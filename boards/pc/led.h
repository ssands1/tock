#include "tock.h"

#define DRIVER_NUM_LEDS 0x00000002
#define NUM_LEDS 1
#define ERROR -1

int led_on(int led_num);
int led_off(int led_num);
int led_toggle(int led_num);

// Returns the number of LEDs on the host platform.
int led_count(void);