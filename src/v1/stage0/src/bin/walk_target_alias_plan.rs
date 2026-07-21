#![allow(clippy::disallowed_macros)]

//! Walk-target-sourced alias codemod — plan-only phase (namespace-resolution-design §13).
//!
//! Emits deduped `AliasBindingRow`-shaped plan rows sourced from SymbolIndex walk targets,
//! never import lists. Pair with `resolution_divergence_census` silent-pick telemetry (#6967).
//!
//! ```text
//! cargo run -p v1-compiler --bin walk_target_alias_plan
//! cargo run -p v1-compiler --bin walk_target_alias_plan -- --source-root dag
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    format_walk_target_alias_plan, resolution_divergence_census_source_roots,
    walk_target_alias_plan_live, whole_tree_resolve_exclusion_substrings,
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
            eprintln!("walk_target_alias_plan: {flag} requires a value");
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
                eprintln!("walk_target_alias_plan: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots = resolution_divergence_census_source_roots(&ws);
    }

    let plan = walk_target_alias_plan_live(&source_roots, &exclude).map_err(|e| {
        eprintln!("walk_target_alias_plan: resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    let report = format_walk_target_alias_plan(&plan);
    println!("{report}");
    if plan.global_bare_lcp_events == 0 && plan.fn_parent_first_hit_events == 0 {
        eprintln!("walk_target_alias_plan: no silent-pick events in resolved corpus");
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
