# Gotchas

Append-only log of problems hit and how they were solved. Newest at the top. Grep this first when stuck.

## Template

```
## YYYY-MM-DD — Short title
**Symptom:** what I saw
**Cause:** what was actually wrong
**Fix:** exact steps
**Why this bit me:** (optional) the conceptual gap that let it happen
```

---

## 2026-04-18 — `/dev/ttyACM0` permission denied even though I'm in `dialout`
**Symptom:**
```
espflash: Failed to open serial port /dev/ttyACM0
         Error while connecting to device
```
`getent group dialout` confirms my user is in the group, but `groups` in the shell doesn't list it.
**Cause:** Group membership is loaded at login time. If `dialout` is added via NixOS `configuration.nix` (or `usermod -aG`) after a session started, existing shells keep their old group set.
**Fix:** Either `newgrp dialout` in the current shell (spawns a subshell with the group active), or fully log out / reboot so all new sessions pick it up.
**Why this bit me:** On NixOS especially, `configuration.nix` changes apply system-wide immediately, but existing graphical sessions don't re-read groups. This is not NixOS-specific — same thing happens on Ubuntu after `usermod -aG`.

---

## 2026-04-18 — `esp_idf_svc::hal::prelude` does not exist
**Symptom:**
```
error[E0432]: unresolved import `esp_idf_svc::hal::prelude`
  --> src/main.rs:2:23
2 | use esp_idf_svc::hal::prelude::*;
  |                       ^^^^^^^ could not find `prelude` in `hal`
```
**Cause:** `esp-idf-hal` removed its `prelude` module (gone by 0.46 — we're on 0.46.2 via git). Older tutorials and the snippet in `initial_plan.md` still use it.
**Fix:** Import `Peripherals` from its real path:
```rust
use esp_idf_svc::hal::peripherals::Peripherals;
```
**Why this bit me:** Glob-importing from a "prelude" module is a Rust convention for convenience, but preludes are not stable API — they get reshuffled between minor versions. Prefer explicit module paths unless you have a reason to glob-import.

---

## 2026-04-17 — Plan wrongly claimed no onboard controllable LED
**Symptom:** `initial_plan.md` says the ESP32-S3-N16R8 has no user-controllable LED and Step 2 tells you to wire an external LED + 330Ω on GPIO 5.
**Cause:** The plan was written without inspecting the actual board layout. This dev board does have a **WS2812B RGB LED on GPIO 48**, plus UART activity LEDs on GPIO 43 (TX) and 44 (RX).
**Fix:** Prefer the onboard RGB LED for Step 2 (no wiring). Note: GPIO 48 is only connected to the LED if the "RGB" solder jumper on the board is bridged — check with a multimeter or inspect the pad visually. See [hardware/pinout.md](hardware/pinout.md).
**Why this bit me:** Assumptions about unfamiliar hardware made without reading the board's own user guide. When starting a new board, always read its user guide cover-to-cover once before writing a plan.

---
