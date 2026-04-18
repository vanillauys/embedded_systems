use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use log::info;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // WS2812B RGB LED on GPIO 48, driven via the RMT peripheral.
    // RMT generates the precise 800 kHz NRZ bitstream WS2812B needs — far
    // more reliable than bit-banging from software.
    let mut driver = Ws2812Esp32RmtDriver::new(
        peripherals.rmt.channel0,
        peripherals.pins.gpio48,
    )?;

    // Low brightness (30/255). Full brightness at close range is uncomfortable.
    let colors = [
        ("red",   LedPixelColorGrb24::new_with_rgb(30, 0, 0)),
        ("green", LedPixelColorGrb24::new_with_rgb(0, 30, 0)),
        ("blue",  LedPixelColorGrb24::new_with_rgb(0, 0, 30)),
        ("off",   LedPixelColorGrb24::new_with_rgb(0, 0, 0)),
    ];

    info!("WS2812B cycle on GPIO 48 — LED should show R → G → B → off at 1 Hz");

    let mut i = 0usize;
    loop {
        let (name, color) = &colors[i];
        info!("LED → {name}");
        // WS2812B expects 3 bytes in G,R,B order. `LedPixelColorGrb24` already
        // arranges them correctly; we just hand the driver the raw bytes.
        let pixel: [u8; 3] = color.as_ref().try_into().unwrap();
        driver.write_blocking(pixel.into_iter())?;
        FreeRtos::delay_ms(1000);
        i = (i + 1) % colors.len();
    }
}
