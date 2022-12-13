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
#include <Arduino.h>

namespace ipdm
{

#define NUM_ELEMS(x) (sizeof (x) / sizeof (x)[0])

// NOTE: Keep in mind this value will overflow at 4294967296ms
// NOTE: This will roughly take into account the system clock prescaler set by
// set_clock_prescaler()
unsigned long timestamp_age(unsigned long timestamp_ms);

// NOTE: Keep in mind this uses the value that will overflow at 4294967296ms
// NOTE: This will roughly take into account the system clock prescaler set by
// set_clock_prescaler()
bool timestamp_younger_than(unsigned long timestamp_ms, unsigned long max_age);

static bool ENM_compare_and_update(unsigned long &t0, const unsigned long &interval)
{
	bool trigger_now = timestamp_age(t0) >= interval;
	if(trigger_now)
		t0 = millis();
	return trigger_now;
}

#define EVERY_N_MILLISECONDS(ms) for(static unsigned long t0 = 0; ::ipdm::ENM_compare_and_update(t0, ms); )

static void util_print_timestamp(Stream &dst)
{
	char format_buf[17];
	uint32_t t = millis();
	uint32_t ms = t % 1000;
	t /= 1000;
	uint32_t s = t % 60;
	t /= 60;
	uint32_t m = t % 60;
	t /= 60;
	uint32_t h = t;
	if(h == 0 && m == 0)
		snprintf(format_buf, sizeof format_buf, "%02i.%03is: ", s, ms);
	else if(h == 0)
		snprintf(format_buf, sizeof format_buf, "%02im%02i.%03is: ", m, s, ms);
	else
		snprintf(format_buf, sizeof format_buf, "%02ih%02im%02i.%03is: ", h, m, s, ms);
	dst.print(format_buf);
}

// Macros for reading resistor divided ADC pins directly to voltage

#define ADC_FACTOR_MV_RESISTORS(highohm, lowohm) (5000.0f / 1024.0f * ((float)(highohm) + (float)(lowohm)) / float(lowohm))
#define ADC_FACTOR16_MV_RESISTORS(highohm, lowohm) (uint32_t)(65536.0f * ADC_FACTOR_MV_RESISTORS(highohm, lowohm))
#define ADC_VAL_TO_MV_FACTOR16(adc, factor16) (uint16_t)(((uint32_t)(adc) * (factor16)) >> 16)
#define ADC_VAL_TO_MV_RESISTORS(adc, highohm, lowohm) ADC_VAL_TO_MV_FACTOR16(adc, ADC_FACTOR16_MV_RESISTORS(highohm, lowohm))
#define analogRead_mV_factor16(pin, factor16) ADC_VAL_TO_MV_FACTOR16(analogRead(pin), factor16)
#define analogRead_mV_resistors(pin, highohm, lowohm) ADC_VAL_TO_MV_RESISTORS(analogRead(pin), highohm, lowohm)

// Macros for reporting value changes on console

#define REPORT_BOOL(var) \
	{\
		static bool reported_value = false;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			if(var)\
				CONSOLE.println(F("TRUE"));\
			else\
				CONSOLE.println(F("FALSE"));\
			reported_value = var;\
		}\
	}

#define REPORT_UINT16(var) \
	{\
		static uint16_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_UINT32(var) \
	{\
		static uint32_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_INT16(var) \
	{\
		static int16_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_UINT16_HYS(var, hys) \
	{\
		static uint16_t reported_value = 0;\
		if(abs(((int32_t)var - (int32_t)reported_value)) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_INT16_HYS(var, hys) \
	{\
		static int16_t reported_value = 0;\
		if(abs((int16_t)((int32_t)var - (int32_t)reported_value)) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_INT32_HYS(var, hys) \
	{\
		static int32_t reported_value = 0;\
		if(abs((int32_t)((int32_t)var - (int32_t)reported_value)) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_UINT32_HYS(var, hys) \
	{\
		static uint32_t reported_value = 0;\
		if(abs(((int32_t)var - (int32_t)reported_value)) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(var);\
			reported_value = var;\
		}\
	}

#define REPORT_UINT16_FORMAT(var, hys, mul, unit) \
	{\
		static uint16_t reported_value = 0;\
		if(abs((int16_t)((int32_t)var - (int32_t)reported_value)) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.print((float)var * mul);\
			CONSOLE.println(F(unit));\
			reported_value = var;\
		}\
	}

#define REPORT_INT16_FORMAT(var, hys, mul, unit) \
	{\
		static int16_t reported_value = 0;\
		if(abs(var - reported_value) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.print((float)var * mul);\
			CONSOLE.println(F(unit));\
			reported_value = var;\
		}\
	}

#define REPORT_FLOAT(var, hys, mul, unit) \
	{\
		static float reported_value = 0;\
		if(abs(var - reported_value) > (hys)){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.print((float)var * mul);\
			CONSOLE.println(F(unit));\
			reported_value = var;\
		}\
	}

#define REPORT_ENUM(var, names) \
	{\
		static uint8_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println(names[var]);\
			reported_value = var;\
		}\
	}

#define REPORT_UINT16_BITMAP(var, num_bits, names) \
	{\
		static uint16_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			for(uint16_t i=0; i<num_bits; i++){\
				if(var & bit(i)){\
					CONSOLE.print(names[i]);\
					CONSOLE.print(" ");\
				}\
			}\
			CONSOLE.println();\
			reported_value = var;\
		}\
	}

#define DEBUG_PRINT_LOCATION(delay_ms) { CONSOLE.print(__FILE__ ":"); CONSOLE.println(__LINE__); delay(delay_ms); }

} // namespace idpm
