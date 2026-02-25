/// Build script that uses `cmakr` to compile the C shared library
/// before the Rust binary is built.
///
/// This demonstrates how to integrate `cmakr` into a Cargo build pipeline.
/// The compiled shared library (`test_lib`) is placed into `OUT_DIR`,
/// and the Rust linker is instructed to search there.
use cmakr::Cmd;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Use cmakr to configure and build the C shared library.
    // The CMake source is in the same directory as this build script.
    let result = Cmd::default()
        .set_path(".")
        .set_preset("default")
        .set_binary_path(&format!("{}/cmake-build", out_dir))
        .set_output_path(&format!("{}/cmake-out", out_dir))
        .build();

    if let Err(e) = result {
        panic!("CMake build failed: {}", e);
    }

    // Tell cargo to look for shared libraries in the output directory.
    println!("cargo::rustc-link-search=native={}/cmake-out", out_dir);

    // Link the shared library built by CMake.
    println!("cargo::rustc-link-lib=dylib=test_lib");

    // Re-run this build script if the C source files change.
    println!("cargo::rerun-if-changed=func.c");
    println!("cargo::rerun-if-changed=func.h");
    println!("cargo::rerun-if-changed=CMakeLists.txt");
    println!("cargo::rerun-if-changed=CMakePresets.json");
}
