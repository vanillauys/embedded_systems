# Progress Log

Single source of truth for where the project is. Update when finishing a step.

## Current state

- **Step in progress:** 2 — Blink an LED
- **Project path:** `temp_monitor/` (Cargo project)
- **Last action:** ✅ Step 1 complete. Board flashed and logging `tick=N` every second via USB CDC. First tick at ~353 ms after boot. See [steps/01-hello-world.md](steps/01-hello-world.md) for what was learned.
- **Next action:** Start Step 2 — drive the onboard WS2812B RGB LED on GPIO 48. First: check whether the "RGB" solder jumper on the board is bridged (multimeter continuity between GPIO 48 pad and the LED data pin, or visual inspection). Then pick a driver crate — likely `ws2812-esp-idf-driver` (uses the RMT peripheral). No external wiring should be needed.

## Checklist

- [x] 0 — Install toolchain (`espup`, `espflash`, `ldproxy`, `cargo-generate`)
- [x] 1 — Hello world / Serial output
- [ ] 2 — Blink LED (GPIO output)  ← _you are here_
- [ ] 3 — DS18B20 read (kit adapter)
- [ ] 4 — DS18B20 read (bare probe + manual pull-up)
- [ ] 5 — Wi-Fi connect + HTTP GET
- [ ] 6 — Deep sleep loop
- [ ] 7 — Button wakeup (ext0)
- [ ] 8 — HTTPS POST to webhook.site
- [ ] 9 — Battery power (18650 + MH-CD42)
- [ ] 10 — NVS persistent config

## History

| Date | Step | What happened |
|---|---|---|
| 2026-04-17 | 0 | Toolchain installed. Board on desk, blinking power LED. |
| 2026-04-17 | 1 | Project scaffolded with `cargo generate esp-rs/esp-idf-template`. `main.rs` updated to looping log. Build not yet run. |
| 2026-04-17 | — | Component datasheets downloaded to `docs/hardware/datasheets/`, converted with `pdftotext -layout`. Index at `datasheets/README.md`. Components index at `hardware/components.md` updated with local links. |
| 2026-04-18 | 1 | ✅ Step 1 complete. First build of ESP-IDF C SDK took ~5–7 min. Incremental build ~2.5s. First flash + monitor run succeeded after rebooting to pick up `dialout` group membership. Log output confirmed at 1Hz, first tick at boot+353ms. |
