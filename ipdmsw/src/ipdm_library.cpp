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
#include "ipdm_pca9539.hpp"
#include "params.h"

namespace ipdm
{

IpdmPca9539 pca9539(0x74);

bool switched_5v_ok = false;
uint32_t active_clock_divider = 1;

static constexpr uint16_t MIN_VBAT_MV_FOR_5VSW = 7000;

void setup()
{
	// Set LOUT1...LOUT6, HOUT1...HOUT6 as outputs
	for(uint8_t i=4; i<16; i++){
		pca9539.pinMode(i, OUTPUT);
		pca9539.digitalWrite(i, LOW);
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

		modules.run_timeouts(CONSOLE);
		params.clear_timed_out_values();
	}

	EVERY_N_MILLISECONDS(1000){
		params.report_if_changed(CONSOLE);
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
