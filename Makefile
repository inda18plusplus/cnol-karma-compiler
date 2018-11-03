
KARMA_SOURCE = karma/sum.kar

run: build
	@cargo run $(KARMA_SOURCE)

build:
	@clear
	@cargo build
