#![allow(clippy::disallowed_macros)]

//! Emit the whole-corpus semantic oracle snapshot (corpus fingerprint,
//! per-module diagnostics/emit repr, full EmitGraphInfo) as JSON on stdout.
//! Used to capture the frozen baseline fixture for func_env whole-corpus
//! semantic equivalence testing (see func_env_semantic_equivalence_test.rs
//! for the pinned BASELINE_COMMIT).

use std::process::ExitCode;

use serde::Serialize;
use v1_compiler::cli_run::{whole_corpus_semantic_oracle_snapshot, witness_exclusion_substrings};

#[derive(Serialize)]
struct CaptureOutput<'a> {
    baseline_commit: &'a str,
    diagnostic_fingerprint: &'a str,
    rust_corpus_repr: &'a str,
    emit_graph_fingerprint: &'a str,
    corpus_fingerprint: &'a str,
    modules_resolved: usize,
    per_module_rows: usize,
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths = witness_exclusion_substrings();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(args.get(i).cloned().ok_or_else(|| {
                    eprintln!("capture_func_env_semantic_oracle: --source-root requires a value");
                    ExitCode::from(2)
                })?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(args.get(i).cloned().ok_or_else(|| {
                    eprintln!(
                        "capture_func_env_semantic_oracle: --exclude-subpath requires a value"
                    );
                    ExitCode::from(2)
                })?);
            }
            other => {
                eprintln!("capture_func_env_semantic_oracle: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("capture_func_env_semantic_oracle: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let oracle =
        whole_corpus_semantic_oracle_snapshot(&source_roots, &exclude_subpaths).map_err(|e| {
            eprintln!("capture_func_env_semantic_oracle: snapshot failed:\n{e}");
            ExitCode::from(2)
        })?;

    let output = CaptureOutput {
        baseline_commit: "aeb1739ec5c",
        diagnostic_fingerprint: &oracle.diagnostic_fingerprint,
        rust_corpus_repr: &oracle.rust_corpus_repr,
        emit_graph_fingerprint: &oracle.emit_graph_fingerprint,
        corpus_fingerprint: &oracle.corpus_fingerprint,
        modules_resolved: oracle.modules_resolved,
        per_module_rows: oracle.per_module_rows,
    };
    println!(
        "{}",
        serde_json::to_string(&output).map_err(|e| {
            eprintln!("capture_func_env_semantic_oracle: serialize failed: {e}");
            ExitCode::from(2)
        })?
    );
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
