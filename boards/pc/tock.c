#include <inttypes.h>
#include <stdio.h>

int command(uint32_t driver, uint32_t command, int data, int arg2) {
    fprintf(stderr, "%u,%u,%d,%d\n", driver, command, data, arg2);
    return 0;
}

// enum { count, on, off, toggle };
// int leds[NUM_LEDS];
    // if (driver != DRIVER_NUM_LEDS) {
    //     printf("Error: only LED supported\n");
    //     return ERROR;
    // }
    // assert(data >= 0 && data < NUM_LEDS);

    // switch (command) {
    //     case count:
    //         return NUM_LEDS;
    //     case on:
    //         leds[data] = on;
    //         break;
    //     case off:
    //         leds[data] = !on;
    //         break;
    //     case toggle:
    //         leds[data] = !leds[data];
    //         break;
    //     default:
    //         return ERROR;
    // }
    // printf(leds[data] ? "LED ON!\n" : "led off\n");