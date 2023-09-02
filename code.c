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
    (*p).x = 5;
    (*p).y = 3;
    (*p).i.a = 'H';
    (*p).i.b = 49;
    return 0;
}

void putchar(char c);

int main()
{
    Point p;
    change(&p);
    putchar(p.i.a);
    putchar(10);
    return p.x + p.y + p.i.b;
}