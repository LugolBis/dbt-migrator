//! Python bindings generated via PyO3, packagable as a wheel with Maturin
//! (`maturin build --release --features python`).
//!
//! The Python API exposes a single function, `migrate_project`, which takes
//! the same parameters as the CLI and returns the migration report serialized
//! as JSON (so the Python side does not need to know the Rust types: it
//! receives a plain string to `json.loads()`).

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::config::MigrateConfig;
use crate::dbt::metadata;
use crate::sql::normalizer::DefaultNormalizer;

/// Migrates all `.sql` files found under `sql_path` to dbt `ref()` references,
/// using metadata from `project_dir`.
///
/// # Arguments
/// * `project_dir` — root of the dbt project (contains `dbt_project.yml`).
/// * `sql_path` — SQL file or directory to migrate.
/// * `dialect` — default SQL dialect (e.g., `"mssql"`).
/// * `config_path` — optional path to `migrate_config.yml`.
///
/// Returns the migration report serialized as JSON.
#[pyfunction]
#[pyo3(signature = (project_dir, sql_path, dialect="mssql".to_string(), config_path=None))]
fn migrate_project(
    project_dir: String,
    sql_path: String,
    dialect: String,
    config_path: Option<String>,
) -> PyResult<String> {
    let project_dir = std::path::PathBuf::from(project_dir);
    let sql_path = std::path::PathBuf::from(sql_path);

    let config = match config_path {
        Some(path) => MigrateConfig::load(std::path::Path::new(&path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        None => MigrateConfig::default(),
    };
    let dialect_name = config.dialect.clone().unwrap_or(dialect);

    let index =
        metadata::build_index(&project_dir).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let files = metadata::collect_sql_files(&sql_path);
    let normalizer = DefaultNormalizer;

    let report = crate::pipeline::runner::run(
        &files,
        &project_dir,
        &dialect_name,
        &index,
        &normalizer,
        &config,
    );

    serde_json::to_string(&report).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Entrypoint of the Rust CLI - used by pip to configure the `dbt_migrator` command.
#[pyfunction]
fn cli_main(py: Python<'_>) -> PyResult<i32> {
    let sys = py.import_bound("sys")?;
    let argv: Vec<String> = sys.getattr("argv")?.extract()?;
    Ok(crate::app::run(argv))
}

/// Python module `dbt_migrator` (name determined by `[lib] name` in Cargo.toml
/// and used by Maturin for the importable module name).
#[pymodule]
fn dbt_migrator(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(migrate_project, m)?)?;
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
