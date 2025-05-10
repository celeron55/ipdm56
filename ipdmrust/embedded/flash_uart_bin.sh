#!/bin/bash

# Firmware path
FIRMWARE_PATH="$1"
if [ -z "$FIRMWARE_PATH" ]; then
    echo "Usage: $0 <firmware_path> [serial port (/dev/ttyUSB0)]"
    exit 1
fi

if [ $# -eq 2 ]; then
	SERIAL_PORT="$2"
else
	SERIAL_PORT=/dev/ttyUSB0
fi

# Use the "dfu" serial console command to put the device into DFU mode
stty -F "$SERIAL_PORT" 115200 cs8 -cstopb -parenb -echo raw || echo "stty failed"
echo -e "\ndfu" > "$SERIAL_PORT" || echo "echo failed"
sleep 2

stm32flash -R -w "$FIRMWARE_PATH" -b 115200 "$SERIAL_PORT"
