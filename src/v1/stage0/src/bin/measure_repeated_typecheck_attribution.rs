#![allow(clippy::disallowed_macros)]

//! Repeated-typecheck attribution probe (entry-graph-union slice 2 / lane ci-cost).
//!
//! Resolves the production-selected entry set (or an explicit `--entry` list) against
//! ONE shared `MultiEntryIndex` and records, per (entry, typed module content key):
//! cache hit/miss/refused, typecheck compute wall on miss, first-computing entry,
//! later-requester count, and per-entry reconcile/assembly timing.
//!
//! The slice-2 decision metric is `decision_ratio`:
//! `repeated_typecheck_compute_ns / total_typecheck_compute_ns` — the share of
//! typecheck wall spent on modules already computed by an earlier entry against
//! ONE shared typed cache. `cache_hit_ratio` is reported separately for
//! diagnostics only. Compare `decision_ratio` against slice-1's membership
//! duplication factor to see whether repeated membership becomes avoided work.
//!
//! Measurement only — no union implementation.
//!
//! ```text
//! GUNBC_CI_DIFF_BASE=<sha> measure_repeated_typecheck_attribution \
//!   --source-root dag --source-root src/v2 \
//!   --scan-dir dag/test/claim --max-entries 6
//! ```

use std::process::ExitCode;

use v1_compiler::cli_run::{
    measure_repeated_typecheck_attribution, render_repeated_typecheck_attribution_json,
    witness_exclusion_substrings,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_repeated_typecheck_attribution: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut discovery_scope_dirs: Vec<String> = Vec::new();
    let mut explicit_entries: Vec<String> = Vec::new();
    let mut max_entries: Option<usize> = None;
    let mut exclude_substrings = witness_exclusion_substrings();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                scan_dirs.push(require_value(&args, i, "--scan-dir")?);
            }
            "--discovery-scope-dir" => {
                i += 1;
                discovery_scope_dirs.push(require_value(&args, i, "--discovery-scope-dir")?);
            }
            "--entry" => {
                i += 1;
                explicit_entries.push(require_value(&args, i, "--entry")?);
            }
            "--max-entries" => {
                i += 1;
                let raw = require_value(&args, i, "--max-entries")?;
                max_entries = Some(raw.parse().map_err(|_| {
                    eprintln!(
                        "measure_repeated_typecheck_attribution: --max-entries must be a usize"
                    );
                    ExitCode::from(2)
                })?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_substrings.push(require_value(&args, i, "--exclude-subpath")?);
            }
            other => {
                eprintln!("measure_repeated_typecheck_attribution: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("measure_repeated_typecheck_attribution: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }
    if scan_dirs.is_empty() && explicit_entries.is_empty() {
        eprintln!(
            "measure_repeated_typecheck_attribution: --scan-dir required unless --entry is given"
        );
        return Err(ExitCode::from(2));
    }

    let measured = measure_repeated_typecheck_attribution(
        &source_roots,
        &scan_dirs,
        &exclude_substrings,
        &discovery_scope_dirs,
        &explicit_entries,
        max_entries,
    )
    .map_err(|e| {
        eprintln!("measure_repeated_typecheck_attribution: REFUSED — {e}");
        ExitCode::from(2)
    })?;

    println!(
        "[typecheck-attribution-measurement] {}",
        render_repeated_typecheck_attribution_json(&measured)
    );
    eprintln!(
        "measure_repeated_typecheck_attribution: N={} hits={} misses={} repeated_misses={} \
         decision_ratio={} cache_hit_ratio={} membership_duplication_factor={} peak_rss={}",
        measured.selected_count(),
        measured.total_cache_hits,
        measured.total_cache_misses,
        measured.repeated_typecheck_misses,
        measured
            .decision_ratio
            .map(|f| format!("{f:.4}"))
            .unwrap_or_else(|| "n/a".to_string()),
        measured
            .cache_hit_ratio
            .map(|f| format!("{f:.4}"))
            .unwrap_or_else(|| "n/a".to_string()),
        measured
            .membership_duplication_factor
            .map(|f| format!("{f:.4}"))
            .unwrap_or_else(|| "n/a".to_string()),
        measured
            .peak_rss_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".to_string()),
    );

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
