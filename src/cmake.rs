use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct CMakePreset {
    name: String,
    #[serde(default = "default_hidden")]
    hidden: bool,
}

impl CMakePreset {
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CMakePresets {
    // verision: u32,
    #[allow(non_snake_case)]
    configurePresets: Vec<CMakePreset>,
}

impl CMakePresets {
    pub fn new<T>(path: T) -> Result<Self, Box<dyn std::error::Error>> 
    where 
        T: Into<PathBuf>, {
        let path = path.into();

        let path = if path.ends_with("CMakePresets.json") {
            path
        } else {
            path.join("CMakePresets.json")
        };

        let content = std::fs::read_to_string(path)?;
        let presets: CMakePresets = serde_json::from_str(&content)?;

        Ok(presets)
    }

    pub fn get_preset(&self, name: &str) -> Option<&CMakePreset> {
        self.configurePresets.iter().find(|p| p.name == name)
    }
}

fn default_hidden() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cmake_preset() {
        let presets = CMakePresets::new("test").unwrap();
        let preset = presets.get_preset("default").expect("Failed to get preset default");
        assert_eq!(preset.get_name(), "default");
    }
}
