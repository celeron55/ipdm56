/*
ipdmsw - iPDM56 firmware template
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

#include "src/ipdm_library.h"
#include "src/ipdm_util.h"

constexpr int IGNITION_PIN = 7;

void setup()
{
	ipdm::setup();

	Serial.begin(115200);

	// The MCP2515 CAN controllers will be initialized with these speeds and
	// filters when the 5Vsw rail is powered up using ipdm::enable_switched_5v()
	ipdm::can1_params.speed = CAN_500KBPS;
	ipdm::can2_params.speed = CAN_500KBPS;
	// You can set the CAN filters like this. By default all messages pass.
	/*ipdm::can1_params.filter1_mask = 0xfff;
	ipdm::can1_params.filter1_ids[0] = 0x123;
	ipdm::can1_params.filter1_ids[1] = 0x456;
	ipdm::can1_params.filter2_mask = 0xff0;
	ipdm::can1_params.filter2_ids[0] = 0x120;
	ipdm::can1_params.filter2_ids[1] = 0x340;
	ipdm::can1_params.filter2_ids[2] = 0x560;
	ipdm::can1_params.filter2_ids[3] = 0x780;*/
}

void loop()
{
	ipdm::loop();

	// Consider D7 / IN10 to be the ignition pin and switch 5Vsw according to it
	if(digitalRead(IGNITION_PIN)){
		ipdm::enable_switched_5v();
	} else {
		ipdm::disable_switched_5v();
	}

	// Read incoming CAN1 frames
	ipdm::can_receive(ipdm::can1, handle_can1_frame);
	ipdm::can_receive(ipdm::can2, handle_can2_frame);

	// Send some CAN frames
	EVERY_N_MILLISECONDS(1000){
		CAN_FRAME frame;
		frame.id = 0x123;
		frame.length = 8;
		frame.data.bytes[0] = 0x13;
		frame.data.bytes[1] = 0x37;
		frame.data.bytes[2] = 0;
		frame.data.bytes[3] = 0;
		frame.data.bytes[4] = 0;
		frame.data.bytes[5] = 0;
		frame.data.bytes[6] = 0;
		frame.data.bytes[7] = 0;
		ipdm::can_send(ipdm::can1, frame);
	}

	// Print out input changes using a helper macro
	EVERY_N_MILLISECONDS(100){
		REPORT_BOOL(digitalRead(2));
		REPORT_BOOL(digitalRead(3));
		REPORT_BOOL(digitalRead(4));
		REPORT_BOOL(digitalRead(7));
		REPORT_UINT16_HYS(analogRead(A0), 50);
		REPORT_UINT16_HYS(analogRead(A1), 50);
		REPORT_UINT16_HYS(analogRead(A2), 50);
		REPORT_UINT16_HYS(analogRead(A3), 50);
		REPORT_UINT16_HYS(analogRead(A6), 50);
		REPORT_UINT16_HYS(analogRead_mV_factor16(ipdm::VBAT_PIN, ipdm::ADC_FACTOR16_VBAT), 50);
	}

	// Do a fancy demo cycle on all outputs and the ED0...ED3 lines
	EVERY_N_MILLISECONDS(1000){
		// NOTE: ED0...ED3 can also be used as digital inputs
		ipdm::digitalWrite(ipdm::ED0, !ipdm::digitalRead(ipdm::ED0));
		ipdm::digitalWrite(ipdm::ED1, !ipdm::digitalRead(ipdm::ED1));
		ipdm::digitalWrite(ipdm::ED2, !ipdm::digitalRead(ipdm::ED2));
		ipdm::digitalWrite(ipdm::ED3, !ipdm::digitalRead(ipdm::ED3));

		static uint8_t counter = 0;
		counter++;
		if(counter >= 3)
			counter = 0;

		Serial.print("Demo counter: ");
		Serial.println(counter);

		ipdm::digitalWrite(ipdm::HOUT1, counter == 0);
		ipdm::digitalWrite(ipdm::HOUT2, counter == 1);
		ipdm::digitalWrite(ipdm::HOUT3, counter == 2);

		ipdm::digitalWrite(ipdm::HOUT4, counter == 1);
		ipdm::digitalWrite(ipdm::HOUT5, counter == 1);
		ipdm::digitalWrite(ipdm::HOUT6, counter == 1);

		ipdm::digitalWrite(ipdm::LOUT1, counter == 0);
		ipdm::digitalWrite(ipdm::LOUT2, counter == 1);
		ipdm::digitalWrite(ipdm::LOUT3, counter == 2);

		ipdm::digitalWrite(ipdm::LOUT4, counter == 0);
		ipdm::digitalWrite(ipdm::LOUT5, counter == 1);
		ipdm::digitalWrite(ipdm::LOUT6, counter == 2);
	}
}

void handle_can1_frame(const CAN_FRAME &frame)
{
	Serial.print("can1 received frame id=0x");
	Serial.println(frame.id, HEX);

	/* Example:
	if(frame.id == 0x108){
		foobar = frame.data.bytes[0];
		return;
	}*/
}

void handle_can2_frame(const CAN_FRAME &frame)
{
	Serial.print("can2 received frame id=0x");
	Serial.println(frame.id, HEX);
}
