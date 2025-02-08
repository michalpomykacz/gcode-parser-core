import gcode_parser_core

def main():
    # Parsing G-code from a string
    gcode = "G1 X10 Y10\nG1 X20 Y20"
    metadata = gcode_parser_core.parse_gcode(gcode)
    print("Parsed metadata from string:", metadata)

    # Parsing G-code from a file
    metadata_from_file = gcode_parser_core.parse_gcode_from_file("../example.gcode")
    print("Parsed metadata from file:", metadata_from_file)

if __name__ == "__main__":
    main()
