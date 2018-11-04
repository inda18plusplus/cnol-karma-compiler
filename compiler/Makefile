

run:
	@mkdir -p out
	@cargo run > out/out.ll
	@llc -filetype=obj out/out.ll
	@gcc -no-pie out/out.o -o out/a.out
	@./out/a.out

build:
	@cargo build
