# gcode-parser-core

`gcode-parser-core` is a Rust library for parsing G-code files and extracting metadata. It exposes both Python bindings (via PyO3) and a C API for integration with C/C++ projects.

## Prerequisites

- **Rust Toolchain:** Install from [rustup](https://rustup.rs/).
- **Python (3.7+):** To use the Python bindings.
- **Maturin:** For building and installing the Python extension.
  ```sh
  pip install maturin
  ```

## Develop
```sh
maturin develop
```
builds the crate and installs it as a python module directly in the current virtualenv.


## Building
```sh
cargo build --release
```


## Python example
The Python bindings are provided via PyO3. To build and install the Python module, use maturin.

```sh
maturin develop --release
```

## C++ Example
The C API is defined in gcode_parser_core.h and implemented in Rust. Given example demonstrates how to call the API from C++.

### Ensure the Rust Library is Built:
When building for your C++ use case, disable the default features so that the PyO3 code (and thus the dependency on Python) isn’t compiled in:
```sh
cargo build --release --no-default-features
```
Verify that the library file (e.g., libgcode_parser_core.so on Linux) is located in the target/release directory.

### Compile the C++ Example:
```sh
g++ -std=c++11 -Iinclude examples/cpp/main.cpp -Ltarget/release -Wl,-rpath=$(pwd)/target/release -lgcode_parser_core -o examples/cpp/example_cpp
```
- The -Iinclude flag tells the compiler where to find the header.
- The -Ltarget/release flag tells the linker where to find the compiled shared library.
- The -lgcode_parser_core flag links against the shared library (make sure the library name matches your build artifact).
- -Wl,-rpath flag: Instructs the linker to embed the path to the shared library into your executable.

### Run the C++ Example:
```sh
./examples/cpp/example_cpp
```

To generate new headers for the C API, run:
```sh
cargo install cbindgen
cbindgen --config examples/cpp/cbindgen.toml --crate gcode-parser-core --output examples/cpp/gcode_parser_core.h
```
