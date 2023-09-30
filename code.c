#include "test.h"

typedef struct Point {
    int x;
    int y;
} Point;

int main() {
    Point* array = DYNLIST(Point);
    for (int i = 0; i < 10; i = i + 1) {
        Point p = {.x = i, .y = i * 33};
        DYNLIST_PUSH(array, p);
    }
    int length = DYNLIST_LEN(array);    
    DYNLIST_FREE(array);
    return length;
}