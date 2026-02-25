# cmakr

A Rust library for programmatically invoking CMake configure and build steps.

Designed for use in Cargo `build.rs` scripts, `cmakr` provides a builder-pattern API to compile C/C++ CMake projects and link the resulting artifacts into your Rust binary.

## Features

- Builder pattern API for constructing CMake invocations
- CMake Presets support (`CMakePresets.json`)
- Separate source, build, and output directory configuration
- Custom `-D` variable definitions
- Synchronous (`build()`) and asynchronous (`spawn()`) execution
- Automatic directory creation for build and output paths
- Windows `\\?\` path normalization for cross-platform compatibility

## Requirements

- [CMake](https://cmake.org/) available on `PATH`
- A build system backend (e.g., [Ninja](https://ninja-build.org/), Make) if specified by your preset

## Installation

Add `cmakr` to your `build-dependencies` in `Cargo.toml`:

```toml
[build-dependencies]
cmakr = "0.1.0"
```

## Usage

### Basic `build.rs` Example

```rust
// build.rs
use cmakr::Cmd;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let result = Cmd::default()
        .set_path(".")                                          // CMake source directory
        .set_preset("default")                                  // CMakePresets.json preset name
        .set_binary_path(&format!("{}/cmake-build", out_dir))   // Intermediate build files
        .set_output_path(&format!("{}/cmake-out", out_dir))     // Final artifacts (libs, binaries)
        .add_define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")      // Custom -D definitions
        .build();

    if let Err(e) = result {
        panic!("CMake build failed: {}", e);
    }

    // Tell cargo where to find the compiled library
    println!("cargo::rustc-link-search=native={}/cmake-out", out_dir);
    println!("cargo::rustc-link-lib=dylib=my_lib");

    // Re-run if sources change
    println!("cargo::rerun-if-changed=CMakeLists.txt");
    println!("cargo::rerun-if-changed=src/my_lib.c");
}
```

### Asynchronous Execution

Use `spawn()` to run CMake in a background thread:

```rust
use cmakr::Cmd;

let rx = Cmd::default()
    .set_path("./my_project")
    .set_preset("default")
    .spawn();

// Do other work while cmake runs...

let result = rx.recv().unwrap();
match result {
    Ok(()) => println!("Build succeeded"),
    Err(e) => eprintln!("Build failed: {}", e),
}
```

### [Demo of `cmakr` and `bindgen`](https://github.com/CoraBlack/cmakr-demo)

## API Reference

### `Cmd`

| Method | Description |
|--------|-------------|
| `Cmd::default()` | Creates a new builder with default settings |
| `.set_path(path)` | Sets the CMake source directory (`-S`). Default: `"."` |
| `.set_binary_path(path)` | Sets the build directory (`-B`). Default: `"build"` |
| `.set_output_path(path)` | Sets the artifact output directory. Default: `"build"` |
| `.set_preset(name)` | Uses a preset from `CMakePresets.json` (`--preset=`) |
| `.add_define(name, value)` | Adds a CMake cache variable (`-D<name>=<value>`) |
| `.add_arg(arg)` | Adds an extra argument to the CMake command |
| `.build()` | Runs configure + build synchronously, returns `Result` |
| `.spawn()` | Runs configure + build in a background thread, returns `Receiver` |

### Execution Steps

When `build()` or `spawn()` is called, `cmakr` performs two CMake invocations:

1. **Configure** - `cmake -S <source> -B <binary> [--preset=<name>] [-D...] [args]`
2. **Build** - `cmake --build <binary> [args]`

The `output_path` is applied via `CMAKE_RUNTIME_OUTPUT_DIRECTORY`, `CMAKE_LIBRARY_OUTPUT_DIRECTORY`, and `CMAKE_ARCHIVE_OUTPUT_DIRECTORY`.

## CMakePresets.json

`cmakr` reads presets from a standard `CMakePresets.json` file in the source directory. Example:

```json
{
    "version": 4,
    "configurePresets": [
        {
            "name": "default",
            "displayName": "Default",
            "generator": "Ninja",
            "cacheVariables": {
                "CMAKE_BUILD_TYPE": "Debug"
            }
        }
    ]
}
```

Hidden presets (`"hidden": true`) are excluded from lookup.

## Example Project

The `test/` directory contains a complete working example: a CMake shared library linked into a Rust binary via `build.rs`. See `test/build.rs` for the integration pattern.

## License

MIT
