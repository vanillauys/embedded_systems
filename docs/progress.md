# Progress Log

Single source of truth for where the project is. Update when finishing a step.

## Current state

- **Step in progress:** 3 — Read DS18B20 (kit adapter)  *(Step 2 deferred)*
- **Project path:** `temp_monitor/` (Cargo project)
- **Last action:** ✅ Step 1 complete. Step 2 paused: the "RGB" solder jumper on this board is not bridged and there's no soldering iron on hand yet, so the onboard WS2812B on GPIO 48 is disconnected. Step 2 PR is open as draft at vanillauys/embedded_systems#1.
- **Next action:** Start Step 3 on branch `step_03_ds18b20_kit`. Wire the DFR0198 adapter to GPIO 4 / 3V3 / GND, add `one-wire-bus` and `ds18b20` crates, rewrite `main.rs` to read + log temperature every 2s. See [steps/03-ds18b20-kit.md](steps/03-ds18b20-kit.md).

## Checklist

- [x] 0 — Install toolchain (`espup`, `espflash`, `ldproxy`, `cargo-generate`)
- [x] 1 — Hello world / Serial output
- [ ] 2 — Blink LED (GPIO output)  *(deferred — RGB jumper not bridged, no soldering iron yet)*
- [ ] 3 — DS18B20 read (kit adapter)  ← _you are here_
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
| 2026-04-18 | 2 → 3 | Step 2 deferred: visual inspection confirms the "RGB" solder jumper on the board is not bridged, so GPIO 48 is disconnected from the onboard WS2812B LED. No soldering iron on hand. PR #1 converted to draft. Reordering to Step 3 (DS18B20 kit) since all hardware for it is already on desk. |
