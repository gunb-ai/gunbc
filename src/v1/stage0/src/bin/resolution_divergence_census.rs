#![allow(clippy::disallowed_macros)]

//! Namespace-resolution-design §12.4 census — read-only comparison of
//! `lookup_resolved_sig` vs SymbolIndex containment walk over the floor corpus.
//!
//! Run:
//! ```text
//! cargo run -p v1-compiler --bin resolution_divergence_census
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    format_resolution_divergence_census, resolution_divergence_census_live,
    resolution_divergence_census_source_roots, whole_tree_resolve_exclusion_substrings,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn run() -> Result<ExitCode, ExitCode> {
    let ws = workspace_root();
    let roots = resolution_divergence_census_source_roots(&ws);
    let exclude = whole_tree_resolve_exclusion_substrings();
    let census = resolution_divergence_census_live(&roots, &exclude).map_err(|e| {
        eprintln!("resolution_divergence_census: whole-tree resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    let report = format_resolution_divergence_census(&census);
    println!("{report}");
    if census.sites_checked == 0 {
        eprintln!("resolution_divergence_census: no bare call sites in resolved corpus");
        return Err(ExitCode::from(2));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
