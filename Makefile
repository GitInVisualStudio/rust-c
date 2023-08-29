OUTPUTFILE = target/debug/output

build:
	cargo build
run:
	cargo run
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
	