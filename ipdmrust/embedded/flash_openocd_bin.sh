#!/bin/bash

# OpenOCD configurations
INTERFACE_CFG="interface/stlink.cfg"
TARGET_CFG="target/stm32f4x.cfg"
OPENOCD_BIN_PATH="/usr/bin/openocd"  # Adjust if OpenOCD is installed in a different location

# Firmware path
FIRMWARE_PATH="$1"
if [ -z "$FIRMWARE_PATH" ]; then
    echo "Usage: $0 <firmware_path>"
    exit 1
fi

# Flashing process
$OPENOCD_BIN_PATH -f $INTERFACE_CFG -f $TARGET_CFG \
-c "init" \
-c "reset halt" \
-c "flash write_image erase $FIRMWARE_PATH 0x08000000 bin" \
-c "reset run" \
-c "exit"
