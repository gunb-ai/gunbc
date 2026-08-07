#![allow(clippy::disallowed_macros)]
// The meters stay compiled in the default-feature build so the two configurations share
// one file; only the measurement body is gated.
#![allow(dead_code)]

//! Private-footprint decomposition for ONE discovery worker (dashboard work item
//! node://adhoc-2a689db3-964).
//!
//! THE QUESTION. A runner slot is MemoryHigh 13 GiB / MemoryMax 14 GiB and a
//! successful serial floor measures 10.3–11.0 GiB, so a second worker has roughly
//! 2–3 GiB of headroom. The cross-worker store (`shared_typecheck_store.rs`) shares
//! ONLY `typed_module_cache`; every other term on `MultiEntryIndex` stays per worker.
//! This harness measures what that per-worker remainder is MADE OF.
//!
//! NOT THIS LANE: no width-1-vs-width-2 A/B (that is bright-koi-166's measurement),
//! no `DiscoveryWidthPolicy` change, no shared-store change. This binary builds ONE
//! index on ONE thread and never admits a second worker.
//!
//! INSTRUMENT — two independent meters, reported side by side, never blended:
//!
//!   1. LIVE HEAP, from a counting global allocator (`CountingAlloc`). Exact bytes
//!      currently live as requested from the allocator: immune to allocator retention,
//!      to shared pages, and to copy-on-write, and it is the only meter used for
//!      per-term attribution. It does NOT see mmap'd file backing, thread stacks, or
//!      allocator metadata/fragmentation.
//!   2. RSS / VmHWM, from `/proc/self/status`. What the cgroup actually charges. It
//!      cannot be attributed to a term, so it is reported only as a stage total and as
//!      the gap against live heap.
//!
//! Where the two disagree, the gap is reported as a gap. Neither is corrected by the
//! other and no term is sized by subtraction from RSS.
//!
//! ATTRIBUTION IS BY EXCLUSIVE DROP, not by shell sizing. The Rc→Arc spike receipt
//! (`docs/plans/rc-to-arc-share-spike.md` §2.2) records that per-field shallow sizing
//! under-counts and must not be summed for a crossover; this harness therefore clears
//! one term at a time and measures the live-heap RELEASE. A byte still reachable from
//! another field (Rc-shared structure) is not released and so is NOT attributed to the
//! term dropped — the deliberate consequence being that shared structure shows up as a
//! residue that no term claims, rather than as an invented split.
//!
//! SEPARABILITY IS MEASURED, NOT ASSUMED. Drop order is a parameter (`--drop-order`).
//! Two terms that share structure attribute differently depending on which is dropped
//! first; running both orders and differencing is what licenses calling a term
//! separable. A term whose exclusive release changes with order is reported as NOT
//! separable from its partner rather than being split.
//!
//! Usage:
//!   measure_worker_private_memory [--retention armed|unarmed]
//!                                 [--drop-order declared|reverse] [--entries N]
//!
//! Requires `--features interp_test_witness`; a default-feature build refuses.

use std::alloc::{GlobalAlloc, Layout, System};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "interp_test_witness")]
use v1_compiler::cli_run::{
    build_multi_entry_index, drop_attributable_terms_for_test, drop_private_term_for_test,
    entry_closure_paths_for_test, force_both_closure_edges_for_test,
    force_pool_bare_census_for_test, force_pool_parse_for_test, force_pool_qualified_fill_for_test,
    force_tree_bare_census_for_test, install_schedule_retention_for_test,
    private_term_entry_counts_for_test, resolve_entry_with_index_for_discovery_corpus,
    resolved_graph_memo_keys_for_test, schedule_entry_completed_for_test, workspace_root,
};

/// The index force/drop handles this harness needs are `interp_test_witness`-gated, so a
/// default-feature build produces a binary that refuses rather than one that silently
/// measures nothing.
#[cfg(not(feature = "interp_test_witness"))]
fn main() -> ExitCode {
    eprintln!(
        "measure_worker_private_memory: built without `interp_test_witness`; rebuild with \
         `--features interp_test_witness`. Refusing rather than reporting an empty measurement."
    );
    ExitCode::FAILURE
}

// --- Meter 1: counting global allocator -------------------------------------
//
// Relaxed ordering is sound for these counters: they are read only at stage
// boundaries on the same single thread that does the allocating, so there is no
// cross-thread ordering claim being made. PEAK is a per-allocation running max and is
// therefore exact for this thread, not a sample.

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let now = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let out = System.realloc(ptr, layout, new_size);
        if !out.is_null() {
            if new_size >= layout.size() {
                let now = LIVE.fetch_add(new_size - layout.size(), Ordering::Relaxed) + new_size
                    - layout.size();
                PEAK.fetch_max(now, Ordering::Relaxed);
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        out
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

fn live_bytes() -> usize {
    LIVE.load(Ordering::Relaxed)
}

fn peak_bytes() -> usize {
    PEAK.load(Ordering::Relaxed)
}

// --- Meter 2: /proc/self/status ---------------------------------------------

fn proc_status_kib(field: &str) -> usize {
    let Ok(text) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            return rest
                .trim_start_matches(':')
                .trim()
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(0);
        }
    }
    0
}

fn rss_bytes() -> usize {
    proc_status_kib("VmRSS") * 1024
}

fn hwm_bytes() -> usize {
    proc_status_kib("VmHWM") * 1024
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// A stage boundary snapshot. `live` is the attributable meter; `rss`/`hwm` are
/// context and are never differenced into a per-term number.
fn stage(label: &str, prev_live: usize) -> usize {
    let live = live_bytes();
    let delta = live as i64 - prev_live as i64;
    println!(
        "[wpm] stage={label} live_mib={:.1} delta_mib={:+.1} rss_mib={:.1} hwm_mib={:.1} \
         alloc_peak_mib={:.1}",
        mib(live),
        delta as f64 / (1024.0 * 1024.0),
        mib(rss_bytes()),
        mib(hwm_bytes()),
        mib(peak_bytes()),
    );
    live
}

#[cfg(feature = "interp_test_witness")]
fn cohort_relative_paths() -> Vec<&'static str> {
    include_str!("p1_cohort_roster.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(feature = "interp_test_witness")]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let arg_value = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    // `--retention armed` drives the production drain step per entry-completion, so the
    // warm reports a worker's STEADY-STATE hold rather than a retain-all accumulation.
    // `unarmed` (default) is the retain-all pole.
    let retention = arg_value("--retention").unwrap_or_else(|| "unarmed".to_string());
    if retention != "armed" && retention != "unarmed" {
        eprintln!("measure_worker_private_memory: --retention must be armed|unarmed");
        return ExitCode::FAILURE;
    }
    let drop_order = arg_value("--drop-order").unwrap_or_else(|| "declared".to_string());
    if drop_order != "declared" && drop_order != "reverse" {
        eprintln!("measure_worker_private_memory: --drop-order must be declared|reverse");
        return ExitCode::FAILURE;
    }
    let entry_limit: usize = arg_value("--entries")
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);

    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir to workspace root");

    // The SAME two roots and the SAME 50-entry cohort the P1 probe drives, so this
    // decomposition and the width A/B are read against one population.
    let source_roots = vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ];
    let entries: Vec<String> = cohort_relative_paths()
        .into_iter()
        .take(entry_limit)
        .map(|rel| ws.join(rel).to_string_lossy().into_owned())
        .collect();

    println!(
        "[wpm] config retention={retention} drop_order={drop_order} entries={} source_roots={:?}",
        entries.len(),
        source_roots
    );

    let mut prev = stage("baseline", live_bytes());
    let baseline = prev;

    // --- Construction, staged in dependency order. Each delta is that term's
    // MARGINAL retained heap given everything before it.
    let index = build_multi_entry_index(&source_roots);
    prev = stage("index_shell", prev);
    let after_shell = prev;

    if let Err(e) = force_pool_parse_for_test(&index) {
        eprintln!("measure_worker_private_memory: pool_parse failed: {e}");
        return ExitCode::FAILURE;
    }
    prev = stage("pool_parse", prev);

    if let Err(e) = force_pool_qualified_fill_for_test(&index) {
        eprintln!("measure_worker_private_memory: pool_qualified_fill failed: {e}");
        return ExitCode::FAILURE;
    }
    prev = stage("pool_qualified_fill", prev);

    for root in &source_roots {
        if let Err(e) = force_tree_bare_census_for_test(&index, root) {
            eprintln!("measure_worker_private_memory: tree_bare_census({root}) failed: {e}");
            return ExitCode::FAILURE;
        }
    }
    prev = stage("tree_bare_census", prev);

    if let Err(e) = force_pool_bare_census_for_test(&index) {
        eprintln!("measure_worker_private_memory: pool_bare_census failed: {e}");
        return ExitCode::FAILURE;
    }
    prev = stage("pool_bare_census", prev);

    if let Err(e) = force_both_closure_edges_for_test(&index) {
        eprintln!("measure_worker_private_memory: both_closure_edges failed: {e}");
        return ExitCode::FAILURE;
    }
    prev = stage("both_closure_edges", prev);
    let after_construction = prev;

    // Arm schedule retention over the cohort through the SAME closure authority
    // production arms from (`entry_closure_paths_for_test` wraps the loader's own
    // closure), with the eviction switch passed explicitly rather than through the
    // process-global env.
    if retention == "armed" {
        let mut per_entry: Vec<(String, Vec<String>)> = Vec::new();
        for entry in &entries {
            match entry_closure_paths_for_test(&index, entry) {
                Ok(paths) => per_entry.push((entry.clone(), paths)),
                // Production skips an unloadable closure too: its modules become counted
                // RetentionUnknown (retained) at reconcile. Skipping matches that arm.
                Err(_) => continue,
            }
        }
        println!("[wpm] retention armed over {} entr(y/ies)", per_entry.len());
        install_schedule_retention_for_test(&index, per_entry, true);
        prev = stage("retention_armed", prev);
    }

    // --- Warm: the worker's real per-entry resolve path. This is what populates
    // parse_cache, the diag caches, the typed cache and resolved_graph_memo. Under
    // `--retention armed` each entry-completion runs the production drain, so what the
    // warm accumulates is what a real worker actually holds.
    let mut resolved = 0usize;
    let mut failed = 0usize;
    let mut prev_entry: Option<(String, Option<String>)> = None;
    for (i, entry) in entries.iter().enumerate() {
        let keys_before: std::collections::HashSet<String> =
            resolved_graph_memo_keys_for_test(&index)
                .into_iter()
                .collect();
        match resolve_entry_with_index_for_discovery_corpus(&index, entry) {
            Ok(_) => resolved += 1,
            Err(_) => failed += 1,
        }
        if retention == "armed" {
            // Recover this entry's subject key by observing what the resolve added to the
            // memo, rather than re-deriving the subject digest in a second place.
            let subject = resolved_graph_memo_keys_for_test(&index)
                .into_iter()
                .find(|k| !keys_before.contains(k));
            // Production completes the PREVIOUS entry when the entry advances, so the
            // last entry stays held — reproduced exactly here.
            if let Some((prev_name, prev_subject)) = prev_entry.take() {
                if let Err(e) =
                    schedule_entry_completed_for_test(&index, &prev_name, prev_subject.as_deref())
                {
                    eprintln!("measure_worker_private_memory: entry_completed({prev_name}): {e}");
                    return ExitCode::FAILURE;
                }
            }
            prev_entry = Some((entry.clone(), subject));
        }
        if (i + 1) % 10 == 0 {
            prev = stage(&format!("warm_{}", i + 1), prev);
        }
    }
    prev = stage("warm_complete", prev);
    let after_warm = prev;
    println!("[wpm] warm resolved={resolved} failed={failed}");

    for (name, count) in private_term_entry_counts_for_test(&index) {
        println!("[wpm] count term={name} entries={count}");
    }

    println!(
        "[wpm] totals baseline_mib={:.1} construction_mib={:.1} warm_mib={:.1} \
         worker_private_mib={:.1} rss_mib={:.1} hwm_mib={:.1}",
        mib(baseline),
        mib(after_construction - after_shell),
        mib(after_warm.saturating_sub(after_construction)),
        mib(after_warm - baseline),
        mib(rss_bytes()),
        mib(hwm_bytes()),
    );

    // --- Exclusive-drop attribution. Clearing a term releases only the bytes NOTHING
    // else still holds. Sum-of-exclusives < total is the Rc-shared residue, reported
    // as such and never distributed across the terms that share it.
    let mut terms: Vec<&str> = drop_attributable_terms_for_test().to_vec();
    if drop_order == "reverse" {
        terms.reverse();
    }
    let before_drops = live_bytes();
    let mut attributed = 0usize;
    for term in &terms {
        let before = live_bytes();
        if !drop_private_term_for_test(&index, term) {
            eprintln!("measure_worker_private_memory: unknown term {term}");
            return ExitCode::FAILURE;
        }
        let after = live_bytes();
        let released = before.saturating_sub(after);
        attributed += released;
        println!(
            "[wpm] drop term={term} exclusive_release_mib={:.1} live_after_mib={:.1}",
            mib(released),
            mib(after),
        );
    }

    let residue = before_drops
        .saturating_sub(attributed)
        .saturating_sub(baseline);
    println!(
        "[wpm] attribution drop_order={drop_order} dropped_total_mib={:.1} \
         attributed_mib={:.1} unattributed_residue_mib={:.1} live_after_all_drops_mib={:.1}",
        mib(before_drops - baseline),
        mib(attributed),
        mib(residue),
        mib(live_bytes()),
    );
    println!(
        "[wpm] note unattributed residue = bytes still reachable after every term was \
         cleared (source_files root + Rc-shared structure). Not split across terms."
    );

    // Keep the index alive to the end so no term is released early by drop-order luck.
    drop(index);
    ExitCode::SUCCESS
}
