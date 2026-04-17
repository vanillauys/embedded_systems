# Commands Cheat Sheet

Run from `temp_monitor/` unless noted.

## Build

```bash
cargo build --release      # release build (what you flash)
cargo build                # debug build (larger, slower, rarely needed)
cargo check                # type-check only, no codegen — great for quick feedback
```

First build is 5–10 min (compiles ESP-IDF C SDK, ~2GB). Subsequent builds are ~10–30s.

## Flash + monitor

```bash
cargo run --release        # builds + flashes + opens serial monitor (via .cargo/config.toml runner)
```

Exit monitor: `Ctrl+]`.

Manual equivalents (if `cargo run` misbehaves):

```bash
espflash flash --monitor target/xtensa-esp32s3-espidf/release/temp_monitor
espflash monitor           # monitor only, don't flash
```

## Check the board is connected

```bash
lsusb                      # look for Silicon Labs / CH340 / FTDI or Espressif
ls /dev/ttyUSB* /dev/ttyACM*   # serial ports — ESP32-S3 native-USB usually /dev/ttyACM0
sudo dmesg -w              # watch enumeration as you plug it in
```

If flashing fails to detect the chip: hold **BOOT**, press **RST**, release **BOOT** → board enters download mode.

## Environment

```bash
source ~/export-esp.sh     # sets up Xtensa toolchain paths (espup installed this)
rustup toolchain list      # should include "esp"
```

On NixOS, may need `nix develop` / `nix-shell` with `pkg-config`, `python3`, `cmake`, `ninja`, `llvmPackages.libclang`, `openssl`. See [nix.md](nix.md).

## Clean

```bash
cargo clean                # nukes target/ — next build will be slow again
```

Don't do this unless you need to — you'll wait 10 min for the next compile.
