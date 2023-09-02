typedef struct Point {
    int x;
    int y;
} Point;

int change(Point* p) {
    (*p).x = 5;
    (*p).y = 3;
    return 0;
}

int main() {
    Point p;
    change(&p);
    return p.x + p.y;
}