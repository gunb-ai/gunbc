#![allow(clippy::disallowed_macros)]

//! Walk-target-sourced alias codemod — apply phase (namespace-resolution-design §13).
//!
//! Consumes deduped plan rows, runs the binding-identity oracle, and emits
//! `alias <binding> = <qualified.path>` into declaring modules.
//!
//! ```text
//! cargo run -p v1-compiler --bin walk_target_alias_apply
//! cargo run -p v1-compiler --bin walk_target_alias_apply -- --source-root dag
//! cargo run -p v1-compiler --bin walk_target_alias_apply -- --write
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    format_walk_target_alias_apply, resolution_divergence_census_source_roots,
    walk_target_alias_apply_live, whole_tree_resolve_exclusion_substrings,
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
            eprintln!("walk_target_alias_apply: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let ws = workspace_root();
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude = whole_tree_resolve_exclusion_substrings();
    let mut write = false;

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
            "--write" => write = true,
            other => {
                eprintln!("walk_target_alias_apply: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots = resolution_divergence_census_source_roots(&ws);
    }

    let report = walk_target_alias_apply_live(&source_roots, &exclude, write).map_err(|e| {
        eprintln!("walk_target_alias_apply: {e}");
        ExitCode::from(1)
    })?;
    let output = format_walk_target_alias_apply(&report);
    println!("{output}");
    if report.edits.is_empty() && report.refused.is_empty() {
        eprintln!("walk_target_alias_apply: no alias rows to apply in resolved corpus");
        return Err(ExitCode::from(2));
    }
    if !report.refused.is_empty() {
        return Err(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
