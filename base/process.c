#include <stdint.h>
#include <unistd.h>

void process_exit(int64_t status) {
    _exit(status);
}

