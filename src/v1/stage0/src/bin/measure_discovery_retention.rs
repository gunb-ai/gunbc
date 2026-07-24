#![allow(clippy::disallowed_macros)]

//! Discovery-retention byte-attribution probe (run 30030807467 swap-crawl diagnosis).
//!
//! Replays the width=1 discovery resolve loop over an explicit slice of REAL witness
//! entries against ONE process-shared-style `MultiEntryIndex`, printing a per-entry
//! receipt line (live VmRSS + VmHWM + per-cache entry counts) so the growth curve
//! attributes bytes by correlation — and, under the experiment arms, by subtraction:
//!
//!   arm A (baseline)              — every cache retained, the CI drain regime;
//!   arm B (--drop-resolved-memo)  — the per-subject resolved-graph ReferenceTier is
//!                                   cleared at each entry boundary (the structure
//!                                   schedule-retention eviction never drops, and the
//!                                   Rc holder that pins every closure module's
//!                                   TypedModule env web even after typed-cache
//!                                   eviction removes its map entry);
//!   arm C (--drop-entry-sources)  — the per-entry closure source vectors clear too.
//!
//! peak(A) − peak(B) ≈ the retained-graph mass on this slice; the per-entry slope of
//! B once typed_cache flattens ≈ the residual per-entry retention (heads, interner,
//! diagnostics). The `--refcount-report` prints the schedule-arming histogram over
//! the slice (the eviction-power bound) without resolving anything.
//!
//! `--real-drain` switches to the REAL width=1 discovery drain over the scan dirs
//! (`run_discovery_corpus_with_options`, Serial, selection Off, hermetic): witnesses
//! execute, drain-level arming + schedule/graph eviction run exactly as the CI
//! inline drain, and the streamed `[floor-drain]` receipts carry the curve. Pair a
//! `GUNBC_SCHEDULE_RETENTION_EVICT=0` run (retain-all pole) against a default run
//! for the end-to-end retention A/B with verdict equivalence.
//!
//! Without `--real-drain`: measurement instrument only — resolves (and optionally
//! builds an eval context); never runs witnesses, never writes.

use std::process::ExitCode;

use v1_compiler::cli_run::{
    build_multi_entry_index, current_rss_vmrss_bytes, index_retention_snapshot,
    make_eval_context, peak_rss_vhwm_bytes, probe_clear_entry_closure_sources,
    probe_clear_resolved_graph_memo, probe_entry_import_closure, resolve_entry_with_index,
    run_discovery_corpus_with_options, witness_exclusion_substrings, DiscoveryCorpusOptions,
    DiscoveryWidthPolicy, NodeFrontierSelectionMode,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_discovery_retention: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn walk_test_entries(dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<std::path::PathBuf> = read.flatten().map(|e| e.path()).collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            walk_test_entries(&child, out);
        } else if child
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with("_test.dag"))
            .unwrap_or(false)
        {
            out.push(child.to_string_lossy().into_owned());
        }
    }
}

fn mib(b: Option<u64>) -> String {
    b.map(|v| format!("{:.1}", v as f64 / (1024.0 * 1024.0)))
        .unwrap_or_else(|| "?".into())
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut entries: Vec<String> = Vec::new();
    let mut limit: usize = usize::MAX;
    let mut skip: usize = 0;
    let mut drop_resolved_memo = false;
    let mut drop_entry_sources = false;
    let mut with_eval_context = false;
    let mut refcount_report = false;
    let mut real_drain = false;

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
            "--entry" => {
                i += 1;
                entries.push(require_value(&args, i, "--entry")?);
            }
            "--limit" => {
                i += 1;
                limit = require_value(&args, i, "--limit")?
                    .parse()
                    .map_err(|_| ExitCode::from(2))?;
            }
            "--skip" => {
                i += 1;
                skip = require_value(&args, i, "--skip")?
                    .parse()
                    .map_err(|_| ExitCode::from(2))?;
            }
            "--drop-resolved-memo" => drop_resolved_memo = true,
            "--drop-entry-sources" => drop_entry_sources = true,
            "--with-eval-context" => with_eval_context = true,
            "--refcount-report" => refcount_report = true,
            "--real-drain" => real_drain = true,
            other => {
                eprintln!("measure_discovery_retention: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("measure_discovery_retention: at least one --source-root required");
        return Err(ExitCode::from(2));
    }

    if real_drain {
        // The REAL width=1 drain over the scan dirs: run_discovery_corpus_with_options
        // (Serial ≡ the CI inline-drain regime for retention: drain-level arming, one
        // private process-shared index, per-entry completion driving schedule + graph
        // eviction, [floor-drain] receipts streaming). Witnesses actually execute
        // (hermetic), so this is the end-to-end arm for the retention A/B — pair a
        // GUNBC_SCHEDULE_RETENTION_EVICT=0 run (retain-all pole) against a default
        // run and diff the RSS curves + verdicts.
        eprintln!(
            "[probe] real_drain roots={source_roots:?} scan_dirs={scan_dirs:?} (Serial, selection Off, hermetic)"
        );
        let summary = run_discovery_corpus_with_options(
            &source_roots,
            &scan_dirs,
            &[],
            ExecutionMode::Hermetic,
            DiscoveryWidthPolicy::Serial,
            DiscoveryCorpusOptions {
                node_frontier_selection: NodeFrontierSelectionMode::Off,
                explicit_roster_only: false,
                exclude_substrings: witness_exclusion_substrings(),
                ..Default::default()
            },
        )
        .map_err(|e| {
            eprintln!("measure_discovery_retention: real drain failed: {e}");
            ExitCode::from(1)
        })?;
        eprintln!(
            "[probe] real_drain done total={} passed={} skipped={} failures={} \
             rss_cur_mib={} rss_peak_mib={}",
            summary.total,
            summary.passed,
            summary.skipped,
            summary.failures.len(),
            mib(current_rss_vmrss_bytes()),
            mib(peak_rss_vhwm_bytes()),
        );
        for f in summary.failures.iter().take(200) {
            eprintln!("[probe] real_drain FAIL {f}");
        }
        return Ok(ExitCode::SUCCESS);
    }

    for dir in &scan_dirs {
        walk_test_entries(std::path::Path::new(dir), &mut entries);
    }
    let entries: Vec<String> = entries.into_iter().skip(skip).take(limit).collect();
    if entries.is_empty() {
        eprintln!("measure_discovery_retention: no entries (use --entry or --scan-dir)");
        return Err(ExitCode::from(2));
    }

    eprintln!(
        "[probe] config entries={} drop_resolved_memo={drop_resolved_memo} \
         drop_entry_sources={drop_entry_sources} with_eval_context={with_eval_context} \
         roots={source_roots:?}",
        entries.len()
    );

    let index_build_started = std::time::Instant::now();
    let index = build_multi_entry_index(&source_roots);
    eprintln!(
        "[probe] index_built ms={} rss_cur_mib={} rss_peak_mib={}",
        index_build_started.elapsed().as_millis(),
        mib(current_rss_vmrss_bytes()),
        mib(peak_rss_vhwm_bytes()),
    );

    if refcount_report {
        // The schedule-arming histogram over this slice: how much can per-entry
        // eviction EVER free, on this schedule, at module grain?
        let mut rc: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut closure_sum = 0usize;
        for entry in &entries {
            let closure = probe_entry_import_closure(&index, entry);
            closure_sum += closure.len();
            for m in closure {
                *rc.entry(m).or_insert(0) += 1;
            }
        }
        let mut counts: Vec<usize> = rc.values().copied().collect();
        counts.sort_unstable();
        let pct = |p: usize| -> usize {
            if counts.is_empty() {
                0
            } else {
                counts[(counts.len().saturating_sub(1)) * p / 100]
            }
        };
        let n = entries.len();
        let full = counts.iter().filter(|&&c| c == n && n > 0).count();
        eprintln!(
            "[probe] refcount_report entries={n} modules={} closure_avg={} \
             rc_p50={} rc_p90={} rc_max={} modules_at_full={full}",
            rc.len(),
            if n == 0 { 0 } else { closure_sum / n },
            pct(50),
            pct(90),
            counts.last().copied().unwrap_or(0),
        );
    }

    for (idx, entry) in entries.iter().enumerate() {
        let t = std::time::Instant::now();
        let resolved = resolve_entry_with_index(&index, entry);
        let resolve_ms = t.elapsed().as_millis();
        let outcome = match &resolved {
            Ok((graph, si)) => {
                let modules = graph.modules.len();
                if with_eval_context {
                    let ctx = make_eval_context(graph, si.clone(), ExecutionMode::Hermetic);
                    drop(ctx);
                }
                format!("ok modules={modules}")
            }
            Err(e) => format!("ERR {}", e.lines().next().unwrap_or("").chars().take(120).collect::<String>()),
        };
        drop(resolved);
        if drop_resolved_memo {
            let _ = probe_clear_resolved_graph_memo(&index);
        }
        if drop_entry_sources {
            let _ = probe_clear_entry_closure_sources(&index);
        }
        let snap = index_retention_snapshot(&index);
        eprintln!(
            "[probe] entry={}/{} resolve_ms={resolve_ms} {outcome} \
             typed={} parse={} memo={} sources_cache_note=cleared_by_arm={} intern={} \
             rss_cur_mib={} rss_peak_mib={} file={entry}",
            idx + 1,
            entries.len(),
            snap.typed_module_cache_entries,
            snap.parse_cache_entries,
            snap.resolved_graph_memo_entries,
            drop_entry_sources,
            snap.intern_table_entries,
            mib(current_rss_vmrss_bytes()),
            mib(peak_rss_vhwm_bytes()),
        );
    }

    let snap = index_retention_snapshot(&index);
    eprintln!(
        "[probe] final typed={} parse={} memo={} intern={} rss_cur_mib={} rss_peak_mib={}",
        snap.typed_module_cache_entries,
        snap.parse_cache_entries,
        snap.resolved_graph_memo_entries,
        snap.intern_table_entries,
        mib(current_rss_vmrss_bytes()),
        mib(peak_rss_vhwm_bytes()),
    );
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
