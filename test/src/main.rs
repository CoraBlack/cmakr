// Example binary demonstrating FFI calls to the C shared library
// built by `cmakr` in `build.rs`.
//
// The `test_lib` shared library exports `hello()` and `add()`,
// which are called here via Rust's FFI mechanism.

unsafe extern "C" {
    // Prints "Hello, world!" to stdout.
    fn hello();

    // Returns the sum of two integers.
    fn add(a: i32, b: i32) -> i32;
}

fn main() {
    println!("Calling C functions from test_lib via FFI:");

    unsafe { hello() };

    let result = unsafe { add(3, 4) };
    println!("add(3, 4) = {}", result);
}
