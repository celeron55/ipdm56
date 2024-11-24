iPDM56 v2.0
===========

iPDM56 v2.0 fixing the board
----------------------------

1. U6 (the 3.3V regulator) needs to be rotated so that (when looking at it so
   that there's one top pin and two bottom pins) the top pin moves in place of
   the right pin, and the right pin moves in place of the top pin. The remaining
   pin that was originally on the left has to be soldered to GND, which is
   handily found on the nearby capacitor and can be reached by adding a solder
   blob.

2. If you intend to use RS232 to flash the STM32:
    - You have three ALTERNATIVE fixes:
        1. Parallel R3001 (originally a 100k resistor) with a 1.0k resistor
            - Downside: This increases the standby drain of the board by 3 mA on
              the 3.3V line, which corresponds to about 1.5 mA on the 12V supply
        2. Desolder RA1H5V1
            - Downside: The 5V external output pin won't provide 5V
        3. Replace RA1H5V1 with a 330k...470k 0603x4 resistor array
            - Downside: You need to acquire the replacement resistor array
            - Downside: I have not tested this fix in reality (yet)
    - R3001 is a pull-up resistor for the `PERIPHERALS_ENABLE` line, and 100k is
      too high due to bad resistance choices on the Q2H5V1 NPN transistor base
      and won't let the voltage on the `PERIPHERALS_ENABLE` line rise high
      enough to enable the U9 RS232 transceiver. Once the STM32 is up and
      running, it will keep the line high and RS232 will work regardless of this
      fix.

iPDM56 v2.0 flashing
--------------------

The board can be flashed and debugged by using SWD, which is available on the
EXT2 connector.

The board is designed to be flashed by using the ROM bootloader. You can read
about the behavior of the STM32 ROM bootloader e.g. in the STM32 application
note AN2606.

The ROM bootloder is reached by pulling the `EXT_RESET_N` line to ground for
about 3 seconds, until LED6 lights up, and then released. This resets the STM32
into the ROM bootloader.

The ROM bootloader is exited by pulling the `EXT_RESET_N` line to ground for
a short time (less than 1 second). This resets the STM32 into running the
program from flash.

The ROM bootloader supports these alternative flashing options:
- USB
    - Supported by e.g. dfu-util
- RS232
    - Supported by e.g. stm32flasher
- CAN
    - To enable CAN flashing, bridge JP3001. This enables CAN flashing via CAN2.

