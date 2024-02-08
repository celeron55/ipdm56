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
#include <avr/wdt.h>

namespace ipdm
{

bool switched_5v_ok = false;
uint32_t active_clock_divider = 1;

static constexpr uint16_t MIN_VBAT_MV_FOR_5VSW = 7000;

void setup()
{
	ipdm::io_begin();
	enable_watchdog();
}

void loop()
{
	ipdm::time_loop();

	reset_watchdog();

	EVERY_N_MILLISECONDS(100){
		bool switched_5v_ok_was = switched_5v_ok;
		switched_5v_ok = (analogRead_mV_factor16(VBAT_PIN, ADC_FACTOR16_VBAT) >= MIN_VBAT_MV_FOR_5VSW && digitalRead(ACTIVATE_5VSW_PIN));

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
	if(digitalRead(ACTIVATE_5VSW_PIN))
		return;

	pinMode(ACTIVATE_5VSW_PIN, OUTPUT);
	digitalWrite(ACTIVATE_5VSW_PIN, HIGH);

	// At about 140ms and below the startup console messages seem to get weird
	delay(200);

	if(analogRead_mV_factor16(VBAT_PIN, ADC_FACTOR16_VBAT) >= MIN_VBAT_MV_FOR_5VSW){
		switched_5v_ok = true;

		CONSOLE.println("-!- 5Vsw ON");

		try_can_init();

		if(!can_initialized){
			CONSOLE.println("-!- CAN interfaces failed to initialize. Keep in mind that"
					" 12V input is needed to power up the switched 5V rail (5Vsw).");
		}
	} else {
		CONSOLE.println("-!- 5Vsw not ON due to low Vbat");
	}
}

void disable_switched_5v()
{
	if(!digitalRead(ACTIVATE_5VSW_PIN))
		return;

	CONSOLE.println("-!- 5Vsw OFF");

	digitalWrite(ACTIVATE_5VSW_PIN, LOW);

	switched_5v_ok = false;
}

bool status_switched_5v()
{
	return switched_5v_ok;
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

void power_save_delay(unsigned long duration_ms)
{
	disable_switched_5v();

	// FIXME: For some reason the delay ends up actually being about a fourth of
	// the specified delay, so just multiply it here for now...
	duration_ms *= 4;

	bool long_delay = false;
	if(duration_ms >= 100){
		CONSOLE.println("-!- Entering power_save_delay");
		// Let the console message be printed at current clock speed
		delay(10);
		duration_ms -= 10;
		long_delay = true;
	}

	set_clock_prescaler(8);

	for(;;){
		loop();
		if(duration_ms >= 100){
			delay(100);
			duration_ms -= 100;
		} else {
			delay(duration_ms);
			break;
		}
	}

	set_clock_prescaler(0);

	enable_switched_5v();

	if(long_delay){
		CONSOLE.println("-!- Returning from power_save_delay");
	}
}

void enable_watchdog()
{
	wdt_disable();
	delay(3000);
	wdt_enable(WDTO_2S);
}

void reset_watchdog()
{
	wdt_reset();
}

} // namespace ipdm
