//! Pool temperature monitor — firmware entry point.
//!
//! The firmware is organised around a small set of **services** —
//! self-contained modules that own their hardware resources and expose a
//! narrow API to `main`. Today there are three:
//!
//! | Service | Module | Status |
//! |---------|--------|--------|
//! | DS18B20 temperature sensor | [`sensor`] | working |
//! | Wi-Fi (captive-portal provisioning) | [`wifi`] | stub — step 5 |
//! | HTTP(S) API client | [`api`] | stub — step 8 |
//!
//! To enable or disable a service, (un)comment the relevant line in the
//! "Services" block in `main()` below — and any paired calls inside the
//! main loop. Stubs `bail!` if their `start` is called, so you can't
//! silently run with a half-configured service.
//!
//! See [`docs/concepts/service-modules.md`] for the full rationale behind
//! this shape.

mod api;
mod sensor;
mod wifi;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use log::{error, info};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // ================================================================
    // Services — (un)comment to enable / disable.
    // ================================================================

    // Sensor (step 3): DS18B20 on GPIO 4.
    let mut sensor = sensor::TemperatureSensor::new(peripherals.pins.gpio4)?;

    // Wi-Fi (step 5): SoftAP provisioning via esp-idf-svc.
    // When enabled: also uncomment the `post_reading` call in the loop below.
    // let _wifi = wifi::Wifi::start(peripherals.modem)?;

    // API (step 8): HTTP(S) POST of readings to a backend (e.g. webhook.site).
    // When enabled: also uncomment the `api.post_reading(r)?` call below.
    // let api = api::ApiClient::new("https://webhook.site/<uuid>");

    // ================================================================
    // Main loop.
    // ================================================================

    info!("Services started. Entering main loop.");
    loop {
        match sensor.read_all() {
            Ok(readings) => {
                if readings.is_empty() {
                    info!("No DS18B20 devices on bus");
                }
                for r in &readings {
                    info!(
                        "{:016X?}  {:6.2} °C  (res {:?})",
                        r.address, r.temperature, r.resolution
                    );
                    // api.post_reading(r)?;
                }
            }
            Err(e) => error!("sensor: {:?}", e),
        }
        FreeRtos::delay_ms(2000);
    }
}
