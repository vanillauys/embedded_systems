# Components

Physical parts on the desk for this project. For detailed datasheets and schematics, see [datasheets/](datasheets/) (local copies + `pdftotext` exports for grep).

## Power / prototyping

- **Resistor 4.7 kΩ** (50-pack) — used as external pull-up for DS18B20 1-Wire data line when the bare probe is wired directly (no adapter).
- **Half-size breadboard**, 400 tie points.
- **Jumper wires** — M-M and M-F packs.
- **Samsung INR18650-35E** 3400 mAh in single-cell holder — paired with MH-CD42 for battery stage.
- **MH-CD42** charge/boost module — 3.7 V → 5 V boost, USB-C charging, 4-LED indicator, battery protection.

## MCU board

**ESP32-S3-N16R8 dev board** — 16 MB flash, 8 MB PSRAM, dual USB-C. Built around the ESP32-S3-WROOM-1 module (Xtensa dual-core LX7).

Annotated photos: [board layout](ESP32-S3-N16-R8-015-BoardLayout.jpg) · [full pinout](ESP32-S3-N16-R8-016-Pinout.jpg)

Onboard LEDs: red 3.3V power (always on, not controllable), **WS2812B RGB LED on GPIO 48** (needs "RGB" solder jumper bridged to be driven), UART TX LED on GPIO 43, UART RX LED on GPIO 44. See [pinout.md](pinout.md) for details.

- Module datasheet: [datasheets/esp32-s3-wroom-1-datasheet.pdf](datasheets/esp32-s3-wroom-1-datasheet.pdf) · [Espressif original](https://documentation.espressif.com/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf)
- Dev board user guide: [datasheets/esp32-s3-n16r8-user-guide.pdf](datasheets/esp32-s3-n16r8-user-guide.pdf) · [microrobotics original](https://github.com/microrobotics/ESP32-S3-N16R8/blob/main/ESP32-S3-N16R8_User_Guide.pdf)
- Dev board schematic: [datasheets/esp32-s3-yd-schematic.pdf](datasheets/esp32-s3-yd-schematic.pdf) · [99tech original](https://99tech.com.au/mx-m/esp32/esp32-s3-yd_schematics.pdf)

## Temperature sensor

**DS18B20** — 1-Wire digital thermometer, 9–12 bit, ±0.5 °C, -55 to +125 °C. Waterproof stainless probe on 1 m cable. Red=VCC, yellow=DATA, black=GND.

Two physical flavours of the same sensor in this kit:

- **Bare probe** — wire directly to ESP32 with an external 4.7 kΩ pull-up between DATA and 3.3 V. Used in step 4.
- **DFRobot DFR0198 kit** — probe + "Plugable Terminal V2" adapter board (pull-up already on-board, screw terminals, 3-pin JST). Used in step 3.

- Chip datasheet: [datasheets/dfr0198-ds18b20-datasheet.pdf](datasheets/dfr0198-ds18b20-datasheet.pdf) · [DFRobot mirror](https://dfimg.dfrobot.com/wiki/17518/DFR0198_ds8b20-waterproof-temperature-sensor_datasheet_1.0.pdf)
- DFRobot wiki: [datasheets/dfrobot-ds18b20-wiki.html](datasheets/dfrobot-ds18b20-wiki.html) · [online](https://wiki.dfrobot.com/dfr0198)

## Enclosure (save for later)

- **Gainta G212C** IP65 enclosure — post-prototype packaging.
