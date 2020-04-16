export TOCK_KERNEL_VERSION := $(shell git describe --always 2> /dev/null || echo "1.3+")

ifndef DEBUG
CARGO_FLAGS += --release
endif

all: kernel playground

playground: playground.c led.c led.h tock.h tock.c
	gcc -o $@ $^

kernel:
	cargo build $(CARGO_FLAGS)

run: playground
	cargo run $(CARGO_FLAGS)

@PHONY: clean
clean:
	cargo clean
	rm playground
