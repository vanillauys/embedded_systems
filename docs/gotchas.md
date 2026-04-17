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

## 2026-04-17 — Plan wrongly claimed no onboard controllable LED
**Symptom:** `initial_plan.md` says the ESP32-S3-N16R8 has no user-controllable LED and Step 2 tells you to wire an external LED + 330Ω on GPIO 5.
**Cause:** The plan was written without inspecting the actual board layout. This dev board does have a **WS2812B RGB LED on GPIO 48**, plus UART activity LEDs on GPIO 43 (TX) and 44 (RX).
**Fix:** Prefer the onboard RGB LED for Step 2 (no wiring). Note: GPIO 48 is only connected to the LED if the "RGB" solder jumper on the board is bridged — check with a multimeter or inspect the pad visually. See [hardware/pinout.md](hardware/pinout.md).
**Why this bit me:** Assumptions about unfamiliar hardware made without reading the board's own user guide. When starting a new board, always read its user guide cover-to-cover once before writing a plan.

---
