mod parser;

use std::ffi::CString;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;

/// Parses G-code from a string (Python function).
#[cfg(feature = "python")]
#[pyfunction]
fn parse_gcode(gcode: &str) -> PyResult<String> {
    let metadata = parser::parse_gcode_metadata(gcode);
    Ok(metadata)
}

/// Parses G-code from a file (Python function).
#[cfg(feature = "python")]
#[pyfunction]
fn parse_gcode_from_file(file_path: &str) -> PyResult<String> {
    match parser::parse_gcode_from_file(file_path) {
        Ok(metadata) => Ok(metadata),
        Err(e) => Err(pyo3::exceptions::PyIOError::new_err(e.to_string())),
    }
}

/// The Python module, only compiled if the "python" feature is enabled.
#[cfg(feature = "python")]
#[pymodule]
fn gcode_parser_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_gcode, m)?)?;
    m.add_function(wrap_pyfunction!(parse_gcode_from_file, m)?)?;
    Ok(())
}

/// C API function to parse G-code from a string.
#[no_mangle]
pub extern "C" fn parse_gcode_c(gcode: *const std::os::raw::c_char) -> *mut std::os::raw::c_char {
    use std::ffi::{CStr, CString};
    unsafe {
        if gcode.is_null() {
            return std::ptr::null_mut();
        }
        let c_str = CStr::from_ptr(gcode);
        if let Ok(str_slice) = c_str.to_str() {
            let metadata = parser::parse_gcode_metadata(str_slice);
            match CString::new(metadata) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

/// C API function to parse G-code from a file.
#[no_mangle]
pub extern "C" fn parse_gcode_from_file_c(file_path: *const std::os::raw::c_char) -> *mut std::os::raw::c_char {
    use std::ffi::{CStr, CString};
    use std::ptr;

    unsafe {
        if file_path.is_null() {
            return ptr::null_mut();
        }
        let c_str = CStr::from_ptr(file_path);
        if let Ok(path) = c_str.to_str() {
            match parser::parse_gcode_from_file(path) {
                Ok(metadata) => match CString::new(metadata) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => ptr::null_mut(),
                },
                Err(_) => ptr::null_mut(),
            }
        } else {
            ptr::null_mut()
        }
    }
}

/// C API function to free a string allocated by `parse_gcode_c` or `parse_gcode_from_file_c`.
#[no_mangle]
pub extern "C" fn free_string(s: *mut std::os::raw::c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
