/// Parses the G-code input and extracts metadata.
///
/// Currently, this is a dummy implementation that returns the first line of the input.
/// In a real-world scenario, you would parse the entire G-code file and extract the
/// necessary metadata.
pub fn parse_gcode_metadata(gcode: &str) -> String {
    // For illustration, return the first line prefixed with a message.
    let first_line = gcode.lines().next().unwrap_or("No data");
    format!("Parsed metadata: {}", first_line)
}
