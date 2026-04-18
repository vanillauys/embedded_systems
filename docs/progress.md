# Progress Log

Single source of truth for where the project is. Update when finishing a step.

## Current state

- **Step in progress:** 4 — DS18B20 bare probe + manual 4.7 kΩ pull-up  *(Step 2 still deferred)*
- **Project path:** `temp_monitor/` (Cargo project)
- **Last action:** ✅ Step 3 complete. DFR0198 kit read working: address `8A00001125DC9D28` (family code `28` = DS18B20), 12-bit resolution, thermal response confirmed (hand-warming → 32 °C, release → drifts back to ~28.5 °C). Full writeup in [steps/03-ds18b20-kit.md](steps/03-ds18b20-kit.md); concept note at [concepts/one-wire-protocol.md](concepts/one-wire-protocol.md).
- **Next action:** Step 4 is ~minutes of work: swap the DFR0198 adapter for the bare probe + one of the 4.7 kΩ resistors from the pack. Same firmware — proves the adapter was "just" a pull-up + terminals. No code change expected.

## Checklist

- [x] 0 — Install toolchain (`espup`, `espflash`, `ldproxy`, `cargo-generate`)
- [x] 1 — Hello world / Serial output
- [ ] 2 — Blink LED (GPIO output)  *(deferred — RGB jumper not bridged, no soldering iron yet)*
- [x] 3 — DS18B20 read (kit adapter)
- [ ] 4 — DS18B20 read (bare probe + manual pull-up)  ← _you are here_
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
| 2026-04-18 | 2 → 3 | Step 2 deferred: visual inspection confirms the "RGB" solder jumper on the board is not bridged, so GPIO 48 is disconnected from the onboard WS2812B LED. No soldering iron on hand. PR #1 converted to draft. Reordering to Step 3 (DS18B20 kit) since all hardware for it is already on desk. |
| 2026-04-18 | 3 | ✅ Step 3 complete. DS18B20 reads over 1-Wire on GPIO 4 via DFR0198 kit adapter. Address `8A00001125DC9D28`, 12-bit resolution, cycle ~780 ms. Debug journey: stale `input_output_od` API, `Ds18b20::new` generic-E inference via helper fn, wire-colour mismapping, intermittent screw-terminal contact diagnosed via `bus.reset()` logging + wire wiggle. See gotchas + concepts/one-wire-protocol.md. |
