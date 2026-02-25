//! # cmakr
//!
//! A Rust library for programmatically invoking CMake configure and build steps.
//!
//! `cmakr` provides a builder-pattern API to construct and execute CMake commands,
//! with support for CMake presets, custom defines, output directory configuration,
//! and both synchronous and asynchronous execution.
//!
//! ## Quick Start
//!
//! ```no_run
//! use cmakr::Cmd;
//!
//! // Synchronous build
//! let result = Cmd::default()
//!     .set_path("./my_project")
//!     .set_preset("default")
//!     .set_binary_path("./build")
//!     .set_output_path("./bin")
//!     .add_define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
//!     .build();
//!
//! assert!(result.is_ok());
//! ```
//!
//! ```no_run
//! use cmakr::Cmd;
//!
//! // Asynchronous build
//! let rx = Cmd::default()
//!     .set_path("./my_project")
//!     .set_preset("default")
//!     .spawn();
//!
//! let result = rx.recv().unwrap();
//! assert!(result.is_ok());
//! ```

pub mod cmake;
pub mod cmd;

pub use cmd::Cmd;
