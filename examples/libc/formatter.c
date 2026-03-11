#include <stdio.h>
#include <stdint.h>

void print_i64(int64_t v) { printf("%ld", v); }
void print_f64(double v)  { printf("%f", v); }
void print_bool(int v)    { printf("%s", v ? "true" : "false"); }

void print_str(const char* s) { printf("%s", s); }
void print_char(char c)       { putchar(c); }
void println()                { putchar('\n'); }

void print_hex(int64_t v) { printf("0x%lx", v); }
void print_ptr(void* p)   { printf("%p", p); }

void println_i64(int64_t v)   { printf("%ld\n", v); }
void println_f64(double v)    { printf("%f\n", v); }
void println_bool(int v)      { printf("%s\n", v ? "true" : "false"); }
void println_str(const char* s) { printf("%s\n", s); }