
int fib(int n) {
    if (n <= 1) 
        return 1;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    typedef struct Test {
        int x;
        int y;
    } Test;
    Test a;
    Test* ptr = &a;
    ptr->y = 3;
    ptr->x = 33;
    int buf = 8623487;
    return a.x + a.y + fib(7);
}