#include <stdio.h>
#include <stdint.h>

void print_integer(int64_t value) {
    printf("%ld\n", value);
    fflush(stdout);
}

void print_float(double value) {
    printf("%f\n", value);
    fflush(stdout);
}

void print_boolean(int value) {
    if (value) {
        printf("true");
    } else {
        printf("false");
    }
    fflush(stdout);
}

void print_string(const char* string) {
    printf("%s\n", string);
    fflush(stdout);
}

void print_character(char character) {
    putchar(character);
    fflush(stdout);
}

void print_newline() {
    putchar('\n');
    fflush(stdout);
}

void print_hexadecimal(int64_t value) {
    printf("0x%lx\n", value);
    fflush(stdout);
}

void print_pointer(void* pointer) {
    printf("%p\n", pointer);
    fflush(stdout);
}
