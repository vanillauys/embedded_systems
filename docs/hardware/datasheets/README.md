# Datasheets & Schematics

Local copies of vendor documentation for the components in this project. Each PDF has a sibling `.txt` produced by `pdftotext -layout` for grep/search.

> Re-generate text: `pdftotext -layout FILE.pdf FILE.txt`

## ESP32-S3-WROOM-1 module (the chip module itself)

- [esp32-s3-wroom-1-datasheet.pdf](esp32-s3-wroom-1-datasheet.pdf) · [.txt](esp32-s3-wroom-1-datasheet.txt)
- Espressif official datasheet v1.8 — Xtensa dual-core LX7, Wi-Fi 2.4 GHz + BLE 5, up to 36 GPIOs, 16 MB flash / 16 MB PSRAM variants.
- Use for: electrical characteristics, pin descriptions, strapping pins, power sequencing, module dimensions.
- Source: <https://documentation.espressif.com/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf>

## ESP32-S3-N16R8 dev board (the physical board on the desk)

- [esp32-s3-n16r8-user-guide.pdf](esp32-s3-n16r8-user-guide.pdf) · [.txt](esp32-s3-n16r8-user-guide.txt)
- Vendor user guide for the specific dev board (Microrobotics / generic YD design). Two USB-C ports: **USB UART** (CH343P) and **native USB** on the ESP32-S3. Explains which port does what.
- Use for: board layout, USB port behavior, boot/reset buttons, onboard jumpers.
- Source: <https://github.com/microrobotics/ESP32-S3-N16R8/blob/main/ESP32-S3-N16R8_User_Guide.pdf>

## ESP32-S3-YD dev board schematic

- [esp32-s3-yd-schematic.pdf](esp32-s3-yd-schematic.pdf) · [.txt](esp32-s3-yd-schematic.txt)
- Full electrical schematic for this exact dev board family.
- Use for: tracing what's actually connected to each GPIO, voltage regulator topology, USB-C wiring, LED / button connections.
- Note: the `.txt` export is useless (pure visual schematic). Open the PDF when you need this.
- Source: <https://99tech.com.au/mx-m/esp32/esp32-s3-yd_schematics.pdf>

## DS18B20 temperature sensor (Maxim/Analog Devices chip)

- [dfr0198-ds18b20-datasheet.pdf](dfr0198-ds18b20-datasheet.pdf) · [.txt](dfr0198-ds18b20-datasheet.txt)
- Chip datasheet shipped with the DFRobot DFR0198 kit. 1-Wire digital thermometer, 9–12 bit resolution, ±0.5 °C accuracy, -55 to +125 °C range.
- Use for: 1-Wire protocol, ROM commands (0x33 READ ROM, 0x55 MATCH ROM, 0xCC SKIP ROM), function commands (0x44 CONVERT T, 0xBE READ SCRATCHPAD), timing slots, parasite power mode, pull-up sizing.
- Source: <https://dfimg.dfrobot.com/wiki/17518/DFR0198_ds8b20-waterproof-temperature-sensor_datasheet_1.0.pdf>

## DFRobot DFR0198 wiki page

- [dfrobot-ds18b20-wiki.html](dfrobot-ds18b20-wiki.html) — raw HTML, no pandoc/html2text available to convert.
- Same tech content as the PDF above but with the terminal-block adapter wiring diagrams specific to DFR0198.
- Source: <https://wiki.dfrobot.com/dfr0198>
- Open in a browser when needed; otherwise prefer the PDF.
