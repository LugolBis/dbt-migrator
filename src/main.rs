//! Native binary - wrapper against `dbt_migrator::app::run`, who
//! contains all of the logic shared with the Python binding
//! (`src/python.rs::cli_main`).

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(dbt_migrator::app::run(args));
}
