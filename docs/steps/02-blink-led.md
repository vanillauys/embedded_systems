# Step 2 — Blink an LED

**Status:** in progress (branch `step_02_blink_led`)
**Goal:** Control a physical LED from the ESP32 — either the onboard WS2812B RGB LED on GPIO 48, or an external LED on GPIO 5 as fallback.

## What the plan originally said

`initial_plan.md` assumed no onboard user LED and told us to wire an external LED + 330 Ω resistor on GPIO 5. Reading the board layout image showed that's wrong — this board has a WS2812B RGB LED on GPIO 48. See [gotchas.md](../gotchas.md) for the catch.

## Two paths

### A. Onboard RGB LED (preferred — no wiring)

- GPIO 48 → WS2812B addressable RGB LED.
- **Requires** the "RGB" solder jumper on the board to be bridged. Inspect the board first — there's a small two-pad jumper near the LED.
- WS2812B is addressable over a single data line using a precise 800 kHz NRZ protocol. ESP32's RMT peripheral is the right way to drive it; don't bit-bang.
- Candidate driver crates (pick one after comparing):
  - [`ws2812-esp-idf-driver`](https://github.com/cat-in-136/ws2812-esp-idf-driver) — thin wrapper over ESP-IDF's RMT. Matches our `esp-idf-*` stack.
  - [`smart-leds`](https://github.com/smart-leds-rs/smart-leds) + a backend — embedded-hal ecosystem, more portable but may need more glue.

### B. External LED on GPIO 5 (fallback)

- Wire: GPIO 5 → 330 Ω → LED anode → cathode → GND.
- Simplest possible output: `PinDriver::output(gpio5)` → `set_high()`/`set_low()` with `FreeRtos::delay_ms(500)`.
- Teaches `PinDriver`, GPIO output mode, but skips the RMT/WS2812 protocol side.

## What we want to learn

- How to take a `Peripherals` singleton and hand off a specific pin to a driver.
- How the RMT peripheral works (option A) or plain push-pull GPIO output (option B).
- How ownership + the typestate pattern play out when you move a pin into a driver.

## Open questions (resolve before coding)

- [ ] Is the "RGB" solder jumper already bridged on this specific board? (Visual inspection or multimeter.)
- [ ] Which driver crate for option A? Tradeoffs, last release date, API ergonomics.
- [ ] Do we want to blink a single colour, cycle RGB, or fade? (Start simple: solid red on/off at 1 Hz.)

## Implementation plan

_Fill in as we go:_

1. Resolve the jumper question → pick option A or B.
2. Add dependency to `Cargo.toml`.
3. Write minimal `main.rs` that blinks at 1 Hz.
4. Flash, confirm.
5. Optional: cycle R → G → B, one colour per second.

## Code

_To be added._

## What I learned

_To be filled in after we finish._

## Next step

[Step 3 — Read the DS18B20 (kit adapter)](03-ds18b20-kit.md) (not yet written)
