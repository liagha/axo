#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>

int64_t file_write(int64_t file, uint8_t* buffer, uint64_t length) {
    return write(file, buffer, length);
}

uint64_t file_read(int64_t file, uint8_t* buffer, uint64_t count) {
    return read(file, buffer, count);
}

int64_t file_open(const char* path, int64_t flags, int64_t mode) {
    return open(path, flags, mode);
}

int64_t file_close(int64_t file) {
    return close(file);
}

int64_t file_unlink(const char* path) {
    return unlink(path);
}

int64_t file_seek(int64_t file, int64_t offset, int64_t whence) {
    return lseek(file, offset, whence);
}

