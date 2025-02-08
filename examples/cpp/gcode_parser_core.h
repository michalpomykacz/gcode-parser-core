#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

/// C API function to parse G-code from a string.
char *parse_gcode_c(const char *gcode);

/// C API function to parse G-code from a file.
char *parse_gcode_from_file_c(const char *file_path);

/// C API function to free a string allocated by `parse_gcode_c` or `parse_gcode_from_file_c`.
void free_string(char *s);

}  // extern "C"
