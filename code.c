int putchar(char c);
int print_int(int value);

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
    char a = 1;
    char b = 4;
    char c = 8;
    char d = 16;
    char e = 38;
    for (char i = 0; i < 4; i = i + 1) {
        char* value = &a - i;
        print_int(*value);
        putchar(10);
    }
    return 0;
}