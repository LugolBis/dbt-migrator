use std::path::PathBuf;

use dbt_migrator::config::MigrateConfig;
use dbt_migrator::dbt::metadata;
use dbt_migrator::sql::normalizer::DefaultNormalizer;
use dbt_migrator::sql::rewriter::rewrite_sql;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

#[test]
fn resout_reference_complete_et_partielle() {
    let index = metadata::build_index(&fixture_root()).expect("index construit");
    let dialect = dbt_migrator::sql::dialects::dialect_from_name("mssql").unwrap();
    let normalizer = DefaultNormalizer;
    let config = MigrateConfig::default();

    let sql = "SELECT * FROM [analytics].[marts_finance].[fct_orders]";
    let (rewritten, report) =
        rewrite_sql("test.sql", sql, dialect.as_ref(), &index, &normalizer, &config)
            .expect("réécriture réussie");

    assert!(rewritten.contains("{{ ref('fct_orders') }}"));
    assert_eq!(report.replacements.len(), 1);
    assert_eq!(report.warnings.len(), 0);
    assert_eq!(report.conflicts.len(), 0);
}

#[test]
fn conserve_les_references_non_resolues() {
    let index = metadata::build_index(&fixture_root()).expect("index construit");
    let dialect = dbt_migrator::sql::dialects::dialect_from_name("mssql").unwrap();
    let normalizer = DefaultNormalizer;
    let config = MigrateConfig::default();

    let sql = "SELECT * FROM dbo.table_externe_non_modelisee";
    let (rewritten, report) =
        rewrite_sql("test.sql", sql, dialect.as_ref(), &index, &normalizer, &config)
            .expect("réécriture réussie");

    assert!(rewritten.to_lowercase().contains("table_externe_non_modelisee"));
    assert!(!rewritten.contains("ref("));
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn signale_les_conflits_sans_les_resoudre_silencieusement() {
    let index = metadata::build_index(&fixture_root()).expect("index construit");
    let dialect = dbt_migrator::sql::dialects::dialect_from_name("mssql").unwrap();
    let normalizer = DefaultNormalizer;
    let config = MigrateConfig::default();

    // "dim_customers" existe dans deux schémas différents dans la fixture
    // (models/marts et models/marts/legacy) : sans qualification de schéma,
    // la résolution par table seule est ambiguë.
    let sql = "SELECT * FROM dim_customers";
    let (rewritten, report) =
        rewrite_sql("test.sql", sql, dialect.as_ref(), &index, &normalizer, &config)
            .expect("réécriture réussie");

    assert!(!rewritten.contains("ref("));
    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].candidates.len(), 2);
}

#[test]
fn une_regle_de_config_explicite_leve_un_conflit() {
    use dbt_migrator::config::ConflictRule;

    let index = metadata::build_index(&fixture_root()).expect("index construit");
    let dialect = dbt_migrator::sql::dialects::dialect_from_name("mssql").unwrap();
    let normalizer = DefaultNormalizer;

    let mut config = MigrateConfig::default();
    config.conflicts.push(ConflictRule {
        script: "test.sql".to_string(),
        table: "dim_customers".to_string(),
        model: "dim_customers".to_string(),
    });

    let sql = "SELECT * FROM dim_customers";
    let (rewritten, report) =
        rewrite_sql("test.sql", sql, dialect.as_ref(), &index, &normalizer, &config)
            .expect("réécriture réussie");

    assert!(rewritten.contains("{{ ref('dim_customers') }}"));
    assert_eq!(report.conflicts.len(), 0);
    assert_eq!(report.replacements.len(), 1);
}
