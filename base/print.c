#include <stdio.h>
#include <stdint.h>

void print_integer(int64_t value) {
    printf("%ld", value);
}

void print_float(double value) {
    printf("%f", value);
}

void print_boolean(int value) {
    if (value) {
        printf("true");
    } else {
        printf("false");
    }
}

void print_string(const char* string) {
    printf("%s", string);
}

void print_character(char character) {
    putchar(character);
}

void print_newline() {
    putchar('\n');
}

void print_hexadecimal(int64_t value) {
    printf("0x%lx", value);
}

void print_pointer(void* pointer) {
    printf("%p", pointer);
}

