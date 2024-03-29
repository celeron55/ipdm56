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

// Parameter: timestamp: millis() timestamp
// NOTE: Keep in mind this value will overflow at 4294967296ms
unsigned long timestamp_age(unsigned long timestamp);

// NOTE: Keep in mind this uses the value that will overflow at 4294967296ms
bool timestamp_younger_than(unsigned long timestamp, unsigned long max_age);

static bool ENM_compare_and_update(unsigned long &t0, const unsigned long &interval)
{
	bool trigger_now = timestamp_age(t0) >= interval;
	if(trigger_now)
		t0 = millis();
	return trigger_now;
}

#define EVERY_N_MILLISECONDS(ms) for(static unsigned long t0 = 0; ::ipdm::ENM_compare_and_update(t0, ms); )

static void print_timestamp(Stream &dst, uint32_t t)
{
	char format_buf[17];
	int ms = t % 1000;
	t /= 1000;
	int s = t % 60;
	t /= 60;
	int m = t % 60;
	t /= 60;
	int h = t;
	if(h == 0 && m == 0)
		snprintf(format_buf, sizeof format_buf, "%02i.%03is: ", s, ms);
	else if(h == 0)
		snprintf(format_buf, sizeof format_buf, "%02im%02i.%03is: ", m, s, ms);
	else
		snprintf(format_buf, sizeof format_buf, "%02ih%02im%02i.%03is: ", h, m, s, ms);
	dst.print(format_buf);
}

static void util_print_timestamp(Stream &dst)
{
	print_timestamp(dst, millis());
}

// Called by ipdm::loop()
void time_loop();

// Deprecated
static uint32_t longtime() { return millis(); }

} // namespace idpm

