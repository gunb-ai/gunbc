#![allow(clippy::disallowed_macros)]

//! Namespace-resolution-design §12.4 census — read-only comparison of
//! `lookup_resolved_sig` vs SymbolIndex containment walk.
//!
//! ```text
//! cargo run -p v1-compiler --bin resolution_divergence_census
//! cargo run -p v1-compiler --bin resolution_divergence_census -- --source-root dag
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    format_resolution_divergence_census, resolution_divergence_census_live,
    resolution_divergence_census_source_roots, resolution_divergence_silent_pick_refusal,
    whole_tree_resolve_exclusion_substrings,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("resolution_divergence_census: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let ws = workspace_root();
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude = whole_tree_resolve_exclusion_substrings();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                let rel = require_value(&args, i, "--source-root")?;
                source_roots.push(ws.join(&rel).to_string_lossy().into_owned());
            }
            "--exclude-subpath" => {
                i += 1;
                exclude.push(require_value(&args, i, "--exclude-subpath")?);
            }
            other => {
                eprintln!("resolution_divergence_census: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots = resolution_divergence_census_source_roots(&ws);
    }

    let census = resolution_divergence_census_live(&source_roots, &exclude).map_err(|e| {
        eprintln!("resolution_divergence_census: resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    let report = format_resolution_divergence_census(&census);
    println!("{report}");
    if census.sites_checked == 0 {
        eprintln!("resolution_divergence_census: no bare call sites in resolved corpus");
        return Err(ExitCode::from(2));
    }
    if let Some(refusal) = resolution_divergence_silent_pick_refusal(&census) {
        eprintln!("{refusal}");
        return Err(ExitCode::from(1));
    }
    println!("SILENT-PICK-GATE: clean (0 silent picks, sites_checked={})", census.sites_checked);
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
