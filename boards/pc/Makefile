export TOCK_KERNEL_VERSION := $(shell git describe --always 2> /dev/null || echo "1.3+")

ifndef DEBUG
CARGO_FLAGS += --release
endif

all: kernel playground

# No $^ to not include header files in gcc (which would cause pragma warnings)
playground: playground.c libsimc/internal/alarm.h libsimc/internal/alarm_internal.c libsimc/alarm.h libsimc/alarm_timer.c libsimc/timer.h libsimc/led.c libsimc/led.h libsimc/tock.h libsimc/tock.c
	gcc -o $@ playground.c libsimc/internal/alarm_internal.c libsimc/alarm_timer.c libsimc/led.c libsimc/tock.c

kernel:
	cargo build $(CARGO_FLAGS)

run: playground
	cargo run $(CARGO_FLAGS)

@PHONY: clean
clean:
	cargo clean
	rm playground
