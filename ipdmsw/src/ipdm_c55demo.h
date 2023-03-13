/*
The C55demo algorithm
Copyright (c) 2023 Perttu "celeron55" Ahola

This algorithm is compatible with CHAdeMO. To respect the trademark, the name
is not used furthermore within this source code file.

License
-------
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

Example usage
-------------
#include "src/ipdm_c55demo.h"

constexpr int C55DEMO_CONN_CHECK_PIN =  2; // D2 = IN7 (1k pull-up to Vbat)
constexpr int C55DEMO_D1_PIN         =  3; // D3 = IN8 (1k pull-down to GND)
constexpr int C55DEMO_D2_PIN         =  4; // D4 = IN9 (1k pull-up to Vbat)
constexpr int C55DEMO_CHG_ENABLE_PIN = ipdm::LOUT5;
constexpr int C55DEMO_CONTACTOR_PIN  = ipdm::LOUT6;
constexpr int SYSTEM_POWER_ON_OUTPUT_PIN = ipdm::HOUT1; // (Optional)

ipdm::C55demoAlgorithm c55demo(your maximum battery charge voltage);

void setup()
{
	enable_watchdog();
	ipdm::setup();

	pinMode(C55DEMO_CONN_CHECK_PIN, INPUT_PULLUP);
	pinMode(C55DEMO_D1_PIN, INPUT);
	pinMode(C55DEMO_D2_PIN, INPUT_PULLUP);

	Serial.begin(115200);

	// C55demo CAN
	ipdm::can2_params.speed = CAN_500KBPS;
	ipdm::can2_params.filter1_mask = 0xfff;
	ipdm::can2_params.filter1_ids[0] = 0x108;
	ipdm::can2_params.filter1_ids[1] = 0x108;
	ipdm::can2_params.filter2_mask = 0x109;
	ipdm::can2_params.filter2_ids[0] = 0x109;
	ipdm::can2_params.filter2_ids[1] = 0x109;
	ipdm::can2_params.filter2_ids[2] = 0x109;
	ipdm::can2_params.filter2_ids[3] = 0x109;
}

void loop()
{
	reset_watchdog();
	ipdm::loop();

	// Enable CAN controllers if charger is plugged in
	if(!digitalRead(C55DEMO_CONN_CHECK_PIN) || digitalRead(C55DEMO_D1_PIN)){
		ipdm::enable_switched_5v();
	} else {
		ipdm::disable_switched_5v();
	}

	// Read incoming CAN2 frames
	ipdm::can_receive(ipdm::can2, handle_can2_frame);

	EVERY_N_MILLISECONDS(100){
		// Continuously feed this bunch of values into c55demo.update()
		ipdm::C55demoInput in;
		in.d1_high = ipdm::digitalRead(C55DEMO_D1_PIN); // D1 input state
		in.d2_high = ipdm::digitalRead(C55DEMO_D2_PIN); // D2 input state
		in.conn_check_high = ipdm::digitalRead(C55DEMO_CONN_CHECK_PIN); // CONN_CHK input state
		in.ntc1_celsius = -128; // Optional
		in.ntc2_celsius = -128; // Optional
		in.bms_main_contactor_closed = (set to true if pack contactors are closed);
		in.rail_voltage_V = (set to HV rail voltage measured by the car outside of the battery);
		in.bms_pack_voltage_V = (set to pack voltage measured by the BMS);
		in.bms_max_charge_current_A = (set to maximum allowed charge current by the BMS);
		in.bms_soc_percent = (set to SoC value given by the BMS); // Optional
		in.vehicle_parked = true; // Optional (set to true if the vehicle is parked)

		c55demo.update(in);

		// Copy digital output states from the algorithm to actual outputs
		ipdm::digitalWrite(C55DEMO_CHG_ENABLE_PIN, c55demo.output.charging_enable);
		ipdm::digitalWrite(C55DEMO_CONTACTOR_PIN, c55demo.output.close_c55demo_contactor);

		// To keep the system powered up when C55demo is plugged in, you can do
		// something like this (optional)
		ipdm::digitalWrite(SYSTEM_POWER_ON_OUTPUT_PIN, !ipdm::digitalRead(C55DEMO_CONN_CHECK_PIN));

		// Let C55demo send stuff on CANbus using your callback function
		c55demo.send_can_frames(c55demo_send_frame);

		if(c55demo.output.close_bms_contactor){
			// Make sure pack contactors are closed
		} else {
			// Pack contactors don't need to be closed
		}

		if(c55demo.output.disable_inverter){
			// Disable inverter for driving
		} else {
			// Enable inverter for driving
		}

		// The charger's voltage and current are available in:
		Serial.print(c55demo.charger_status.present_output_voltage);
		Serial.print("V ");
		Serial.print(c55demo.charger_status.present_charging_current);
		Serial.println("A");

		// Other useful values can be found in:
		c55demo.vehicle_status.status;
		c55demo.vehicle_status.faults;
		c55demo.charger_status.status;
		c55demo.charger_status.available_current;
		c55demo.charger_status.present_charging_current;
		c55demo.charger_status.remaining_charging_time_minutes;
	}
}

// Define the callback function for sending CAN frames
void c55demo_send_frame(uint16_t id, uint8_t bytes[8])
{
	CAN_FRAME frame;
	frame.id = id;
	frame.length = 8;
	memcpy(frame.data.bytes, bytes, 8);
	ipdm::can_send(ipdm::can2, frame);
}

// Feed received CAN frames into C55demo
void handle_can2_frame(const CAN_FRAME &frame)
{
	c55demo.handle_can_frame(frame.id, frame.data.bytes);
}
*/

#pragma once
#include <stdint.h>

namespace ipdm
{

// CANbus protocol
// ---------------

// Vehicle frames: 0x100, 0x101, 0x102
// Charger frames: 0x108, 0x109

#define VEHICLE_FAULT_OVER_VOLTAGE 1
#define VEHICLE_FAULT_UNDER_VOLTAGE 2
#define VEHICLE_FAULT_CURRENT_DEVIATION 4
#define VEHICLE_FAULT_OVER_TEMPERATURE 8
#define VEHICLE_FAULT_VOLTAGE_DEVIATION 16

#define VEHICLE_STATUS_CHARGE_ENABLED 1
#define VEHICLE_STATUS_NOT_PARK 2
#define VEHICLE_STATUS_FAULT 4
#define VEHICLE_STATUS_CONTACTOR_OPEN 8
#define VEHICLE_STATUS_REQUEST_STOP_BEFORE_CHARGING 16 // Weird flag, not used at all

#define CHARGER_STATUS_CHARGING 1
#define CHARGER_STATUS_FAULT 2
#define CHARGER_STATUS_CONNECTOR_LOCKED 4
#define CHARGER_STATUS_INCOMPATIBLE 8
#define CHARGER_STATUS_MALFUNCTION 16
#define CHARGER_STATUS_STOPPED 32

static const char* VEHICLE_STATUS_NAMES[5] = {
	"CHARGE_ENABLED",
	"NOT_PARK",
	"FAULT",
	"CONTACTOR_OPEN",
	"REQUEST_STOP",
};

static const char* CHARGER_STATUS_NAMES[6] = {
	"CHARGING",
	"FAULT",
	"CONNECTOR_LOCKED",
	"INCOMPATIBLE",
	"MALFUNCTION",
	"STOPPED",
};

// Algorithm
// ---------

struct C55demoInput
{
	bool d1_high = false; // High = 12V (active), low = 0V
	bool d2_high = false; // High = 12V, low = 0V (active)
	bool conn_check_high = false; // false = plugged in
	int8_t ntc1_celsius = -128;
	int8_t ntc2_celsius = -128;
	int16_t rail_voltage_V = 0;
	int16_t bms_pack_voltage_V = 0;
	bool bms_main_contactor_closed = false;
	uint8_t bms_max_charge_current_A = 0;
	uint8_t bms_soc_percent = 0;
	bool vehicle_parked = false;
};

struct C55demoOutput
{
	bool disable_inverter = true;
	bool charging_enable = false;
	bool close_c55demo_contactor = false;
	bool close_bms_contactor = false;
};

enum C55demoState {
	CS_WAITING_SEQ1,             // Until d1 (seq 1 input) activates
	CS_WAITING_PARAMETERS,       // Until we have valid parameters from charger
	CS_WAITING_BMS_CONTACTOR,    // Until BMS reports main contactor closed
	CS_PERMITTING_CHARGE,        // 0.5s delay, then next state (IEEE 2030.1.1 A.6)
	CS_PERMITTING_CHARGE_PHASE2, // Until d2 (seq 2 input) and connector lock activates after insulation test; then close contactor
	CS_WAITING_CHARGER_TO_START_CHARGING, // Until charger does not have "stopped" status and reports remaining charging time
	CS_CHARGING,                 // Until battery is full or something else happens
	CS_REQUESTING_STOP_NICELY,   // Until current request has been lowered to 0
	CS_REQUESTING_STOP,          // 1.75s delay, then next state (IEEE 2030.1.1 A.6)
	CS_REQUESTING_STOP_PHASE2,   // Until charger reports <5A current; then open contactor
	CS_WAITING_CONNECTOR_UNLOCK, // Until charger reports connector lock open
	CS_ENDED,

	CS_COUNT,
};

extern const char *C55demoState_STRINGS[CS_COUNT];

struct C55demoAlgorithm
{
	const int16_t target_charge_voltage_V = 0;
	const uint8_t charge_end_A = 0;
	const int16_t capacity_kWh = 0;

	C55demoOutput output;

	C55demoState c55demo_state = CS_WAITING_SEQ1;

	unsigned long c55demo_start_timestamp = 0;
	unsigned long permit_charge_timestamp = 0;
	unsigned long contactor_close_timestamp = 0;
	unsigned long requesting_stop_timestamp = 0;
	unsigned long charger_last_correct_voltage_timestamp = 0;
	unsigned long bms_last_correct_voltage_timestamp = 0;
	unsigned long last_received_from_c55demo_timestamp = 0;
	unsigned long current_request_adjusted_timestamp = 0;

	struct VehicleConstant {
		// 0x100
		uint16_t maximum_voltage = 0;
		uint8_t charged_rate_reference = 100;
		// 0x101
		uint8_t maximum_charging_time_minutes = 102;
		// 0x102
		// NOTE: If 2 doesn't work, try 1 instead
		uint8_t c55demo_version = 2; // 0 = <0.9, 1 = 0.9/0.9.1, 2 = 1.0.0/1.0.1
		uint16_t target_battery_voltage = 0; // V
	} vehicle_constant;

	struct VehicleStatus {
		// 0x101
		uint8_t estimated_charging_time_minutes = 102; // Optional or not really?
		// 0x102
		uint8_t charging_current_request = 0; // A
		uint8_t faults = 0;
		uint8_t status = VEHICLE_STATUS_CONTACTOR_OPEN;
		uint8_t charged_rate = 0;
	} vehicle_status;

	struct ChargerStatus {
		// 0x108
		bool supports_contactor_welding_detection = false;
		uint16_t available_voltage = 0;
		uint8_t available_current = 0;
		uint16_t threshold_voltage = 0;
		// 0x109
		uint8_t c55demo_version = 0;
		uint16_t present_output_voltage = 0;
		uint8_t present_charging_current = 0;
		uint8_t status = 0;
		uint8_t remaining_charging_time_minutes = 0;
	} charger_status;

	C55demoAlgorithm(
		int16_t target_charge_voltage_V,
		uint8_t charge_end_A=10
	):
		target_charge_voltage_V(target_charge_voltage_V),
		capacity_kWh(capacity_kWh),
		charge_end_A(charge_end_A)
	{
		vehicle_constant.maximum_voltage = target_charge_voltage_V + 2; // Add some slop
		vehicle_constant.target_battery_voltage = target_charge_voltage_V;
	}

	// Shall be called every 100ms
	// Result is found in the "output" member variable
	void update(const C55demoInput &input);

	// Shall be called every 100ms
	void handle_can_frame(uint16_t id, uint8_t bytes[8]);

	// Shall be called every 100ms
	void send_can_frames(void (*send_frame)(uint16_t id, uint8_t bytes[8]));

	bool get_request_main_contactor();
	bool get_request_inverter_disable(const C55demoInput &input);
	void stop_charge_if_needed(const C55demoInput &input);
	void permit_charge();
	void close_contactor_and_start_charging();
	void stop_charging_nicely();
	void stop_charging();
	void open_contactor_and_start_waiting_for_connector_unlock();
	void report_status_on_console();
};
} // namespace ipdm
