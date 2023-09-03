typedef struct Point
{
    int x;
    int y;
} Point;

Point create()
{
    Point p = {.x = 5, .y = 3};
    return p;
}

int main()
{
    return create().x + create().y;
}