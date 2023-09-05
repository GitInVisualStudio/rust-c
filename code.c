void* malloc(long size);
void free(void* ptr);
void putchar(char c);

int print_string(char* s) {
    while (*s) {
        putchar(*s);
        s = s + 1;
    }
    return 0;
}

int main()
{
    int* nums = malloc(4 * 10);
    for (int i = 0; i < 10; i = i+1) {
        nums[i] = i;
    }
    for (int i = 0; i < 10; i = i+1) {
        char* string = "Number: ";
        print_string(string);
        putchar(nums[i] + '0');
        putchar(10);
    }
    free(nums);
    return 0;
}