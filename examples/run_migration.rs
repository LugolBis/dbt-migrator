//! Example of `dbt_migrator` usage as library, without use the CLI.
//!
//! Execute it with : `cargo run --example run_migration`

use dbt_migrator::config::MigrateConfig;
use dbt_migrator::dbt::metadata;
use dbt_migrator::sql::normalizer::DefaultNormalizer;

fn main() -> anyhow::Result<()> {
    let project_dir = std::path::Path::new("tests/fixtures/demo_project");
    let sql_dir = project_dir.join("sql_legacy");

    let index = metadata::build_index(project_dir)?;
    println!("{} model(s) indexed from dbt_project.yml", index.len());

    let files = metadata::collect_sql_files(&sql_dir);

    // Replace it by your own implementation to fit to you use cases
    let normalizer = DefaultNormalizer;

    let mut config = MigrateConfig::default();
    config.output_mode = dbt_migrator::config::OutputMode::Copy;

    let report = dbt_migrator::pipeline::runner::run(
        &files,
        project_dir,
        "mssql",
        &index,
        &normalizer,
        &config,
    );

    for file in &report.files {
        println!("\n--- {} ---", file.file);
        for r in &file.replacements {
            println!("  Replaced : {} -> ref('{}')", r.original, r.model);
        }
        for w in &file.warnings {
            println!("  Not resolved : {}", w.original);
        }
        for c in &file.conflicts {
            println!(
                "  Conflict : {} ({} candidates)",
                c.original,
                c.candidates.len()
            );
        }
    }

    println!(
        "\nTotal : {} replacement(s), {} warnings(s), {} conflict(s)",
        report.total_replacements(),
        report.total_warnings(),
        report.total_conflicts()
    );

    Ok(())
}
