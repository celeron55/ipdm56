#!/bin/bash

FIRMWARE_PATH="$1"
SERIAL_PORT="${2:-/dev/ttyUSB0}"

# Cooked-ish mode: no echo, no flow control
stty -F "$SERIAL_PORT" 115200 cs8 -cstopb -parenb -ixon -ixoff -echo

# Open port for read/write, keep open
exec 3<> "$SERIAL_PORT"

# Optional: flush
printf "\r" >&3
sleep 0.1

# Send "dfu" one character at a time
# Yeah RX reading on the firmware effed as the single-byte interrupt can't keep
# up. Need to set up DMA there.
for c in d f u; do
    printf "%s" "$c" >&3
    sleep 0.05  # 50ms between characters
done

# Send carriage return
printf "\r" >&3
sleep 0.2

# Wait for the STM32 ROM bootloader start
sleep 2

# Flash firmware
stm32flash -R -w "$FIRMWARE_PATH" -b 115200 "$SERIAL_PORT"
