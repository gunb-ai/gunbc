#![allow(clippy::disallowed_macros)]

use im::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::cli_run::{
    build_multi_entry_index, check_floor_filename_hygiene, closure_subject_for_entry,
    discover_floor_corpus_rows, make_eval_context_with_runtime_options, peak_rss_vhwm_bytes,
    precompute_whole_tree_published_mock_keys, resolve_entry_with_index, run_claim_measured,
    witness_exclusion_substrings, ClaimOutcome, DiscoveryRow, MultiEntryIndex,
};
use v1_compiler::recorded_fixture::RecordedFixtureStore;
use v1_compiler::v1_compiler_compile::ResolvedGraph;
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext};
use v1_compiler::v1_std_core::NewlineIndex;

type ResolvedEntry = (Rc<ResolvedGraph>, Rc<HashMap<String, Rc<NewlineIndex>>>);

#[derive(Default)]
struct ResolveTimings {
    resolves: u64,
    resolve_ms: u128,
    witnesses: u64,
    witness_ms: u128,
}

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

fn peak_rss_bytes() -> Option<u64> {
    peak_rss_vhwm_bytes()
}

/// Peak RSS of terminated child processes (Linux `getrusage(RUSAGE_CHILDREN)`).
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
fn children_max_rss_bytes() -> Option<u64> {
    // Hand-decoded `struct rusage` (no libc dep): layout assumes Linux lp64
    // (x86_64 / aarch64) — 144-byte buffer, `ru_maxrss` at byte offset 32 after
    // two 16-byte `timeval`s. Non-lp64 Linux is gated out below; syscall failure → None.
    // If this probe survives phase-0, replace with `libc::rusage` (dissolves with PerformanceReceipt).
    extern "C" {
        fn getrusage(who: i32, usage: *mut std::ffi::c_void) -> i32;
    }
    const RUSAGE_CHILDREN: i32 = -1;
    const RU_MAXRSS_OFFSET: usize = 32; // after two struct timeval (16 bytes each)
    let mut buf = [0u8; 144];
    if unsafe { getrusage(RUSAGE_CHILDREN, buf.as_mut_ptr().cast()) } != 0 {
        return None;
    }
    let ru_maxrss = i64::from_ne_bytes(
        buf[RU_MAXRSS_OFFSET..RU_MAXRSS_OFFSET + 8]
            .try_into()
            .ok()?,
    );
    Some(ru_maxrss as u64 * 1024)
}

#[cfg(all(target_os = "linux", not(target_pointer_width = "64")))]
fn children_max_rss_bytes() -> Option<u64> {
    None
}

#[cfg(not(target_os = "linux"))]
fn children_max_rss_bytes() -> Option<u64> {
    None
}

fn emit_rss_measurement(label: &str) {
    match peak_rss_bytes() {
        Some(bytes) => eprintln!("[measurement] {label}: {bytes} bytes (VmHWM)"),
        None => eprintln!("[measurement] {label}: unavailable (no /proc/self/status)"),
    }
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

/// Path-valued arguments resolve against the PROCESS CWD at the CLI boundary, refusing
/// on a nonexistent path — never falling back to the compile-time-baked workspace root
/// (`v1_compiler::cli_run::resolve_cli_path_arg`; DESIGN §5 fail-open closed there).
fn require_path_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    let given = require_value(args, idx, flag)?;
    match v1_compiler::cli_run::resolve_cli_path_arg("claim_batch", flag, &given) {
        Ok(resolved) => Ok(resolved),
        Err(msg) => {
            eprintln!("{msg}");
            Err(ExitCode::from(2))
        }
    }
}

struct EntryGroup {
    entry: String,
    functions: Vec<String>,
}

struct ParsedArgs {
    source_roots: Vec<String>,
    entry_groups: Vec<EntryGroup>,
    discovery: Option<DiscoveryConfig>,
    execution_mode: ExecutionMode,
    fixture_store: Option<PathBuf>,
    eval_budget_ms: Option<u64>,
    pre_push: bool,
}

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
    let mut execution_mode = ExecutionMode::Hermetic;
    let mut fixture_store: Option<PathBuf> = None;
    let mut eval_budget_ms: Option<u64> = None;
    let mut pre_push = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_path_value(args, i, "--source-root")?);
            }
            "--entry" => {
                i += 1;
                let entry = require_path_value(args, i, "--entry")?;
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
                scan_dirs.push(require_path_value(args, i, "--scan-dir")?);
            }
            "--notice-title" => {
                i += 1;
                notice_title = require_value(args, i, "--notice-title")?;
            }
            "--claim-run" => {}
            "--wet" => execution_mode = ExecutionMode::Wet,
            "--hermetic" => execution_mode = ExecutionMode::Hermetic,
            "--record" => execution_mode = ExecutionMode::Record,
            "--eval-budget-ms" => {
                i += 1;
                let v = require_value(args, i, "--eval-budget-ms")?;
                let ms: u64 = v.parse().map_err(|_| {
                    eprintln!(
                        "claim_batch: --eval-budget-ms requires a positive integer, got {v:?}"
                    );
                    ExitCode::from(2)
                })?;
                eval_budget_ms = Some(ms);
            }
            "--fixture-store" => {
                i += 1;
                fixture_store = Some(PathBuf::from(require_value(args, i, "--fixture-store")?));
            }
            "--pre-push" => pre_push = true,
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
        eval_budget_ms,
        pre_push,
    })
}

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
            if timings.resolves == 1 {
                emit_rss_measurement("post-first-entry-resolve-rss");
            }
            Ok((graph, source_indices))
        }
        Err(msg) => {
            eprintln!("claim_batch: resolve failed for {}:\n{}", entry, msg);
            Err(ExitCode::from(1))
        }
    }
}

fn run_claim_timed(
    ctx: &InterpContext,
    closure_subject: &str,
    function: &str,
    any_failed: &mut bool,
    timings: &mut ResolveTimings,
) {
    let (outcome, receipt) = run_claim_measured(ctx, closure_subject, function);
    report_outcome(function, outcome, any_failed);
    eprintln!(
        "[witness] {}: {}ms subject={} eval_self={:.3}ms",
        function,
        receipt.wall_nanos / 1_000_000,
        receipt.subject_key,
        receipt.eval_self_nanos as f64 / 1.0e6,
    );
    print_eval_profile(function);
    timings.witnesses += 1;
    timings.witness_ms += receipt.wall_nanos / 1_000_000;
}

fn print_eval_profile(function: &str) {
    use v1_compiler::v1_interpreter::{
        eval_profile_snapshot, expr_variant_name, EXPR_VARIANT_COUNT,
    };
    let prof = eval_profile_snapshot();
    let total_ns: u128 = prof.self_nanos.iter().sum();
    let total_count: u64 = prof.counts.iter().sum();
    if total_count == 0 {
        return;
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
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    eval_budget_ms: Option<u64>,
    any_failed: &mut bool,
    timings: &mut ResolveTimings,
) -> Result<(), ExitCode> {
    let (graph, source_indices) = resolve_timed(index, &group.entry, timings)?;
    let closure_subject = closure_subject_for_entry(index, &group.entry).map_err(|e| {
        eprintln!("claim_batch: closure subject for {}: {e}", group.entry);
        ExitCode::from(1)
    })?;
    let ctx = make_eval_context_with_runtime_options(
        &graph,
        source_indices,
        execution_mode,
        fixture_store,
        whole_tree_published_keys,
    );
    ctx.set_witness_eval_budget(eval_budget_ms);
    for function in &group.functions {
        run_claim_timed(&ctx, &closure_subject, function, any_failed, timings);
        // The eval-call memo's eviction scope is the witness frame, not this
        // shared per-entry ctx — ctx-lifetime retention of argument+result
        // values across witnesses is byte-unbounded (20GiB-class kills).
        v1_compiler::v1_interpreter::eval_call_memo_frame_exit(&ctx);
    }
    Ok(())
}

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
    if parsed.pre_push {
        return Ok(v1_compiler::cli_run::handle_pre_push());
    }
    let source_roots = parsed.source_roots;
    let execution_mode = parsed.execution_mode;
    let eval_budget_ms = parsed.eval_budget_ms;
    let fixture_store_path = parsed.fixture_store;
    validate_fixture_flags(execution_mode, &fixture_store_path)?;
    let fixture_store = fixture_store_rc(&fixture_store_path);

    if source_roots.is_empty() {
        eprintln!("claim_batch: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }

    // Funnel host-effect traces per the .dag output policy (see claim_executor).
    v1_compiler::cli_run::install_output_policy(&source_roots);
    // GUNBC_FLOOR_PHASE_PROFILE support (same as claim_executor): without this,
    // claim_batch diagnostics cannot attribute time to resolve/typecheck/eval
    // phases — a 20-minute silent resolve is uninterpretable.
    let _phase_profile = v1_compiler::cli_run::PhaseProfile::install_from_env();

    let (entry_groups, discovery_notice) = if let Some(disc) = parsed.discovery {
        if let Err(e) = check_floor_filename_hygiene(&source_roots) {
            eprintln!("claim_batch: {e}");
            return Err(ExitCode::from(2));
        }
        let excludes = witness_exclusion_substrings();
        let mut rows = match discover_floor_corpus_rows(&source_roots, &disc.scan_dirs, &excludes) {
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
                        // claim_batch runs every row (no selection); the undeclared
                        // fail-closed default is the honest fill for a transient row.
                        reads_live_tree: true,
                    });
                }
            }
        }
        if rows.is_empty() {
            eprintln!("claim_batch: roster produced no rows (empty corpus → fail closed)");
            return Err(ExitCode::from(2));
        }
        // P4 advisory-first: predict the memory-packed width per witness from its
        // derived space bound, logged for offline comparison against the run's peak
        // RSS. Gated (opt-in); no scheduling change.
        if std::env::var("GUNBC_REALIZE_ADVISORY").is_ok() {
            v1_compiler::cli_run::emit_realize_advisory_for_rows(&source_roots, &rows);
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

    let index = build_multi_entry_index(&source_roots);

    let whole_tree_published_keys = match precompute_whole_tree_published_mock_keys(&source_roots) {
        Ok(keys) => {
            emit_rss_measurement("post-mock-precompute-rss");
            if let Some(bytes) = children_max_rss_bytes() {
                eprintln!("[measurement] post-mock-precompute-children-max-rss: {bytes} bytes (getrusage RUSAGE_CHILDREN)");
            }
            if keys.is_empty() {
                eprintln!(
                    "claim_batch: whole-tree published mock corpus — no dag/ corpora precomputed; \
                     using entry-closure fallback per witness"
                );
                None
            } else {
                eprintln!(
                    "claim_batch: whole-tree published mock corpus — {} operation key(s)",
                    keys.len()
                );
                Some(Rc::new(keys))
            }
        }
        Err(e) => {
            eprintln!("claim_batch: whole-tree published mock corpus precompute failed: {e}");
            return Err(ExitCode::from(1));
        }
    };

    let flatten_baseline = v1_compiler::v1_interpreter::flatten_counters_snapshot();
    let stats_requested = std::env::var_os("GUNBC_INTERP_STATS").is_some_and(|v| v != "0");

    let mut any_failed = false;
    let mut timings = ResolveTimings::default();

    if entry_groups.len() == 1 {
        let group = &entry_groups[0];
        // Two lines so the log cannot lie about sequencing: the past-tense line
        // printed BEFORE the resolve once mis-attributed a resolve-phase OOM to
        // witness eval (eager-ram-612 bisect, 2026-07-10).
        eprintln!(
            "claim_batch: resolving {} once ({} witness(es))",
            group.entry,
            group.functions.len()
        );
        let (graph, source_indices) = resolve_timed(&index, &group.entry, &mut timings)?;
        eprintln!(
            "claim_batch: resolved {}; running {} witness(es)",
            group.entry,
            group.functions.len()
        );
        let closure_subject = closure_subject_for_entry(&index, &group.entry).map_err(|e| {
            eprintln!("claim_batch: closure subject for {}: {e}", group.entry);
            ExitCode::from(1)
        })?;
        let ctx = make_eval_context_with_runtime_options(
            &graph,
            source_indices,
            execution_mode,
            fixture_store.clone(),
            whole_tree_published_keys.clone(),
        );
        ctx.set_witness_eval_budget(eval_budget_ms);
        for function in &group.functions {
            run_claim_timed(
                &ctx,
                &closure_subject,
                function,
                &mut any_failed,
                &mut timings,
            );
            // Witness frame exit on the single-entry fast path too — this is
            // the exact path the 6-witness 20GiB-kill recipe runs (the memo
            // must not retain values across witnesses sharing this ctx).
            v1_compiler::v1_interpreter::eval_call_memo_frame_exit(&ctx);
        }
        if stats_requested {
            print_interp_stats(&ctx, flatten_baseline);
        }
    } else {
        for group in &entry_groups {
            eprintln!(
                "claim_batch: resolving {} ({} witness(es))",
                group.entry,
                group.functions.len()
            );
            // A group whose resolve fails is COUNTED — every enrolled witness in
            // it reports FAIL and the batch continues (exit stays 1 via
            // any_failed). Aborting the whole batch on the first red entry
            // truncated the measurement: each run revealed only the NEXT red
            // class, and everything alphabetically after it went unmeasured.
            if run_witnesses(
                &index,
                group,
                execution_mode,
                fixture_store.clone(),
                whole_tree_published_keys.clone(),
                eval_budget_ms,
                &mut any_failed,
                &mut timings,
            )
            .is_err()
            {
                for function in &group.functions {
                    println!("FAIL {} (entry resolve failed: {})", function, group.entry);
                }
                any_failed = true;
            }
        }
        if stats_requested {
            print_interp_stats_multi_entry(flatten_baseline);
        }
    }

    eprintln!(
        "[resolve-summary] {} resolve(s) in {}ms; {} witness(es) in {}ms",
        timings.resolves, timings.resolve_ms, timings.witnesses, timings.witness_ms,
    );

    emit_rss_measurement("per-shard-peak-rss");
    if let Some(bytes) = children_max_rss_bytes() {
        eprintln!("[measurement] children-max-rss: {bytes} bytes (getrusage RUSAGE_CHILDREN)");
    }

    if any_failed {
        return Ok(ExitCode::from(1));
    }

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
mod cli_path_resolution_wiring_tests {
    use super::parse_args;
    use v1_compiler::cli_run::workspace_root;

    fn args(v: &[&str]) -> Vec<String> {
        std::iter::once("claim_batch")
            .chain(v.iter().copied())
            .map(str::to_string)
            .collect()
    }

    /// Absolute existing paths pass parse_args unchanged (resolution applied, no rewrite).
    #[test]
    fn absolute_existing_args_parse_and_survive_resolution() {
        let ws = workspace_root();
        let root = ws.join("dag").to_string_lossy().into_owned();
        let entry = ws
            .join("dag/test/claim/commit_workflow_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let parsed = parse_args(&args(&[
            "--source-root",
            &root,
            "--entry",
            &entry,
            "--function",
            "commit_workflow_witnesses",
        ]))
        .unwrap_or_else(|_| panic!("absolute existing paths must parse"));
        assert_eq!(parsed.source_roots, vec![root]);
        assert_eq!(parsed.entry_groups.len(), 1);
        assert_eq!(parsed.entry_groups[0].entry, entry);
    }

    /// A nonexistent --source-root refuses at the CLI boundary (exit-code error), never
    /// proceeding to a partial run.
    #[test]
    fn nonexistent_source_root_refuses_at_parse() {
        assert!(parse_args(&args(&["--source-root", "/no/such/tree"])).is_err());
    }

    /// Relative args resolve against the PROCESS CWD, not the baked workspace root.
    /// `cargo test` runs bin unit tests with cwd == the package dir (src/v1/stage0),
    /// where `dag` does not exist — while it DOES exist under the baked workspace root.
    /// parse_args must therefore agree with the cwd, whichever it is: refuse when
    /// cwd/dag is absent (a baked-root fallback would accept), accept when present.
    #[test]
    fn relative_source_root_follows_process_cwd_not_baked_root() {
        let cwd = std::env::current_dir().expect("test cwd");
        let cwd_has_dag = cwd.join("dag").is_dir();
        let result = parse_args(&args(&[
            "--source-root",
            "dag",
            "--entry",
            "dag/test/claim/commit_workflow_witness_test.dag",
            "--function",
            "commit_workflow_witnesses",
        ]));
        assert_eq!(
            result.is_ok(),
            cwd_has_dag,
            "relative --source-root must resolve against the process cwd {} (dag present: {cwd_has_dag}), never the baked workspace root {}",
            cwd.display(),
            workspace_root().display()
        );
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
}
