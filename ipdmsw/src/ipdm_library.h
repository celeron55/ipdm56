/*
ipdm - iPDM56 firmware
Copyright (c) 2022 Perttu "celeron55" Ahola

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#pragma once
#include "can_common.h"
#include "mcp_can.h"
#include "ipdm_io.h"
#include "ipdm_util.h"

namespace ipdm
{

// This should be called as first thing in the application setup()
void setup();

// This housekeeping function should be called in the application loop()
void loop();

// This allows switching off the switched 5V rail (5Vsw), for power saving
// purposes.
// This switches off the CAN controllers and PHYs also
void enable_switched_5v();
void disable_switched_5v();
// This allows monitoring the 5Vsw line. It may be OFF even if it's been
// enabled, due to missing Vbat.
bool status_switched_5v();

// Extra powersave options

// This allows dividing the 16MHz CPU clock with a divider of:
// 0=1(16MHz), 1=2, 2=4, 3=8, 4=16(1MHz), 5=32, 6=64, 7=128, 8=256(62.5kHz)
// This lowers standby power consumption (with 5Vsw off):
// prescaler_bit=0 ->   16 MHz -> 18mA (default)
// prescaler_bit=1 ->    8 MHz -> 12mA
// prescaler_bit=2 ->    4 MHz -> 10mA
// prescaler_bit=4 ->    1 MHz ->  7mA
// prescaler_bit=8 -> 62.5 kHz ->  6mA
// NOTE: When using this, all delay()s will multiply and the serial baud rate
// will divide up accordingly.
// NOTE: ipdm::delay() takes this prescaler into account
void set_clock_prescaler(uint8_t prescaler_bit);

// Returns the actual divider value in use (not the prescaler bit)
uint32_t get_active_clock_divider();

// This delay is able to compensate for the clock divider, unlike Arduino's
// delay().
void delay(unsigned long ms);

// This function does not return until the time has passed
// NOTE: Outputs will be left in their current state and may consume power
// NOTE: Returns with clock prescaler set to 0 = 16MHz
void power_save_delay(unsigned long duration_ms);

// These are called automatically by setup() and loop() respectively, but if you
// need to, you can call them separately.
void enable_watchdog();
void reset_watchdog();

} // namespace ipdm
