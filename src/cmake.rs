//! CMake presets parsing and lookup.
//!
//! This module handles reading and deserializing `CMakePresets.json` files,
//! providing access to the configure presets defined within.

use std::path::PathBuf;

use serde::Deserialize;

/// A CMake variable definition consisting of a name-value pair.
///
/// Used to pass `-D<name>=<value>` arguments to the CMake configure step.
pub(crate) struct Defination {
    pub name: String,
    pub value: String,
}

/// A single CMake configure preset.
///
/// Represents one entry in the `configurePresets` array of a `CMakePresets.json` file.
/// Hidden presets (with `hidden: true`) are excluded from lookup by [`CMakePresets::get_preset`].
#[derive(Deserialize)]
pub(crate) struct CMakePreset {
    name: String,
    #[serde(default = "default_hidden")]
    hidden: bool,
}

impl CMakePreset {
    /// Returns the name of this preset.
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

/// A collection of CMake configure presets parsed from a `CMakePresets.json` file.
///
/// This struct deserializes the top-level JSON object and extracts the
/// `configurePresets` array. Unknown fields (such as `version`) are silently ignored.
///
/// # Example
///
/// ```ignore
/// use cmakr::cmake::CMakePresets;
///
/// let presets = CMakePresets::new("./my_project").unwrap();
/// if let Some(preset) = presets.get_preset("default") {
///     println!("Found preset: {}", preset.get_name());
/// }
/// ```
#[derive(Deserialize)]
pub(crate) struct CMakePresets {
    #[serde(rename = "configurePresets")]
    configure_presets: Vec<CMakePreset>,
}

impl CMakePresets {
    /// Creates a new [`CMakePresets`] by reading and parsing a `CMakePresets.json` file.
    ///
    /// # Arguments
    ///
    /// * `path` - A path that can be either:
    ///   - A directory containing a `CMakePresets.json` file (e.g., `"./my_project"`)
    ///   - A direct path to the JSON file (e.g., `"./my_project/CMakePresets.json"`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (I/O error)
    /// - The JSON content is malformed or does not match the expected schema
    pub fn new<T>(path: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: Into<PathBuf>,
    {
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

    /// Finds a non-hidden preset by name.
    ///
    /// Returns `None` if no preset with the given name exists, or if the
    /// matching preset has `hidden: true`.
    ///
    /// # Arguments
    ///
    /// * `name` - The preset name to search for.
    pub fn get_preset(&self, name: &str) -> Option<&CMakePreset> {
        self.configure_presets
            .iter()
            .find(|p| p.name == name && !p.hidden)
    }
}

/// Default value for the `hidden` field in [`CMakePreset`].
fn default_hidden() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cmake_preset() {
        let presets = CMakePresets::new("test").unwrap();
        let preset = presets
            .get_preset("default")
            .expect("Failed to get preset default");
        assert_eq!(preset.get_name(), "default");
    }
}
