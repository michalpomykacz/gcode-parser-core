use std::fs::File;
use std::io::{self, Read};

/// Parses G-code from a file by reading its content into a string.
pub fn parse_gcode_from_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(parse_gcode_metadata(&content))
}

/// Existing function to parse G-code from a string.
pub fn parse_gcode_metadata(gcode: &str) -> String {
    // Your existing parsing logic
    format!("Parsed G-code: {}", gcode)
}
