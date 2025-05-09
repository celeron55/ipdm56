#!/bin/sh
set -euv
dir="$( cd "$( dirname "$0" )" && pwd )"
cd "$dir"

./flash_uart_bin.sh ../target/thumbv7em-none-eabihf/release/embedded.bin
