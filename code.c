void *malloc(long size);
void free(void *ptr);
void putchar(char c);
void print_int(int i);

void print_int(int i) {
    if (i < 10) {
        putchar(i + '0');
        return;
    }
    print_int(i / 10);
    putchar(i % 10 + '0');
}

struct Point
{
    int x;
    int y;
};

int print_string(char *s)
{
    while (*s)
    {
        putchar(*s);
        s = s + 1;
    }
    return 0;
}

int main()
{
    struct Point *points = malloc(sizeof(struct Point) * 10);
    for (int i = 0; i < 10; i = i + 1)
    {
        struct Point tmp = {.x = i, .y = i * 33};
        points[i] = tmp;
    }

    for (int i = 0; i < 10; i = i + 1) {
        struct Point tmp = points[i];
        print_string("X: ");
        print_int(tmp.x);
        print_string(" Y: ");
        print_int(tmp.y);
        putchar(10);
    }
    free(points);
    return 0;
}