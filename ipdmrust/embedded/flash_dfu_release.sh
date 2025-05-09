#!/bin/sh
set -euv
dir="$( cd "$( dirname "$0" )" && pwd )"
cd "$dir"

# Check if a parameter (serial port) is provided. If it is, then use the "dfu"
# serial console command to put the device into DFU mode
if [ $# -eq 1 ]; then
	SERIAL_PORT="$1"
	stty -F "$SERIAL_PORT" 115200 cs8 -cstopb -parenb -echo raw || echo "stty failed"
	echo -e "\ndfu" > "$SERIAL_PORT" || echo "echo failed"
	sleep 2
fi

./flash_dfu_bin.sh ../target/thumbv7em-none-eabihf/release/embedded.bin
