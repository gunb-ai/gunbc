#![allow(clippy::disallowed_macros)]

//! Emit the whole-corpus semantic oracle snapshot (diagnostic fingerprint +
//! rust_corpus_repr) as JSON on stdout. Used to capture the pre-change baseline
//! at commit 57223267a2 for func_env scope-chain equivalence testing.

use std::process::ExitCode;

use v1_compiler::cli_run::{whole_corpus_semantic_oracle_snapshot, FLOOR_DISCOVERY_EXCLUDES};

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|sub| (*sub).to_string())
        .collect();
    exclude_subpaths.extend([
        "test/fixture/".to_string(),
        "/test/".to_string(),
        "nat_semiring_rung".to_string(),
        "lens/application/empty_required_lenses_skip_gate.dag".to_string(),
        "lens/application/rejecting_lens_blocks_before_compile.dag".to_string(),
    ]);

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

    println!(
        "{{\"baseline_commit\":\"57223267a2\",\"diagnostic_fingerprint\":\"{}\",\"rust_corpus_repr\":\"{}\",\"modules_resolved\":{}}}",
        oracle.diagnostic_fingerprint,
        oracle.rust_corpus_repr,
        oracle.modules_resolved,
    );
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
