int puts(char* string);

int main()
{
    char a[] = {'H', 'i', '!', 0};
    puts(a);
    *(a + 1) = 5;
    puts(a);
    return 0;
}