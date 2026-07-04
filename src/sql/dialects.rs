use sqlparser::dialect::{
    AnsiDialect, BigQueryDialect, Dialect, DuckDbDialect, GenericDialect, MsSqlDialect,
    MySqlDialect, PostgreSqlDialect, SnowflakeDialect,
};

use crate::error::Result;

/// Build the `sqlparser` dialect from the name.
/// Use it to extend the functionalities and add the support of others dialect.
pub fn dialect_from_name(name: &str) -> Result<Box<dyn Dialect>> {
    match name.to_lowercase().as_str() {
        "ansi" => Ok(Box::new(AnsiDialect {})),
        "mssql" | "sqlserver" | "sql_server" | "tsql" => Ok(Box::new(MsSqlDialect {})),
        "mysql" | "mariadb" => Ok(Box::new(MySqlDialect {})),
        "postgres" | "postgresql" => Ok(Box::new(PostgreSqlDialect {})),
        "duckdb" => Ok(Box::new(DuckDbDialect {})),
        "snowflake" => Ok(Box::new(SnowflakeDialect {})),
        "bigquery" => Ok(Box::new(BigQueryDialect {})),
        _ => Ok(Box::new(GenericDialect {})),
    }
}
