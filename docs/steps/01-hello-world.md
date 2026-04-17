# Step 1 — Hello World (Serial Output)

**Status:** in progress (project scaffolded, not yet flashed)
**Goal:** Confirm the Xtensa Rust toolchain works, the board enumerates over USB, and log output reaches the host.

## What we did

### 1. Scaffolded the project

```bash
cd ~/stuff/embedded_systems
cargo generate esp-rs/esp-idf-template cargo
# project name: temp_monitor
# MCU: esp32s3
# advanced: false
# std: true
```

This produced `temp_monitor/` with all the plumbing wired up:

- `rust-toolchain.toml` pins `channel = "esp"` — cargo auto-selects the Xtensa toolchain.
- `.cargo/config.toml` sets target `xtensa-esp32s3-espidf` and a `runner = "espflash flash --monitor"`, so `cargo run` does the flashing.
- `Cargo.toml` has `[patch.crates-io]` pointing esp-idf-{sys,hal,svc} to git. Known-good versions; avoids the manual version-matching trap.
- `build.rs` = one line, bridges Cargo → ESP-IDF's CMake build.
- `sdkconfig.defaults` = ESP-IDF Kconfig defaults. Don't touch yet.

### 2. Replaced `src/main.rs` with a looping log

```rust
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
```

Added `anyhow = "1"` to `[dependencies]` for the `?` operator on `main`'s `Result`.

## What's worth noticing

- **`link_patches()`** — a no-op call that the linker uses as an anchor to keep certain ESP-IDF C runtime symbols alive. If you skip it you'll get unresolved-symbol link errors. Weird but required.
- **`EspLogger::initialize_default()`** — binds the `log` crate's macros (`info!`, `warn!`, `error!`) to ESP-IDF's logging system, which then writes to UART / USB-CDC.
- **`Peripherals::take()?`** — Rust ownership applied to hardware. You can take it exactly once; a second `take()` returns `None`. That's how the HAL prevents two parts of your program from both claiming GPIO 5.
- **`FreeRtos::delay_ms(1000)`** — not a busy-wait. It yields to FreeRTOS, which can schedule other tasks (Wi-Fi, logging, etc.) while you're "asleep". This matters later.
- **`count.wrapping_add(1)`** — Rust panics on integer overflow in debug builds by default. `wrapping_add` is explicit "I want modular arithmetic". Not strictly needed for a u32 counting seconds, but it's a habit worth forming.

## Flash + run

```bash
cd temp_monitor
cargo build --release      # 5–10 min first time
cargo run --release        # flashes and opens monitor
```

Expected output:

```
I (1234) temp_monitor: Hello from ESP32-S3! tick=0
I (2234) temp_monitor: Hello from ESP32-S3! tick=1
...
```

Exit monitor with `Ctrl+]`.

## What I learned

_Fill in after flashing. Especially: anything surprising, any error I hit, timing of the build, what the first log lines look like._

## Next step

[Step 2 — Blink an LED](02-blink-led.md) (not yet written)
