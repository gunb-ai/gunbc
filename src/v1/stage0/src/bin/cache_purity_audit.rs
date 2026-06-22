//! cache_purity_audit — the standalone warm==cold audit over the floor discovery corpus
//! (DESIGN §5; ROADMAP §2 P3). For every discovered entry, a WARM resolved-graph cache hit must be
//! canonically `==` the COLD compute it cached — the write→read CODEC is the only difference under
//! test. A divergence is a located, typed, LOUD `CachePurityViolation` → exit 1.
//!
//! This is the MANUAL / out-of-band diagnostic tool. The CI floor's CONTINUOUS gate is the
//! in-process PIGGYBACK (`cache_purity_audit::audit_warm_readback_against_cold`, armed by
//! `GUNBC_CACHE_PURITY_AUDIT=1`), which audits each cold cache WRITE the floor already performs —
//! ~1× cost, no second resolve. The earlier co-run orchestrator (background process, K-way
//! sharding, fail-closed join) was deleted: measured on the real fleet it was NET −18min and OOM'd
//! runners (it double-resolved the corpus beside the width-8 floor). This bin re-resolves the
//! corpus cold+warm, so run it OFF the floor critical path.
//!
//! This gate REQUIRES `GUNBC_RESOLVED_GRAPH_CACHE_DIR` set to an EMPTY dir (so COLD genuinely
//! misses+computes+writes); it refuses a vacuous audit (DESIGN §5/§6).
//!
//! §5 honest residual: SOUND over the CI corpus, NOT COMPLETE over prod-only realizations absent
//! from CI. Codec depth note: a large fraction of the corpus is too deep for serde's 128-level
//! decode, so production `read_cached_file` Misses→recomputes on them (FAIL-SAFE, uncached) — a
//! cache EFFECTIVENESS gap, not a soundness hole (see the audited reach split below).
//!
//! Exit: 0 = clean; 1 = any violation / setup error (fail-closed); 2 = usage error.

#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cache_purity_audit::{audit_floor_discovery_corpus, CorpusAuditReport};

const DEFAULT_SCAN_DIR: &str = "dsl/test/claim";

struct Args {
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    notice_title: String,
    max_entries: Option<usize>,
}

fn run() -> Result<ExitCode, ExitCode> {
    let raw: Vec<String> = std::env::args().collect();
    let mut a = Args {
        source_roots: Vec::new(),
        scan_dirs: Vec::new(),
        notice_title: "cache purity audit".to_string(),
        max_entries: None,
    };

    let mut i = 1;
    while i < raw.len() {
        match raw[i].as_str() {
            "--source-root" => {
                i += 1;
                a.source_roots.push(arg_value(&raw, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                a.scan_dirs.push(arg_value(&raw, i, "--scan-dir")?);
            }
            "--max-entries" => {
                i += 1;
                a.max_entries = Some(parse_usize(
                    &arg_value(&raw, i, "--max-entries")?,
                    "--max-entries",
                )?);
            }
            "--notice-title" => {
                i += 1;
                a.notice_title = arg_value(&raw, i, "--notice-title")?;
            }
            other => {
                eprintln!("cache_purity_audit: unknown argument '{other}'");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if a.source_roots.is_empty() {
        eprintln!("cache_purity_audit: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    if a.scan_dirs.is_empty() {
        a.scan_dirs.push(DEFAULT_SCAN_DIR.to_string());
    }

    println!("::group::{}", a.notice_title);
    let report = match audit_floor_discovery_corpus(&a.source_roots, &a.scan_dirs, a.max_entries) {
        Ok(r) => r,
        Err(setup_err) => {
            println!("::error title={}::{setup_err}", a.notice_title);
            println!("::endgroup::");
            return Err(ExitCode::from(1));
        }
    };

    for v in &report.violations {
        println!("::error title={}::{v}", a.notice_title);
    }
    let coverage = coverage_line(&report);
    let code = if report.violations.is_empty() {
        println!("cache purity audit: {coverage} — warm==cold (no lossy/stale decode)");
        emit_depth_notes(&a, &report);
        ExitCode::SUCCESS
    } else {
        println!(
            "cache purity audit FAILED: {}/{} audited entries diverged warm!=cold ({coverage}) — the \
             cache codec is lossy/stale (a warm hit serves a different graph than a cold recompute)",
            report.violations.len(),
            report.entries_audited
        );
        ExitCode::from(1)
    };
    println!("::endgroup::");
    Ok(code)
}

fn coverage_line(report: &CorpusAuditReport) -> String {
    format!(
        "audited {} of {} discovered entries [decoded {} · miss-on-read(deep, fail-safe) {} · rejected {} · skipped {}]",
        report.entries_audited, report.entries_discovered, report.decoded, report.miss_on_read, report.rejected, report.skipped
    )
}

fn emit_depth_notes(a: &Args, report: &CorpusAuditReport) {
    if a.max_entries.is_some() {
        println!(
            "note: BOUNDED — a FAST SMOKE, not the soundness gate (codec fidelity is depth-dependent; \
             run without --max-entries for the full sound sweep)"
        );
    }
    if report.miss_on_read > 0 {
        println!(
            "note: {} entr(ies) too DEEP to decode (serde 128-level limit) → production read_cached_file \
             Misses → recomputes (FAIL-SAFE, uncached, NOT lossy); a cache-effectiveness gap",
            report.miss_on_read
        );
    }
    println!(
        "note: sound over the audited corpus; NOT complete over prod-only realizations absent from CI \
         (DESIGN §5 honest edge)"
    );
}

fn parse_usize(raw: &str, flag: &str) -> Result<usize, ExitCode> {
    raw.parse::<usize>().map_err(|_| {
        eprintln!("cache_purity_audit: {flag} expects a non-negative integer, got '{raw}'");
        ExitCode::from(2)
    })
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
