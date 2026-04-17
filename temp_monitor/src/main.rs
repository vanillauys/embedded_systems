use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::*;
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let _peripherals = Peripherals::take()?;

    let mut count: u32 = 0;
    loop {
        info!("Hello from ESP32-S3! tick={count}");
        count = count.wrapping_add(1);
        FreeRtos::delay_ms(1000);
    }
}
