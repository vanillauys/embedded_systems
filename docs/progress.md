# Progress Log

Single source of truth for where the project is. Update when finishing a step.

## Current state

- **Step in progress:** 5 — Wi-Fi captive-portal provisioning (branch `step_05_wifi_captive_portal`, implementation complete, **untested on hardware**).  *(Step 2 still deferred, Step 4 re-wiring still pending.)*
- **Project path:** `temp_monitor/` (Cargo project)
- **Last action:** Service-module refactor merged to `main` (PR #3 → `a46cf05`). Implemented the full captive portal: `wifi/` module is now a directory with `mod.rs` (state machine), `credentials.rs` (NVS), `ap.rs` (SoftAP), `sta.rs` (STA client), `dns.rs` (UDP DNS hijack), `http.rs` (form + save/reboot). `main.rs` enables the service. `sdkconfig.defaults` bumps HTTP stack + lwIP sockets. Build is clean. Full walkthrough in [steps/05-wifi-captive-portal.md](steps/05-wifi-captive-portal.md); concept note at [concepts/captive-portal.md](concepts/captive-portal.md).
- **Next action:** Flash and test. First boot should expose `PoolMon-XXXX`; phone-join should auto-open the captive portal onto the config form; submitting credentials should reboot into STA mode on the home network. Expect iteration — this is a lot of untested code.

## Checklist

- [x] 0 — Install toolchain (`espup`, `espflash`, `ldproxy`, `cargo-generate`)
- [x] 1 — Hello world / Serial output
- [ ] 2 — Blink LED (GPIO output)  *(deferred — RGB jumper not bridged, no soldering iron yet)*
- [x] 3 — DS18B20 read (kit adapter)
- [ ] 4 — DS18B20 read (bare probe + manual pull-up)  *(pending — just re-wiring, no code change)*
- [ ] 5 — Wi-Fi connect + captive-portal provisioning  ← _you are here_ (implementation done, awaiting flash test)
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
| 2026-04-19 | — | PR #2 (Step 3) squash-merged to `main`. Started service-module refactor: `main.rs` now orchestrates three services — `sensor` (real DS18B20), `wifi` (stub, step 5, esp-idf-svc provisioning), `api` (stub, step 8). Pattern documented in concepts/service-modules.md. |
| 2026-04-19 | — | Refactor PR #3 merged to `main` (→ `a46cf05`). |
| 2026-04-19 | 5 | Implemented Step 5 end-to-end: custom captive-portal Wi-Fi provisioning (6 files under `wifi/`). SoftAP `PoolMon-XXXX`, UDP DNS hijack, HTTP form on `/`, POST `/save` writes NVS + reboots. Boot-time state machine: if creds present → STA; else AP + portal. sdkconfig bumps for HTTPD stack + lwIP sockets. Build clean. Not yet flashed. Walkthrough in `steps/05-wifi-captive-portal.md`. |
