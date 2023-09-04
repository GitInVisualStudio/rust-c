# rust-c
c compiler implemented in bad rust code

compiles **C Code** to [x86 Assembly/GNU assembly](https://en.wikibooks.org/wiki/X86_Assembly/GNU_assembly_syntax).

instead of actually studying for my exams i decided to learn some Rust. And the best way to learn things is just to do them. Therefore, don't expect too much from the source code as this is my first project in Rust. 
# usage
you have to have gcc installed

this will compile the code in ```code.c``` and execute it
```
  make asm
```
# features

**Variables**
* primitives: char, int, long
```c
int main() {
  int foo = 5;
  char bar = 3;
  return foo + bar;
}
```
**structs & typedef**
```c
typedef char bool;
typedef struct Foo {
  bool value;
} Foo;

struct Bar {
  Foo foo;
};
```
**pointer**
* also supportes pointer arithmeic (altho all pointer are treated as char*)
```c
typedef struct Point {
  int x;
  int y;
} Point;

int sum(Point* p) {
  return p->x + p->y; //(*p).x also works
}
```
**functions**
* up to 6 parameter (all defined types are supported)
```c
Point origin() {
  return {
    .x = 0,
    .y = 0
  }
}
```
**basics**
* for-, while-, if-statements
* operators: ```&& || >= <= > < + - * / % &(ref) *(deref)```
* just simple assignments ```var = expression```
* arrays and pointers are treated the same way, but there are array epxressions like: ```int array[] = {1, 2, 3, 4};```
  
**limitations**
* there are no type casts! (void* does convert implicitly tho, same as all primitive types)
* structs as parameters & return value are implemented in a super weird way, not like c does it.
* also expressions can only use up to 6 registers
* i somewhat tested the compiler but i am sure there are many unknown bugs
