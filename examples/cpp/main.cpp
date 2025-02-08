#include <stdio.h>
#include "gcode_parser_core.h"

int main() {
    // Parse G-code from a string
    char* result = parse_gcode_c("G1 X10 Y10\nG1 X20 Y20");
    if (result) {
        printf("Parsed metadata from string: %s\n", result);
        free_string(result);
    }

    // Parse G-code from a file
    char* file_result = parse_gcode_from_file_c("../example.gcode");
    if (file_result) {
        printf("Parsed metadata from file: %s\n", file_result);
        free_string(file_result);
    }

    return 0;
}
