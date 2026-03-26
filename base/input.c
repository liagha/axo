#include <stdio.h>
#include <stdlib.h>

const char* get_input(const char* prompt) {
    if (prompt != NULL) {
        printf("%s", prompt);
        fflush(stdout);  
    }

    size_t capacity = 64;
    size_t length = 0;
    char* buffer = malloc(capacity);

    if (buffer == NULL) {
        return NULL;
    }

    int character;
    while ((character = getchar()) != '\n' && character != EOF) {
        if (length + 1 >= capacity) {
            capacity *= 2;
            char* new_buffer = realloc(buffer, capacity);
            if (new_buffer == NULL) {
                free(buffer);
                return NULL;
            }
            buffer = new_buffer;
        }
        buffer[length++] = (char)character;
    }

    buffer[length] = '\0';
    return buffer;
}

