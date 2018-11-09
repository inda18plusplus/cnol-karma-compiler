

run:
	@clear
	@mkdir -p out
	@cargo run ../karma/simple.kar > out/out.ll
	@llc -filetype=obj out/out.ll
	@gcc -no-pie out/out.o -o out/a.out
	@./out/a.out

build:
	@cargo build
