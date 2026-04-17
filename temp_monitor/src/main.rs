use ds18b20::{Ds18b20, Resolution};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use log::{error, info};
use one_wire_bus::{OneWire, OneWireResult};

// Scan the bus, issue a single simultaneous conversion, read every DS18B20
// found, and log its temperature. Returns the count of successfully read
// devices.
//
// Generics: `P` is whatever pin type we passed in; `E` is that pin's error
// type. Bounding both `InputPin::Error` and `OutputPin::Error` to the same
// `E` lets the compiler infer error types all the way through — including
// `Ds18b20::new`'s phantom type parameter.
fn read_all<P, E>(bus: &mut OneWire<P>, delay: &mut Ets) -> OneWireResult<usize, E>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: core::fmt::Debug,
{
    // CONVERT T (0x44) to every device on the bus at once.
    ds18b20::start_simultaneous_temp_measurement(bus, delay)?;

    // 12-bit conversion takes up to 750 ms. Library-provided worst-case delay.
    Resolution::Bits12.delay_for_measurement_time(delay);

    let mut search_state = None;
    let mut count = 0;
    loop {
        match bus.device_search(search_state.as_ref(), false, delay)? {
            Some((address, state)) => {
                search_state = Some(state);
                if address.family_code() != ds18b20::FAMILY_CODE {
                    info!("Skipping non-DS18B20 device {:?}", address);
                    continue;
                }
                let sensor = Ds18b20::new(address)?;
                let data = sensor.read_data(bus, delay)?;
                info!(
                    "{:?}  {:6.2} °C  (res {:?})",
                    address, data.temperature, data.resolution
                );
                count += 1;
            }
            None => break,
        }
    }
    Ok(count)
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // GPIO 4 as open-drain I/O — 1-Wire demands the line can be pulled LOW
    // or released (high-Z), never actively driven HIGH.
    // `Pull::Up` enables the ESP32's internal pull-up as a safety net; the
    // real pull-up driving the bus is the 4.7 kΩ on the DFR0198 adapter
    // (or external, in the bare-probe wiring for step 4). Internal in
    // parallel just makes the bus slightly stiffer — no harm.
    let pin = PinDriver::input_output_od(peripherals.pins.gpio4, Pull::Up)?;

    // Ets = ESP-IDF microsecond delay. 1-Wire bit slots are 1–60 µs so
    // millisecond-granularity delay (FreeRtos::delay_us rounds up to 1 ms)
    // would wreck the protocol timing.
    let mut delay = Ets;

    let mut bus = OneWire::new(pin).map_err(|e| anyhow::anyhow!("OneWire init: {:?}", e))?;

    info!("DS18B20 reader starting — scanning 1-Wire bus on GPIO 4");

    loop {
        match read_all(&mut bus, &mut delay) {
            Ok(0) => info!("No DS18B20 devices found on bus"),
            Ok(_) => {}
            Err(e) => error!("read_all: {:?}", e),
        }
        FreeRtos::delay_ms(2000);
    }
}
