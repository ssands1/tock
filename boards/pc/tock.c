#include <inttypes.h>
#include <stdio.h>

enum Syscall { COMMAND, SUBSCRIBE, ALLOW, YIELD, MEMOP };

int command(uint32_t driver, uint32_t command, int data, int arg2) {
    fprintf(stderr, "%d,%u,%u,%d,%d\n", COMMAND, driver, command, data, arg2);
    return 0;
}
