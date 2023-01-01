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

// This file (src/params.h) implements a global parameter database according to
// user application's definitions in param_def.h

#pragma once
#include <stdint.h>
#include <Stream.h>

struct Modules {
	#define MODULE_DEF(name, timeout_ms) \
		struct { \
			uint8_t timeout_counter = 255; /* incremented at 100ms interval, 255=dead */ \
			void timeout_reset(){ \
				timeout_counter = 0; \
			} \
			void run_timeout(Stream &stream){ \
				if((timeout_ms) == 0 || timeout_counter == 255) return; \
				timeout_counter++; \
				if(timeout_counter > ((timeout_ms) / 100)){ \
					timeout_counter = 255; \
					stream.println(F("-!- " #name " timed out")); \
				} \
			} \
			bool alive(){ \
				return timeout_counter != 255; \
			} \
		} name;
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis)
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF

	void run_timeouts(Stream &stream);
};

extern Modules modules;

struct Params {
	#define MODULE_DEF(name, timeout_ms)
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis) \
		struct { \
			type value = (default_value); \
			const type& set(const type &new_value){ \
				value = new_value; \
				/* This could be enabled to have automatic reset on set() */ \
				/*modules.module_name.timeout_counter = 0;*/ \
				return value; \
			} \
			void clear_timed_out_value(){ \
				if(modules.module_name.timeout_counter == 255) \
					value = (default_value); \
			} \
			bool alive(){ \
				return modules.module_name.timeout_counter != 255; \
			} \
			void report_value(Stream &stream){ \
				stream.print(F("-!- " #module_name "_" #name " = ")); \
				stream.println(value); \
			} \
		} module_name##_##name;
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF

	// Shall be called at 100ms interval
	void clear_timed_out_values();

	void report_if_changed(Stream &stream);

	void report_all_values(Stream &stream);
};

extern Params params;

