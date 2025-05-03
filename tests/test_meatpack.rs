#[cfg(test)]
mod tests {
    use gcode_parser_core::meatpack::*;

    // Test a single packed byte using nibble-based packing.
    // For example, to encode "G1" with the default table (MEAT_PACK):
    // 'G' is at index 13 and '1' is at index 1.
    // In the default branch the decoder outputs:
    //   table[c & 0xF] then table[c >> 4].
    // To get "G1", we need c such that:
    //   c & 0xF == 13 (for 'G')
    //   c >> 4 == 1  (for '1')
    // That is: c = (1 << 4) | 13 = 0x1D.
    #[test]
    fn test_basic_nibble_packing() {
        let input = [0x1D];
        let expected = b"G1";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test that two consecutive newlines are collapsed to a single newline.
    // In the default table, newline is at index 12.
    // If both nibbles equal 12, the packed byte is: (12 << 4) | 12 = 0xCC.
    // The first newline is appended, the second is skipped.
    #[test]
    fn test_newline_collapsing() {
        let input = [0xCC];
        let expected = b"\n";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test the full-width sequence branch.
    // When a byte 0xFF is encountered (and it isn’t part of a command preamble),
    // the next two bytes are appended as full-width characters.
    #[test]
    fn test_full_width_sequence() {
        let input = [0xFF, b'A', b'B'];
        let expected = b"AB";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test the branch for high nibble full-width.
    // If (c & 0xF0) == 0xF0 then:
    //   first output: table[c & 0xF]
    //   second output: the next byte (full width)
    // For example, if c = 0xF1 then:
    //   table[c & 0xF] = table[1] = b'1'
    // and if the next byte is 'Z', the output should be "1Z".
    #[test]
    fn test_high_nibble_full_width() {
        let input = [0xF1, b'Z'];
        let expected = b"1Z";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test the branch for low nibble full-width.
    // If (c & 0xF) == 0xF then:
    //   first output: the next byte (full width)
    //   second output: table[c >> 4]
    // For example, if c = 0x2F then:
    //   c >> 4 = 2 so table[2] = b'2'
    // and if the next byte is 'Y', the output should be "Y2".
    #[test]
    fn test_low_nibble_full_width() {
        let input = [0x2F, b'Y'];
        let expected = b"Y2";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test the command preamble that disables "nospace" mode.
    // The command sequence is: 0xFF, 0xFF, DISABLE_NOSPACES.
    // Since DISABLE_NOSPACES (246) resets the table to MEAT_PACK (the default),
    // the subsequent packed data should decode as usual.
    // For example, using the packed byte 0x1D to output "G1".
    #[test]
    fn test_disable_nospaces_command() {
        let input = [0xFF, 0xFF, DISABLE_NOSPACES, 0x1D];
        let expected = b"G1";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test the command preamble that enables "nospace" mode.
    // ENABLE_NOSPACES (247) sets the table to MEAT_PACK_NOSP.
    // In MEAT_PACK, index 11 is a space (' '), but in MEAT_PACK_NOSP index 11 is 'E'.
    // For example, consider a packed byte 0x1B:
    //   lower nibble: 0xB (11) and upper nibble: 0x1.
    // With MEAT_PACK the output would be " 1" (space then '1'),
    // but with MEAT_PACK_NOSP it becomes "E1".
    #[test]
    fn test_enable_nospaces_command() {
        let input = [0xFF, 0xFF, ENABLE_NOSPACES, 0x1B];
        let expected = b"E1";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test toggling packing off and then back on.
    // When packing is disabled (with DISABLE_PACKING), bytes are copied directly.
    // Then re-enabling packing (with ENABLE_PACKING) resumes nibble decoding.
    // For example, raw bytes "RAW" should be output as-is,
    // followed by a packed byte 0x1D decoding to "G1".
    #[test]
    fn test_disable_and_enable_packing() {
        let input: Vec<u8> = [
            &[0xFF, 0xFF, DISABLE_PACKING][..],
            b"RAW",
            &[0xFF, 0xFF, ENABLE_PACKING, 0x1D][..],
        ]
            .concat();
        let expected = b"RAWG1";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Test a complex mixed sequence that combines several branches.
    // The sequence is:
    //   1. Packed nibble: 0x1D -> "G1"
    //   2. Full width sequence: 0xFF, 'X', 'Y' -> "XY"
    //   3. High nibble branch: 0xF1, 'Z' -> "1Z"
    //   4. Low nibble branch: 0x2F, 'W' -> "W2"
    //   5. Disable packing command, then raw "abc"
    //   6. Enable packing command, then packed byte 0x1D -> "G1"
    // Expected output: "G1XY1ZW2abcG1"
    #[test]
    fn test_mixed_sequence() {
        let input: Vec<u8> = [
            &[0x1D][..],
            &[0xFF, b'X', b'Y'][..],
            &[0xF1, b'Z'][..],
            &[0x2F, b'W'][..],
            &[0xFF, 0xFF, DISABLE_PACKING][..],
            b"abc",
            &[0xFF, 0xFF, ENABLE_PACKING, 0x1D][..],
        ]
            .concat();
        let expected = b"G1XY1ZW2abcG1";
        let output = decode_meatpack(&input).unwrap();
        assert_eq!(output, expected);
    }

    // Error tests:

    // Test that a command preamble missing its command byte produces an error.
    #[test]
    fn test_error_command_preamble_incomplete() {
        let input = [0xFF, 0xFF];
        let result = decode_meatpack(&input);
        assert!(result.is_err());
    }

    // Test that a full-width sequence with insufficient bytes produces an error.
    #[test]
    fn test_error_full_width_incomplete() {
        let input = [0xFF, b'A']; // Missing one more byte
        let result = decode_meatpack(&input);
        assert!(result.is_err());
    }

    // Test that a high-nibble branch missing the following full-width byte produces an error.
    #[test]
    fn test_error_high_nibble_incomplete() {
        let input = [0xF1]; // Should be followed by one byte
        let result = decode_meatpack(&input);
        assert!(result.is_err());
    }

    // Test that a low-nibble branch missing the following full-width byte produces an error.
    #[test]
    fn test_error_low_nibble_incomplete() {
        let input = [0x2F]; // Should be followed by one byte
        let result = decode_meatpack(&input);
        assert!(result.is_err());
    }
}
