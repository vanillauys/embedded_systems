 Pool Temperature Monitor — ESP32-S3 Firmware (Rust)

## Goal

Learn how the ESP32-S3 works by building a battery-powered pool temperature monitor in Rust. Focus entirely on firmware and hardware — no backend, no frontend, no cloud. Just the ESP32 reading a sensor, connecting to Wi-Fi, and printing/posting data.

## Hardware on the desk

- **MCU:** ESP32-S3-N16R8 dev board (generic Espressif reference design, 16MB flash, 8MB PSRAM, USB-C). No built-in controllable LED — the red LED visible when plugged in is the power indicator.
- **Temperature sensor (kit):** DFRobot KIT0021 — DS18B20 waterproof probe + "Plugable Terminal V2" adapter board (has pull-up resistor on-board). Adapter has screw terminals for the probe wires (red=VCC, yellow=DATA, black=GND) and a 3-pin JST header for signal/VCC/GND.
- **Temperature sensor (bare):** DS18B20 waterproof probe, 1m cable (red=VCC, yellow=DATA, black=GND). Requires external 4.7kΩ pull-up resistor between DATA and 3.3V.
- **Resistors:** 4.7kΩ (pack of 50)
- **Power:** Samsung INR18650-35E 3400mAh in single-cell wire holder → MH-CD42 charge/boost module (3.7V→5V, battery protection, USB-C charging, 4-LED indicator)
- **Breadboard:** Half-size 400 tie points
- **Jumper wires:** Male-male + male-female packs
- **Enclosure:** Gainta G212C IP65 (save for later)

## Toolchain

The ESP32-S3 uses the **Xtensa** architecture, which requires a special Rust toolchain. We use the `esp-idf-hal` ecosystem (Rust bindings over Espressif's official ESP-IDF C framework). This gives us Wi-Fi, BLE, NVS, OTA, and all ESP-IDF features from Rust.

**Required tools:**
- `espup` — installs the Xtensa Rust toolchain and LLVM fork
- `cargo-espflash` or `espflash` — flashes firmware to the board
- `ldproxy` — linker proxy required by esp-idf-sys
- `esp-idf` — pulled automatically by the build system (no manual install needed)

**Setup commands (run once):**
```bash
# Install espup (manages the Xtensa Rust toolchain)
cargo install espup
espup install  # installs the esp Rust toolchain + LLVM

# Source the environment (add to shell profile)
source $HOME/export-esp.sh  # or wherever espup tells you

# Install flash tooling
cargo install espflash
cargo install ldproxy

# Verify
rustup toolchain list  # should show esp
```

**Note for NixOS:** You may need to install some system deps via nix. Key ones: `pkg-config`, `python3`, `cmake`, `ninja`, `libclang`, `openssl`. If the build fails complaining about missing system libraries, add them to your nix environment. The ESP-IDF build system pulls and compiles the C SDK during first build — this takes 5-10 minutes and ~2GB disk. Subsequent builds are fast.

## Project Structure

```
~/stuff/embedded_systems/
├── Cargo.toml
├── build.rs                    # ESP-IDF build integration
├── rust-toolchain.toml         # Pin to esp toolchain
├── sdkconfig.defaults          # ESP-IDF Kconfig defaults
├── src/
│   └── main.rs
└── README.md
```

## Cargo.toml

```toml
[package]
name = "pool-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-idf-svc = { version = "0.50", features = ["binstart"] }
esp-idf-hal = "0.45"
esp-idf-sys = { version = "0.36", features = ["binstart"] }
log = "0.4"
anyhow = "1"
one-wire-bus = "0.2"
ds18b20 = "0.2"

[build-dependencies]
embuild = "0.32"
```

**Important:** These version numbers may be stale. On first setup, check https://github.com/esp-rs/esp-idf-svc for the latest compatible set of `esp-idf-svc`, `esp-idf-hal`, and `esp-idf-sys` versions — they must all be from the same release cycle. The esp-rs ecosystem moves fast.

## rust-toolchain.toml

```toml
[toolchain]
channel = "esp"
```

## build.rs

```rust
fn main() {
    embuild::espidf::sysenv::output();
}
```

## sdkconfig.defaults

```
CONFIG_ESP_MAIN_TASK_STACK_SIZE=8192
CONFIG_ESP_SYSTEM_EVENT_TASK_STACK_SIZE=4096
```

## Pin Assignments

- GPIO 4 — DS18B20 data line (1-Wire, needs open-drain capable pin)
- GPIO 5 — Status LED (wire an LED + 330Ω to GND, or just use log output for now)
- GPIO 0 — Button (active low, internal pull-up, RTC-capable for deep sleep wakeup)
- 5V — power in (from USB or MH-CD42)
- GND — common ground

## Learning path (do these in order, each builds on the last)

### Step 1: Hello world — Serial output
- Create the project, flash a sketch that logs "Hello from ESP32-S3" every second
- Confirms: Xtensa toolchain works, board is detected, USB serial works
- No wiring needed — just USB cable

```rust
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::delay::FreeRtos;
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let _peripherals = Peripherals::take()?;

    loop {
        info!("Hello from ESP32-S3!");
        FreeRtos::delay_ms(1000);
    }
}
```

**Flash command:** `cargo build --release && espflash flash target/xtensa-esp32s3-espidf/release/pool-monitor --monitor`

Or just: `espflash flash --monitor target/xtensa-esp32s3-espidf/release/pool-monitor`

### Step 2: Blink an LED (GPIO output)
- Wire LED + 330Ω resistor from GPIO 5 to GND on the breadboard
- Toggle it on/off every 500ms
- Learns: PinDriver, GPIO output, basic peripheral access in esp-idf-hal

### Step 3: Read the DS18B20 (kit version)
- Wire the DFRobot adapter board: signal→GPIO 4, VCC→3.3V, GND→GND
- Screw probe wires into adapter terminals (red=VCC, yellow=DATA, black=GND)
- Read temperature, log every 2 seconds
- Learns: 1-Wire protocol, open-drain GPIO, the `one-wire-bus` + `ds18b20` crates
- **Note:** 1-Wire on ESP32 in Rust requires an open-drain GPIO output. Use `PinDriver::input_output_od()` for the data pin.

### Step 4: Read the DS18B20 (manual version)
- Wire the bare probe directly: red→3.3V, black→GND, yellow→GPIO 4
- Add 4.7kΩ resistor between 3.3V and GPIO 4 on the breadboard
- Same code as step 3 — should work identically
- Learns: what the adapter board was doing (just a pull-up + terminal block)

### Step 5: Connect to Wi-Fi
- Hardcode SSID/password, connect, log IP address and RSSI
- Make an HTTP GET to http://httpbin.org/get to prove connectivity
- Learns: `EspWifi`, `BlockingWifi`, `EspHttpConnection`, networking stack
- Wi-Fi requires the `EspEventLoop` and `EspDefaultNvsPartition` — these are singletons you take once at startup

### Step 6: Deep sleep
- Read temp → log → deep sleep 10 seconds (short for testing) → wake → repeat
- Use `esp_idf_svc::sys::esp_deep_sleep()` for the sleep call
- Learns: deep sleep, RTC memory (use `#[link_section = ".rtc.data"]` for persistent vars), power management
- **Important:** After deep sleep, `main()` runs from the top again. There is no "resume". All state is lost except RTC memory.

### Step 7: Button wakeup
- Wire tactile button between GPIO 0 and GND (GPIO 0 has internal pull-up)
- Add ext0 wakeup source: `esp_sleep_enable_ext0_wakeup(gpio_num_t_GPIO_NUM_0, 0)`
- Detect wake reason with `esp_sleep_get_wakeup_cause()` and log whether it was timer or button
- Learns: ext0 wakeup, GPIO wakeup sources, wake cause detection

### Step 8: POST data over HTTPS
- Full loop: wake → read temp → connect Wi-Fi → POST JSON → sleep
- Use https://webhook.site as a temporary backend (free, shows incoming requests in browser)
- JSON body: `{"deviceId":"pool-01","tempC":25.5,"bootCount":42,"rssi":-65}`
- Learns: `EspHttpConnection`, HTTP POST, manual JSON string formatting (or use `serde_json` if you add it), TLS

### Step 9: Battery power
- Wire: 18650 in holder → MH-CD42 BAT+/BAT- → MH-CD42 OUT+/OUT- → ESP32 5V/GND
- Unplug USB, confirm device boots and runs on battery
- Check: does it wake, read, and go back to sleep correctly without USB?
- Learns: power path, voltage regulation, real-world power constraints

### Step 10: NVS configuration
- Store/retrieve settings in NVS: device ID, sleep interval, alert thresholds
- Use `EspDefaultNvs` or raw `nvs_flash` bindings
- Read config at boot, apply it (e.g., variable sleep duration)
- Learns: non-volatile storage, persistent config across reboots and deep sleep

## Important ESP32-S3 + Rust gotchas

1. **First build is slow.** The build system downloads and compiles the entire ESP-IDF C SDK (~2GB, 5-10 min). After that, incremental builds are ~10-30 seconds. Be patient on the first `cargo build`.

2. **Xtensa toolchain:** ESP32-S3 is Xtensa, not RISC-V. You need the `esp` Rust toolchain from `espup`, not standard Rust. If you see errors about unknown target `xtensa-esp32s3-espidf`, your toolchain isn't set up. Run `espup install` again and source the env file.

3. **USB Serial:** This board uses native USB CDC. Logging output goes through the ESP-IDF logging system (`log` crate + `EspLogger`). Use `espflash flash --monitor` to see output after flashing.

4. **GPIO 0 / BOOT button:** If the board won't flash, hold BOOT → press RST → release BOOT to enter download mode. GPIO 0 is the BOOT pin — using it as a button in normal operation is fine, just don't hold it during reset.

5. **3.3V logic:** All GPIOs are 3.3V. DS18B20 is fine on 3.3V. Never connect 5V to a GPIO.

6. **Deep sleep current:** Expect ~100-500μA on this generic dev board (voltage regulator stays on). Fine for learning.

7. **NixOS specifics:** The ESP-IDF build needs `python3`, `cmake`, `ninja`, `pkg-config`. You might need to run inside a `nix-shell` or `devShell` with these available. If `libclang` is missing, add `llvmPackages.libclang` to your environment and set `LIBCLANG_PATH`.

8. **esp-rs version compatibility:** `esp-idf-svc`, `esp-idf-hal`, and `esp-idf-sys` must all be from the same release. Check the esp-rs GitHub repos for the latest compatible versions before starting. Using mismatched versions is the #1 source of cryptic build errors.

9. **Open-drain for 1-Wire:** The DS18B20 needs an open-drain GPIO. In esp-idf-hal, use `PinDriver::input_output_od(pin)` — not `PinDriver::output(pin)`. Regular push-pull output will not work with 1-Wire.

10. **cargo-espflash vs espflash:** `espflash` is the standalone tool; `cargo-espflash` integrates with cargo. Either works. Use whichever you prefer: `espflash flash --monitor target/...` or `cargo espflash flash --monitor`.

## Current Task

Start with Step 1: Set up the Rust ESP-IDF project and flash a "Hello from ESP32-S3" serial output to confirm the toolchain and board work.
