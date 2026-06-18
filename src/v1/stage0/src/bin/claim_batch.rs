//! Batch Bool-witness runner: build the module source index ONCE from the
//! source roots, then resolve each entry's import closure and run its witness
//! functions — all in a single process.
//!
//! Two use-cases are supported in one invocation:
//!
//! 1. **Shared-entry batch** (original #4719 pattern): one `--entry` with
//!    many `--function`/`--functions` flags.  The import closure is resolved
//!    once; all named functions run against the same graph.  This is the shape
//!    the node-frontier rows-fn's green pass (gate-3) uses.
//!
//! 2. **Multi-entry batch** (lens-gate extension, #4719 follow-on): multiple
//!    `--entry` flags each followed by its own `--function`/`--functions`
//!    flags.  The module source index is built once; each entry's closure is
//!    resolved separately in the same process.  This is what the v2 lens CI
//!    rows-fn's GREEN pass uses.
//!
//! Motivation. The v2 node-frontier rows-fn (now invoked from the single
//! `ci_claim_gate` binary) runs N Bool witnesses
//! that ALL share a single `--entry` file, differing only in `--function`.
//! The v2 lens CI rows-fn (same host) has one witness per
//! entry file.  Both previously ran `gunbc run --claim-run` once per
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
//! 3. **Discovery batch** (CI floor; absorbed from the retired
//!    `ci_claim_gate` binary). Instead of a hand-typed `--entry`/`--function`
//!    roster, reflect over the discovered `unified_claim_*` BoolWitness corpus
//!    plus the single-representation `test fn`/`test data` decls under the scan
//!    dirs, group the discovered rows by entry, and run them through the SAME
//!    batch loop. No hand roster, no rows-fn. The perturb pass that
//!    `ci_claim_gate` carried was flag-gated (`--perturb-check`) and never run
//!    in CI, so it is intentionally dropped with that binary.
//!
//! 4. **Discovery + explicit append** (consolidated CI floor witnesses):
//!    `--roster-from-discovery` may be combined with explicit `--entry`/
//!    `--function` rows. Discovered rows run first; explicit rows are appended
//!    with (entry, function) dedup. `--scan-dir` is optional when explicit rows
//!    are given (the `test fn` walk still runs over all `--source-root`s).
//!
//! Usage (discovery — the CI floor shape):
//!   claim_batch --source-root <dir> [--source-root <dir> ...] \
//!               --roster-from-discovery \
//!               [--scan-dir <dir> ...] \
//!               [--entry <file.dag> --function <fn> ...] \
//!               [--notice-title <title>] [--wet] [--hermetic]
//!
//! Exit codes: 0 = all witnesses returned Bool(true); 1 = any witness failed,
//! returned non-Bool, raised a runtime error, or resolve failed; 2 = usage
//! error (including: discovery roster produced an empty corpus → fail closed).
//!
//! Set GUNBC_INTERP_STATS=1 to print the phase-0 memory measurement report
//! (ctrl#1533) on stderr after the run.  In multi-entry mode only the
//! flatten-counter delta and RSS lines are printed (mutation counters are
//! per-context and each context is dropped after its entry's witnesses finish).
//!
//! Set GUNBC_INTERP_PROFILE=1 to additionally print, per witness, the
//! per-instruction eval breakdown (`[eval-profile]`): each `ExprData` variant's
//! eval count and self-time, sorted by self-time.  This answers "where do the
//! witness ms go" for the cost/complexity lens witnesses whose trivial Bool
//! result hides a multi-million-node symbolic tree walk.

// Binary entrypoint: it reports witness results directly on stdout/stderr, so
// println!/eprintln! are appropriate here (the disallowed-macros lint is aimed
// at library crates that should return structured errors). The generated
// main.rs carries the same allow.
#![allow(clippy::disallowed_macros)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::cli_run::{
    build_multi_entry_index, check_floor_filename_hygiene, discover_floor_corpus_rows,
    make_eval_context_with_fixture_store, resolve_entry_with_index, run_claim, ClaimOutcome,
    DiscoveryRow, MultiEntryIndex,
};
use v1_compiler::recorded_fixture::RecordedFixtureStore;
use v1_compiler::v1_compiler_compile::ResolvedGraph;
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext};
use v1_compiler::v1_std_core::NewlineIndex;

/// What `resolve_entry_with_index` hands back: the resolved import-closure graph
/// plus the per-file newline indices used for span reporting.
type ResolvedEntry = (Rc<ResolvedGraph>, Rc<HashMap<String, Rc<NewlineIndex>>>);

/// Per-phase wall-clock / closure-size accounting for the green pass.
///
/// The lens + node-frontier CI jobs spend most of their wall in this binary's
/// resolves (`build_module_index` + closure-compile is ~5-13s per distinct
/// entry; the witness eval that follows is comparatively cheap). The CI latency
/// attack (2026-06-13) needs that split visible per row, so every resolve
/// reports its wall plus closure size (modules scanned + resolved items) and
/// every witness reports its eval wall. The end-of-run summary lets a reader
/// confirm "resolve dominates" at a glance instead of timestamp archaeology.
#[derive(Default)]
struct ResolveTimings {
    resolves: u64,
    resolve_ms: u128,
    witnesses: u64,
    witness_ms: u128,
}

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
    let (snap_calls, snap_items) = v1_compiler::v1_interpreter::flatten_counters_snapshot();
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
    let intern = ctx.interner_stats_snapshot();
    eprintln!("[interp-stats] symbol interning receipt (#4799; this context):");
    eprintln!(
        "  {:<12} {:>12} calls  {:>16} distinct  {:>16} hits  (dedup {:.1}x; {} heap bytes)",
        "intern",
        intern.calls,
        intern.distinct,
        intern.hits,
        if intern.distinct == 0 {
            0.0
        } else {
            intern.calls as f64 / intern.distinct as f64
        },
        intern.heap_bytes
    );
    eprintln!("[interp-stats] retained value accounting (data cache + pure-call memo):");
    eprint!("{}", ctx.account_retained_memory(&[]));
    eprintln!("[interp-stats] process memory:");
    eprint!("{}", peak_rss_lines());
}

// Multi-entry mode: per-context mutation counters are not available (each
// context is dropped after its entry finishes). Print flatten counters + RSS.
fn print_interp_stats_multi_entry(flatten_baseline: (u64, u64)) {
    let (snap_calls, snap_items) = v1_compiler::v1_interpreter::flatten_counters_snapshot();
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

// The discovery ROSTER (exclude set, `unified_claim_*`/`test fn`/`test data`
// reflection, filename hygiene, `*_test.dag`-only convention) is the single
// authority in `cli_run` (`discover_floor_corpus_rows` /
// `check_floor_filename_hygiene`), shared with the scheduler's `DiscoveryBatch`
// node — see those functions. This bin keeps only its timing/stats run loop.

/// Parsed CLI config. Roster is sourced one of two ways: an explicit
/// `--entry`/`--function` list (`entry_groups`), or `--roster-from-discovery`
/// reflection over the scan dirs (`discovery`). The two are mutually exclusive.
struct ParsedArgs {
    source_roots: Vec<String>,
    entry_groups: Vec<EntryGroup>,
    discovery: Option<DiscoveryConfig>,
    /// Witness execution mode. Phase 1 default: Wet (CI unchanged).
    /// `--hermetic` replays recorded fixtures when `--fixture-store` is set, else
    /// modeled `mock_response`; `--record` is wet capture into the fixture store.
    execution_mode: ExecutionMode,
    /// Directory for recorded fixture JSON files (`--fixture-store <path>`).
    fixture_store: Option<PathBuf>,
}

/// Discovery-mode config (set when `--roster-from-discovery` is given).
struct DiscoveryConfig {
    scan_dirs: Vec<String>,
    notice_title: String,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, ExitCode> {
    let mut source_roots: Vec<String> = Vec::new();
    let mut entry_groups: Vec<EntryGroup> = Vec::new();
    let mut roster_from_discovery = false;
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut notice_title = "v2 CI claim gate".to_string();
    // Phase 1: default Wet so CI behavior is unchanged. Phase 2 flips default
    // to Hermetic; `--wet` then opts into live dispatch for real-I/O witnesses.
    let mut execution_mode = ExecutionMode::Wet;
    let mut fixture_store: Option<PathBuf> = None;

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
            "--roster-from-discovery" => roster_from_discovery = true,
            "--scan-dir" => {
                i += 1;
                scan_dirs.push(require_value(args, i, "--scan-dir")?);
            }
            "--notice-title" => {
                i += 1;
                notice_title = require_value(args, i, "--notice-title")?;
            }
            // Accepted for call-site parity with `gunbc run`; this bin is always
            // claim-run (Bool witnesses).
            "--claim-run" => {}
            "--wet" => execution_mode = ExecutionMode::Wet,
            "--hermetic" => execution_mode = ExecutionMode::Hermetic,
            "--record" => execution_mode = ExecutionMode::Record,
            "--fixture-store" => {
                i += 1;
                fixture_store = Some(PathBuf::from(require_value(args, i, "--fixture-store")?));
            }
            other => {
                eprintln!("claim_batch: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    let discovery = if roster_from_discovery {
        if scan_dirs.is_empty() && entry_groups.is_empty() {
            eprintln!(
                "claim_batch: --roster-from-discovery requires at least one --scan-dir and/or explicit --entry row"
            );
            return Err(ExitCode::from(2));
        }
        Some(DiscoveryConfig {
            scan_dirs,
            notice_title,
        })
    } else {
        None
    };

    Ok(ParsedArgs {
        source_roots,
        entry_groups,
        discovery,
        execution_mode,
        fixture_store,
    })
}

/// Report one witness outcome on stdout (PASS/FAIL line consumed by callers),
/// flipping `any_failed` on any non-pass. Shared by the single- and multi-entry
/// paths so the failure-reporting shape stays in one place.
fn report_outcome(function: &str, outcome: ClaimOutcome, any_failed: &mut bool) {
    match outcome {
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

/// Resolve one entry's closure, timing the resolve and reporting its wall plus
/// closure size (modules in the import closure, resolved items) on stderr.
fn resolve_timed(
    index: &MultiEntryIndex,
    entry: &str,
    timings: &mut ResolveTimings,
) -> Result<ResolvedEntry, ExitCode> {
    let started = Instant::now();
    match resolve_entry_with_index(index, entry) {
        Ok((graph, source_indices)) => {
            let ms = started.elapsed().as_millis();
            eprintln!(
                "[resolve] {}: {}ms  ({} modules, {} resolved items in closure)",
                entry,
                ms,
                graph.modules.len(),
                graph.item_registry.len(),
            );
            timings.resolves += 1;
            timings.resolve_ms += ms;
            Ok((graph, source_indices))
        }
        Err(msg) => {
            eprintln!("claim_batch: resolve failed for {}:\n{}", entry, msg);
            Err(ExitCode::from(1))
        }
    }
}

/// Run one witness, timing its eval and reporting it on stderr.
fn run_claim_timed(
    ctx: &InterpContext,
    function: &str,
    any_failed: &mut bool,
    timings: &mut ResolveTimings,
) {
    // Under GUNBC_INTERP_PROFILE=1 the interpreter records a per-instruction
    // eval histogram; reset before this witness so the breakdown is per-row.
    v1_compiler::v1_interpreter::eval_profile_reset();
    let started = Instant::now();
    let outcome = run_claim(ctx, function);
    let ms = started.elapsed().as_millis();
    report_outcome(function, outcome, any_failed);
    eprintln!("[witness] {}: {}ms", function, ms);
    print_eval_profile(function);
    timings.witnesses += 1;
    timings.witness_ms += ms;
}

/// Print the per-instruction eval breakdown for the witness just run, sorted by
/// self-time. No-op unless GUNBC_INTERP_PROFILE=1 produced a profile (all-zero
/// snapshot ⇒ profiling was off). Answers "where do the witness ms go" —
/// each `ExprData` variant's eval count and self-nanoseconds (gross frame time
/// minus child eval frames), so a reader sees the hot instruction directly.
fn print_eval_profile(function: &str) {
    use v1_compiler::v1_interpreter::{
        eval_profile_snapshot, expr_variant_name, EXPR_VARIANT_COUNT,
    };
    let prof = eval_profile_snapshot();
    let total_ns: u128 = prof.self_nanos.iter().sum();
    let total_count: u64 = prof.counts.iter().sum();
    if total_count == 0 {
        return; // profiling disabled
    }
    let mut rows: Vec<usize> = (0..EXPR_VARIANT_COUNT)
        .filter(|&i| prof.counts[i] > 0)
        .collect();
    rows.sort_by(|&a, &b| prof.self_nanos[b].cmp(&prof.self_nanos[a]));
    eprintln!(
        "[eval-profile] {}: {} node-evals, {:.3}ms self-time total (sorted by self-time)",
        function,
        total_count,
        total_ns as f64 / 1.0e6,
    );
    for i in rows {
        let ns = prof.self_nanos[i];
        let count = prof.counts[i];
        eprintln!(
            "  {:<16} {:>12} evals  {:>10.3}ms self ({:>5.1}%)  {:>8.0}ns/eval",
            expr_variant_name(i),
            count,
            ns as f64 / 1.0e6,
            if total_ns == 0 {
                0.0
            } else {
                100.0 * ns as f64 / total_ns as f64
            },
            if count == 0 {
                0.0
            } else {
                ns as f64 / count as f64
            },
        );
    }
}

fn fixture_store_rc(path: &Option<PathBuf>) -> Option<Rc<RecordedFixtureStore>> {
    path.as_ref()
        .map(|p| Rc::new(RecordedFixtureStore::open(p.clone())))
}

fn validate_fixture_flags(
    execution_mode: ExecutionMode,
    fixture_store: &Option<PathBuf>,
) -> Result<(), ExitCode> {
    if execution_mode.is_record() && fixture_store.is_none() {
        eprintln!("claim_batch: --record requires --fixture-store <path>");
        return Err(ExitCode::from(2));
    }
    Ok(())
}

fn run_witnesses(
    index: &MultiEntryIndex,
    group: &EntryGroup,
    execution_mode: ExecutionMode,
    fixture_store: Option<Rc<RecordedFixtureStore>>,
    any_failed: &mut bool,
    timings: &mut ResolveTimings,
) -> Result<(), ExitCode> {
    let (graph, source_indices) = resolve_timed(index, &group.entry, timings)?;
    let ctx =
        make_eval_context_with_fixture_store(&graph, source_indices, execution_mode, fixture_store);
    for function in &group.functions {
        run_claim_timed(&ctx, function, any_failed, timings);
    }
    Ok(())
}

/// Group discovered rows into per-entry `EntryGroup`s, preserving the discovery
/// sort order (rows are sorted by (entry, function) in `discover_floor_corpus_rows`).
fn group_discovered_rows(rows: Vec<DiscoveryRow>) -> Vec<EntryGroup> {
    let mut groups: Vec<EntryGroup> = Vec::new();
    for row in rows {
        match groups.last_mut() {
            Some(g) if g.entry == row.entry => g.functions.push(row.function),
            _ => groups.push(EntryGroup {
                entry: row.entry,
                functions: vec![row.function],
            }),
        }
    }
    groups
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_args(&args)?;
    let source_roots = parsed.source_roots;
    let execution_mode = parsed.execution_mode;
    let fixture_store_path = parsed.fixture_store;
    validate_fixture_flags(execution_mode, &fixture_store_path)?;
    let fixture_store = fixture_store_rc(&fixture_store_path);

    if source_roots.is_empty() {
        eprintln!("claim_batch: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }

    // Discovery mode: reflect over the scan dirs for the roster, optionally append
    // explicit `--entry` rows, group by entry, and run through the SAME batch
    // loop the explicit-only path uses.
    let (entry_groups, discovery_notice) = if let Some(disc) = parsed.discovery {
        // Filename hygiene (fail-closed): no `__` in any `.dag` basename under the
        // source roots — folder-based naming, no legacy flat-dir encoding (#5051,
        // preserved from the retired ci_claim_gate).
        if let Err(e) = check_floor_filename_hygiene(&source_roots) {
            eprintln!("claim_batch: {e}");
            return Err(ExitCode::from(2));
        }
        let mut rows = match discover_floor_corpus_rows(&source_roots, &disc.scan_dirs) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("claim_batch: discovery roster failed: {e}");
                return Err(ExitCode::from(2));
            }
        };
        let mut seen: std::collections::BTreeSet<(String, String)> = rows
            .iter()
            .map(|r| (r.entry.clone(), r.function.clone()))
            .collect();
        for group in &parsed.entry_groups {
            for function in &group.functions {
                if seen.insert((group.entry.clone(), function.clone())) {
                    rows.push(DiscoveryRow {
                        label: function.clone(),
                        entry: group.entry.clone(),
                        function: function.clone(),
                    });
                }
            }
        }
        if rows.is_empty() {
            eprintln!("claim_batch: roster produced no rows (empty corpus → fail closed)");
            return Err(ExitCode::from(2));
        }
        rows.sort_by(|a, b| {
            a.entry
                .cmp(&b.entry)
                .then_with(|| a.function.cmp(&b.function))
        });
        (group_discovered_rows(rows), Some(disc.notice_title))
    } else {
        (parsed.entry_groups, None)
    };

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

    let flatten_baseline = v1_compiler::v1_interpreter::flatten_counters_snapshot();
    let stats_requested = std::env::var_os("GUNBC_INTERP_STATS").is_some_and(|v| v != "0");

    let mut any_failed = false;
    let mut timings = ResolveTimings::default();

    if entry_groups.len() == 1 {
        // Single-entry path: keep `ctx` alive for the full stats report.
        let group = &entry_groups[0];
        eprintln!(
            "claim_batch: resolved {} once; running {} witness(es)",
            group.entry,
            group.functions.len()
        );
        let (graph, source_indices) = resolve_timed(&index, &group.entry, &mut timings)?;
        let ctx = make_eval_context_with_fixture_store(
            &graph,
            source_indices,
            execution_mode,
            fixture_store.clone(),
        );
        for function in &group.functions {
            run_claim_timed(&ctx, function, &mut any_failed, &mut timings);
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
            run_witnesses(
                &index,
                group,
                execution_mode,
                fixture_store.clone(),
                &mut any_failed,
                &mut timings,
            )?;
        }
        if stats_requested {
            print_interp_stats_multi_entry(flatten_baseline);
        }
    }

    // Phase split for the CI latency attack: confirm resolve dominates the green
    // pass at a glance (the index build above is amortized once across all rows).
    eprintln!(
        "[resolve-summary] {} resolve(s) in {}ms; {} witness(es) in {}ms",
        timings.resolves, timings.resolve_ms, timings.witnesses, timings.witness_ms,
    );

    if any_failed {
        return Ok(ExitCode::from(1));
    }

    // Discovery mode emits the CI floor notice (parity with the retired
    // ci_claim_gate): `"{title}: {N} discriminating witness(es) passed"`.
    if let Some(title) = discovery_notice {
        println!("{title}: {total_witnesses} discriminating witness(es) passed");
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}

#[cfg(test)]
mod discovery_exclude_tests {
    use v1_compiler::cli_run::floor_discovery_path_excluded;

    #[test]
    fn real_ingest_gate_only_path_excluded_on_ci_style_paths() {
        let paths = [
            "src/v2/test/claim/program_assembly/real_ingest_test.dag",
            "/opt/actions-runner/work/gunbc/gunbc/src/v2/test/claim/program_assembly/real_ingest_test.dag",
        ];
        for path in paths {
            assert!(
                floor_discovery_path_excluded(path),
                "expected gate-only overlay witness excluded: {path}"
            );
        }
    }

    #[test]
    fn sg2_manual_lane_unified_claim_excluded_on_ci_style_paths() {
        let paths = [
            "src/v2/compiler/manual/sg2_type_expression_projection.dag",
            "/opt/actions-runner/work/gunbc/gunbc/src/v2/compiler/manual/sg2_type_expression_projection.dag",
        ];
        for path in paths {
            assert!(
                floor_discovery_path_excluded(path),
                "expected manual-lane sg2 unified_claim excluded from floor discovery: {path}"
            );
        }
    }
}
