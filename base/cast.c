#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

uint8_t* string_pointer(const char* string) {
    return (uint8_t*)string;
}

uint8_t* integer_pointer(int64_t value) {
    return (uint8_t*)value;
}

int64_t pointer_integer(uint8_t* pointer) {
    return (int64_t)pointer;
}

uint64_t integer_uint64(int64_t value) {
    return (uint64_t)value;
}

uint8_t integer_uint8(int64_t value) {
    return (uint8_t)value;
}

int32_t uint8_character(uint8_t value) {
    return (int32_t)value;
}

uint8_t character_uint8(int32_t value) {
    return (uint8_t)value;
}

int64_t character_integer(int32_t value) {
    return (int64_t)value;
}

const char* float_string(double value) {
    char* buffer = malloc(64);
    if (buffer) {
        snprintf(buffer, 64, "%f", value);
    }
    return buffer;
}

const char* pointer_string(uint8_t* pointer) {
    return (const char*)pointer;
}
