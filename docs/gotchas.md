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

## 2026-04-19 — Occasional 1-Wire `UnexpectedResponse` while Wi-Fi is active
**Symptom:** When the Wi-Fi radio is doing scans / associating / handling HTTP requests, the DS18B20 reader occasionally errors `sensor: UnexpectedResponse` or returns no devices, then self-heals on the next 2 s cycle.
**Cause:** 1-Wire is bit-banged — each bit slot is 1–60 µs wide and relies on precise `Ets` microsecond delays. The Wi-Fi driver task preempts us occasionally; when it does during a read, the timing window gets stretched and the device either misses a response or returns garbage the library flags as `UnexpectedResponse`.
**Fix:** None required — the next cycle recovers. If this ever becomes a real problem (e.g. sustained Wi-Fi traffic causing >10% reading loss), pin the sensor task to the core Wi-Fi *isn't* running on (ESP32-S3 has two cores), or switch to the `esp-idf-hal::onewire` driver which uses hardware RMT for bit timing instead of CPU cycles.
**Why this bit me:** Hadn't considered that Wi-Fi brings real-time constraints into the sensor loop. Before Step 5 the loop ran uninterrupted; now it shares cycles with a live radio stack.

---

## 2026-04-18 — Intermittent screw-terminal contact on 1-Wire bus
**Symptom:** After fixing all wiring colours, `bus.reset()` kept returning `Ok(false)` (no presence pulse). The probe was cool, wiring was correct, and the bus was electrically healthy (no `BusNotHigh`). Yet: 90 seconds of `No DS18B20 devices found on bus`, then — after gentle wiggling of one probe wire at its screw terminal — `RESET: presence pulse detected ✓` and clean temperature readings every 2 s.
**Cause:** One of the three screw-terminal contacts on the DFR0198 adapter wasn't actually clamping the probe's stripped conductor. The wire looked seated, even tugged firmly, but the copper strands weren't reaching the screw.
**Fix:** Back the screw all the way out, strip a fresh ~5 mm, twist the strands tight, push all the way in, re-tighten firmly. Tug-test again.
**Why this bit me:** Intermittent hardware faults produce partial-working / partial-not symptoms that look like software bugs. Adding an explicit `bus.reset()` diagnostic (`presence pulse detected ✓` vs `no presence pulse`) plus a physical wiggle test was what surfaced this as "flickers to working when I move that wire" — classic bad contact.

---

## 2026-04-18 — Misread DFRobot JST cable mapping at first glance
**Symptom:** First run of step 3 returned `No DS18B20 devices found on bus`. After re-wiring (swapping what I thought needed swapping) the next run returned `BusNotHigh` — DATA pin stuck LOW.
**Cause:** Mis-described the JST cable's wire → silkscreen-label mapping on the first look. I thought black = DATA / red = VCC / green = GND. Actually the cable is **standard**: **black = GND (−), red = VCC (+), green = DATA (D)**. The second wiring crossed DATA to GND, producing `BusNotHigh`.
**Fix:** Green → GPIO 4, red → 3V3, black → GND. The probe-side screw terminals are separate and unchanged: yellow → A (D), red → B (+), black → C (−).
**Why this bit me:** The silkscreen labels on the adapter (`D`, `+`, `−`) are the only ground truth. Wire colours are a vendor convention that I should verify rather than guess. Two tight, bright-lit looks at the board beat one rushed glance every time.

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
