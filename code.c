int putchar(int value);
int print_int(int value);
int fib(int n);

int fib(int n) {
    if (n <= 2) return 1;
    return fib(n - 1) + fib(n - 2);
}

int print_int(int value) {
    if (value < 10) {
        putchar(value + 48);
        return 0;
    }
    print_int(value / 10);
    putchar(value % 10 + 48);
    return 0;
}

int main() {
    for (int i = 0; i < 40; i = i + 1) {
        print_int(fib(i));
        putchar(10);
    }
    return 0;
}