#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "tock.h"

typedef enum { COMMAND, SUBSCRIBE, YIELD } Syscall_t;

typedef struct {
    subscribe_cb *cb;
    int arg0;
    int arg1;
    int arg2;
    void* ud;
} tock_task_t;

#define TASK_QUEUE_SIZE  16
static tock_task_t task_queue[TASK_QUEUE_SIZE];
static int task_cur  = 0;
static int task_last = 0;

// TODO: Use dedicated pipe instead of stderr
long make_call(Syscall_t sys, uint32_t driver, uint32_t subdriver, size_t arg3, 
                                                                  size_t arg4) {
    fprintf(stderr, "%u,%u,%u,%zu,%zu\n", sys, driver, subdriver, arg3, arg4);
    // printf("%u,%u,%u,%zu,%zu\n", sys, driver, subdriver, arg3, arg4);

    char inbox[32] = {0};
    int num_bytes = read(STDIN_FILENO, inbox, sizeof(inbox) / sizeof(char) - 1);
    if (num_bytes <= 0) {
        printf("ERROR: couldn't read properly");
        return -1;
    }

    long response = atol(inbox);
    if (response < 0) printf("Warning: syscall returned %ld\n", response);
    // printf("Read %d bytes; got a number from Rust: %ld\n", num_bytes, response);
    
    return response;
}

int tock_enqueue(subscribe_cb cb, int arg0, int arg1, int arg2, void* ud) {
    int next_task_last = (task_last + 1) % TASK_QUEUE_SIZE;
    if (next_task_last == task_cur) {
        return -1;
    }

    task_queue[task_last].cb   = cb;
    task_queue[task_last].arg0 = arg0;
    task_queue[task_last].arg1 = arg1;
    task_queue[task_last].arg2 = arg2;
    task_queue[task_last].ud   = ud;
    task_last = next_task_last;

    return task_last;
}

void yield_for(bool *cond) {
    while (!*cond) {
        yield();
    }
}

// TODO: Verify correctness
void yield(void) {
    if (task_cur != task_last) {
        tock_task_t task = task_queue[task_cur];
        task_cur = (task_cur + 1) % TASK_QUEUE_SIZE;
        task.cb(task.arg0, task.arg1, task.arg2, task.ud);
    } else {
        long callback = make_call(YIELD, 0, 0, 0, 0);
    }
}

int command(uint32_t driver, uint32_t subdriver, int data, int arg2) {
    return make_call(COMMAND, driver, subdriver, (size_t) data, (size_t) arg2);
}

int subscribe(uint32_t driver, uint32_t subdriver, subscribe_cb cb, void* userdata) {
    return make_call(SUBSCRIBE, driver, subdriver, (size_t) cb, (size_t) userdata);
}