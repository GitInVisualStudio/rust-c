void* calloc(int size, int n);
void* realloc(void* ptr, int size);
void free(void* ptr);

#define DEFAULT_SIZE 128

typedef struct DynList {
    long capacity;
    long length;
} DynList;

#define DYNLIST(type) ((type*) (calloc(sizeof(DynList) + DEFAULT_SIZE * sizeof(type), 1) + sizeof(long) * 2))

#define DYNLIST_CAST(list) ((DynList*)(((char*)list) - sizeof(long) * 2))

#define DYNLIST_LEN(list) (DYNLIST_CAST(list)->length)
#define DYNLIST_CAP(list) (DYNLIST_CAST(list)->capacity)
#define DYNLIST_FREE(list) (free(DYNLIST_CAST(list)))

#define DYNLIST_PUSH(list, element) {\
    DynList* l = DYNLIST_CAST(list);\
    if (l->capacity == 0) {\
        l->capacity = DEFAULT_SIZE;\
    }\
    if (l->length == l->capacity) {\
        list = (typeof(list))realloc(l, sizeof(DynList) + l->capacity * 2) + sizeof(long) * 2;\
        l = DYNLIST_CAST(list);\
        l->capacity = l->capacity * 2;\
    }\
    list[l->length] = element;\
    l->length = l->length + 1;\
}

