void putchar(char c);
void print_int(int value);
void print_int(int value) {
    if (value < 10) {
        putchar(value + '0');
        return;
    }
    print_int(value / 10);
    putchar(value % 10 + '0');
}

typedef struct Node
{
    int value;
    void *next;
} Node;

typedef struct LinkedList
{
    Node *first;
    Node *last;
} LinkedList;

Node *malloc(int size);
void free(Node *ptr);

void push(LinkedList *list, int value)
{
    Node *node = malloc(sizeof(Node));
    node->value = value;

    if (list->first == 0)
    {
        list->first = node;
        list->last = node;
        return;
    }
    list->last->next = node;
    list->last = node;
}

void test() {
    LinkedList list;
    list.first = 0;
    list.last = 0;

    for (int i = 90; i < 100; i = i + 1) {
        push(&list, i);
    }

    Node* current = list.first;
    while (current) {
        print_int(current->value);
        putchar(10);
        Node* next = current->next;
        free(current);
        current = next;
    }
}

void main()
{
    test();
}