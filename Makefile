OUTPUTFILE = target/debug/output
COMPILER = target/debug/rust-compiler

run:
	cargo run code.c output.s -ast
build:
	cargo build
tokens:
	cargo run code.c output.s -tokens
asm: run
	gcc -o  $(OUTPUTFILE) output.s
	./$(OUTPUTFILE)
test:
	gcc -S -o output.s code.c
	gcc -o $(OUTPUTFILE) output.s
	./$(OUTPUTFILE)
cmp:
	gcc -o  $(OUTPUTFILE) output.s
	./$(OUTPUTFILE)
run-test: build
	python3 tests/test.py  $(COMPILER)

valgrind: run
	gcc -o  $(OUTPUTFILE) output.s
	valgrind $(OUTPUTFILE)