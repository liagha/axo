#include <stdlib.h>
#include <stdbool.h>

typedef struct {
    void** items;
    size_t capacity;
    size_t total;
} Vector;

Vector* vector_create() {
    Vector* vector = malloc(sizeof(Vector));

    if (!vector) {
        return NULL;
    }

    vector->capacity = 8;
    vector->total = 0;
    vector->items = malloc(sizeof(void*) * vector->capacity);

    if (!vector->items) {
        free(vector);
        return NULL;
    }

    return vector;
}

size_t vector_count(Vector* vector) {
    return vector->total;
}

static bool vector_resize(Vector* vector, size_t capacity) {
    void** items = realloc(vector->items, sizeof(void*) * capacity);

    if (items) {
        vector->items = items;
        vector->capacity = capacity;
        return true;
    }
    
    return false;
}

bool vector_push(Vector* vector, void* item) {
    if (vector->capacity == vector->total) {
        if (!vector_resize(vector, vector->capacity * 2)) {
            return false;
        }
    }
    
    vector->items[vector->total++] = item;

    return true;
}

bool vector_set(Vector* vector, size_t index, void* item) {
    if (index < vector->total) {
        vector->items[index] = item;
        return true;
    }
    return false;
}

void* vector_get(Vector* vector, size_t index) {
    if (index < vector->total) {
        return vector->items[index];
    }
    
    return NULL;
}

bool vector_delete(Vector* vector, size_t index) {
    if (index >= vector->total) {
        return false;
    }

    for (size_t i = index; i < vector->total - 1; i++) {
        vector->items[i] = vector->items[i + 1];
    }

    vector->total--;

    if (vector->total > 0 && vector->total <= vector->capacity / 4) {
        vector_resize(vector, vector->capacity / 2);
    }

    return true;
}

void vector_free(Vector* vector) {
    free(vector->items);
    free(vector);
}

