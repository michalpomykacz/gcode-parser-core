/// The 15 common characters used for MeatPack encoding.
pub const MEAT_PACK: [u8; 15] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'.', b' ', b'\n', b'G', b'X',
];

/// The 15 common characters used when whitespace is removed.
pub const MEAT_PACK_NOSP: [u8; 15] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'.', b'E', b'\n', b'G', b'X',
];

pub const DISABLE_NOSPACES: u8 = 246;
pub const ENABLE_NOSPACES: u8 = 247;
pub const DISABLE_PACKING: u8 = 250;
pub const ENABLE_PACKING: u8 = 251;

/// Decodes a MeatPack-compressed buffer.
///
/// The input is expected to follow the MeatPack protocol:
///   - When packing is enabled, two characters are packed per byte using a lookup table.
///   - A special byte 0xFF is used to signal control commands or full-width (unpacked) characters.
///   - There is also support for a “nospace” table and turning packing on/off.
///
/// In the output, double newlines are collapsed (as in the Python version).
pub fn decode_meatpack(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut packing = true;
    let mut table = &MEAT_PACK;
    let mut skip = 0;
    let mut prev: Option<u8> = None;
    let len = input.len();
    let mut i = 0;

    while i < len {
        if skip > 0 {
            skip -= 1;
            i += 1;
            continue;
        }
        let c = input[i];

        // Check for the special command preamble: two 0xFF bytes in a row.
        if c == 0xFF && (i + 1 < len && input[i + 1] == 0xFF) {
            if i + 2 >= len {
                return Err("Unexpected end of input after command preamble".to_string());
            }
            let cmd = input[i + 2];
            skip = 2; // Skip the next two bytes that we’ve just processed.
            match cmd {
                DISABLE_NOSPACES => {
                    table = &MEAT_PACK;
                }
                ENABLE_NOSPACES => {
                    table = &MEAT_PACK_NOSP;
                }
                DISABLE_PACKING => {
                    packing = false;
                }
                ENABLE_PACKING => {
                    packing = true;
                }
                _ => return Err(format!("Unsupported command {}", cmd)),
            }
            i += 1;
            continue;
        }

        // If packing is disabled, simply copy the byte.
        if !packing {
            output.push(c);
            prev = Some(c);
            i += 1;
            continue;
        }

        // When packing is enabled, check for the different packing modes.

        // 1. If the byte is 0xFF (but not a command preamble), then the next two bytes are full-width.
        if c == 0xFF {
            if i + 2 >= len {
                return Err("Unexpected end of input after 0xFF in packed mode".to_string());
            }
            append_packed(&mut output, &mut prev, input[i + 1]);
            append_packed(&mut output, &mut prev, input[i + 2]);
            skip = 2;
            i += 1;
            continue;
        }
        // 2. If the high nibble is 0xF0, then the lower nibble indexes into the table and the next byte is full width.
        if c & 0xF0 == 0xF0 {
            if i + 1 >= len {
                return Err("Unexpected end of input after high nibble flag".to_string());
            }
            let lower_nibble = c & 0xF;
            // Since c is not 0xFF here, lower_nibble is in 0..15.
            append_packed(&mut output, &mut prev, table[lower_nibble as usize]);
            append_packed(&mut output, &mut prev, input[i + 1]);
            skip = 1;
            i += 1;
            continue;
        }
        // 3. If the low nibble is 0x0F, then the next byte is full width and the high nibble indexes into the table.
        if c & 0xF == 0xF {
            if i + 1 >= len {
                return Err("Unexpected end of input after low nibble flag".to_string());
            }
            append_packed(&mut output, &mut prev, input[i + 1]);
            let upper_nibble = c >> 4;
            append_packed(&mut output, &mut prev, table[upper_nibble as usize]);
            skip = 1;
            i += 1;
            continue;
        }
        // 4. Otherwise, both nibbles are packable—each nibble indexes into the current table.
        let lower_nibble = c & 0xF;
        let upper_nibble = c >> 4;
        append_packed(&mut output, &mut prev, table[lower_nibble as usize]);
        append_packed(&mut output, &mut prev, table[upper_nibble as usize]);
        i += 1;
    }

    Ok(output)
}

/// Appends a byte to the output, ignoring double newlines (0x0A).
fn append_packed(output: &mut Vec<u8>, prev: &mut Option<u8>, byte: u8) {
    // If the byte is a newline (10) and the previous byte was also newline, skip it.
    if byte == b'\n' && prev == &Some(b'\n') {
        return;
    }
    output.push(byte);
    *prev = Some(byte);
}