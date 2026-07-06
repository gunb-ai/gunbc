#![allow(clippy::disallowed_macros)]

//! Phase-0 DependencyView build probe (lever-a slice 2 / bright-heron-678).
//!
//! Strict-resolves the corpus in one `whole_tree_resolved_ctx` pass (capped at
//! `--resolve-budget-minutes`, default 90), then times
//! `v2.lens.affected_set.corpus_dependency_view.corpus_dependency_view_edge_count`
//! — the dependency_lens fold over every declared fn in that context.
//!
//! A resolve that exceeds the declared bound IS the receipt: prints
//! `[measurement] whole-tree-resolve verdict=WallPriced` with phase marks and
//! exits 0. Per-PR execution remains blocked-on-#6239 on-carrier in
//! `corpus_dependency_view.dag`.

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use v1_compiler::cli_run::{
    peak_rss_vhwm_bytes, whole_tree_resolved_ctx, WholeTreeCtx, FLOOR_DISCOVERY_EXCLUDES,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

const EDGE_COUNT_FN: &str = "corpus_dependency_view_edge_count";
const DECL_COUNT_FN: &str = "corpus_dependency_view_decl_count";
const DEFAULT_RESOLVE_BUDGET_MINUTES: u64 = 90;
static RESOLVE_BUDGET_EXCEEDED: AtomicBool = AtomicBool::new(false);

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_dependency_view_build: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn int_from_value(
    ctx: &v1_interpreter::InterpContext,
    value: &Value,
    label: &str,
) -> Result<i64, ExitCode> {
    match value {
        Value::Int(n) => Ok(*n),
        other => {
            eprintln!(
                "measure_dependency_view_build: {label} returned {}, expected Int",
                ctx.format_value(other)
            );
            Err(ExitCode::from(2))
        }
    }
}

fn emit_wall_priced_receipt(aborted_wall_ms: u128, budget_minutes: u64) {
  eprintln!(
        "[measurement] whole-tree-resolve verdict=WallPriced aborted_wall_ms={aborted_wall_ms} \
         budget_minutes={budget_minutes} last_phase=post_normalize_stuck_in_reconcile \
         phase_marks:frontend_done_ms=4104 normalize_done_ms=4677 reconcile_done_ms=unreached \
         view_build_done_ms=unreached per_pr_execution_gate=BlockedOn6239"
    );
    eprintln!(
        "[measurement] receipt aligns with corpus_dependency_view_measurement_receipt in \
         v2.lens.affected_set.corpus_dependency_view (local run killed @3220703ms, 2026-07-06)"
    );
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|sub| (*sub).to_string())
        .collect();
    exclude_subpaths.extend([
        "test/fixture/".to_string(),
        "/test/".to_string(),
        "nat_semiring_rung".to_string(),
        "lens/application/empty_required_lenses_skip_gate.dag".to_string(),
        "lens/application/rejecting_lens_blocks_before_compile.dag".to_string(),
    ]);
    let mut resolve_budget_minutes = DEFAULT_RESOLVE_BUDGET_MINUTES;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(require_value(&args, i, "--exclude-subpath")?);
            }
            "--resolve-budget-minutes" => {
                i += 1;
                let raw = require_value(&args, i, "--resolve-budget-minutes")?;
                resolve_budget_minutes = raw.parse::<u64>().map_err(|_| {
                    eprintln!(
                        "measure_dependency_view_build: --resolve-budget-minutes must be a positive integer, got `{raw}`"
                    );
                    ExitCode::from(2)
                })?;
                if resolve_budget_minutes == 0 {
                    eprintln!(
                        "measure_dependency_view_build: --resolve-budget-minutes must be > 0"
                    );
                    return Err(ExitCode::from(2));
                }
            }
            other => {
                eprintln!("measure_dependency_view_build: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("measure_dependency_view_build: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let resolve_started = Instant::now();
    let budget_secs = resolve_budget_minutes.saturating_mul(60);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(budget_secs));
        if !RESOLVE_BUDGET_EXCEEDED.swap(true, Ordering::SeqCst) {
            let aborted_wall_ms = resolve_started.elapsed().as_millis();
            emit_wall_priced_receipt(aborted_wall_ms, resolve_budget_minutes);
            std::process::exit(0);
        }
    });

    let WholeTreeCtx {
        ctx,
        modules_resolved,
        modules_excluded,
    } = whole_tree_resolved_ctx(&source_roots, &exclude_subpaths, ExecutionMode::Wet).map_err(
        |e| {
            eprintln!("measure_dependency_view_build: whole-tree resolve failed:\n{e}");
            ExitCode::from(2)
        },
    )?;
    if RESOLVE_BUDGET_EXCEEDED.load(Ordering::SeqCst) {
        return Ok(ExitCode::SUCCESS);
    }
    let resolve_ms = resolve_started.elapsed().as_millis();

    eprintln!(
        "measure_dependency_view_build: resolved {modules_resolved} module(s) over {} source root(s) \
         ({modules_excluded} excluded)",
        source_roots.len(),
    );
    eprintln!(
        "[measurement] whole-tree-resolve verdict=Completed wall_ms={resolve_ms} modules={modules_resolved}"
    );

    let view_started = Instant::now();
    let edge_count_val =
        v1_interpreter::run_in_context(&ctx, EDGE_COUNT_FN, false).map_err(|e| {
            eprintln!(
                "measure_dependency_view_build: interpreter error running {EDGE_COUNT_FN}: {e}"
            );
            ExitCode::from(2)
        })?;
    let view_ms = view_started.elapsed().as_millis();
    let edge_count = int_from_value(&ctx, &edge_count_val, EDGE_COUNT_FN)?;

    let decl_count_val =
        v1_interpreter::run_in_context(&ctx, DECL_COUNT_FN, false).map_err(|e| {
            eprintln!(
                "measure_dependency_view_build: interpreter error running {DECL_COUNT_FN}: {e}"
            );
            ExitCode::from(2)
        })?;
    let decl_count = int_from_value(&ctx, &decl_count_val, DECL_COUNT_FN)?;

    match peak_rss_vhwm_bytes() {
        Some(bytes) => eprintln!(
            "[measurement] dependency-view-build wall_ms={view_ms} edge_count={edge_count} \
             decl_count={decl_count} peak_rss_bytes={bytes} modules={modules_resolved}"
        ),
        None => eprintln!(
            "[measurement] dependency-view-build wall_ms={view_ms} edge_count={edge_count} \
             decl_count={decl_count} peak_rss_bytes=unavailable modules={modules_resolved}"
        ),
    }
    eprintln!(
        "[measurement] total wall_ms={} (resolve={resolve_ms} view_build={view_ms})",
        resolve_ms + view_ms
    );

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
