
KARMA_SOURCE = karma/fib.kar

run: build
	@cargo run $(KARMA_SOURCE)

build:
	@clear
	@cargo build
