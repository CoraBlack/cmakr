use std::{path::PathBuf, sync::mpsc::{self, Receiver}, thread};

use crate::cmake::{CMakePresets};

type ExecResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub struct Cmd {
    args: Vec<String>,
    path: Option<PathBuf>,
    binary_path: PathBuf,
    output_path: PathBuf,
    preset:  Option<String>,
    defines: Vec<String>,
}

impl Cmd {
    pub fn new() -> Self {
        Self { 
            args: Vec::new(),
            path: None,
            binary_path: PathBuf::from("build"),
            output_path: PathBuf::from("build"),
            preset:  None,
            defines: Vec::new(),
        }
    }

    pub fn add_arg<T>(mut self, arg: T) -> Self 
    where 
        T: Into<String>, {
        self.args.push(arg.into());
        self
    }

    pub fn set_path<T>(mut self, path: T) -> Self 
    where 
        T: Into<String>, {
        self.path = Some(PathBuf::from(path.into()));
        self
    }

    pub fn set_binary_path<T>(mut self, path: T) -> Self 
    where 
        T: Into<String>, {
        self.binary_path = PathBuf::from(path.into());
        self
    }

    pub fn set_output_path<T>(mut self, path: T) -> Self 
    where 
        T: Into<String>, {
        self.output_path = PathBuf::from(path.into());
        self
    }

    pub fn set_preset<T>(mut self, preset: T) -> Self 
    where 
        T: Into<String>, {
        self.preset = Some(preset.into());
        self
    }

    pub fn add_define<T>(mut self, define: T) -> Self 
    where 
        T: Into<String>, {
        self.defines.push(format!("-D{}", define.into()));
        self
    }

    pub fn build(&mut self) -> ExecResult {
        self.execute()
    } 

    pub fn spawn(mut self) -> Receiver<ExecResult> {
        let (tx, rx) = mpsc::channel();

        let tx = tx.clone();
        thread::spawn(move || {
            let _ = tx.send(self.execute());
        });

        rx
    }

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
        let mut preset_arg = String::new();
        if let Some(preset_name) = &self.preset {
            let presets = CMakePresets::new(&cmake_path).expect("Failed to get cmake presets");
            let Some(preset) = presets.get_preset(preset_name) else {
                panic!("preset {} not found", preset_name);
            };

            preset_arg = format!("--preset={}", preset.get_name());
        }

        // binary path and output path must be exists, if not exists, create it
        check_dir_exists_and_create(&self.binary_path)?;
        check_dir_exists_and_create(&self.output_path)?;
        let mut output_path_args = Vec::new();
        output_path_args.push(format!("-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={}", self.output_path.canonicalize().unwrap().to_str().unwrap()));
        output_path_args.push(format!("-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={}", self.output_path.canonicalize().unwrap().to_str().unwrap()));
        output_path_args.push(format!("-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={}", self.output_path.canonicalize().unwrap().to_str().unwrap()));
    
        // configure cmake
        let status = std::process::Command::new("cmake")
            .args(["-S", cmake_path.to_str().unwrap()])
            .args(["-B", self.binary_path.to_str().unwrap()])
            .arg(preset_arg)
            .args(self.defines.clone())
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

fn check_dir_exists_and_create(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn execute_cmake() {
        let cmd = Cmd::new()
        .set_path("./test/")
        .set_preset("default")
        .build();

        assert_eq!(cmd.is_ok(), true);
    }

    #[test]
    fn spawn_cmake() {
        let rx = Cmd::new()
        .set_path("./test/")
        .set_binary_path("./build")
        .set_output_path("./bin")
        .set_preset("default")
        .spawn();

        let result = rx.recv().unwrap();
        assert_eq!(result.is_ok(), true);
    }
}