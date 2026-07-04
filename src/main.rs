mod cli;

use clap::Parser;

use cli::Cli;
use dbt_migrator::config::MigrateConfig;
use dbt_migrator::dbt::metadata;
use dbt_migrator::sql::normalizer::DefaultNormalizer;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let config = match &cli.config {
        Some(path) => MigrateConfig::load(path)?,
        None => MigrateConfig::default(),
    };
    let dialect_name = config
        .dialect
        .clone()
        .unwrap_or_else(|| cli.dialect.clone());

    log::info!("Build index resolution from {:?}", cli.project_dir);
    let index = metadata::build_index(&cli.project_dir)?;
    log::info!("{} model(s) indexed", index.len());

    let files = metadata::collect_sql_files(&cli.sql_path);
    log::info!("{} SQL file(s) to process", files.len());

    let normalizer = DefaultNormalizer;
    let report = dbt_migrator::pipeline::runner::run(
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

    std::fs::create_dir_all(&cli.report_dir)?;
    let (json_path, csv_path) = dbt_migrator::sql::report::default_report_paths(&cli.report_dir);
    report.write_json(&json_path)?;
    report
        .write_csv(&csv_path)
        .map_err(|e| anyhow::anyhow!("Write CSV : {e}"))?;

    log::info!("Reports written : {json_path:?}, {csv_path:?}");
    Ok(())
}
