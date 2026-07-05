# dbt-migrator

**Migrate legacy SQL scripts to dbt models** with safe, lineage‑aware `{{ ref(...) }}` replacement.

[![Crates.io](https://img.shields.io/crates/v/dbt_migrator)](https://crates.io/crates/dbt_migrator)
[![PyPI](https://img.shields.io/pypi/v/dbt_migrator)](https://pypi.org/project/dbt-migrator/)

---

## Why dbt_migrator ?

In modern data platforms, **lineage** is the foundation of trust. dbt excels at providing column‑level and table‑level lineage – but only if all references are expressed via `{{ ref(...) }}`. Legacy SQL scripts often contain hard‑coded database/schema/table names, breaking lineage and making impact analysis, testing, and CI/CD impossible.

`dbt_migrator` bridges that gap:

- **Parses** legacy SQL using a full SQL parser (`sqlparser‑rs`), not fragile regex.
- **Matches** table references against your actual dbt model files (respecting schema, database, and tags inheritance from `dbt_project.yml`).
- **Rewrites** them to `{{ ref('model_name') }}` **only** when an unambiguous match exists – conflicts are flagged, never silently resolved.
- **Works with any dialect** – Snowflake, BigQuery, Redshift, SQL Server, DuckDB, and more (via `sqlparser`).
- **Delivers reports** (JSON/CSV) of all replacements, warnings, and conflicts, so you can review changes before committing.

The result: **clean, dbt‑native SQL** with full lineage, enabling:

- Reliable **impact analysis** (“Which dashboards break if I change this model?”)
- **Automated testing** (data tests, freshness, etc.) with full dependency resolution.
- **CI/CD pipelines** that only run models affected by a pull request.
- Improved **data quality** through visibility and governance.

---

## Key Features

- 🧠 **Smart resolution** – three‑level priority (full `db.schema.table` → `schema.table` → bare `table`) with explicit conflict detection.
- 🔍 **Parses real SQL** – uses `sqlparser‑rs` AST visitor, no fragile regex.
- 📦 **dbt‑aware** – reads `dbt_project.yml`, traverses `models:` hierarchy, accumulates `schema`/`database`/`tags` per model directory.
- ⚙️ **Configurable** – per‑table rules, output modes (overwrite or `.migrated` copy), thread control.
- 📊 **Rich reports** – JSON for automation, CSV for human review.
- 🐍 **Python bindings** – use it as a library in your dbt orchestration scripts (via PyO3/Maturin).
- 🔀 **Multithreaded** – processes thousands of files in parallel (Rayon), with optional single‑thread mode for debugging.

---

## Quick Start

### Command‑line Interface

```bash
# Install from source (requires Rust)
cargo install dbt_migrator

# Or run directly
cargo run -- \
  --project-dir /path/to/dbt_project \
  --sql-path /path/to/legacy_sql \
  --config migrate_config.yml \
  --report-dir ./reports
```

### Python Package (via PyO3)

```bash
pip install dbt-migrator
```

```python
from dbt_migrator import migrate_project

report = migrate_project(
    project_dir="path/to/dbt_project",
    sql_path="path/to/legacy_sql",
    config_path="migrate_config.yml"
)
print(report.summary())
```

The Python API returns the same report data (replacements, warnings, conflicts) as the CLI, ready for integration into your CI/CD workflows.

---

## Configuration

`migrate_config.yml` (example):

```yaml
dialect: "snowflake"        # or "bigquery", "mssql", "duckdb", …
multithreading: true
output_mode: "overwrite"    # or "copy" (saves as .migrated)

conflicts:
  - script: "report_orders.sql"
    table: "dim_customers"
    target: "dim_customers_master"  # explicit override

normalization:
  # Optional regex for non‑standard table names (e.g., Dremio paths)
  regex: "(?P<schema>[^/]+)/(?P<table>[^/]+)"
```

All settings are optional; defaults are documented in the [example config](migrate_config.example.yml).

---

## How It Works

1. **Parse `dbt_project.yml`** – extract model paths and the `models:` tree.
2. **Build resolution index** – walk every `.sql` file under `model-paths`, accumulate schema/database/tags from the `models:` hierarchy (exact dbt inheritance logic: deepest schema wins, tags union).
3. **Parse each legacy SQL file** – using the chosen dialect.
4. **Visit every table reference** – with `sqlparser`’s `VisitorMut`:
   - Try explicit rules from config.
   - Query the index (full → schema.table → table, with conflict detection).
   - If unambiguous, replace `ObjectName` with `{{ ref('model') }}` (AST‑level rewrite).
   - Otherwise, log a warning/conflict and keep the original.
5. **Write back** (overwrite or side‑by‑side) and **generate report**.

> ⚠️ **Important**: The tool uses the `dbt_project.yml` alone (not `manifest.json`), so it’s a **reasonable approximation** of dbt’s final model names. If you have a `manifest.json` available, you can plug it into the index as a more accurate source.

---

## Demo

A [demo project](tests/fixtures/demo_project/) is included. Four legacy SQL references are tested:

| Legacy Reference                             | Result                         |
| -------------------------------------------- | ------------------------------ |
| `[analytics].[marts_finance].[fct_orders]` | `{{ ref('fct_orders') }}`    |
| `marts.dim_customers`                      | `{{ ref('dim_customers') }}` |
| `dbo.legacy_unmapped_table`                | kept (unmapped, warning)       |
| `dim_customers` (ambiguous)                | kept (conflict reported)       |

Run the tests:

```bash
cargo test
```

Or using Python:

```python
pytest tests/
```

---

## Build from Source

### Rust (CLI)

```bash
cargo build --release
target/release/dbt_migrator --help
```

### Python Wheel

```bash
pip install maturin
maturin build --release --features python
# Wheel is in target/wheels/
```

For development:

```bash
maturin develop --features python
```

---

## Limitations & Future Plans

- **No `manifest.json` support yet** – using `dbt_project.yml` only. A future version will optionally read `manifest.json` for 100% accuracy.
- **Tree‑based fallback** – disabled by default (spec’s “last resort”); can be enabled via config flag, but we recommend resolving conflicts manually.

---

## Contributing

Issues and pull requests are welcome !

---

## License

GPL-3.0-or-later

---

**Give your dbt project the lineage it deserves.** Try `dbt_migrator` today.
