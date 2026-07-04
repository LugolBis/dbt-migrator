use std::path::PathBuf;

use clap::Parser;

/// dbt_migrator - Tool to migrate legacy SQL codebase to dbt (ref()).
#[derive(Debug, Parser)]
#[command(name = "dbt_migrator", version, about)]
pub struct Cli {
    /// Path to the root of the dbt project (who contains dbt_project.yml).
    #[arg(long)]
    pub project_dir: PathBuf,

    /// File .sql or directory with .sql files to migrate.
    #[arg(long)]
    pub sql_path: PathBuf,

    /// Optional config file (migrate_config.yml).
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// SQL Dialect used for the migration (used if absent from the config, default = "mssql").
    #[arg(long, default_value = "mssql")]
    pub dialect: String,

    /// Output folder used for the reports (JSON + CSV).
    #[arg(long, default_value = ".")]
    pub report_dir: PathBuf,
}
