
KARMA_SOURCE = ../karma/sum.kar

run:
	@cargo run --release $(KARMA_SOURCE)

debug:
	@cargo run --release --features debug -- $(KARMA_SOURCE)

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
