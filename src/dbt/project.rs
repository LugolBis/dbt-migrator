use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{DbtMigratorError, Result};

/// Parsed representation of the `dbt_project.yml``
#[derive(Debug, Deserialize)]
pub struct DbtProjectFile {
    pub name: String,

    #[serde(default)]
    pub profile: Option<String>,

    #[serde(default, alias = "source-paths", rename(deserialize = "model-paths"))]
    pub model_paths: Vec<String>,

    #[serde(default)]
    pub models: serde_yaml::Value,
}

impl DbtProjectFile {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join("dbt_project.yml");
        if !path.exists() {
            return Err(DbtMigratorError::DbtProjectNotFound(
                project_root.to_path_buf(),
            ));
        }
        let raw = std::fs::read_to_string(&path).map_err(|source| DbtMigratorError::Io {
            path: path.clone(),
            source,
        })?;
        let mut parsed: DbtProjectFile =
            serde_yaml::from_str(&raw).map_err(|source| DbtMigratorError::Yaml { path, source })?;

        if parsed.model_paths.is_empty() {
            parsed.model_paths.push("models".to_string());
        }
        Ok(parsed)
    }

    /// Absolute rooy of models folder
    pub fn model_roots(&self, project_root: &Path) -> Vec<PathBuf> {
        self.model_paths
            .iter()
            .map(|p| project_root.join(p))
            .filter(|p| p.exists())
            .collect()
    }
}
