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
#include "ipdm_io.h"
#include <Wire.h>

namespace ipdm
{

constexpr uint8_t REG_INPUT = 0;
constexpr uint8_t REG_OUTPUT = 2;
constexpr uint8_t REG_INVERT = 4;
constexpr uint8_t REG_CONFIG = 6;

struct IpdmPca9539
{

	uint8_t addr;
	union {
		struct {
			uint8_t low;
			uint8_t high;
		} conf8;
		uint16_t conf16;
	};
	union {
		struct {
			uint8_t low;
			uint8_t high;
		} out8;
		uint16_t out16;
	};

	IpdmPca9539(uint8_t addr): addr(addr)
	{
		Wire.begin();
		out16 = 0;
	}

	void pinMode(uint8_t pin, uint8_t mode)
	{
		/*CONSOLE.print("IpdmPca9539::pinMode(pin=");
		CONSOLE.print(pin);
		CONSOLE.print(", mode=");
		CONSOLE.print(mode);
		CONSOLE.println(")");*/

		if(pin >= 16)
			return;

		if(mode == OUTPUT){
			conf16 &= ~(1<<pin);
		} else {
			conf16 |= (1<<pin);
		}
		set_value(addr, REG_CONFIG, conf8.low);
		set_value(addr, REG_CONFIG+1, conf8.high);
	}

	void digitalWrite(uint8_t pin, bool state)
	{
		/*CONSOLE.print("IpdmPca9539::digitalWrite(pin=");
		CONSOLE.print(pin);
		CONSOLE.print(", state=");
		CONSOLE.print(state);
		CONSOLE.println(")");*/

		if(pin >= 16)
			return;

		// Set output state
		if(state){
			out16 |= (1<<pin);
		} else {
			out16 &= ~(1<<pin);
		}
		set_value(addr, REG_OUTPUT, out8.low);
		set_value(addr, REG_OUTPUT+1, out8.high);

		// Set output mode if not already set
		if(conf16 & (1<<pin)){
			conf16 &= ~(1<<pin);
			set_value(addr, REG_CONFIG, conf8.low);
			set_value(addr, REG_CONFIG+1, conf8.high);
		}
	}

	bool digitalRead(uint8_t pin)
	{
		if(pin >= 16)
			return LOW;

		if(!(conf16 & (1<<pin))){
			// Is output
			return out16 & (1<<pin);
		}

		if(pin < 8){
			uint8_t d = get_value(addr, REG_INPUT);
			return d & (1<<pin);
		} else {
			uint8_t d = get_value(addr, REG_INPUT+1);
			return d & (1<<(pin-8));
		}
	}

	void set_value(uint8_t addr, uint8_t reg, uint8_t value)
	{
		Wire.beginTransmission(addr);
		Wire.write(reg);
		Wire.write(value);
		uint8_t r = Wire.endTransmission();
		if(r){
			CONSOLE.print("WARNING: IpdmPca9539: set_value(): Wire.endTransmission returned ");
			CONSOLE.println(r);
		}
	}

	uint16_t get_value(uint8_t addr, uint8_t reg)
	{
		Wire.beginTransmission(addr);
		Wire.write(reg);
		uint8_t r = Wire.endTransmission();
		if(r){
			CONSOLE.print("WARNING: IpdmPca9539: get_value(): Wire.endTransmission returned ");
			CONSOLE.println(r);
			return 0;
		}
		// TODO: Change to Wire.requestFrom(addr, (uint8_t)1)
		if(Wire.requestFrom((int)addr, 1) != 1){
			CONSOLE.println("WARNING: IpdmPca9539: get_value(): Wire.requestFrom returned !=1");
			return 0;
		}
		return Wire.read();
	}
};

} // namespace ipdm
