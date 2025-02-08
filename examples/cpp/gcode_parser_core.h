#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

/// A C API function to parse a G-code string. The input is a C string and
/// the return value is a newly allocated C string which the caller is responsible
/// for freeing using `free_string`.
///
/// # Safety
/// This function uses raw pointers.
char *parse_gcode_c(const char *gcode);

/// A C API helper function to free strings allocated by `parse_gcode_c`.
///
/// # Safety
/// The pointer must have been allocated by `parse_gcode_c`.
void free_string(char *s);

}  // extern "C"
