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

#include "ipdm_library.h"
#include "ipdm_can.h"
#include "mcp_can.h"
#include "ipdm_pca9539.hpp"

namespace ipdm
{

MCP_CAN can1(CAN1_CS_PIN);
MCP_CAN can2(CAN2_CS_PIN);

CanParameters can1_params;
CanParameters can2_params;

bool can_initialized = false;

static bool can_init(MCP_CAN &can, const CanParameters &params, const char *log_title)
{
	/*CONSOLE.print(log_title);
	CONSOLE.println(F(": Initializing MCP2515"));*/

	for(uint8_t i=0; i<10; i++){
		if(can.begin(MCP_STDEXT, can1_params.speed, MCP_8MHZ) == CAN_OK){
			/*CONSOLE.print(log_title);
			CONSOLE.println(F(": MCP2515 init ok"));*/
			// Allow messages to be transmitted
			can.setMode(MCP_NORMAL);
			break;
		} else {
			CONSOLE.print(log_title);
			CONSOLE.println(F(": MCP2515 init failed"));
		}
		delay(100);
	}

	if(can.init_Mask(0, params.filter1_extended_id,
			params.filter1_extended_id ? params.filter1_mask : (params.filter1_mask << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(0, params.filter1_extended_id,
			params.filter1_extended_id ? params.filter1_ids[0] : (params.filter1_ids[0] << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(1, params.filter1_extended_id,
			params.filter1_extended_id ? params.filter1_ids[1] : (params.filter1_ids[1] << 16))
			== MCP2515_FAIL) goto filter_fail;
	
	if(can.init_Mask(1, params.filter2_extended_id,
			params.filter2_extended_id ? params.filter2_mask : (params.filter2_mask << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(2, params.filter2_extended_id,
			params.filter2_extended_id ? params.filter2_ids[0] : (params.filter2_ids[0] << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(3, params.filter2_extended_id,
			params.filter2_extended_id ? params.filter2_ids[1] : (params.filter2_ids[1] << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(4, params.filter2_extended_id,
			params.filter2_extended_id ? params.filter2_ids[2] : (params.filter2_ids[2] << 16))
			== MCP2515_FAIL) goto filter_fail;
	if(can.init_Filt(5, params.filter2_extended_id,
			params.filter2_extended_id ? params.filter2_ids[3] : (params.filter2_ids[3] << 16))
			== MCP2515_FAIL) goto filter_fail;

	return true;

filter_fail:
	CONSOLE.print(log_title);
	CONSOLE.println(F(": FAILED to set MCP2515 filters"));
	return false;
}

void try_can_init()
{
	can_initialized = true;
	if(!can_init(can1, can1_params, "can1")) can_initialized = false;
	if(!can_init(can2, can2_params, "can2")) can_initialized = false;
}

bool can_send(MCP_CAN &mcp_can, const CAN_FRAME &frame)
{
	if(!can_initialized)
		return false;
	return (mcp_can.sendMsgBuf(frame.id, 0, frame.length,
			frame.data.bytes) == CAN_OK);
}

bool can_receive(MCP_CAN &mcp_can, CAN_FRAME &frame)
{
	if(!can_initialized)
		return false;
	memset(&frame, 0, sizeof frame);
	uint8_t r = mcp_can.readMsgBuf(&frame.id, &frame.length, frame.data.bytes);
	return (r == CAN_OK);
}

void can_receive(MCP_CAN &mcp_can, void (*handle_frame)(const CAN_FRAME &frame))
{
	if(!can_initialized)
		return false;
	for(uint8_t i=0; i<10; i++){
		CAN_FRAME frame;
		if(!can_receive(mcp_can, frame))
			break;

		handle_frame(frame);
	}
}

} // namespace ipdm
