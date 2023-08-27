OUTPUTFILE = target/debug/output

build:
	cargo build
run:
	cargo run
asm: run
	gcc -o  $(OUTPUTFILE) output.s
	./$(OUTPUTFILE)
test:
	gcc -o  $(OUTPUTFILE) code.c
	./$(OUTPUTFILE)
cmp:
	gcc -o  $(OUTPUTFILE) output.s
	./$(OUTPUTFILE)
	