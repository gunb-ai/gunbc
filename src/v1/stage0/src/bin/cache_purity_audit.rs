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
//!                      [--scan-dir <dir> ...] [--notice-title <title>]
//!
//! Exit: 0 = every audited entry byte-identical warm==cold; 1 = any violation or a setup error
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
    let report = match audit_floor_discovery_corpus(&source_roots, &scan_dirs) {
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

    if report.violations.is_empty() {
        println!(
            "cache purity audit: {} entries audited, warm==cold byte-identical (cache is pure)",
            report.entries_audited
        );
        // §5 honest edge — record the coverage boundary every run, not silently.
        println!(
            "note: sound over the CI discovery corpus ({} entries), NOT complete over prod-only \
             realizations absent from it (DESIGN §5 honest edge)",
            report.entries_audited
        );
        println!("::endgroup::");
        Ok(ExitCode::SUCCESS)
    } else {
        println!(
            "cache purity audit FAILED: {}/{} entries diverged warm!=cold (the cache codec is \
             lossy/stale — a warm hit silently serves a different graph than a cold recompute)",
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
