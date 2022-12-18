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
#include "mcp_can.h"
#include "ipdm_pca9539.hpp"

namespace ipdm
{

IpdmPca9539 pca9539(0x74);

MCP_CAN can1(CAN1_CS_PIN);
MCP_CAN can2(CAN2_CS_PIN);

CanParameters can1_params;
CanParameters can2_params;

static bool can_initialized = false;
static bool switched_5v_ok = false;
static uint32_t active_clock_divider = 1;

static constexpr uint16_t MIN_VBAT_MV_FOR_5VSW = 7000;

void setup()
{
	// Set LOUT1...LOUT6, HOUT1...HOUT6 as outputs
	for(uint8_t i=4; i<16; i++){
		pca9539.pinMode(i, OUTPUT);
		pca9539.digitalWrite(i, LOW);
	}
}

static bool can_init(MCP_CAN &can, const CanParameters &params, const char *log_title)
{
	/*CONSOLE.print(log_title);
	CONSOLE.println(": Initializing MCP2515");*/

	for(uint8_t i=0; i<10; i++){
		if(can.begin(MCP_STDEXT, can1_params.speed, MCP_8MHZ) == CAN_OK){
			/*CONSOLE.print(log_title);
			CONSOLE.println(": MCP2515 init ok");*/
			// Allow messages to be transmitted
			can.setMode(MCP_NORMAL);
			break;
		} else {
			CONSOLE.print(log_title);
			CONSOLE.println(": MCP2515 init failed");
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
	CONSOLE.println(": FAILED to set MCP2515 filters");
	return false;
}

static void try_can_init()
{
	can_initialized = true;
	if(!can_init(can1, can1_params, "can1")) can_initialized = false;
	if(!can_init(can2, can2_params, "can2")) can_initialized = false;
}

bool can_send(MCP_CAN &mcp_can, const CAN_FRAME &frame)
{
	return (mcp_can.sendMsgBuf(frame.id, 0, frame.length,
			frame.data.bytes) == CAN_OK);
}

bool can_receive(MCP_CAN &mcp_can, CAN_FRAME &frame)
{
	memset(&frame, 0, sizeof frame);
	uint8_t r = mcp_can.readMsgBuf(&frame.id, &frame.length, frame.data.bytes);
	return (r == CAN_OK);
}

void can_receive(MCP_CAN &mcp_can, void (*handle_frame)(const CAN_FRAME &frame))
{
	for(uint8_t i=0; i<10; i++){
		CAN_FRAME frame;
		if(!can_receive(mcp_can, frame))
			break;

		handle_frame(frame);
	}
}

void loop()
{
	EVERY_N_MILLISECONDS(100){
		bool switched_5v_ok_was = switched_5v_ok;
		switched_5v_ok = (analogRead_mV_factor16(VBAT_PIN, ADC_FACTOR16_VBAT) >= MIN_VBAT_MV_FOR_5VSW && digitalRead(POWERSW_PIN));

		REPORT_BOOL(switched_5v_ok);

		if(switched_5v_ok && !can_initialized){
			try_can_init();
			if(can_initialized){
				CONSOLE.println("-!- 5Vsw ok; CAN initialized");
			}
		}
		if(!switched_5v_ok){
			can_initialized = false;
		}
	}
}

void enable_switched_5v()
{
	if(digitalRead(POWERSW_PIN))
		return;

	pinMode(POWERSW_PIN, OUTPUT);
	digitalWrite(POWERSW_PIN, HIGH);

	// At about 140ms and below the startup console messages seem to get weird
	delay(200);

	if(analogRead_mV_factor16(VBAT_PIN, ADC_FACTOR16_VBAT) >= MIN_VBAT_MV_FOR_5VSW){
		switched_5v_ok = true;

		CONSOLE.println("-!- 5Vsw ON");

		try_can_init();

		if(!can_initialized){
			CONSOLE.println("-!- CAN interfaces failed to initialize. Keep in mind that"
					"12V input is needed to power up the switched 5V rail (5Vsw).");
		}
	}
}

void disable_switched_5v()
{
	if(!digitalRead(POWERSW_PIN))
		return;

	CONSOLE.println("-!- 5Vsw OFF");

	digitalWrite(POWERSW_PIN, LOW);

	switched_5v_ok = false;
}

bool status_switched_5v()
{
	return switched_5v_ok;
}

void pinMode(int pin, uint8_t mode)
{
	if(pin < 5600 || pin > 5615)
		return ::pinMode(pin, mode);
	pca9539.pinMode(pin - 5600, mode);
}

void digitalWrite(int pin, bool state)
{
	if(pin < 5600 || pin > 5615)
		return ::digitalWrite(pin, state);
	pca9539.digitalWrite(pin - 5600, state);
}

bool digitalRead(int pin)
{
	if(pin < 5600 || pin > 5615)
		return ::digitalRead(pin);
	// We don't want to do this automatically, because this way the output state
	// of the pin can't be checked by using this function, when the pin is being
	// used as output.
	//pca9539.pinMode(pin - 5600, INPUT);
	return pca9539.digitalRead(pin - 5600);
}

uint16_t analogRead(int pin)
{
	return ::analogRead(pin);
}

void set_clock_prescaler(uint8_t prescaler_bit)
{
	CLKPR = (1<<CLKPCE);
	CLKPR = prescaler_bit & 0x0f;

	active_clock_divider = (1<<prescaler_bit);
}

uint32_t get_active_clock_divider()
{
	return active_clock_divider;
}

void delay(unsigned long ms)
{
	::delay(ms / active_clock_divider);
}

} // namespace ipdm
