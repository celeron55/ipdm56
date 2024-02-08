#include "ipdm_util.h"
#include "ipdm_library.h"

namespace ipdm
{

uint32_t longtime_counter = 0;

uint32_t longtime()
{
	// Base our time on Arduino's millis(), but scale it according to the
	// prescaler
	static uint32_t last_millis = 0;
	// This millis() here should be the only one you can find in your
	// ipdm_library based program. Everything else should use idpm::longtime().
	const uint32_t now_millis = millis();
	const uint32_t t = now_millis - last_millis;
	uint32_t divider = get_active_clock_divider();
	uint32_t actually_elapsed = t / divider;
	if(actually_elapsed > 0){
		longtime_counter += actually_elapsed;
		last_millis = now_millis;
	}
	return longtime_counter;
}

unsigned long timestamp_age(unsigned long timestamp)
{
	return longtime() - timestamp;
}

bool timestamp_younger_than(unsigned long timestamp, unsigned long max_age)
{
	// Timestamp is assumed to be initialized to 0. This means that a timestamp
	// of 0 is infinitely old.
	if(timestamp == 0)
		return false;
	return timestamp_age(timestamp) < max_age;
}

void time_loop()
{
	// Call longtime so that ongoing alterations to the clock prescaler are
	// taken into account even while longtime() isn't being called by any user
	// code
	(void)longtime();
}

} // namespace ipdm
