#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::emit_witness_entry_eligibility_census;

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let root = std::env::current_dir().map_err(|e| {
        eprintln!("witness_entry_eligibility_census_emit: cwd: {e}");
        ExitCode::from(2)
    })?;
    let tsv = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs/probes/witness_entry_eligibility_census.tsv"));
    let hist = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs/probes/witness_entry_eligibility_histogram.txt"));
    match emit_witness_entry_eligibility_census(&tsv, &hist) {
        Ok(count) => {
            eprintln!(
                "witness_entry_eligibility_census_emit: wrote {} ({count} entries)",
                tsv.display()
            );
            eprintln!(
                "witness_entry_eligibility_census_emit: wrote {}",
                hist.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("witness_entry_eligibility_census_emit: {e}");
            Err(ExitCode::from(1))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
