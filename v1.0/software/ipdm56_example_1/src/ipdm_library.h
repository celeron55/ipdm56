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

namespace ipdm
{

// This should be called as first thing in the application setup()
void setup();

// This housekeeping function should be called in the application loop()
void loop();

// This allows switching off the switched 5V rail, for power saving purposes.
// This switches off the CAN controllers and PHYs also
void enable_switched_5v();
void disable_switched_5v();

void pinMode(int pin, uint8_t mode);
void digitalWrite(int pin, bool state);
bool digitalRead(int pin);
uint16_t analogRead(int pin);

// These are the CAN interfaces
extern MCP_CAN can1;
extern MCP_CAN can2;

// You want to set CAN parameters using these, so that every time you disable
// and enable the switched 5V rail, control_switched_5v() can re-initialize the
// CAN controllers automatically
struct CanParameters
{
	uint8_t speed = CAN_500KBPS;
	// These correspond to the MCP2515 filters
	bool filter1_extended_id = false;
	uint32_t filter1_mask = 0x000;
	uint32_t filter1_ids[2] = {0xfff, 0xfff};
	bool filter2_extended_id = false;
	uint32_t filter2_mask = 0x000;
	uint32_t filter2_ids[4] = {0xfff, 0xfff, 0xfff, 0xfff};
};
extern CanParameters can1_params;
extern CanParameters can2_params;

bool can_send(MCP_CAN &mcp_can, const CAN_FRAME &frame);

bool can_receive(MCP_CAN &mcp_can, CAN_FRAME &frame);
void can_receive(MCP_CAN &mcp_can, void (*handle_frame)(const CAN_FRAME &frame));

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

void delay(unsigned long ms);

} // namespace ipdm
