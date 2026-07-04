use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::config::schema::{MigrateConfig, OutputMode};
use crate::dbt::index::ResolutionIndex;
use crate::error::DbtMigratorError;
use crate::sql::normalizer::Normalizer;
use crate::sql::report::{FileReport, MigrationReport};
use crate::sql::rewriter;

/// Execute the migration on the files using multi-threading (if the configuration allow it).
pub fn run(
    files: &[PathBuf],
    project_root: &Path,
    dialect_name: &str,
    index: &ResolutionIndex,
    normalizer: &(dyn Normalizer + Sync),
    config: &MigrateConfig,
) -> MigrationReport {
    let num_threads = if config.multithreading {
        0 // 0 means this is Rayon who choose the number of thread to use.
    } else {
        1
    };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to build the rayon Thread pool");

    let file_reports: Vec<FileReport> = pool.install(|| {
        files
            .par_iter()
            .map(|path| {
                process_one_file(path, project_root, dialect_name, index, normalizer, config)
            })
            .collect()
    });

    MigrationReport {
        files: file_reports,
    }
}

fn process_one_file(
    path: &Path,
    project_root: &Path,
    dialect_name: &str,
    index: &ResolutionIndex,
    normalizer: &(dyn Normalizer + Sync),
    config: &MigrateConfig,
) -> FileReport {
    let script_id = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    let dialect = match crate::sql::dialects::dialect_from_name(dialect_name) {
        Ok(d) => d,
        Err(e) => return FileReport::error(path, e.to_string()),
    };

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return FileReport::error(
                path,
                DbtMigratorError::Io {
                    path: path.to_path_buf(),
                    source: e,
                }
                .to_string(),
            )
        }
    };

    let (rewritten, report) = match rewriter::rewrite_sql(
        &script_id,
        &source,
        dialect.as_ref(),
        index,
        normalizer,
        config,
    ) {
        Ok(result) => result,
        Err(e) => return FileReport::error(path, e.to_string()),
    };

    if let Err(e) = write_output(path, &rewritten, config.output_mode) {
        let mut report = report;
        report.error = Some(format!("Failed to write dbt model : {e}"));
        return report;
    }

    report
}

fn write_output(path: &Path, content: &str, mode: OutputMode) -> std::io::Result<()> {
    match mode {
        OutputMode::Overwrite => fs::write(path, content),
        OutputMode::Copy => {
            let mut migrated = path.to_path_buf();
            let new_ext = match path.extension().and_then(|e| e.to_str()) {
                Some(ext) => format!("{ext}.migrated"),
                None => "migrated".to_string(),
            };
            migrated.set_extension(new_ext);
            fs::write(migrated, content)
        }
    }
}
