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

typedef struct Inner {
    int z;
} Inner;

struct Point
{
    int x;
    int y;
    Inner inner;
};

int main()
{
    struct Point *points = (struct Point*)malloc(sizeof(struct Point) * 10);
    for (int i = 0; i < 10; i = i + 1)
    {
        struct Point tmp = {.x = i, .y = i * 33, .inner = {.z = i * 2}};
        tmp.inner = (struct Inner){.z = 5};
        points[i] = tmp;
    }

    for (int i = 0; i < 10; i = i + 1) {
        struct Point tmp = points[i];
        puts("X: ");
        print_int(tmp.x);
        puts("\nY: ");
        print_int(tmp.y);
        puts("\nY: ");
        print_int(tmp.inner.z);
        putchar('\n');
        putchar('\n');
    }
    free(points);
    return 0;
}