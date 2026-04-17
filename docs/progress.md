# Progress Log

Single source of truth for where the project is. Update when finishing a step.

## Current state

- **Step in progress:** 1 — Hello world / Serial output
- **Project path:** `temp_monitor/` (Cargo project)
- **Last action:** Scaffolded project from `esp-idf-template`, replaced `main.rs` with looping hello-world, added `anyhow` dep.
- **Next action:** Run `cargo build --release` in `temp_monitor/`. Expect a 5–10 min first build while ESP-IDF C SDK downloads and compiles. Then `cargo run --release` with the board plugged in.

## Checklist

- [x] 0 — Install toolchain (`espup`, `espflash`, `ldproxy`, `cargo-generate`)
- [ ] 1 — Hello world / Serial output  ← _you are here_
- [ ] 2 — Blink LED (GPIO output)
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
