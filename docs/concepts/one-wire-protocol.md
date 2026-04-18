# 1-Wire Protocol

A master-slave serial protocol invented by Dallas Semiconductor (now Maxim / Analog Devices) that runs **signal and optionally power on a single wire**, with a shared ground. Used by the DS18B20 temperature sensor and many ID chips.

## Electrical shape

- One data line (e.g. GPIO 4 on our ESP32) + ground.
- The line idles HIGH. A **pull-up resistor** (typically 4.7 kΩ to 3V3) holds it there.
- **Every device on the bus uses open-drain output**: it can pull the line LOW (to GND) or release it (high-Z, line floats HIGH via the pull-up). Nothing can drive the line HIGH actively. That's why:
  - Contention is impossible — worst case, two devices both pull LOW = still LOW, no shorts.
  - You **must** use `PinDriver::input_output_od(...)` on the ESP32. Push-pull output (`PinDriver::output`) would fight the pull-up and break the bus.
- "Parasite power" is a variant where devices draw power from the data line via a diode when it's HIGH. Not used here — our wiring has a separate VCC.

## Timing

1-Wire is bit-banged. Each bit is a 60–120 µs window divided into two halves:

- **Master writes 0:** pull LOW for 60–120 µs.
- **Master writes 1:** pull LOW briefly (~1–15 µs) then release; pull-up brings the line HIGH for the rest of the slot.
- **Master reads:** pull LOW for ~6 µs, release, sample the line ~15 µs after the start of the slot. Slave device either holds LOW (= 0) or has already released (= 1).

These slots are **microsecond-timed** — this is why the code uses `Ets` (ESP-IDF's microsecond delay) inside the protocol layer. `FreeRtos::delay_us()` rounds up to the next 1 ms OS tick and would wreck the timing.

## The protocol layers

A 1-Wire transaction always starts with a **RESET**:

1. Master pulls LOW for ≥480 µs, then releases.
2. Within 15–60 µs, any device on the bus answers with a **presence pulse** — it pulls LOW for 60–240 µs.
3. If no presence → no device. If presence → continue.

Then two command bytes are exchanged in sequence:

### ROM commands (addressing)
- `0x33 READ ROM` — read the 64-bit ROM ID. Only works with a single device on the bus (multiple devices = bit-level collision on the wire; the library's search algorithm resolves this).
- `0x55 MATCH ROM` — address a specific device by its 64-bit ID. Every other device then ignores the following command.
- `0xCC SKIP ROM` — broadcast; all devices react. Used for `start_simultaneous_temp_measurement` so every DS18B20 on the bus converts at the same time.
- `0xF0 SEARCH ROM` — the bit-level enumeration algorithm that lets the master discover every ROM ID on the bus.

### Function commands (device-specific)
For the DS18B20:
- `0x44 CONVERT T` — trigger a temperature conversion. Takes up to 750 ms at 12-bit resolution; shorter at lower resolutions.
- `0xBE READ SCRATCHPAD` — read 9 bytes: temperature (2 bytes), alarm high/low (2 bytes), config (1 byte), reserved (3 bytes), CRC (1 byte).
- `0x4E WRITE SCRATCHPAD` — set alarm thresholds and conversion resolution.
- `0x48 COPY SCRATCHPAD` — persist the scratchpad config to the device's EEPROM.

## ROM ID structure

Every 1-Wire device has a unique factory-burned 64-bit serial number:

```
| Family (8 bits) | Serial (48 bits) | CRC (8 bits) |
```

The family code is the lowest byte. `0x28` is DS18B20 specifically. The `ds18b20` crate exposes it as `ds18b20::FAMILY_CODE`. On our bus, the first device address we saw was `8A00001125DC9D28` — notice the `28` at the end confirming it's a DS18B20.

## Useful references

- [DS18B20 datasheet](../hardware/datasheets/dfr0198-ds18b20-datasheet.pdf) — full protocol timing diagrams, command set, scratchpad layout.
- [Maxim App Note 126](https://www.maximintegrated.com/en/design/technical-documents/app-notes/1/126.html) — the canonical 1-Wire reference.
- Crate source: [`one-wire-bus` on GitHub](https://github.com/fuchsnj/one-wire-bus) — pin bit-banging and ROM search algorithm.
