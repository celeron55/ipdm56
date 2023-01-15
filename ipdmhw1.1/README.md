iPDM56 v1.1
===========

iPDM56 v1.1 flashing
--------------------

Bootloader:
- Install Arduino.
- Select Arduino Uno as the board in Arduino.
- Connect an ISP programmer like USBasp to the ISP header. GND is closer to board edge.
- Select the ISP prorammer in Arduino's Tools menu.
- Use Arduino's "Burn bootloader" option in the Tools menu.

Uploading program:
- Select Arduino Uno as the board in Arduino.
- Plug in a USB 2.0 micro B cable to the USB connector, OR
- Cut your most hated USB cable in half and connect it to the 56-pin connector pins (according to the iPDM56 connector pinout):
	- 10: USB 5V  (red)
	- 12: USB D-  (white)
	- 14: USB D+  (green)
	- 16: USB GND (black)

