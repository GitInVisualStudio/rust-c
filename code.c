void putchar(char c);

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

int push(LinkedList *list, int value)
{
    Node *node = malloc(12);
    node->value = value;

    if (list->first == 0)
    {
        list->first = node;
        list->last = node;
        return 0;
    }
    list->last->next = node;
    list->last = node;
    return 0;
}

int main()
{
    LinkedList list;
    list.first = 0;
    list.last = 0;

    for (int i = 0; i < 10; i = i + 1) {
        push(&list, i);
    }

    Node* current = list.first;
    while (current) {
        putchar(current->value+ '0');
        putchar(10);
        Node* next = current->next;
        free(current);
        current = next;
    }

    return 0;
}