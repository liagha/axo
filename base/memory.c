#include <stdint.h>
#include <stdlib.h>
#include <sys/mman.h>

void* allocate_memory(uint64_t size) {
    return malloc(size);
}

void free_memory(void* pointer) {
    free(pointer);
}

void* reallocate_memory(void* pointer, uint64_t size) {
    return realloc(pointer, size);
}

uint8_t* memory_map(uint8_t* address, uint64_t length, int64_t protection, int64_t flags, int64_t file, int64_t offset) {
    return mmap(address, length, protection, flags, file, offset);
}

int64_t memory_unmap(uint8_t* address, uint64_t length) {
    return munmap(address, length);
}
