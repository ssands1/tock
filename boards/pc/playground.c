#include <stdio.h>
#include "libsimc/led.h"
#include "libsimc/alarm.h"

void my_cb(int a, int b, int c, void* vp) {
    printf("This is within a callback\n");
}

int main() {
    setbuf(stdout, NULL); // auto-flush after each printf

    printf("There are %d LED(s) on this board\n", led_count());
    
    led_on(0);
    led_off(0);

    // // quick and dirty sleeper
    // int i = 0;
    // while (i++ < 1000000000) 
    //     if (i % 100000000 == 0)
    //         printf(".\n");

    led_off(0);
    led_toggle(0);
    led_on(1);

    printf("\nTesting Alarm:\n");
    alarm_t alarm;
    // alarm_at(0, my_cb, NULL, &alarm);
    printf("Alarm reads: %d\n", alarm_read());

    return 0;
}
