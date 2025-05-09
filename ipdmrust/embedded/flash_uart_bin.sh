#!/bin/bash

# Firmware path
FIRMWARE_PATH="$1"
if [ -z "$FIRMWARE_PATH" ]; then
    echo "Usage: $0 <firmware_path>"
    exit 1
fi

stm32flash -w "$FIRMWARE_PATH" -b 115200 /dev/ttyUSB0
