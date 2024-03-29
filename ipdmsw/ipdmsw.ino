/*
ipdmsw - iPDM56 firmware template
Copyright (c) 2023 Perttu "celeron55" Ahola

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
#include "src/ipdm_can.h"
#include "src/params.h"

// Matrix inputs/outputs
constexpr int UNUSED_M1          = A0; // A0 = M1
constexpr int UNUSED_M2          = A1; // A1 = M2
constexpr int UNUSED_M3          = A2; // A2 = M3
constexpr int UNUSED_M4          = A3; // A3 = M4
constexpr int UNUSED_M5          = A6; // A6 = M5
constexpr int UNUSED_M6          =  2; // D2 = M6
constexpr int UNUSED_M7          =  3; // D3 = M7
constexpr int UNUSED_M8          = ipdm::ED4; // ED4 = M8
constexpr int UNUSED_M9          = ipdm::ED5; // ED5 = M9
constexpr int IGNITION_PIN       = ipdm::ED6; // ED6 = M10

// Outputs
constexpr int UNUSED_LOUT1                = ipdm::LOUT1;
constexpr int UNUSED_LOUT2                = ipdm::LOUT2;
constexpr int UNUSED_LOUT3                = ipdm::LOUT3;
constexpr int UNUSED_LOUT4                = ipdm::LOUT4;
constexpr int UNUSED_LOUT5                = ipdm::LOUT5;
constexpr int UNUSED_LOUT6                = ipdm::LOUT6;
constexpr int UNUSED_HOUT1                = ipdm::HOUT1;
constexpr int POWER_STEERING_POWER_PIN    = ipdm::HOUT2;
constexpr int UNUSED_HOUT3                = ipdm::HOUT3;
constexpr int UNUSED_HOUT4                = ipdm::HOUT4;
constexpr int UNUSED_HOUT5                = ipdm::HOUT5;
constexpr int UNUSED_HOUT6                = ipdm::HOUT6;
constexpr int BATTERY_HEATER_SIGNAL_PIN   = ipdm::AOUT1;
constexpr int UNUSED_AOUT2                = ipdm::AOUT2;

void setup()
{
	Serial.begin(115200);

	ipdm::setup();

	ipdm::pinMode(IGNITION_PIN, INPUT);

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

	EVERY_N_MILLISECONDS(30000){
		ipdm::util_print_timestamp(Serial);
		Serial.println("-!- iPDM56 running");
	}

	// Consider D7 (IN10) to be the ignition pin and switch 5Vsw according to it
	//if(ipdm::digitalRead(IGNITION_PIN)){
	// Always enable switched 5V for testing CANbus
	if(true){
		ipdm::enable_switched_5v();
	} else {
		ipdm::disable_switched_5v();
	}

	// Read incoming CAN frames
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
		REPORT_BOOL(ipdm::digitalRead(2));
		REPORT_BOOL(ipdm::digitalRead(3));
		REPORT_BOOL(ipdm::digitalRead(4));
		REPORT_BOOL(ipdm::digitalRead(IGNITION_PIN));
		REPORT_UINT16_HYS(analogRead(A0), 50);
		REPORT_UINT16_HYS(analogRead(A1), 50);
		REPORT_UINT16_HYS(analogRead(A2), 50);
		REPORT_UINT16_HYS(analogRead(A3), 50);
		REPORT_UINT16_HYS(analogRead(A6), 50);
		REPORT_UINT16_HYS(analogRead_mV_factor16(ipdm::VBAT_PIN, ipdm::ADC_FACTOR16_VBAT), 100);
	}

	// Example: Control power steering pump in the simplest possible way
	EVERY_N_MILLISECONDS(1000){
		bool power_steering_power = digitalRead(IGNITION_PIN);
		ipdm::digitalWrite(POWER_STEERING_POWER_PIN, power_steering_power);
		REPORT_BOOL(power_steering_power);
	}

	// Example: PWM / analog output
	EVERY_N_MILLISECONDS(1000){
		bool do_heat_battery = true;
		// NOTE: Output range: 0...255 maps to 14...0V on ipdmhw1.0 AOUT1,2
		analogWrite(BATTERY_HEATER_SIGNAL_PIN, do_heat_battery ? 138 : 0);
	}

	// Do a fancy demo cycle on all outputs (except POWER_STEERING_POWER_PIN =
	// ipdm::HOUT2 which is used in the example code above)
	EVERY_N_MILLISECONDS(1000){
		static uint8_t counter = 0;
		counter++;
		if(counter >= 3)
			counter = 0;

		Serial.print("Demo counter: ");
		Serial.println(counter);

		ipdm::digitalWrite(ipdm::HOUT1, counter == 0);
		//ipdm::digitalWrite(ipdm::HOUT2, counter == 1);
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
	/*Serial.print("can1 received frame id=0x");
	Serial.println(frame.id, HEX);*/

	// Handling of received CAN messages

	// Example: Outlander OBC

	if(frame.id == 0x377){
		/* - B0+B1 = 12V Battery voltage	(h04DC=12,45V -> 0,01V/bit)
		- B2+B3 = 12V Supply current	(H53=8,3A -> 0,1A/bit)
		- B4 = Temperature 1		(starts at -40degC, +1degC/bit)
		- B5 = Temperature 2		(starts at -40degC, +1degC/bit)
		- B6 = Temperature 3		(starts at -40degC, +1degC/bit)
		- B7 = Statusbyte		(h20=standby, h21=error, h22=in operation)
		-  - bit0(LSB) = Error
		-  - bit1	= In Operation
		-  - bit3      =
		-  - bit4      =
		-  - bit5      = Ready
		-  - bit6	=
		-  - bit7(MSB) = */
		params.obc_battery_12v_voltage.set(word(frame.data.bytes[0], frame.data.bytes[1]));
		params.obc_dcdc_status.set(frame.data.bytes[7]);
		return;
	}
	if(frame.id == 0x389){
		modules.obc.timeout_reset();
		/* - B0 = Battery Voltage (as seen by the charger), needs to be scaled x
		 * 2, so can represent up to 255*2V; used to monitor battery during
		 * charge
		 - B1 = Charger supply voltage, no scaling needed
		 - B6 = Charger Supply Current x 10 */
		params.obc_battery_voltage.set((uint16_t)frame.data.bytes[0] * 2);
		params.obc_supply_voltage.set(frame.data.bytes[1]);
		params.obc_supply_current.set(frame.data.bytes[6]);
		return;
	}
	if(frame.id == 0x38a){
		/* - B0 = temp x 2?
		- B1 = temp x 2?
		- B3 = EVSE Control Duty Cycle (granny cable ~26 = 26%) */
		params.obc_evse_pwm.set(frame.data.bytes[3]);
		return;
	}

	// Example: Outlander CAN-controlled heater

	if(frame.id == 0x398){
		modules.heater.timeout_reset();
		params.heater_heating.set((frame.data.bytes[5] > 0));
		params.heater_hv_present.set((frame.data.bytes[6] != 0x09));
		int16_t temp1 = (int16_t)frame.data.bytes[3] - 40;
		int16_t temp2 = (int16_t)frame.data.bytes[4] - 40;
		params.heater_temperature.set(ipdm::limit_int16(temp1 > temp2 ? temp1 : temp2, -127, 126));
		return;
	}
	if(frame.id == 0x630){
		return;
	}
	if(frame.id == 0x62d){
		return;
	}
	if(frame.id == 0x6bd){
		return;
	}
}

static void print_frame(const CAN_FRAME &frame)
{
	Serial.print("id=0x");
	Serial.print(frame.id, HEX);
	for(uint8_t i=0; i<8; i++){
		Serial.print(" 0x");
		Serial.print(frame.data.bytes[i], HEX);
	}
}

void handle_can2_frame(const CAN_FRAME &frame)
{
#if 1
	Serial.print("can2 received frame ");
	print_frame(frame);
	Serial.println();
#endif

	// Handling of received CAN messages

	if(frame.id == 0x123){
		modules.obc.timeout_reset();
		params.example_module_foobar.set(frame.data.bytes[0]);
		return;
	}
}
