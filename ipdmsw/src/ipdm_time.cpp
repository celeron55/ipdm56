#include "ipdm_util.h"
#include "ipdm_library.h"

// This is the Arduino-internal millisecond counter.
// We manipulate it according to our power save sleeps to keep track of the
// actual time that passes.
extern volatile unsigned long timer0_millis;

namespace ipdm
{

unsigned long timestamp_age(unsigned long timestamp)
{
	return millis() - timestamp;
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
	// Increment timer0_millis to account for ongoing alterations to the clock
	// prescaler
	static uint32_t last_millis = 0;
	const uint32_t now_millis = millis();
	const uint32_t millis_elapsed = now_millis - last_millis;
	uint32_t divider = get_active_clock_divider();
	uint32_t actually_elapsed = millis_elapsed / divider;
	if(actually_elapsed > millis_elapsed){
		noInterrupts();
		timer0_millis += actually_elapsed - millis_elapsed;
		interrupts();
		last_millis = now_millis;
	}
}

} // namespace ipdm
