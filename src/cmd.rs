//! CMake command builder and executor.
//!
//! This module provides the [`Cmd`] struct, a builder-pattern API for constructing
//! and running CMake configure + build steps. It supports synchronous execution
//! via [`Cmd::build`] and asynchronous execution via [`Cmd::spawn`].

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::cmake::{CMakePresets, Defination};

/// The result type returned by CMake execution methods.
///
/// Returns `Ok(())` on success, or an error describing the failure
/// (e.g., cmake not found, configure/build failure, I/O error).
type ExecResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// A builder for constructing and executing CMake commands.
///
/// `Cmd` uses a builder pattern to configure CMake invocation parameters
/// such as source path, build directory, output directory, presets, and
/// custom defines. Once configured, call [`build`](Cmd::build) for synchronous
/// execution or [`spawn`](Cmd::spawn) for asynchronous execution in a background thread.
///
/// The execution performs two steps:
/// 1. **Configure** - runs `cmake -S <source> -B <binary> [--preset=<name>] [defines] [args]`
/// 2. **Build** - runs `cmake --build <binary> [args]`
///
/// # Defaults
///
/// | Field | Default |
/// |-------|---------|
/// | `path` (source directory) | Current directory (`"."`) |
/// | `binary_path` (build directory) | `"build"` |
/// | `output_path` (artifact output) | `"build"` |
/// | `preset` | None |
///
/// # Example
///
/// ```no_run
/// use cmakr::Cmd;
///
/// let result = Cmd::default()
///     .set_path("./my_project")
///     .set_binary_path("./build")
///     .set_output_path("./bin")
///     .set_preset("release")
///     .add_define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
///     .add_arg("-Wno-dev")
///     .build();
///
/// match result {
///     Ok(()) => println!("Build succeeded"),
///     Err(e) => eprintln!("Build failed: {}", e),
/// }
/// ```
pub struct Cmd {
    /// Extra arguments passed to both configure and build steps.
    args: Vec<String>,
    /// CMake source directory (passed as `-S`). Defaults to `"."`.
    path: Option<PathBuf>,
    /// CMake build directory (passed as `-B`). Defaults to `"build"`.
    binary_path: PathBuf,
    /// Output directory for built artifacts (`CMAKE_RUNTIME_OUTPUT_DIRECTORY`,
    /// `CMAKE_LIBRARY_OUTPUT_DIRECTORY`, `CMAKE_ARCHIVE_OUTPUT_DIRECTORY`).
    /// Defaults to `"build"`.
    output_path: PathBuf,
    /// Optional CMake preset name (passed as `--preset=<name>`).
    preset: Option<String>,
    /// Custom CMake variable definitions (passed as `-D<name>=<value>`).
    defines: Vec<Defination>,
}

impl Cmd {
    /// Creates a new [`Cmd`] with default settings.
    ///
    /// The default configuration uses `"build"` as both the binary and output
    /// directory paths. No source path, preset, or custom defines are set.
    pub fn default() -> Self {
        Self {
            args: Vec::new(),
            path: None,
            binary_path: PathBuf::from("build"),
            output_path: PathBuf::from("build"),
            preset: None,
            defines: Vec::new(),
        }
    }

    /// Adds an extra argument to be passed to the CMake command.
    ///
    /// These arguments are appended to both the configure and build steps.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument string (e.g., `"-Wno-dev"`, `"--log-level=WARNING"`).
    pub fn add_arg<T>(mut self, arg: T) -> Self
    where
        T: Into<String>,
    {
        self.args.push(arg.into());
        self
    }

    /// Sets the CMake source directory.
    ///
    /// This is the directory containing `CMakeLists.txt` and optionally
    /// `CMakePresets.json`. Passed to CMake as `-S <path>`.
    ///
    /// If not set, defaults to the current working directory (`"."`).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the source directory.
    pub fn set_path<T>(mut self, path: T) -> Self
    where
        T: Into<String>,
    {
        self.path = Some(PathBuf::from(path.into()));
        self
    }

    /// Sets the CMake build (binary) directory.
    ///
    /// This is where CMake generates build system files and intermediate
    /// artifacts. Passed to CMake as `-B <path>`. The directory is created
    /// automatically if it does not exist.
    ///
    /// Defaults to `"build"`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the build directory.
    pub fn set_binary_path<T>(mut self, path: T) -> Self
    where
        T: Into<String>,
    {
        self.binary_path = PathBuf::from(path.into());
        self
    }

    /// Sets the output directory for final build artifacts.
    ///
    /// This configures `CMAKE_RUNTIME_OUTPUT_DIRECTORY`,
    /// `CMAKE_LIBRARY_OUTPUT_DIRECTORY`, and `CMAKE_ARCHIVE_OUTPUT_DIRECTORY`
    /// so that executables, shared libraries, and static libraries are placed
    /// in the specified directory. The directory is created automatically if
    /// it does not exist.
    ///
    /// Defaults to `"build"`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output directory.
    pub fn set_output_path<T>(mut self, path: T) -> Self
    where
        T: Into<String>,
    {
        self.output_path = PathBuf::from(path.into());
        self
    }

    /// Sets the CMake preset to use.
    ///
    /// The preset name is looked up in the `CMakePresets.json` file located
    /// in the source directory. Hidden presets are excluded from lookup.
    /// Passed to CMake as `--preset=<name>`.
    ///
    /// # Arguments
    ///
    /// * `preset` - The name of the configure preset.
    pub fn set_preset<T>(mut self, preset: T) -> Self
    where
        T: Into<String>,
    {
        self.preset = Some(preset.into());
        self
    }

    /// Adds a CMake cache variable definition.
    ///
    /// Passed to CMake as `-D<name>=<value>` during the configure step.
    ///
    /// # Arguments
    ///
    /// * `define` - The variable name (e.g., `"CMAKE_BUILD_TYPE"`).
    /// * `value` - The variable value (e.g., `"Release"`).
    pub fn add_define<T, U>(mut self, define: T, value: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.defines.push(Defination {
            name: define.into(),
            value: value.into(),
        });
        self
    }

    /// Executes CMake configure and build synchronously.
    ///
    /// This consumes the builder and runs the full CMake workflow
    /// (configure + build) in the current thread, blocking until completion.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `cmake` is not found on `PATH`
    /// - The preset name is invalid or not found
    /// - The configure step fails (non-zero exit code)
    /// - The build step fails (non-zero exit code)
    /// - Any I/O error occurs (directory creation, path resolution, etc.)
    pub fn build(mut self) -> ExecResult {
        self.execute()
    }

    /// Executes CMake configure and build asynchronously in a background thread.
    ///
    /// This consumes the builder and spawns a new thread to run the full CMake
    /// workflow. Returns a [`Receiver`] that will receive the result once the
    /// build completes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cmakr::Cmd;
    ///
    /// let rx = Cmd::default()
    ///     .set_path("./my_project")
    ///     .set_preset("default")
    ///     .spawn();
    ///
    /// // Do other work while cmake runs...
    ///
    /// let result = rx.recv().unwrap();
    /// match result {
    ///     Ok(()) => println!("Build succeeded"),
    ///     Err(e) => eprintln!("Build failed: {}", e),
    /// }
    /// ```
    pub fn spawn(mut self) -> Receiver<ExecResult> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let _ = tx.send(self.execute());
        });

        rx
    }

    /// Internal method that performs the actual CMake configure and build.
    ///
    /// This method:
    /// 1. Verifies that `cmake` is available on `PATH`.
    /// 2. Resolves the preset (if set) from `CMakePresets.json`.
    /// 3. Creates build and output directories if they don't exist.
    /// 4. Runs `cmake -S <source> -B <binary>` with all configured arguments.
    /// 5. Runs `cmake --build <binary>` to compile the project.
    fn execute(&mut self) -> ExecResult {
        // check cmake is exists in path
        if which::which("cmake").is_err() {
            panic!("cmake not found in path");
        }

        // add path arg if path is set
        let cmake_path = match &self.path {
            Some(path) => path.clone(),
            None => PathBuf::from("."),
        };

        // add preset arg if preset is set
        let mut preset_args: Vec<String> = Vec::new();
        if let Some(preset_name) = &self.preset {
            let presets = CMakePresets::new(&cmake_path).expect("Failed to get cmake presets");
            let Some(preset) = presets.get_preset(preset_name) else {
                return Err(format!("preset {} not found", preset_name).into());
            };

            preset_args.push(format!("--preset={}", preset.get_name()));
        }

        // binary path and output path must be exists, if not exists, create it
        check_dir_exists_and_create(&self.binary_path)?;
        check_dir_exists_and_create(&self.output_path)?;
        let output_dir = normalize_path(&self.output_path.canonicalize()?);
        let output_path_args = vec![
            format!("-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={}", output_dir),
            format!("-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={}", output_dir),
            format!("-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={}", output_dir),
        ];

        // configure cmake
        let status = std::process::Command::new("cmake")
            .args(["-S", cmake_path.to_str().unwrap()])
            .args(["-B", self.binary_path.to_str().unwrap()])
            .args(&preset_args)
            .args(
                self.defines
                    .iter()
                    .map(|d| format!("-D{}={}", d.name, d.value)),
            )
            .args(output_path_args)
            .args(self.args.clone())
            .status()?;

        if !status.success() {
            return Err(format!("cmake configure failed with status: {}", status).into());
        }

        // build cmake
        let status = std::process::Command::new("cmake")
            .arg("--build")
            .arg(self.binary_path.clone())
            .args(self.args.clone())
            .status()?;

        if !status.success() {
            return Err(format!("cmake build failed with status: {}", status).into());
        }

        Ok(())
    }
}

/// Ensures a directory exists, creating it (and any parent directories) if necessary.
///
/// # Errors
///
/// Returns an I/O error if directory creation fails.
fn check_dir_exists_and_create(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Converts a path to a normalized string, stripping the Windows `\\?\` extended-length
/// prefix if present. This is necessary because some tools (e.g., GCC's linker) do not
/// recognize UNC-style paths produced by [`std::path::Path::canonicalize`] on Windows.
fn normalize_path(path: &std::path::Path) -> String {
    let s = path.to_str().unwrap_or_default();
    s.strip_prefix(r"\\?\").unwrap_or(s).to_string()
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn execute_cmake() {
        let cmd = Cmd::default()
            .set_path("./test/")
            .set_preset("default")
            .build();

        assert_eq!(cmd.is_ok(), true);
    }

    #[test]
    fn spawn_cmake() {
        let rx = Cmd::default()
            .set_path("./test/")
            .set_binary_path("./build")
            .set_output_path("./bin")
            .spawn();

        let result = rx.recv().unwrap();
        assert_eq!(result.is_ok(), true);
    }
}
