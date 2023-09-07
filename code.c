void *malloc(long size);
void free(void *ptr);
void putchar(char c);
void print_int(int i);
void puts(char* ptr);

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

int main()
{
    struct Point *points = (struct Point*)malloc(sizeof(struct Point) * 10);
    for (int i = 0; i < 10; i = i + 1)
    {
        struct Point tmp = {.x = i, .y = i * 33};
        points[i] = tmp;
    }

    for (int i = 0; i < 10; i = i + 1) {
        struct Point tmp = points[i];
        puts("X: ");
        print_int(tmp.x);
        puts(" Y: ");
        print_int(tmp.y);
        putchar('\n');
    }
    free(points);
    return 0;
}