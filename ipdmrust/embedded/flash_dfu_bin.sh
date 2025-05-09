#!/bin/bash

# Firmware path
FIRMWARE_PATH="$1"
if [ -z "$FIRMWARE_PATH" ]; then
    echo "Usage: $0 <firmware_path>"
    exit 1
fi

dfu-util -a 0 --dfuse-address 0x08000000:leave -D "$FIRMWARE_PATH"
