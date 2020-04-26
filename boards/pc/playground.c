#include <stdio.h>
#include "led.h"

int main() {
    setbuf(stdout, NULL); // auto-flush after each printf

    // quick and dirty sleeper
    int i = 0;
    while (i++ < 1000000000) 
        if (i % 100000000 == 0)
            printf(".\n");

    printf("There are %d LED(s) on this board\n", led_count());

    
    
    led_on(0);
    led_off(0);
    led_off(0);
    led_toggle(0);
    led_on(0);

    return 0;
}

/* 

Notes:
 - look into protocol buffer if json is too hard
 - copy led.c into playground.c and inline the command function there
 - might be a problem using printf because tock also uses printf
 - _start function will be different in the future
 - friday Tock call if done by then.

*/