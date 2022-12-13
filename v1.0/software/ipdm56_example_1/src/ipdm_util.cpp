#include "ipdm_util.h"
#include "ipdm_library.h"

namespace ipdm
{

unsigned long timestamp_age(unsigned long timestamp_ms)
{
	return (millis() - timestamp_ms) * get_active_clock_divider();
}

bool timestamp_younger_than(unsigned long timestamp_ms, unsigned long max_age)
{
	// Timestamp is assumed to be initialized to 0. This means that a timestamp
	// of 0 is infinitely old.
	if(timestamp_ms == 0)
		return false;
	return timestamp_age(timestamp_ms) < max_age;
}

} // namespace ipdm
