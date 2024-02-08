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
#include "ipdm_time.h"
#include <Arduino.h>

namespace ipdm
{

#define NUM_ELEMS(x) (sizeof (x) / sizeof (x)[0])

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

#define REPORT_UINT8(var) \
	{\
		static uint8_t reported_value = 0;\
		if(var != reported_value){\
			::ipdm::util_print_timestamp(CONSOLE);\
			CONSOLE.print(F(">> "#var" = "));\
			CONSOLE.println((uint16_t)var);\
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

#define REPORT_INT8(var) \
	{\
		static int8_t reported_value = 0;\
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

static uint16_t limit_uint16(uint16_t v, uint16_t min, uint16_t max)
{
	if(v < min)
		return min;
	if(v > max)
		return max;
	return v;
}

static int16_t limit_int16(int16_t v, int16_t min, int16_t max)
{
	if(v < min)
		return min;
	if(v > max)
		return max;
	return v;
}

static int32_t limit_int32(int32_t v, int32_t min, int32_t max)
{
	if(v < min)
		return min;
	if(v > max)
		return max;
	return v;
}

} // namespace idpm
