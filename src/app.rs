use clap::Parser;

use crate::cli::Cli;
use crate::config::MigrateConfig;
use crate::dbt::metadata;
use crate::sql::normalizer::DefaultNormalizer;

pub fn run(args: Vec<String>) -> i32 {
    let _ = env_logger::try_init();

    let cli = Cli::parse_from(args);

    let config = match &cli.config {
        Some(path) => match MigrateConfig::load(path) {
            Ok(c) => c,
            Err(e) => return fail(&format!("Invalid configuration : {e}")),
        },
        None => MigrateConfig::default(),
    };
    let dialect_name = config
        .dialect
        .clone()
        .unwrap_or_else(|| cli.dialect.clone());

    log::info!("Build the index resolution from {:?}", cli.project_dir);
    let index = match metadata::build_index(&cli.project_dir) {
        Ok(i) => i,
        Err(e) => return fail(&e.to_string()),
    };
    log::info!("{} model(s) indexed", index.len());

    let files = metadata::collect_sql_files(&cli.sql_path);
    log::info!("{} SQL file(s) to process", files.len());

    let normalizer = DefaultNormalizer;
    let report = crate::pipeline::runner::run(
        &files,
        &cli.project_dir,
        &dialect_name,
        &index,
        &normalizer,
        &config,
    );

    println!(
        "Finished : {} replacement(s), {} warning(s), {} conflict(s)",
        report.total_replacements(),
        report.total_warnings(),
        report.total_conflicts()
    );

    if let Err(e) = std::fs::create_dir_all(&cli.report_dir) {
        return fail(&format!(
            "Failed to create directories {:?}: {e}",
            cli.report_dir
        ));
    }
    let (json_path, csv_path) = crate::sql::report::default_report_paths(&cli.report_dir);
    if let Err(e) = report.write_json(&json_path) {
        return fail(&format!("Failed to write JSON : {e}"));
    }
    if let Err(e) = report.write_csv(&csv_path) {
        return fail(&format!("Failed to write CSV : {e}"));
    }

    log::info!("Reports written : {json_path:?}, {csv_path:?}");
    0
}

fn fail(message: &str) -> i32 {
    eprintln!("Error : {message}");
    1
}
