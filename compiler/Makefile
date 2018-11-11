
# 

run:
	@clear
	@mkdir -p out
	@cargo run ../karma/simple.kar > out/out.ll
	@opt -O3 -S out/out.ll -o out/opt.ll
	@llc -filetype=obj out/opt.ll
	@gcc -no-pie out/opt.o -o out/a.out
	@echo Done! Running...
	@./out/a.out

build:
	@cargo build
