iPDM56 v1.0
===========

iPDM56 v1.0 fixing the board
----------------------------

NOTE: See photos in doc/flashing/

There are 2 bugs on the v1.0 board. This is what you need to do to get it working:
- Cut the trace next to U21 pin 10 (the one nearest to the edge of the board)
- Solder a 1k resistor across the cut. 0805 or smaller should fit fine.
- Solder a 0805 1k resistor in parallel on top of resistor R10 (next to U2).

Additionally the support library requires this change for its proper function:
- Connect Vbat to A7
    * This is easiest done by connecting Vbat to IN6 using a wire jumper or a 0...10 Ohm resistor.
    * The library uses this to determine whether 5Vsw is available and it will not initialize the CAN controllers until it sees voltage at the A7 pin. This check exists for the case where the MCU may be running while 5Vsw not being available when the board is being powered via USB without 12V being applied to Vbat.
    * With this change, IN6 is obviously unavailable for its normal A7 input function.


iPDM56 v1.0 flashing
--------------------

Bootloader:
- Install Arduino.
- Select Arduino Uno as the board in Arduino.
- Connect an ISP programmer like USBasp to the ISP header. GND is closer to board edge.
- Select the ISP prorammer in Arduino's Tools menu.
- Use Arduino's "Burn bootloader" option in the Tools menu.

Uploading program:
- Select Arduino Uno as the board in Arduino.
- Cut your most hated USB cable in half and connect it to the 56-pin connector pins (according to the iPDM56 connector pinout):
	- 10: USB 5V  (red)
	- 12: USB D-  (white)
	- 14: USB D+  (green)
	- 16: USB GND (black)
	* See /doc/flashing/iPDM56_v1.0_flashing_060.jpg
	  (altough of course you probably want to crimp it to the actual connector)


iPDM56 v1.0 application design notes
------------------------------------

- M1...M4 are generally intended to be wired to Vbat, 5Vsw or 5Vp, depending on the application's requirements. This way they are used to provide power to switches and sensors.

- Connect ED0...ED2 to monitor outputs HOUT1...HOUT3, if monitoring of outputs is needed.
 
