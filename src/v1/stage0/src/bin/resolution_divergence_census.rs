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
    project_resolution_divergence_census, record_resolution_divergence_phase,
    resolution_divergence_census_live, resolution_divergence_census_live_closure_scoped,
    resolution_divergence_census_source_roots, whole_tree_probe_exclusion_substrings,
    ResolutionDivergencePhase, ResolutionDivergencePhaseState,
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
    let mut exclude = whole_tree_probe_exclusion_substrings();
    let mut closure_scoped = false;

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
            "--closure-scoped" => {
                closure_scoped = true;
            }
            other => {
                eprintln!("resolution_divergence_census: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if closure_scoped && !source_roots.is_empty() {
        eprintln!("resolution_divergence_census: --closure-scoped takes no --source-root (it reuses the compile-clean gate's own witness_layer_roots)");
        return Err(ExitCode::from(2));
    }

    let census = if closure_scoped {
        // Parent/operator ruling 2026-07-21: reuse the compile-clean gate's closure
        // authority (witness_layer_roots + load_compile_clean_entry_sources) — the
        // same source set the falsifier's whole-tree cold control already resolves
        // cleanly — instead of the default blind directory walk. Additive mode; the
        // default (source_roots empty => dag+src/v2) is untouched below.
        resolution_divergence_census_live_closure_scoped()
    } else {
        if source_roots.is_empty() {
            source_roots = resolution_divergence_census_source_roots(&ws);
        }
        resolution_divergence_census_live(&source_roots, &exclude)
    }
    .map_err(|e| {
        eprintln!("resolution_divergence_census: resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    record_resolution_divergence_phase(
        ResolutionDivergencePhase::OutputProjection,
        ResolutionDivergencePhaseState::Started,
        "format report and gate verdict",
    )
    .map_err(|e| {
        eprintln!("resolution_divergence_census: {e}");
        ExitCode::from(2)
    })?;
    let projection = project_resolution_divergence_census(&census);
    println!("{}", projection.stdout);
    if let Some(stderr) = &projection.stderr {
        eprintln!("{stderr}");
    }
    record_resolution_divergence_phase(
        ResolutionDivergencePhase::OutputProjection,
        ResolutionDivergencePhaseState::Completed,
        projection.receipt_detail,
    )
    .map_err(|e| {
        eprintln!("resolution_divergence_census: {e}");
        ExitCode::from(2)
    })?;
    if projection.exit_code == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Err(ExitCode::from(projection.exit_code))
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
