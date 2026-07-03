use std::path::{Path, PathBuf};

use serde_yaml::Value;
use walkdir::WalkDir;

use crate::dbt::index::{ModelInfo, ResolutionIndex};
use crate::dbt::project::DbtProjectFile;
use crate::error::Result;

/// Config based on the heritage system introduced by dbt configs
#[derive(Debug, Clone, Default)]
struct ScopeConfig {
    database: Option<String>,
    schema: Option<String>,
    tags: Vec<String>,
}

impl ScopeConfig {
    fn apply(&mut self, node: Option<&Value>) {
        let Some(mapping) = node.and_then(Value::as_mapping) else {
            return;
        };

        if let Some(db) = mapping
            .get(Value::String("+database".into()))
            .and_then(Value::as_str)
        {
            self.database = Some(db.to_lowercase());
        }

        if let Some(schema) = mapping
            .get(Value::String("+schema".into()))
            .and_then(Value::as_str)
        {
            self.schema = Some(schema.to_lowercase());
        }

        match mapping.get(Value::String("+tags".into())) {
            Some(Value::String(tag)) => self.tags.push(tag.to_lowercase()),
            Some(Value::Sequence(seq)) => {
                for tag in seq.iter().filter_map(Value::as_str) {
                    self.tags.push(tag.to_lowercase());
                }
            }
            _ => {}
        }
    }
}

/// Build the `ResolutionIndex` with the meta data parsed from `dbt_project.yml` and traverses models folder and it's sub folders
pub fn build_index(project_root: &Path) -> Result<ResolutionIndex> {
    let project_file = DbtProjectFile::load(project_root)?;
    let mut index = ResolutionIndex::default();

    for model_root in project_file.model_roots(project_root) {
        let mut root_scope = ScopeConfig::default();

        let project_node = project_file.models.get(&project_file.name);
        root_scope.apply(project_node);

        for entry in WalkDir::new(&model_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|e| e.to_str()) != Some("sql") {
                continue;
            }

            let scope =
                resolve_scope_for_file(project_node, root_scope.clone(), &model_root, entry.path());

            let model_name = entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let relative_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");

            index.insert(ModelInfo {
                name: model_name,
                database: scope.database,
                schema: scope.schema,
                tags: dedup(scope.tags),
                relative_path,
            });
        }
    }

    Ok(index)
}

fn resolve_scope_for_file(
    project_node: Option<&Value>,
    base_scope: ScopeConfig,
    model_root: &Path,
    file_path: &Path,
) -> ScopeConfig {
    let mut scope = base_scope;
    let mut node = project_node;

    let relative_dir = file_path
        .strip_prefix(model_root)
        .ok()
        .and_then(|p| p.parent());

    if let Some(relative_dir) = relative_dir {
        for component in relative_dir.components() {
            let name = component.as_os_str().to_string_lossy();
            node = node.and_then(|n| n.get(name.as_ref()));
            scope.apply(node);
        }
    }

    scope
}

fn dedup(mut tags: Vec<String>) -> Vec<String> {
    tags.sort();
    tags.dedup();
    tags
}

/// Collect the SQL scripts path who need to be processed.
pub fn collect_sql_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("sql"))
        .map(|e| e.path().to_path_buf())
        .collect()
}
