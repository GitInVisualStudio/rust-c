void putchar(char c);

int print_string(char* string) {
    while (*string) {
        putchar(*string);
        string = string + 1;
    }
    return 0;
}

typedef struct Inner {
    char a;
    long b;
} Inner;


typedef struct Point
{
    int x;
    int y;
    Inner i;
} Point;

int change(Point *p)
{
    p->x = 5;
    p->y = 3;
    p->i.a = 'H';
    p->i.b = 49;
    return 0;
}

int print_point(Point* p) {
    print_string("X: ");
    putchar(p->x + '0');
    print_string(" Y: ");
    putchar(p->y + '0');
    putchar(10);
    print_string("Inner: ");
    putchar(p->i.a);
    putchar(p->i.b);
    putchar(10);
    return 0;
}

int test() {
    Point p;
    change(&p);
    print_point(&p);


    Point* c = &p;
    c[0] = p;
    print_point(c);

    return c->x + c->y;
}

int main()
{
    return test();
}