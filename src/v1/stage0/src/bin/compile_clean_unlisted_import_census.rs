#![allow(clippy::disallowed_macros)]

//! Whole-tree UnlistedImportUse census with binding-source attribution (issue 11).
//!
//! Emits a TSV keyed (file, referenced_name, binding_source) for Dispatch 1 consumption.
//! NOT floor-enrolled — run standalone. Do NOT invoke from cargo tests (whole-tree OOM risk).

use std::collections::BTreeMap;
use std::io::Write;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    compile_clean_unlisted_import_census, workspace_root, UnlistedImportBindingSource,
};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("GUNBC_CI_DIFF_BASE");

    eprintln!("compile_clean_unlisted_import_census: starting whole-tree census…");
    let started = std::time::Instant::now();

    let rows = match compile_clean_unlisted_import_census() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("CENSUS_STATUS error");
            eprintln!("CENSUS_ERROR {e}");
            return ExitCode::from(2);
        }
    };

    let elapsed = started.elapsed();
    let total = rows.len();

    let mut by_source: BTreeMap<UnlistedImportBindingSource, usize> = BTreeMap::new();
    for row in &rows {
        *by_source.entry(row.binding_source).or_default() += 1;
    }

    println!("CENSUS_STATUS ok");
    println!("CENSUS_TOTAL {total}");
    println!("CENSUS_ELAPSED_SECS {:.1}", elapsed.as_secs_f64());

    println!("--- BINDING_SOURCE ---");
    for (source, count) in &by_source {
        println!("SOURCE\t{}\t{count}", source.as_str());
    }

    let pool_coincidence = by_source
        .get(&UnlistedImportBindingSource::PoolCoincidence)
        .copied()
        .unwrap_or(0);
    if pool_coincidence > 0 {
        eprintln!(
            "CLASS_B_WARNING: {pool_coincidence} pool-coincidence rows — #6985 live in CI; \
             floor green is accidental for these rows (import-strip freeze justified)"
        );
    }

    println!("--- TSV ---");
    println!("file\treferenced_name\treferencing_module\tdefiner_module\tbinding_source");
    for row in &rows {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            row.file,
            row.referenced_name,
            row.referencing_module,
            row.definer_module.as_deref().unwrap_or(""),
            row.binding_source.as_str()
        );
    }

    if let Ok(path) = std::env::var("COMPILE_CLEAN_CENSUS_WRITE_TSV") {
        let mut file = std::fs::File::create(&path).expect("create census tsv");
        writeln!(
            file,
            "file\treferenced_name\treferencing_module\tdefiner_module\tbinding_source"
        )
        .expect("write census header");
        for row in &rows {
            writeln!(
                file,
                "{}\t{}\t{}\t{}\t{}",
                row.file,
                row.referenced_name,
                row.referencing_module,
                row.definer_module.as_deref().unwrap_or(""),
                row.binding_source.as_str()
            )
            .expect("write census row");
        }
        eprintln!("CENSUS_WROTE {path}");
    }

    ExitCode::from(if total == 0 { 0 } else { 1 })
}
