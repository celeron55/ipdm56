iPDM56 v1.0
===========

iPDM56 v1.0 fixing the board
----------------------------

NOTE: See photos in doc/flashing/

There are 2 bugs on the v1.0 board. This is what you need to do to get it working:
- Cut the trace next to U21 pin 10 (the one nearest to the edge of the board)
- Solder a 1k resistor across the cut. 0805 or smaller should fit fine.
- Solder a 0805 1k resistor in parallel on top of resistor R10 (next to U2).


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

These will be hardwired in v1.1 and ipdm software will depend on them, so connecting them like so in v1.0 is highly recommended:

- Connect Vbat to A7. This is easiest done by connecting Vbat to IN6. This is
  critical: The library uses this to determine whether 5Vsw is available for
  initializing and running the CAN controllers.

- Connect USB 5V to ED3 by adding a jumper wire underneath the PCB and
  disconnect ED3 from its v1.0 default of M4 (remove resistor R65). This is not
  critical at the moment, the library does not use this for anything for now.

These will be the default connections in v1.1, so connecting them like so in v1.0 is recommended:

- Disconnect ED0...ED2 from M1...M3 by removing resistors R62, R63 and R64.

More pin mapping conventions worth following:

- M1...M4 are generally intended to be wired to Vbat, 5Vsw or 5Vp, depending on the application's requirements. This way they are used to provide power to switches and sensors.

- Connect ED0...ED2 to monitor outputs HOUT1...HOUT3, if monitoring is needed.
 
