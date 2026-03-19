#include <stdint.h>

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

