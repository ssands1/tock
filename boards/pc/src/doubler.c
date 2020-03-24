#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#define ERROR -1

enum {R, W};

int doubler(int i) {
    // need to trap system calls from playgroud
    (void) system("./playground");
    
    int p2c[2],
        c2p[2];

    if (pipe(p2c) == ERROR || pipe(c2p) == ERROR)
        return ERROR;
    
    pid_t pid = fork();
    if (pid == ERROR) 
        return ERROR;
    if (pid == 0) { // child
        char* outbox = "hey";
        char inbox[100];
        
        close(p2c[W]); 
        close(c2p[R]); 
        int rVal = read(p2c[R], inbox, 100); 
        close(p2c[R]); 
        int wVal = write(c2p[W], outbox, strlen(outbox)+1); 
        close(c2p[W]); 
        
        if (rVal == ERROR || wVal == ERROR)
            return ERROR;

        printf("Child reads %s from parent\n", inbox); 
  
        return 1;
    } 

    // parent
    char* outbox = "hi";
    char inbox[100];
    
    close(p2c[R]);
    close(c2p[W]);
    int wVal = write(p2c[W], outbox, strlen(outbox)+1); 
    close(p2c[W]); 
    int rVal = read(c2p[R], inbox, 100); 
    close(c2p[R]); 
    
    if (rVal == ERROR || wVal == ERROR)
        return ERROR;

    printf("Parent reads %s from child\n", inbox); 

    return 2*i;
}
