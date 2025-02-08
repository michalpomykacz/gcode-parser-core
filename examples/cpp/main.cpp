#include <iostream>
#include "gcode_parser_core.h"

int main() {
    // A sample G-code string.
    const char* gcode = "G1 X10 Y10\nG1 X20 Y20";

    // Call the Rust function via the C API.
    char* result = parse_gcode_c(gcode);

    if (result != nullptr) {
        std::cout << "Parsed metadata: " << result << std::endl;
        // Free the allocated string.
        free_string(result);
    } else {
        std::cerr << "Error: Parsing failed." << std::endl;
    }

    return 0;
}
