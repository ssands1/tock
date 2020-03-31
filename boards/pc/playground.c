#include <stdio.h>
#include <unistd.h>
int main() {
    setbuf(stdout, NULL); // auto-flush after each printf

    char inbox[32] = { 0 };
    int r = read(STDIN_FILENO, inbox, sizeof(inbox)/sizeof(char) - 1);
    printf("Read %d bytes and got a message from Rust: %s", r, inbox);
    
    // quick and dirty sleeper
    int i = 0;
    while (i++ < 1000000000) 
        if (i % 100000000 == 0)
            printf(".\n");

    printf("All done\n");
    
    return 0;
}