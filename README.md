iPDM56
======

News
----
2023-01-16:
Hardware version 1.1 design was finished and boards were ordered.

Check out the documentation under ipdmhw1.1/.

2023-01-11:
Hardware version 1.0 is being tested by me (in two vehicles) and by a so-far
anonymous party (in something he comes up with).

In the next few weeks I expect to design version 1.1 with bugs fixed, a few
more outputs crammed in and the design tweaked somewhat. Those will be
documented in this repo, and put on sale on my Etsy shop at:
https://www.etsy.com/fi-en/shop/EVresources/edit?ref=seller-platform-mcnav

2022-12-09:
Hardware version 1.0 was designed and an initial batch of 5 was made.


The hardware
------------
Please check the README.md of your chosen hardware version in its subdirectory.

The software
------------
A GPL 3 licensed utility library and example code is available in the `ipdmsw`
subdirectory.

The product
-----------
The iPDM56 is an intelligent power distribution module. It's basically an
atmega32-based arduino with 6x low-side and 6x high-side 3A PPTC protected
MOSFET outputs. The inputs are protected by resistors and TVSes and filtered by
capacitors. They are configurable using a soldering iron (to make resistor
dividers, pull-ups, filters, and to repurpose them as logic level outputs).

An I/O expander IC is used to get all the outputs out of the small MCU. It has
2x MCP2515 CAN controllers to either work with 2 buses or alternatively they can
be connected to the same bus so that you have more filters and inboxes.

The MCU and I/O expander are intended to be permanently powered on the "5Vp"
rail and eg. the MCP2515s are gated behind a software controlled power switch,
on the "5Vsw" rail.

The configuration is done as an Arduino sketch via USB exposed via the external
56-pin connector. I will write a library for it to make it easy and consistent.
No, you can't make me change my mind on this - this is the exact way I prefer
doing things. But of course one could write a general program for it where you
can configure it to do some common things without writing and uploading a new
sketch.

It's intended to act as the "I want these parts of the system to power up based
on this and this arbitrary logic without the ignition key" system. 

It's intended to act as programmable logic and output driver for pumps, fans,
brake vacuum boosters and whatever.

It' intended to act as the fuse box and relay box for your inverter, charger and
other module's 12V power inputs.

And if you have a simple enough system where you don't need a more complex VCU
than what this is, then it can act as a VCU too. Just configure some of the
inputs as a throttle and whatever and then fit all the software you need into
the small program memory.

Have fun.

- celeron55 @ 8Dromeda Productions

The iPDM56 essay
================

Yes, it's just a circuit board with inputs and outputs and CANbus, which fits into an enclosure.

No, it's not like ANY of the other circuit boards with inputs and outputs and CANbus that fit into an enclosure.

How do you navigate insanity?
-----------------------------

There are many possible designs for everything. Most of the time you don't choose the design, you just end up with one. And most of the time you cannot afford or cannot be arsed to understand the design before you already did yours. And now you need to integrate.

Before you know it, the design will have become insane. It will have no reseblance to your original plans, and you have to re-think all of it. And it will be insane in a completely different way than everyone else's design. Nobody can help you. You will have to live with that design.

The iPDM56 is the "I need to combine CAN and I/O" solution, which you just Slap-On-And-Design-Later. But that's not because you're lazy. It gives you freedom to be lazy, and that's very healthy, but the thing is, even if you try to design everything right from the ground up, your brain capacity and I/O speed just isn't enough to take into account everything. The way the vehicle has to work changes over time due to changing needs, and changing realizations about needs that were always there but your monkey brain wasn't up to it.

The design
----------

I think there's a lot of value in me digging into this and spending the sleepless nights with the super annoying design choices. And I have the experience of having an EV conversion as a daily driver for years (and the experience of starting a second and a third conversion project). I know what has and what hasn't worked, and I know what I have had to do all over again too many times. Almost all aspects of this are a culmination of that. Both the overall idea and all the details.

It's a balance between features and cost. Bang for the buck is important.

The way I think about the cost of the design is, if there's room on the board and adding a certain set of components would probably save on average much more time when working on a project than what the components cost, then the components have to be there. Otherwise, they have to be deleted.

The specific enclosure has also been chosen in this way. It's super useful for the price. Once you have it in your hands you question how you managed to live without one.

Of course the entire project has been also chosen in this way too. I realized you can easily shorten the time needed to make an EV conversion or some other similar-ish project by replacing the time spent arranging and re-arranging fuses, relays and outputs, and cursing the lack of outputs and inputs, with just adding a stupid box and then thinking in software. You can shorten it probably by days, and every year you save more days as you don't need to rip apart the system to make some logic change due to whatever you happened to add or realize.

But I also realized it's not THAT simple. You can't do everything in software. That's why it has those terminals on-board so that it can be modified for specific i/o requirements. In DIY, you always end up with project specific I/O requirements.

But what terminals? What modifications to make possible? How the modifications are supposed to be made? For what purpose? What are the purposes that will not be supported?

What are the default connections? What kind of resistor dividers and input protection?

How many outputs? How many high side and low side outputs? What current? What kind of protection? How many power input pins? How much to oversize the transistors just to make sure? Trace widths? Any analog outputs? What kind? What kind of PWM to support?

As you realize, the design of the iPDM56 itself is insanity. But it has been designed to solve YOUR insanity, at the cost of MY sanity.

The schematics and board layout are published as part of documentation, which make it easy to make modifications according to your project's needs. Absolutely no reverse engineering is required.

Maintaining your project vehicle
--------------------------------

Maintaining your project vehicle is easy during the first weeks of taking it into daily use, as you have all the tools and software lying around.

But what about after a few months? And then after a few years? How likely is it that you still have the same programming dongle, wifi network or laptop after 10 years of ownership? How likely is it that you can get a replacement MCU if the one on the board died?

The answers to these questions lead me to use an ATMEGA328 as the MCU, and set up programming via USB as if it was an Arduino Uno. This makes it as likely as possible that you can program the chip as long as you still have the code around (remember to back it up - how about on a USB flash drive in the glove box?), and if the chip dies, you can at 100% certainty get a replacement one, as the ATMEGA328 is absolutely ubiquitous. Chances are it will still be in production, but if not, you will stumble upon one without even searching for it.

Besides, the fact that the iPDM56 is relatively cheap and multipurpose, you could just have a small stock of them. Maybe even in that glove box, just in case. Regardless of where it is, if you end up starting another project, you have a board ready right away, saving ridiculous amounts of calendar time.

Did I talk about a dead MCU? How likely is it to die?

The iPDM56 has, by default, good protection for the MCU against incoming harm through the external power, input and output pins. Unless you modify it yourself in a way that skips the protection, the MCU will be very long-lasting.

How about the lifespan of the connector?

The built-in PPTC resettable fuses in the iPDM56 are rated roughly at what the connector can handle, so it will certainly not burn up due to overcurrent. The connector also is waterproof, and has a robust locking mechanism. I would say it is unlikely to be an issue in the long term.

Software/firmware
-----------------

When you store software as source code, it has to be compiled before use. You could store compiled binaries, but then you couldn't modify them, and you are very likely to want to modify the behavior during the vehicle's lifetime. That's why you are using the iPDM56, to let you modify the operating logic without touching the wiring or replacing components.

The software stack has to be such that you can get the tools for your 4rd laptop into the future, and they just work. That's why the iPDM56 is set up exactly like a Chinese Arduino Uno clone, when it comes to programming. All it has for extra is the CAN controllers and an I/O expander chip.

The software that is provided by me for use with the iPDM56 comes with all the necessary libraries in the src subdirectory, so that your project can be built on any fresh Arduino install, without the need to set up anything at all.

If you don't like my software, you can simply make your own Arduino project and include your preferred support libraries for the peripheral chips. Those are widely available, you will not run out of options.

The schematics and board layout are published as part of documentation, which make it easy to set up the I/O mapping in your own software. Absolutely no reverse engineering is required.

Furthermore, if you're a hardcore embedded programmer and don't like Arduino tools, you can set up whatever you like, with a bootloader or not, by using the on-board ISP header. What you have is simply an AVR with a couple of peripheral chips.

Your application design
-----------------------

The iPDM56 is designed in such a way that when used correctly, the MCU can run your run-of-the-mill Arduino program at full speed with basically only the current consumption of the AVR. This is done by giving the software control over the 5V line of the CAN controllers, which consume quite a lot of power. Generally speaking, once you detect the system needs to be turned on based on any of your custom requirements, you can turn on the 5Vsw (switched 5V) line, which powers the CAN controllers, and you probably have an output powering all or part of your system, which you turn on. Then you get CAN traffic and can handle the situation further.

A low-power sleep function is provided which turns everything off and puts the AVR into a low power mode, for the specified duration. You may want to do this for example when the 12V battery is running out.

If you need to power up parts of the system and the CANbus in order to e.g. detect a charging cable via an On-Board Charger module, you can for example use a timeout from turning the ignition key to the off position during which you still keep the CANbus and OBC alive, let's say for 24 hours, and after that shut down the CANbus and OBC in order to conserve power. This allows ease of use while not running your 12V battery down. And it's all software defined. If you don't like it, just edit the code.

Example uses
------------

- Powering devices based on sensor inputs and other inputs
	- e.g. Control a brake vacuum pump based on a pressure sensor, but disabling it if the gear shift is in P. Remember to use a relay for the pump though - they consume more amps than the iPDM56 can provide. The iPDM56 can easily provide 5V to the sensor if you jumper the 5Vsw line to one of the multi-purpose outputs. You can get or build a separate module for this, but why bother. It's just a few lines of code, one input and one output.
	- e.g. Powering up the HV system both based on the ignition key, but also based on the charging port status
	- e.g. Powering up the system and starting a pre-heating or pre-cooling cycle based on an otherwise unused remote central locking signal

- Powering devices based on data available on CAN
	- e.g. Running a fan and pump based on BMS temperature data
	- e.g. Generate an HV OK signal to a DC-DC converter based on BMS CAN data

- Controlling devices on CAN
	- e.g. Sending CAN messages to a water heater based on its temperature feedback and BMS precharge status

- Providing switched 12V power to various modules like chargers and heaters based on any combination of inputs, timers and logic you can imagine
	- e.g. Powering the cabin heater pump when the CAN controlled cabin heater is active
	- e.g. Timing out the switched 12V to a charger after 24 hours from turning the ignition off, to conserve the 12V battery but allow EVSE detection
	- e.g. Powering the BMS based on the ignition key, inverter status and charger status

- Publishing readings of sensors on CAN
	- e.g. Make a very cheap CAN temperature sensor by just using an NTC and converting publishing the readings

- Producing PWM to control something based on sensor or CAN input
	- e.g. A PWM controlled PTC heater

- Controlling things based on custom input signals
	- e.g. Modify an input to high impedance and monitor an EVSE PP, CP or a Chademo line in order to power up the BMS and charging devices when needed

- Providing extra inputs and outputs for various things you just have to control in order to make a good or legal experience
	- e.g. Drive a PRND gear indicator
	- e.g. Sound an alarm if the car is in gear when the driver's door is opened
	- e.g. Animate the PRND indicator or any other outputs you happen to have when the car is charging, to indicate the car is charging without adding any extra outputs
	- e.g. Read the various cruise control buttons and provide the info on CAN, because your VCU doesn't have the necessary I/O
	- e.g. Drive a fuel gauge using an analog output

- Translating CAN messages
	- e.g. Translate incompatible traction control messages between an ABS unit and a VCU


