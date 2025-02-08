import gcode_parser_core

def main():
    gcode = "G1 X10 Y10\nG1 X20 Y20"
    metadata = gcode_parser_core.parse_gcode(gcode)
    print("Parsed metadata:", metadata)

if __name__ == "__main__":
    main()