#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / issue 11) — host transport for the
//! whole-tree UnlistedImportUse census with binding-source attribution (Dispatch 1 input).
//!
//! Runs the same resolve kernel as compile-clean (`witness_layer_roots` whole-tree closure
//! + `compile_to_resolved`) but emits every `UnlistedImportUse` row with binding-source
//! classification (listed-import | pool-coincidence | definer-resolvable).
//!
//! NOT floor-enrolled — run standalone. Do NOT invoke from cargo tests (whole-tree OOM risk).
//! Carrier: `CLI_RUN_COMPILE_CLEAN_UNLISTED_IMPORT_CENSUS_SCAFFOLD_MARKER` in `cli_run.rs`.
//!
//! DISSOLUTION: delete this bin and the marker-gated helpers when the namespace-only lane
//! deletes the import grammar and binding-source attribution is modeled in the substrate
//! (`docs/plans/namespace-resolution-design.md` §8) OR a floor-enrolled census lens
//! subsumes this transport. Receipt: `rg cli_run_compile_clean_unlisted_import_census
//! src/v1/stage0` == 1 until deletion.

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
