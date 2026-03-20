#include <stdint.h>
#include <string.h>
#include <ctype.h>
#include <stdlib.h>

uint64_t string_length(const char* string) {
    return strlen(string);
}

uint8_t character_at(const char* string, uint64_t index) {
    return (uint8_t)string[index];
}

int is_whitespace(uint8_t character) {
    return isspace(character);
}

int is_digit(uint8_t character) {
    return isdigit(character);
}

const char* string_substring(const char* string, uint64_t start, uint64_t end) {
    uint64_t length = end - start;
    char* result = malloc(length + 1);
    memcpy(result, string + start, length);
    result[length] = '\0';
    return result;
}

double parse_float(const char* string) {
    return strtod(string, NULL);
}

