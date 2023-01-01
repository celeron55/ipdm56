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

// This file (src/params.cpp) implements a global parameter database according
// to user application's definitions in param_def.h

#include "params.h"

Modules modules;

Params params;

void Modules::run_timeouts(Stream &stream)
{
	#define MODULE_DEF(name, timeout_ms) \
		name.run_timeout(stream);
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis)
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF
}

void Params::clear_timed_out_values()
{
	#define MODULE_DEF(name, timeout_ms)
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis) \
		module_name##_##name.clear_timed_out_value();
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF
}

// Returns true on match
static bool compare_hysteresis(int a, int b, int hysteresis)
{
	return abs(a - b) < hysteresis;
}

static bool compare_hysteresis(uint16_t a, uint16_t b, int hysteresis)
{
	return abs(a - b) < hysteresis;
}

static bool compare_hysteresis(bool a, bool b, int hysteresis)
{
	return a == b;
}

void Params::report_if_changed(Stream &stream)
{
	#define MODULE_DEF(name, timeout_ms)
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis) { \
		if((report_hysteresis) != 0){ \
			static type reported_value = (default_value); \
			if(!compare_hysteresis(module_name##_##name.value, reported_value, report_hysteresis)){ \
				reported_value = module_name##_##name.value; \
				module_name##_##name.report_value(stream); \
			} \
		} \
	}
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF
}

void Params::report_all_values(Stream &stream)
{
	#define MODULE_DEF(name, timeout_ms)
	#define PARAM_DEF(module_name, name, type, default_value, report_hysteresis) { \
		module_name##_##name.report_value(stream); \
	}
	#include "../param_def.h" // Parameters defined by user application
	#undef PARAM_DEF
	#undef MODULE_DEF
}

