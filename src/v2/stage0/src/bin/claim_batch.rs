//! Batch Bool-witness runner: build the module source index ONCE from the
//! source roots, then resolve each entry's import closure and run its witness
//! functions — all in a single process.
//!
//! Two use-cases are supported in one invocation:
//!
//! 1. **Shared-entry batch** (original #4719 pattern): one `--entry` with
//!    many `--function`/`--functions` flags.  The import closure is resolved
//!    once; all named functions run against the same graph.  This is the shape
//!    the `v4-affected-set-node-frontier-gate.sh` `batch_green_pass` uses.
//!
//! 2. **Multi-entry batch** (lens-gate extension, #4719 follow-on): multiple
//!    `--entry` flags each followed by its own `--function`/`--functions`
//!    flags.  The module source index is built once; each entry's closure is
//!    resolved separately in the same process.  This is what
//!    `v4-lens-ci-gate.sh`'s GREEN pass uses.
//!
//! Motivation. The v4 node-frontier CI gate
//! (`scripts/v4-affected-set-node-frontier-gate.sh`) runs N Bool witnesses
//! that ALL share a single `--entry` file, differing only in `--function`.
//! The v4 lens CI gate (`scripts/v4-lens-ci-gate.sh`) has one witness per
//! entry file.  Both gates previously ran `gunbc run --claim-run` once per
//! row, re-resolving the module tree each time (~5-13s per resolve —
//! `build_module_index` + closure-compile dominates).  Multi-entry mode
//! collapses that to ONE filesystem scan + one resolve per distinct entry.
//!
//! The perturb pass still runs per-row through `gunbc run`: each row mutates
//! a DIFFERENT function to `false` in its own temp source-root, so those
//! resolves are genuinely distinct and cannot share an index.
//!
//! Usage (single-entry — unchanged from #4719):
//!   claim_batch --source-root <dir> [--source-root <dir> ...] \
//!               --entry <file.dag> \
//!               --functions f1,f2,... [--function f3 ...] [--claim-run]
//!
//! Usage (multi-entry):
//!   claim_batch --source-root <dir> [--source-root <dir> ...] \
//!               --entry <e1.dag> --function f1 [--functions f2,...] \
//!               --entry <e2.dag> --function g1 [--functions g2,...] \
//!               ... [--claim-run]
//!
//! Exit codes: 0 = all witnesses returned Bool(true); 1 = any witness failed,
//! returned non-Bool, raised a runtime error, or resolve failed; 2 = usage
//! error.
//!
//! Set GUNBC_INTERP_STATS=1 to print the phase-0 memory measurement report
//! (ctrl#1533) on stderr after the run.  In multi-entry mode only the
//! flatten-counter delta and RSS lines are printed (mutation counters are
//! per-context and each context is dropped after its entry's witnesses finish).

// Binary entrypoint: it reports witness results directly on stdout/stderr, so
// println!/eprintln! are appropriate here (the disallowed-macros lint is aimed
// at library crates that should return structured errors). The generated
// main.rs carries the same allow.
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v2_compiler::cli_run::{
    build_multi_entry_index, make_eval_context, resolve_entry_with_index, run_claim, ClaimOutcome,
    MultiEntryIndex,
};
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

// Multi-entry mode: per-context mutation counters are not available (each
// context is dropped after its entry finishes). Print flatten counters + RSS.
fn print_interp_stats_multi_entry(flatten_baseline: (u64, u64)) {
    let (snap_calls, snap_items) = v2_compiler::v2_interpreter::flatten_counters_snapshot();
    let flatten_calls = snap_calls.saturating_sub(flatten_baseline.0);
    let flatten_items = snap_items.saturating_sub(flatten_baseline.1);
    eprintln!("[interp-stats] flatten counters (all entries combined):");
    eprintln!(
        "  {:<12} {:>12} calls  {:>16} items materialized  (avg {:.1}/call)",
        "fm_flatten",
        flatten_calls,
        flatten_items,
        if flatten_calls == 0 {
            0.0
        } else {
            flatten_items as f64 / flatten_calls as f64
        }
    );
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

/// One entry file with its associated witness function names.
struct EntryGroup {
    entry: String,
    functions: Vec<String>,
}

fn parse_args(args: &[String]) -> Result<(Vec<String>, Vec<EntryGroup>), ExitCode> {
    let mut source_roots: Vec<String> = Vec::new();
    let mut entry_groups: Vec<EntryGroup> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(args, i, "--source-root")?);
            }
            "--entry" => {
                i += 1;
                let entry = require_value(args, i, "--entry")?;
                entry_groups.push(EntryGroup {
                    entry,
                    functions: Vec::new(),
                });
            }
            "--function" => {
                i += 1;
                let f = require_value(args, i, "--function")?;
                match entry_groups.last_mut() {
                    Some(g) => g.functions.push(f),
                    None => {
                        eprintln!("claim_batch: --function before --entry");
                        return Err(ExitCode::from(2));
                    }
                }
            }
            "--functions" => {
                i += 1;
                let csv = require_value(args, i, "--functions")?;
                match entry_groups.last_mut() {
                    Some(g) => g.functions.extend(
                        csv.split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                    ),
                    None => {
                        eprintln!("claim_batch: --functions before --entry");
                        return Err(ExitCode::from(2));
                    }
                }
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

    Ok((source_roots, entry_groups))
}

fn run_witnesses(
    index: &MultiEntryIndex,
    group: &EntryGroup,
    any_failed: &mut bool,
) -> Result<(), ExitCode> {
    let (graph, source_indices) = match resolve_entry_with_index(index, &group.entry) {
        Ok(pair) => pair,
        Err(msg) => {
            eprintln!("claim_batch: resolve failed for {}:\n{}", group.entry, msg);
            return Err(ExitCode::from(1));
        }
    };
    let ctx = make_eval_context(&graph, source_indices);
    for function in &group.functions {
        match run_claim(&ctx, function) {
            ClaimOutcome::Pass => println!("PASS {}", function),
            ClaimOutcome::Fail => {
                println!("FAIL {}", function);
                *any_failed = true;
            }
            ClaimOutcome::NotBool { got } => {
                println!(
                    "FAIL {} (returned `{}`, not Bool; --claim-run entries must return Bool)",
                    function, got
                );
                *any_failed = true;
            }
            ClaimOutcome::RuntimeError { message } => {
                println!("FAIL {} (runtime error: {})", function, message);
                *any_failed = true;
            }
        }
    }
    Ok(())
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let (source_roots, entry_groups) = parse_args(&args)?;

    if source_roots.is_empty() {
        eprintln!("claim_batch: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    if entry_groups.is_empty() {
        eprintln!("claim_batch: --entry <file.dag> is required");
        return Err(ExitCode::from(2));
    }
    for group in &entry_groups {
        if group.functions.is_empty() {
            eprintln!(
                "claim_batch: --entry {} has no --function or --functions",
                group.entry
            );
            return Err(ExitCode::from(2));
        }
    }

    let total_witnesses: usize = entry_groups.iter().map(|g| g.functions.len()).sum();
    eprintln!(
        "claim_batch: {} entry group(s), {} witness(es) total; building module index...",
        entry_groups.len(),
        total_witnesses,
    );

    // Build the module source index ONCE for all entries.
    let index = build_multi_entry_index(&source_roots);

    let flatten_baseline = v2_compiler::v2_interpreter::flatten_counters_snapshot();
    let stats_requested = std::env::var_os("GUNBC_INTERP_STATS").is_some_and(|v| v != "0");

    let mut any_failed = false;

    if entry_groups.len() == 1 {
        // Single-entry path: keep `ctx` alive for the full stats report.
        let group = &entry_groups[0];
        eprintln!(
            "claim_batch: resolved {} once; running {} witness(es)",
            group.entry,
            group.functions.len()
        );
        let (graph, source_indices) = match resolve_entry_with_index(&index, &group.entry) {
            Ok(pair) => pair,
            Err(msg) => {
                eprintln!("claim_batch: resolve failed for {}:\n{}", group.entry, msg);
                return Err(ExitCode::from(1));
            }
        };
        let ctx = make_eval_context(&graph, source_indices);
        for function in &group.functions {
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
        if stats_requested {
            print_interp_stats(&ctx, flatten_baseline);
        }
    } else {
        // Multi-entry path: resolve each entry in turn; context dropped per entry.
        for group in &entry_groups {
            eprintln!(
                "claim_batch: resolving {} ({} witness(es))",
                group.entry,
                group.functions.len()
            );
            run_witnesses(&index, group, &mut any_failed)?;
        }
        if stats_requested {
            print_interp_stats_multi_entry(flatten_baseline);
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
