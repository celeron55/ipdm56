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

typedef void (*log_can_frame_cb_t)(uint8_t interface, bool is_tx, const CAN_FRAME &);

void can_set_logger(log_can_frame_cb_t);

// These are mostly meant for internal operation, the user application rarely
// should touch these
extern bool can_initialized;
void try_can_init();

} // namespace ipdm
