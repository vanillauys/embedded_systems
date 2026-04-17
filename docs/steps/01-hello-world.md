# Step 1 — Hello World (Serial Output)

**Status:** ✅ done (2026-04-18)
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

### Things that bit me
1. **`esp_idf_svc::hal::prelude` no longer exists** in `esp-idf-hal` 0.46+. The snippet in `initial_plan.md` was stale. Fixed by importing `Peripherals` explicitly: `use esp_idf_svc::hal::peripherals::Peripherals;`. See [gotchas.md](../gotchas.md).
2. **`/dev/ttyACM0` permission denied** even though I was in the `dialout` group per `getent`. My graphical session had started before the group was added, so the running shell didn't have it. Rebooting fixed it. `newgrp dialout` in the shell also works as a one-off.

### Build timing
- First `cargo build --release` compiled the full ESP-IDF C SDK (esp-idf-sys/hal/svc) — about 5–7 min on this machine.
- Incremental builds after a source change: **~2.5s**.
- Flash time: ~25s for the 400 KB app.

### Things worth noticing from the boot log

```
I (227) cpu_start: GPIO 44 and 43 are used as console UART I/O pins
I (228) cpu_start: cpu freq: 160000000 Hz
I (237) app_init: Project name:     libespidf
I (237) app_init: App version:      488ba71         ← git SHA of HEAD
I (246) app_init: Compile time:     Apr 18 2026 00:03:13
I (251) app_init: ESP-IDF:          v5.5.3
I (272) heap_init: At 3FC94D90 len 00054980 (338 KiB): RAM
I (353) temp_monitor: Hello from ESP32-S3! tick=0
I (1353) temp_monitor: Hello from ESP32-S3! tick=1
```

- **`I (353)`** — milliseconds since boot. First tick fires ~350 ms after reset. That's how fast ESP-IDF boots.
- **`FreeRtos::delay_ms(1000)` is exact** — ticks are spaced to the millisecond (353 → 1353 → 2353 …). Not a busy-wait; the OS scheduler handles it.
- **App version = commit SHA** (`488ba71`). `embuild` auto-embeds that into the ESP-IDF build. Free provenance on every boot.
- **Console UART on GPIO 43/44** — those are also the TX/RX LEDs on the board. While ESP-IDF uses them as the serial console, they blink with every log line. If I ever want to reuse them as GPIOs, I'd have to disable UART console in `sdkconfig`.
- **Heap: 338 KiB RAM + 32 KiB DRAM + 21 KiB + 7 KiB RTCRAM** — plenty for our purposes. PSRAM (8 MB) is available but not used yet.
- **Flash chip:** `boya` — some cheap flash brand, not Winbond. Not a problem unless it shows up in a bug report somewhere.
- **Boot message `W (…) i2c: old driver, migrate to i2c_master.h`** — harmless warning from ESP-IDF itself; not our code.

### What this proves
- Xtensa toolchain produces correct binaries.
- Board enumerates as `/dev/ttyACM0` via native USB (right USB-C port).
- `cargo run --release` → flash → monitor is a one-command loop. ✓
- The Rust `log` crate is wired through `EspLogger` to ESP-IDF's native logging; output is indistinguishable from C-side IDF logs.

## Next step

[Step 2 — Blink an LED](02-blink-led.md). Prefer the onboard WS2812B RGB LED on GPIO 48 (requires bridging the "RGB" solder jumper on the board) over an external LED. See [hardware/pinout.md](../hardware/pinout.md).
