#include <inttypes.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

enum Syscall { COMMAND, SUBSCRIBE, ALLOW };

int command(uint32_t driver, uint32_t command, int data, int arg2) {
    char inbox[32] = {0};
    fprintf(stderr, "%d,%u,%u,%d,%d\n", COMMAND, driver, command, data, arg2);
    int r = read(STDIN_FILENO, inbox, sizeof(inbox) / sizeof(char) - 1);
    int response = atoi(inbox);
    printf("Read %d bytes and got a message from Rust: %u\n", r, response);
    
    return 0;
}
