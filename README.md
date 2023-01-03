iPDM56
======

The hardware
------------
Please check the README.md of your chosen hardware version in its subdirectory.

The software
------------
A GPL 3 licensed utility library and example code is available in the `ipdmsw`
subdirectory.

The idea
--------
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

You can read a longer discussion in [Let's talk about the design of the iPDM56.md](Let's%20talk%20about%20the%20design%20of%20the%20iPDM56.md)

- celeron55 @ 8Dromeda Productions

