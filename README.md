Embedded Rust Raspberrypi Pico (RP2040) OLED Example
====================================================

This project was based off of the [Project template for rp2040-hal](https://github.com/rp-rs/rp2040-project-template). Check their README for setup.

The logic to drive the display via SPI has been ported from the [TIVA-C example](https://github.com/HerrMuellerluedenscheid/tm4c-oled-example). The OLED display is the exact same SPI driven SSD1309.

.. image:: ./rp2050-oled-small.jpg
  :width: 600
  :alt: OLED SPI example

.. list-table:: Wiring
   :widths: 50 50
   :header-rows: 1

   * - OLED pin
     - RP2040 pin
   * - CS
     - GND
   * - DC
     - GP5
   * - RES
     - GP6
   * - SDA
     - GP3
   * - SCL
     - GP2
   * - VCC
     - 3.3V
   * - GND
     - GND

## Run the example

I used [pyOCD](https://pyocd.io/docs/installing):

    pyocd gdbserver -t rp2040_core0

You may want to change the `runner` from `arm-none-eabi-gdb` to e.g. `gdb-multiarch` dependent on the platform and setup you are using.

Then:

    cargo run
