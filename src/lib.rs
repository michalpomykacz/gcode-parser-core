mod parser;

use std::ffi::CString;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;

#[cfg(feature = "python")]
#[pyfunction]
fn parse_gcode(gcode: &str) -> PyResult<String> {
    let metadata = parser::parse_gcode_metadata(gcode);
    Ok(metadata)
}

// The Python module is only compiled if the "python" feature is enabled.
#[cfg(feature = "python")]
#[pymodule]
fn gcode_parser_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_gcode, m)?)?;
    Ok(())
}

/// C API function to parse G-code, available regardless of Python support.
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

/// C API function to free a string allocated by `parse_gcode_c`.
#[no_mangle]
pub extern "C" fn free_string(s: *mut std::os::raw::c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}
