/*
The C55demo algorithm
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

#include "ipdm_c55demo.h"
#include "ipdm_util.h"
#include <Arduino.h>

namespace ipdm
{

// Utilities
// ---------

#define CONSOLE Serial

static void log_println(const char *line)
{
	ipdm::util_print_timestamp(CONSOLE);
	CONSOLE.println(line);
}

static void log_println_P(const char *line)
{
	ipdm::util_print_timestamp(CONSOLE);
	CONSOLE.println((__FlashStringHelper*)line);
}

#define log_println_f(x) log_println_P(PSTR(x))

// Algorithm
// ---------

#define REQUESTING_STOP_NICELY_TIMEOUT_MS 40000
#define REQUESTING_STOP_OPEN_CONTACTOR_TIMEOUT_MS 20000
#define VOLTAGE_SLOP 2

#define C55DEMO_CAN_ALIVE (last_received_from_c55demo_timestamp != 0 && ipdm::timestamp_age(last_received_from_c55demo_timestamp) < 5000)

const char *C55demoState_STRINGS[CS_COUNT] = {
	"WAITING_SEQ1",
	"WAITING_PARAMETERS",
	"WAITING_BMS_CONTACTOR",
	"PERMITTING_CHARGE",
	"PERMITTING_CHARGE_PHASE2",
	"WAITING_CHARGER_TO_START_CHARGING",
	"CHARGING",
	"REQUESTING_STOP_NICELY",
	"REQUESTING_STOP",
	"REQUESTING_STOP_PHASE2",
	"WAITING_CONNECTOR_UNLOCK",
	"ENDED",
};

bool C55demoAlgorithm::get_request_main_contactor()
{
	if(charger_status.present_charging_current > 5) // Contactor-saving failsafe
		return true;
	if(c55demo_state >= CS_WAITING_BMS_CONTACTOR && c55demo_state != CS_ENDED)
		return true;

	return false;
}

bool C55demoAlgorithm::get_request_inverter_disable(const C55demoInput &input)
{
	if(
		!input.conn_check_high ||
		get_request_main_contactor()
	){
		return true;
	}

	return false;
}

void C55demoAlgorithm::stop_charge_if_needed(const C55demoInput &input)
{
	int8_t temperature1 = input.ntc1_celsius;
	int8_t temperature2 = input.ntc2_celsius;

	if(c55demo_state == CS_CHARGING || c55demo_state == CS_PERMITTING_CHARGE ||
			c55demo_state == CS_WAITING_CHARGER_TO_START_CHARGING){
		if(temperature1 > 50 || temperature2 > 50){
			log_println_f("Connector over temperature");
			stop_charging();
		}
		if(input.bms_max_charge_current_A == 0 || !input.bms_main_contactor_closed){
			log_println_f("BMS does not allow charging");
			stop_charging();
		}
	}

	if(c55demo_state == CS_CHARGING || c55demo_state == CS_REQUESTING_STOP_NICELY){
		// Check CHARGER_STATUS_CHARGING and CHARGER_STATUS_STOPPED if contactor was
		// closed more than 5000ms ago.
		// NOTE: Charger sets CHARGER_STATUS_CHARGING and clears CHARGER_STATUS_STOPPED
		//       after vehicle closes its contactor.
		if(ipdm::timestamp_age(contactor_close_timestamp) > 5000){
			if(charger_status.status & CHARGER_STATUS_STOPPED){
				log_println_f("Charger status switched to \"stopped\"");
				stop_charging();
			}
			if(!(charger_status.status & CHARGER_STATUS_CHARGING)){
				log_println_f("Charger status switched to \"not charging\"");
				stop_charging();
			}
		}
	}

	if(charger_status.status & CHARGER_STATUS_MALFUNCTION){
		log_println_f("Charger reports malfunction");
		stop_charging();
	}

	if(!input.d1_high){
		if(c55demo_state == CS_WAITING_PARAMETERS){
			log_println_f("d1 (seq1) deactivation detected, waiting for it again");
			c55demo_state = CS_WAITING_SEQ1;
		} else {
			log_println_f("d1 (seq1) deactivation detected, stopping charging");
			stop_charging();
		}
	}
}

void C55demoAlgorithm::permit_charge()
{
	log_println_f("permit_charge()");

	output.charging_enable = true;

	permit_charge_timestamp = longtime();

	c55demo_state = CS_PERMITTING_CHARGE;
}

void C55demoAlgorithm::close_contactor_and_start_charging()
{
	log_println_f("close_contactor_and_start_charging()");

	output.close_c55demo_contactor = true;
	vehicle_status.status &= ~VEHICLE_STATUS_CONTACTOR_OPEN;

	contactor_close_timestamp = longtime();
	charger_last_correct_voltage_timestamp = longtime();
	bms_last_correct_voltage_timestamp = longtime();

	c55demo_state = CS_WAITING_CHARGER_TO_START_CHARGING;
}

void C55demoAlgorithm::stop_charging_nicely()
{
	log_println_f("stop_charging_nicely()");

	c55demo_state = CS_REQUESTING_STOP_NICELY;
	requesting_stop_timestamp = longtime();
}

void C55demoAlgorithm::stop_charging()
{
	log_println_f("stop_charging()");

	vehicle_status.charging_current_request = 0;
	vehicle_status.status &= ~VEHICLE_STATUS_CHARGE_ENABLED;

	c55demo_state = CS_REQUESTING_STOP;
	requesting_stop_timestamp = longtime();
}

void C55demoAlgorithm::open_contactor_and_start_waiting_for_connector_unlock()
{
	log_println_f("open_contactor_and_start_waiting_for_connector_unlock()");

	output.close_c55demo_contactor = false;
	vehicle_status.status |= VEHICLE_STATUS_CONTACTOR_OPEN;
	vehicle_status.status &= ~VEHICLE_STATUS_CHARGE_ENABLED;

	c55demo_state = CS_WAITING_CONNECTOR_UNLOCK;
}


void C55demoAlgorithm::update(const C55demoInput &input)
{
	REPORT_BOOL(input.d1_high);
	REPORT_BOOL(input.d2_high);
	REPORT_BOOL(input.conn_check_high);
	REPORT_INT8(input.ntc1_celsius);
	REPORT_INT8(input.ntc2_celsius);
	REPORT_INT16_FORMAT(input.rail_voltage_V, 2, 1, " V");
	REPORT_INT16_FORMAT(input.bms_pack_voltage_V, 2, 1, " V");
	REPORT_BOOL(input.bms_main_contactor_closed);
	REPORT_UINT16(input.bms_max_charge_current_A);
	REPORT_UINT8(input.bms_soc_percent);
	REPORT_BOOL(input.vehicle_parked);
	REPORT_BOOL(C55DEMO_CAN_ALIVE);

	if(!C55DEMO_CAN_ALIVE){
		charger_status = ChargerStatus();

		if(input.conn_check_high){
			if(c55demo_state != CS_WAITING_SEQ1){
				log_println_f("CAN and conn_check are inactive; resetting state");
				c55demo_state = CS_WAITING_SEQ1;
				vehicle_status = VehicleStatus();
			}
		}
	}

	switch(c55demo_state){
	case CS_WAITING_SEQ1: {
		if(
			input.d1_high
		){
			if(!input.d2_high){
				EVERY_N_MILLISECONDS(1000){
					log_println_f("d1 (seq1) activation detected");
					log_println_f("* but (seq2) also is. Logical discrepancy, not starting");
				}
			} else {
				log_println_f("d1 (seq1) activation detected");

				c55demo_state = CS_WAITING_PARAMETERS;
				c55demo_start_timestamp = longtime();
				break;
			}
		}

		EVERY_N_MILLISECONDS(5000){
			log_println_f("... Waiting for d1 (seq1) activation");
		}
	} break;
	case CS_WAITING_PARAMETERS: {
		if(!input.d1_high){
			log_println_f("d1 (seq1) deactivation detected, waiting for it again");

			c55demo_state = CS_WAITING_SEQ1;
			break;
		}

		if(
			(
				charger_status.available_current >= 10
				||
				// For efacec
				(charger_status.c55demo_version != 0 ||
					charger_status.remaining_charging_time_minutes > 0)
			)
			&&
			input.bms_max_charge_current_A != 0
		){
			c55demo_state = CS_WAITING_BMS_CONTACTOR;
			break;
		}

		/*log_println_f("Permitting charge for no good reason (DEBUG)");
		c55demo_state = CS_WAITING_BMS_CONTACTOR; break;*/

		EVERY_N_MILLISECONDS(5000){
			if(charger_status.available_current < 10){
				log_println_f("... Waiting for charger available current >= 10A or some other indication of charger being alive");
			}
			if(input.bms_max_charge_current_A == 0){
				log_println_f("... Waiting for BMS to permit charge");
			}
		}
	} break;
	case CS_WAITING_BMS_CONTACTOR: {
		if(!input.d1_high){
			log_println_f("d1 (seq1) deactivation detected, waiting for it again");

			c55demo_state = CS_WAITING_SEQ1;
			break;
		}

		if(input.bms_main_contactor_closed){
			permit_charge();
		}

		/*log_println_f("Permitting charge for no good reason (DEBUG)");
		permit_charge();*/

		EVERY_N_MILLISECONDS(5000){
			if(!input.bms_main_contactor_closed){
				log_println_f("... Waiting for BMS main contactor to close");
			}
		}
	} break;
	case CS_PERMITTING_CHARGE: {
		if(longtime() - permit_charge_timestamp >= 500){
			// IEEE 2030.1.1 A.6: Vehicle charging enabled flag is set 0.0...1.0s
			// after the vehicle charge permission line is activated
			vehicle_status.status |= VEHICLE_STATUS_CHARGE_ENABLED;

			c55demo_state = CS_PERMITTING_CHARGE_PHASE2;

			log_println_f("NOTE: Connector lock and insulation test should occur now.");
			log_println_f("NOTE: Then the charger should pull down C55DEMO_CHARGE_SEQ_2_INPUT_PIN.");
		}
	} break;
	case CS_PERMITTING_CHARGE_PHASE2: {
		stop_charge_if_needed(input);

		// When seq 2 input (active low) and connector lock activate, close
		// contactor and start requesting current
		// Charger does an insulation test before activating seq 2.
		if(
			!input.d2_high &&
			(charger_status.status & CHARGER_STATUS_CONNECTOR_LOCKED)
		){
			close_contactor_and_start_charging();
		}

		EVERY_N_MILLISECONDS(5000){
			if(!(charger_status.status & CHARGER_STATUS_CONNECTOR_LOCKED)){
				log_println_f("... Waiting for connector lock");
			}
			if(input.d2_high){
				log_println_f("... Waiting for seq2 to be pulled low");
			}
		}
	} break;
	case CS_WAITING_CHARGER_TO_START_CHARGING: {
		stop_charge_if_needed(input);

		// Request some current initially
		vehicle_status.charging_current_request = 5;

		// When charger status is not STOPPED and charger reports a charging
		// time, then we are charging and can start requesting current and
		// waiting for a stop request
		if(
			!(charger_status.status & CHARGER_STATUS_STOPPED) &&
			charger_status.remaining_charging_time_minutes > 0
		){
			c55demo_state = CS_CHARGING;
		}

		// If charger reports a non-zero charge current while reporting as being
		// stopped, stop charging
		if(
			(charger_status.status & CHARGER_STATUS_STOPPED) &&
			charger_status.present_charging_current > 0
		){
			log_println_f("Charger reports charging current and being STOPPED"
					" at the same time");
			stop_charging();
		}

		EVERY_N_MILLISECONDS(5000){
			if(charger_status.status & CHARGER_STATUS_STOPPED){
				log_println_f("... Waiting for charger status to not be STOPPED");
			}

			if(charger_status.remaining_charging_time_minutes == 0){
				log_println_f("... Waiting for charger to report a non-zero charging time");
			}
		}
	} break;
	case CS_CHARGING: {
		stop_charge_if_needed(input);

		// Adjust current request at interval (spec allows 20A/s)
		{
			if(longtime() - current_request_adjusted_timestamp > 300){
				current_request_adjusted_timestamp = longtime();

				uint8_t max_current_request = input.bms_max_charge_current_A;

				// Don't believe the charger if it reports 0A available (this
				// code has been written for efacec)
				uint8_t charger_available_current = charger_status.available_current;
				if(charger_available_current == 0)
					charger_available_current = 120;
				if(max_current_request > charger_available_current)
					max_current_request = charger_available_current;

				// Main feedback
				int16_t measured_voltage = input.rail_voltage_V;

				if(vehicle_status.charging_current_request > max_current_request){
					// Decrement twice
					if(vehicle_status.charging_current_request > 0)
						vehicle_status.charging_current_request--;
					if(vehicle_status.charging_current_request > 0)
						vehicle_status.charging_current_request--;
				} else if(measured_voltage < target_charge_voltage_V - VOLTAGE_SLOP){
					// Increment once
					if(vehicle_status.charging_current_request < max_current_request)
						vehicle_status.charging_current_request++;
				} else if(measured_voltage > target_charge_voltage_V){
					// Decrement twice
					if(vehicle_status.charging_current_request > 0)
						vehicle_status.charging_current_request--;
					if(vehicle_status.charging_current_request > 0)
						vehicle_status.charging_current_request--;
				}

				// Stop charging at some point
				if(vehicle_status.charging_current_request < charge_end_A &&
						ipdm::timestamp_age(contactor_close_timestamp) > 180000){
					log_println_f("Charge looks finished");
					stop_charging_nicely();
				}

				// Check voltage deviatiion
				// OBC to charger
				// NOTE: 10V deviation specified by IEEE 2030.1.1 table A.22
				if(labs((int16_t)measured_voltage -
						(int16_t)charger_status.present_output_voltage) <= 10){
					charger_last_correct_voltage_timestamp = longtime();
				}
				if(ipdm::timestamp_age(charger_last_correct_voltage_timestamp) > 5000){
					log_println_f("Charger correct voltage timeout");
					stop_charging();
				}
				// OBC to BMS
				// bms_pack_voltage_V updates too slowly for direct feedback
				if(labs((int16_t)measured_voltage - input.bms_pack_voltage_V) < 5){
					bms_last_correct_voltage_timestamp = longtime();
				}
				if(ipdm::timestamp_age(bms_last_correct_voltage_timestamp) > 5000){
					log_println_f("BMS correct voltage timeout");
					stop_charging();
				}
			}
		}
	} break;
	case CS_REQUESTING_STOP_NICELY: {
		if(longtime() - requesting_stop_timestamp > REQUESTING_STOP_NICELY_TIMEOUT_MS){
			log_println_f("Timed out requesting stop nicely. Requesting not nicely");
			stop_charging();
		}

		// Adjust current request down every 100ms = every call to this function
		{
			if(vehicle_status.charging_current_request > 0){
				vehicle_status.charging_current_request--;
			}

			// Stop charging when current request reaches zero
			if(vehicle_status.charging_current_request == 0){
				stop_charging();
			}
		}
	} break;
	case CS_REQUESTING_STOP: {
		if(longtime() - requesting_stop_timestamp > 1750){
			// IEEE 2030.1.1 A.6: Charge permission deactivates after 1.5...2.0s
			// measured from CANbus charging stop flag
			output.charging_enable = false;

			c55demo_state = CS_REQUESTING_STOP_PHASE2;
			requesting_stop_timestamp = longtime();
		}
	} break;
	case CS_REQUESTING_STOP_PHASE2: {
		if(longtime() - requesting_stop_timestamp > REQUESTING_STOP_OPEN_CONTACTOR_TIMEOUT_MS){
			log_println_f("Timed out requesting stop. Opening contactor");
			open_contactor_and_start_waiting_for_connector_unlock();
			vehicle_status.status |= VEHICLE_STATUS_FAULT;
		}
		// When charger reports <5A current after 7s, open contactor
		// TODO: Check our own current measurement
		if(charger_status.present_charging_current < 5 &&
				ipdm::timestamp_age(requesting_stop_timestamp) > 7000){
			open_contactor_and_start_waiting_for_connector_unlock();
		}
	} break;
	case CS_WAITING_CONNECTOR_UNLOCK: {
		if(!(charger_status.status & CHARGER_STATUS_CONNECTOR_LOCKED)){
			log_println_f("Connector lock is inactive. Charging has ended.");
			c55demo_state = CS_ENDED;
			last_received_from_c55demo_timestamp = 0; // Needed with long timeout in case of problem
		}
	} break;
	case CS_ENDED: {
		EVERY_N_MILLISECONDS(60000){
			log_println_f("Charging has ended");
		}
	} break;
	case CS_COUNT: break;
	}

	vehicle_status.charged_rate = input.bms_soc_percent;

	output.disable_inverter = get_request_inverter_disable(input);
	output.close_bms_contactor = get_request_main_contactor();
}

void C55demoAlgorithm::handle_can_frame(uint16_t id, uint8_t bytes[8])
{
	if(id == 0x108){
		last_received_from_c55demo_timestamp = longtime();

		charger_status.supports_contactor_welding_detection = bytes[0];
		charger_status.available_voltage = bytes[1] | (bytes[2] << 8);
		charger_status.available_current = bytes[3];
		charger_status.threshold_voltage = bytes[4] | (bytes[5] << 8);

		return;
	}
	if(id == 0x109){
		last_received_from_c55demo_timestamp = longtime();

		charger_status.c55demo_version = bytes[0];
		charger_status.present_output_voltage = bytes[1] | (bytes[2] << 8);
		charger_status.present_charging_current = bytes[3];
		charger_status.status = bytes[5];
		charger_status.remaining_charging_time_minutes = bytes[6] == 0xff ?
				(bytes[6] * 6) : bytes[7];

		return;
	}
}

void C55demoAlgorithm::send_can_frames(void (*send_frame)(uint16_t id, uint8_t bytes[8]))
{
	if(c55demo_state == CS_WAITING_SEQ1){
		return;
	}

	{
		uint8_t bytes[8];
		bytes[0] = 0;
		bytes[1] = 0;
		bytes[2] = 0;
		bytes[3] = 0;
		bytes[4] = lowByte(vehicle_constant.maximum_voltage);
		bytes[5] = highByte(vehicle_constant.maximum_voltage);
		bytes[6] = vehicle_constant.charged_rate_reference;
		bytes[7] = 0;
		send_frame(0x100, bytes);
	}
	{
		uint8_t bytes[8];
		bytes[0] = 0;
		bytes[1] = 0xff;
		bytes[2] = vehicle_constant.maximum_charging_time_minutes;
		bytes[3] = vehicle_status.estimated_charging_time_minutes;
		bytes[4] = 0;
		bytes[5] = 0;
		bytes[6] = 0;
		bytes[7] = 0;
		send_frame(0x101, bytes);
	}
	{
		uint8_t bytes[8];
		bytes[0] = vehicle_constant.c55demo_version;
		bytes[1] = lowByte(vehicle_constant.target_battery_voltage);
		bytes[2] = highByte(vehicle_constant.target_battery_voltage);
		bytes[3] = vehicle_status.charging_current_request;
		bytes[4] = vehicle_status.faults;
		bytes[5] = vehicle_status.status;
		bytes[6] = vehicle_status.charged_rate;
		bytes[7] = 0;
		send_frame(0x102, bytes);
	}
}
} // namespace ipdm
