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
//!
//! Set GUNBC_INTERP_STATS=1 to print the phase-0 memory measurement report
//! (ctrl#1533) on stderr after the run: copy-work counters for the
//! copy-on-update collection primitives, sharing-aware byte accounting of the
//! retained context, and peak RSS.

// Binary entrypoint: it reports witness results directly on stdout/stderr, so
// println!/eprintln! are appropriate here (the disallowed-macros lint is aimed
// at library crates that should return structured errors). The generated main.rs
// carries the same allow.
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v2_compiler::cli_run::{make_eval_context, resolve_entry_graph, run_claim, ClaimOutcome};
use v2_compiler::v2_interpreter::InterpContext;

/// VmHWM/VmRSS lines from /proc/self/status (linux best-effort; empty elsewhere).
fn peak_rss_lines() -> String {
    std::fs::read_to_string("/proc/self/status")
        .map(|s| {
            s.lines()
                .filter(|l| l.starts_with("VmHWM") || l.starts_with("VmRSS"))
                .map(|l| format!("  {}\n", l))
                .collect()
        })
        .unwrap_or_default()
}

fn print_interp_stats(ctx: &InterpContext, flatten_baseline: (u64, u64)) {
    eprintln!("[interp-stats] mutation-primitive copy work (this context):");
    eprint!("{}", ctx.mutation_counters_snapshot());
    // Flatten counters are thread-global (the chokepoint also fires inside
    // `Value::eq`, which has no context); the snapshot API is delta-sampled so
    // the row printed here covers exactly this run's witness loop — the same
    // scope as the context-bound counters above it.
    let (snap_calls, snap_items) = v2_compiler::v2_interpreter::flatten_counters_snapshot();
    let flatten_calls = snap_calls.saturating_sub(flatten_baseline.0);
    let flatten_items = snap_items.saturating_sub(flatten_baseline.1);
    eprintln!(
        "  {:<12} {:>12} calls  {:>16} items materialized  (avg {:.1}/call; delta over the witness loop)",
        "fm_flatten",
        flatten_calls,
        flatten_items,
        if flatten_calls == 0 {
            0.0
        } else {
            flatten_items as f64 / flatten_calls as f64
        }
    );
    eprintln!("[interp-stats] retained value accounting (data cache + pure-call memo):");
    eprint!("{}", ctx.account_retained_memory(&[]));
    eprintln!("[interp-stats] process memory:");
    eprint!("{}", peak_rss_lines());
}

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

    // Build the evaluation context ONCE for the shared graph: every witness
    // reuses its fn index and `data` cache, and dropping it at the end of this
    // scope releases all cached values (the cache is context-scoped, not
    // process-global).
    let ctx = make_eval_context(&graph, source_indices);

    // Baseline for the thread-global flatten counters, taken after
    // resolve/context setup so the stats report covers only the witness loop.
    let flatten_baseline = v2_compiler::v2_interpreter::flatten_counters_snapshot();

    // Run each witness against the shared graph.
    let mut any_failed = false;
    for function in &functions {
        match run_claim(&ctx, function) {
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

    if std::env::var_os("GUNBC_INTERP_STATS").is_some_and(|v| v != "0") {
        print_interp_stats(&ctx, flatten_baseline);
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
