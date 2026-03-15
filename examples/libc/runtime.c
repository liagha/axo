#include <stdint.h>

const uint8_t* str_to_ptr(const char* s) {
    return (const uint8_t*)s;
}

uint8_t* int_to_ptr(int64_t v) {
    return (uint8_t*)(uintptr_t)v;
}

int64_t ptr_to_int(const uint8_t* p) {
    return (int64_t)(uintptr_t)p;
}

uint64_t int_to_u64(int64_t v) {
    return (uint64_t)v;
}

uint8_t int_to_u8(int64_t v) {
    return (uint8_t)v;
}
