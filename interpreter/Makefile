
KARMA_SOURCE = ../karma/sum.kar

run: build
	@cargo run $(KARMA_SOURCE)

bench: 
	@echo "=== SUM ==="
	@echo "123456789" | cargo run --features benchmark --release ../karma/sum.kar
	@echo "=== SUM ==="
	@echo 
	@echo 
	@echo 

build:
	@clear
	@cargo build

build-release:
	@clear
	@cargo build --features benchmark --release
