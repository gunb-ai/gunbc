//! cache_purity_audit — the warm==cold shadow audit over the floor discovery corpus, run as
//! a fail-closed CI floor gate (DESIGN §5; ROADMAP §2 P3).
//!
//! For every distinct discovery-corpus entry, a WARM resolved-graph cache hit must be
//! canonically byte-identical to the COLD compute it cached (both through the same cached
//! resolve path, so the only difference under test is the write→read CODEC). A divergence is a
//! located, typed, LOUD `CachePurityViolation` and FAILS the gate (exit 1) — the live
//! continuous falsifier that makes enabling `GUNBC_RESOLVED_GRAPH_CACHE_DIR` safe.
//!
//! This gate REQUIRES `GUNBC_RESOLVED_GRAPH_CACHE_DIR` to be set (it audits the cache that var
//! gates); it refuses to run a vacuous audit (DESIGN §5/§6 — an inert gate is a lie). The CI
//! floor wiring sets it (the bundled enable).
//!
//! §5 honest residual: this gate is SOUND over the CI corpus, NOT COMPLETE over all
//! realizations — a prod-only realization never resolved in CI can still go impure silently.
//!
//! Usage:
//!   cache_purity_audit --source-root <dir> [--source-root <dir> ...] \
//!                      [--scan-dir <dir> ...] [--max-entries <N>] [--notice-title <title>]
//!
//! `--max-entries N` bounds the audit to the first N (sorted) corpus entries — the floor gate
//! passes a bound because the codec is UNIFORM across entries (a sample falsifies a lossy
//! serializer at a per-run cost worth paying; a full sweep is ~one cold resolve of every test
//! entry). Omit it for a full periodic / manual deep run.
//!
//! Exit: 0 = every audited entry equal warm==cold; 1 = any violation or a setup error
//! (fail-closed); 2 = usage error.

#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cache_purity_audit::audit_floor_discovery_corpus;

const DEFAULT_SCAN_DIR: &str = "dsl/test/claim";

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut notice_title = "cache purity audit".to_string();
    let mut max_entries: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(arg_value(&args, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                scan_dirs.push(arg_value(&args, i, "--scan-dir")?);
            }
            "--max-entries" => {
                i += 1;
                let raw = arg_value(&args, i, "--max-entries")?;
                match raw.parse::<usize>() {
                    Ok(n) => max_entries = Some(n),
                    Err(_) => {
                        eprintln!("cache_purity_audit: --max-entries expects a non-negative integer, got '{raw}'");
                        return Err(ExitCode::from(2));
                    }
                }
            }
            "--notice-title" => {
                i += 1;
                notice_title = arg_value(&args, i, "--notice-title")?;
            }
            other => {
                eprintln!("cache_purity_audit: unknown argument '{other}'");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("cache_purity_audit: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    if scan_dirs.is_empty() {
        scan_dirs.push(DEFAULT_SCAN_DIR.to_string());
    }

    println!("::group::{notice_title}");
    let report = match audit_floor_discovery_corpus(&source_roots, &scan_dirs, max_entries) {
        Ok(r) => r,
        Err(setup_err) => {
            // Fail-closed: a setup error (e.g. the cache dir unset) is NOT a green.
            println!("::error title={notice_title}::{setup_err}");
            println!("::endgroup::");
            return Err(ExitCode::from(1));
        }
    };

    for v in &report.violations {
        // Located + loud: one GHA error annotation per violation, naming the un-keyed axis.
        println!("::error title={notice_title}::{v}");
    }

    // "No silent caps" (DESIGN §6): state audited-of-discovered AND the codec's real reach.
    let coverage = format!(
        "audited {} of {} discovered entries [decoded {} · miss-on-read(deep, fail-safe) {} · \
         rejected {} · skipped {}]",
        report.entries_audited,
        report.entries_discovered,
        report.decoded,
        report.miss_on_read,
        report.rejected,
        report.skipped
    );

    if report.violations.is_empty() {
        println!("cache purity audit: {coverage} — warm==cold (no lossy/stale decode)");
        if report.entries_audited < report.entries_discovered {
            println!(
                "note: BOUNDED — this is a FAST SMOKE, not the soundness gate (codec fidelity is \
                 depth-dependent, so a sample is not a guarantee; run without --max-entries for the \
                 full sound sweep)"
            );
        }
        if report.miss_on_read > 0 {
            println!(
                "note: {} entr(ies) too DEEP to decode (serde 128-level limit) → production \
                 read_cached_file Misses → recomputes (FAIL-SAFE, uncached, NOT a lossy decode); a \
                 cache-effectiveness hole, not a §5 soundness hole",
                report.miss_on_read
            );
        }
        println!(
            "note: sound over what was audited; NOT complete over prod-only realizations absent \
             from CI (DESIGN §5 honest edge)"
        );
        println!("::endgroup::");
        Ok(ExitCode::SUCCESS)
    } else {
        println!(
            "cache purity audit FAILED: {}/{} audited entries diverged warm!=cold ({coverage}) — \
             the cache codec is lossy/stale (a warm hit serves a different graph than a cold \
             recompute)",
            report.violations.len(),
            report.entries_audited
        );
        println!("::endgroup::");
        Err(ExitCode::from(1))
    }
}

fn arg_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("cache_purity_audit: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
