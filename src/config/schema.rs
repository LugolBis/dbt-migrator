use std::path::Path;

use serde::Deserialize;

use crate::error::{DbtMigratorError, Result};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Overwrite,
    Copy,
}

impl Default for OutputMode {
    fn default() -> Self {
        OutputMode::Overwrite
    }
}

/// Explicit rule to resolve conflicts between models.
#[derive(Debug, Deserialize, Clone)]
pub struct ConflictRule {
    pub script: String,
    pub table: String,
    pub model: String,
}

/// Config file definition.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct MigrateConfig {
    pub dialect: Option<String>,
    pub normalizer: Option<String>,
    #[serde(default)]
    pub conflicts: Vec<ConflictRule>,
    #[serde(default)]
    pub output_mode: OutputMode,
    #[serde(default)]
    pub multithreading: bool,
    #[serde(default)]
    pub allow_tree_fallback: bool,
}

impl MigrateConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path).map_err(|source| DbtMigratorError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_yaml::from_str(&raw).map_err(|source| DbtMigratorError::Yaml {
            path: path.to_path_buf(),
            source,
        })
    }
}
