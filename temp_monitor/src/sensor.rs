//! DS18B20 temperature sensor on a 1-Wire bus.
//!
//! A thin service wrapper around the `one-wire-bus` + `ds18b20` crates. Takes
//! ownership of a GPIO pin on construction; each `read_all()` call fires a
//! simultaneous conversion on every DS18B20 on the bus and returns every
//! reading it could get.
//!
//! The bus is driven open-drain with the ESP32's internal pull-up as a safety
//! net; the real pull-up is expected on the hardware side (4.7 kΩ on the
//! DFR0198 adapter, or wired externally in the bare-probe setup).
//!
//! See `docs/concepts/one-wire-protocol.md` and
//! `docs/concepts/service-modules.md` for background.

use ds18b20::{Ds18b20, Resolution};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{InputOutput, PinDriver, Pull};
use one_wire_bus::{Address, OneWire, OneWireResult};

/// One temperature reading from a single device on the bus.
pub struct Reading {
    pub address: Address,
    pub temperature: f32,
    pub resolution: Resolution,
}

/// Service handle for a DS18B20 bus on a single GPIO pin.
///
/// The concrete pin type is erased inside `PinDriver` on construction, so the
/// struct itself is only generic over the pin's lifetime. Callers pass the
/// pin directly from `peripherals.pins.gpioN`.
pub struct TemperatureSensor<'d> {
    bus: OneWire<PinDriver<'d, InputOutput>>,
    delay: Ets,
}

impl<'d> TemperatureSensor<'d> {
    /// Take ownership of the pin, configure it open-drain, and wrap it in a
    /// 1-Wire bus driver.
    ///
    /// The bound is on esp-idf-hal's *hardware* pin traits (needed so the
    /// pin can be driven open-drain). `'d` is the lifetime of the borrowed
    /// peripheral — in practice, the lifetime of `Peripherals::take()`'s
    /// return value, i.e. the program.
    pub fn new<T>(pin: T) -> anyhow::Result<Self>
    where
        T: esp_idf_svc::hal::gpio::InputPin + esp_idf_svc::hal::gpio::OutputPin + 'd,
    {
        let driver = PinDriver::input_output_od(pin, Pull::Up)?;
        let bus = OneWire::new(driver)
            .map_err(|e| anyhow::anyhow!("OneWire init: {:?}", e))?;
        Ok(Self { bus, delay: Ets })
    }

    /// Scan the bus and read every DS18B20 found. Non-DS18B20 devices on the
    /// bus (if any) are silently skipped.
    pub fn read_all(&mut self) -> anyhow::Result<Vec<Reading>> {
        read_all_inner(&mut self.bus, &mut self.delay)
            .map_err(|e| anyhow::anyhow!("sensor: {:?}", e))
    }
}

// Generic helper kept as a free function so the compiler can infer the pin's
// error type `E` uniformly through `bus.device_search(...)?`,
// `Ds18b20::new(...)?`, and `read_data(...)?`. The embedded-hal 0.2 trait
// bounds here are distinct from the esp-idf-hal-level bounds used at
// construction time — the former are what `one-wire-bus` uses to bit-bang
// the protocol.
fn read_all_inner<P, E>(
    bus: &mut OneWire<P>,
    delay: &mut Ets,
) -> OneWireResult<Vec<Reading>, E>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: core::fmt::Debug,
{
    ds18b20::start_simultaneous_temp_measurement(bus, delay)?;
    Resolution::Bits12.delay_for_measurement_time(delay);

    let mut out = Vec::new();
    let mut search_state = None;
    loop {
        match bus.device_search(search_state.as_ref(), false, delay)? {
            Some((address, state)) => {
                search_state = Some(state);
                if address.family_code() != ds18b20::FAMILY_CODE {
                    continue;
                }
                let sensor = Ds18b20::new(address)?;
                let data = sensor.read_data(bus, delay)?;
                out.push(Reading {
                    address,
                    temperature: data.temperature,
                    resolution: data.resolution,
                });
            }
            None => break,
        }
    }
    Ok(out)
}
