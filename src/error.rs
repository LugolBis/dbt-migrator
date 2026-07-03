use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbtMigratorError {
    #[error("Error I/O at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Error : dbt_project.yml doesn't exist at {0}")]
    DbtProjectNotFound(PathBuf),

    #[error("Error : Failed to parse YAML {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Error : Failed to parse SQL in the script {path}: {source}")]
    SqlParse {
        path: PathBuf,
        #[source]
        source: sqlparser::parser::ParserError,
    },

    #[error("Error : Unknown SQL dialect : {0}")]
    UnknownDialect(String),

    #[error("Error : Config issue : {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, DbtMigratorError>;
