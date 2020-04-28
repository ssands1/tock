#include <stdio.h>
#include "libsimc/led.h"
#include "libsimc/alarm.h"

int main() {
    setbuf(stdout, NULL); // auto-flush after each printf

    printf("There are %d LED(s) on this board\n", led_count());
    
    led_on(0);
    led_off(0);

    // quick and dirty sleeper
    int i = 0;
    while (i++ < 1000000000) 
        if (i % 100000000 == 0)
            printf(".\n");

    led_off(0);
    led_toggle(0);
    led_on(1);

    // TODO: look at examples to fix this
    alarm_at(3, NULL, NULL, NULL);

    return 0;
}
