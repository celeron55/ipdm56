ipdmrust
========

Example firmware and support crates for iPDM56 v2, written in Rust.


Preparation before doing anything else
--------------------------------------
Symlink your application crate:
$ ln -s /wherever/app app

Or if you don't have a specific application, symlink the example app:
$ ln -s app_example app

Performance benchmarking
------------------------
Heap profiling
- Mainly just to see that Piston is behaving. Heap allocations aren't used on
  the embedded target.
$ cd desktop
$ cargo build
$ valgrind --tool=massif ../target/debug/desktop
$ ms_print massif.out.<pid> | less

Stack profiling:
- This gives an idea about how much memory will be used on the embedded target
  also.
$ cd desktop
$ cargo build
$ valgrind --tool=massif --heap=no --stacks=yes ../target/debug/desktop
$ ms_print massif.out.<pid> | less

Compiling for physical hardware
-------------------------------
On fedora (something similar on others, to install `cc` and `arm-none-eabi-objcopy`):
$ sudo dnf install gcc arm-none-eabi-binutils-cs

$ rustup update
$ rustup component add llvm-tools-preview
$ rustup target add thumbv7em-none-eabihf

$ cd embedded
$ ./build_release.sh

Flashing physical hardware
--------------------------
USB firmware update using automatic DFU mode:
- Connect USB
$ ./flash_dfu_release.sh /dev/ttyACM0

USB firmware update using manual DFU mode:
- Connect USB
- Hold EXT_RESET_N for 3 seconds (this activates reset with BOOT0 high)
- The board should appear as "Product: STM32 BOOTLOADER"
$ ./flash_dfu_release.sh
- OR
$ dfu-util -a 0 --dfuse-address 0x08000000  -D ../target/thumbv7em-none-eabihf/release/embedded.bin

Installing stm32flash for UART flashing:
- Get it from https://sourceforge.net/p/stm32flash/ (stm32flash-0.7.tar.gz)
$ ./configure
$ make -j2
$ ln -s $(pwd)/stm32flash ~/bin/stm32flash
- Make sure ~/bin is in your $PATH

The raw command for UART firmware update uses stm32flash:
$ stm32flash -R -w ../target/thumbv7em-none-eabihf/release/embedded.bin -b 115200 /dev/ttyUSB0

Using an ST-Link v2 clone or similar:
- Connect GND, SWDIO, SWCLK and 3.3V
$ ./flash_openocd_release.sh

- This also kind of works, but RTT logging is not set up so it will complain after flashing succeeds:
$ cargo run --release

Flashing physical hardware via UI8D UART bridge
-----------------------------------------------

Ideally you just run this:
$ cd embedded
$ ./build_release.sh
$ ./flash_uart_release.sh /dev/ttyUSB0

But actually for some reason that doesn't work very well and you need to do it manually:
$ cd embedded
$ ./build_release.sh
$ picocom --baud 115200 -r -l -c -e x /dev/ttyUSB0
# Enter the "dfu" command in picocom and exit (CTRL+X CTRL+Q)
$ stm32flash -R -w ../target/thumbv7em-none-eabihf/release/embedded.bin -b 115200 /dev/ttyUSB0

Monitoring using USB serial
---------------------------
$ picocom --baud 115200 -r -l -c -e x /dev/ttyACM0

Debugging on physical hardware
------------------------------
$ cd embedded
$ ./build_release.sh
$ ./flash_openocd_release.sh
$ openocd -f interface/stlink.cfg -f target/stm32f4x.cfg

In another terminal:
$ cd embedded
$ arm-none-eabi-gdb ../target/thumbv7em-none-eabihf/release/embedded
(gdb) target extended-remote localhost:3333
(gdb) break Reset

Editing
-------
- Use any editor (vim is of course recommended 8-))
- Install bacon and run it in another terminal
$ cargo install --locked bacon
$ bacon

Logging
-------
$ picocom --baud 115200 -r -l -c -e x /dev/ttyACM0 | ts %H:%M:%.S
$ picocom --baud 115200 -r -l -c -e x /dev/ttyACM0 | tee $(date +%Y-%m-%d_%H%M%S).log
$ picocom --baud 115200 -r -l -c -e x /dev/ttyUSB0 | ts %H:%M:%.S
$ picocom --baud 115200 -r -l -c -e x /dev/ttyUSB0 | tee $(date +%Y-%m-%d_%H%M%S).log
