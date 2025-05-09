#!/bin/sh
set -euv
dir="$( cd "$( dirname "$0" )" && pwd )"
cd "$dir"

./flash_openocd_bin.sh ../target/thumbv7em-none-eabihf/debug/embedded.bin
