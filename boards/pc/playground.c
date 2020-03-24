#include <stdio.h>
#include <unistd.h>
int main() {
    printf("Hi, I'm a playground with PID %d\n", getpid());
    return 0;
}