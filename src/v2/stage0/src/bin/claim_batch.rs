//! Batch Bool-witness runner: resolve ONE entry's import closure once, then run
//! many witness functions against the shared resolved graph.
//!
//! Motivation. The v4 node-frontier CI gate
//! (`scripts/v4-affected-set-node-frontier-gate.sh`) runs N Bool witnesses that
//! ALL share a single `--entry` file, differing only in `--function`. Invoking
//! `gunbc run --entry <E> --function <F> --claim-run` once per row re-resolves
//! the entire `src/v4` module closure N times (~5-13s each — the
//! `build_module_index` + closure-compile cost dominates; the witness eval
//! itself is sub-second). This bin walks that resolve ONCE and loops the
//! functions, collapsing N resolves to 1 for the green pass.
//!
//! The gate's perturb pass still runs per-row through `gunbc run`: each row
//! mutates a DIFFERENT function to `false` in its own temp source-root, so those
//! resolves are genuinely distinct and cannot share an index.
//!
//! This is a hand-written CLI bin (like `regen_stage0`), deliberately NOT routed
//! through the generated `main.rs` / emit stage — so adding this capability
//! touches no load-bearing pipeline stage. It reuses the same resolve/run
//! primitives the `gunbc run --claim-run` path uses (see
//! `cli_run::resolve_entry_graph` / `cli_run::run_claim`).
//!
//! Usage:
//!   claim_batch --source-root <dir> [--source-root <dir> ...] \
//!               --entry <file.dag> \
//!               --functions f1,f2,... [--function f3 ...] [--claim-run]
//!
//! Exit codes: 0 = all witnesses returned Bool(true); 1 = any witness failed,
//! returned non-Bool, raised a runtime error, or resolve failed; 2 = usage error.

// Binary entrypoint: it reports witness results directly on stdout/stderr, so
// println!/eprintln! are appropriate here (the disallowed-macros lint is aimed
// at library crates that should return structured errors). The generated main.rs
// carries the same allow.
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v2_compiler::cli_run::{resolve_entry_graph, run_claim, ClaimOutcome};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("claim_batch: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut entry: Option<String> = None;
    let mut functions: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--entry" => {
                i += 1;
                entry = Some(require_value(&args, i, "--entry")?);
            }
            "--function" => {
                i += 1;
                functions.push(require_value(&args, i, "--function")?);
            }
            "--functions" => {
                i += 1;
                let csv = require_value(&args, i, "--functions")?;
                functions.extend(
                    csv.split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                );
            }
            // Accepted for call-site parity with `gunbc run`; this bin is always
            // claim-run (Bool witnesses).
            "--claim-run" => {}
            other => {
                eprintln!("claim_batch: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("claim_batch: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let entry = match entry {
        Some(e) => e,
        None => {
            eprintln!("claim_batch: --entry <file.dag> is required");
            return Err(ExitCode::from(2));
        }
    };
    if functions.is_empty() {
        eprintln!("claim_batch: provide at least one --function or --functions");
        return Err(ExitCode::from(2));
    }

    // Resolve the entry's import closure ONCE.
    let (graph, source_indices) = match resolve_entry_graph(&source_roots, &entry) {
        Ok(pair) => pair,
        Err(msg) => {
            eprintln!("claim_batch: resolve failed for {}:\n{}", entry, msg);
            return Err(ExitCode::from(1));
        }
    };
    eprintln!(
        "claim_batch: resolved {} once; running {} witness(es)",
        entry,
        functions.len()
    );

    // Run each witness against the shared graph.
    let mut any_failed = false;
    for function in &functions {
        match run_claim(&graph, source_indices.clone(), function) {
            ClaimOutcome::Pass => println!("PASS {}", function),
            ClaimOutcome::Fail => {
                println!("FAIL {}", function);
                any_failed = true;
            }
            ClaimOutcome::NotBool { got } => {
                println!(
                    "FAIL {} (returned `{}`, not Bool; --claim-run entries must return Bool)",
                    function, got
                );
                any_failed = true;
            }
            ClaimOutcome::RuntimeError { message } => {
                println!("FAIL {} (runtime error: {})", function, message);
                any_failed = true;
            }
        }
    }

    if any_failed {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
