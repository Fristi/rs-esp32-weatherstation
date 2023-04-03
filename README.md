rs-esp32-ssd1306
---

Simple program to display text on the screen using rs-esp

![esp32-ssd1306](result.jpeg)

### Ingredients

- LilyGO TTGO T-Beam - LoRa 868MHz - NEO-6M GPS - ESP32
- 0.91 inch OLED Display 128*32 pixels blue - I2C

### Getting started

1. Get the drivers for USB to serial connectors https://www.tinytronics.nl/shop/en/drivers-for-usb-to-serial-converters
2. Install espflash `cargo install cargoflash`
3. Install espup `cargo install espup` and after that get the a rustc versian for esp32 `espup install`
4. Make sure to run `. /Users/{home_dir}/export-esp.sh` when you open a new terminal
5. Use `cargo espflash /dev/cu.usbserial-{id}` to flash the device
6. Use `cargo espflash /dev/cu.usbserial-{id} serial-monitor` to see the serial output
7. To use i2c and such there are great resources at https://github.com/esp-rs