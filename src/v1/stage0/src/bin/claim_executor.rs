#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::thread;
use std::time::Instant;

#[cfg(test)]
use v1_compiler::cli_run::workspace_root;
use v1_compiler::cli_run::{
    compute_histogram_data, compute_witness_timing_rows, make_eval_context, resolve_entry_graph,
    run_claim, run_discovery_corpus_with_options, run_value, top_n_slowest_witnesses, ClaimOutcome,
    DiscoveryCorpusOptions, DiscoverySummary, HistogramData, TimingPercentiles, WitnessTimingRow,
    DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N,
};
use v1_compiler::v1_interpreter::{
    color_enabled, eval_data_item_value, paint, run_in_context_with_args, sgr, ExecutionMode,
    InterpContext, Value,
};

#[derive(Clone)]
enum Runnable {
    SingleClaim {
        entry: String,
        function: String,
        use_walk_memo: bool,
    },
    DiscoveryBatch {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        skip_unaffected_node_frontier: bool,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        spawn_width_cap: usize,
    },
    CompileCleanShardBatch {
        entry_paths: Vec<String>,
        spawn_width_cap: usize,
    },
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("claim_executor: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn free_monoid_elems<'a>(value: &'a Value, ctx: &InterpContext) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected a List (Cons/Empty), got {}",
                    other.type_label_public()
                ))
            }
        }
    }
}

fn str_list_from_value(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for elem in free_monoid_elems(value, ctx)? {
        match elem {
            Value::Str(s) => out.push(s.clone()),
            other => {
                return Err(format!(
                    "expected a List<String> element, got {}",
                    other.type_label_public()
                ))
            }
        }
    }
    Ok(out)
}

fn str_field(
    fields: &[(v1_compiler::v1_interpreter::Symbol, Value)],
    name: &str,
    owner: &str,
    ctx: &InterpContext,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!(
            "{}.{} is {}, not String",
            owner,
            name,
            ctx.format_value(other)
        )),
        None => Err(format!("{} missing field `{}`", owner, name)),
    }
}

fn runnable_from_value(value: &Value, ctx: &InterpContext) -> Result<Runnable, String> {
    match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => {
            Ok(Runnable::SingleClaim {
                entry: str_field(fields, "entry", "ClaimRef", ctx)?,
                function: str_field(fields, "function", "ClaimRef", ctx)?,
                use_walk_memo: false,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableSingleClaim") => {
            let entry = str_field(fields, "entry", "RunnableSingleClaim", ctx)?;
            let function = str_field(fields, "function", "RunnableSingleClaim", ctx)?;
            let use_walk_memo = match ctx.field(fields, "profile") {
                Some(Value::Record { fields: pf, .. })
                | Some(Value::Variant { fields: pf, .. }) => {
                    matches!(
                        ctx.field(pf, "heavy_whole_tree_resolve"),
                        Some(Value::Bool(true))
                    )
                }
                _ => false,
            };
            Ok(Runnable::SingleClaim {
                entry,
                function,
                use_walk_memo,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableDiscoveryBatch") => {
            let source_roots = match ctx.field(fields, "source_roots") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => {
                    return Err("RunnableDiscoveryBatch missing field `source_roots`".to_string())
                }
            };
            let scan_dirs = match ctx.field(fields, "scan_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => return Err("RunnableDiscoveryBatch missing field `scan_dirs`".to_string()),
            };
            let explicit_entries = match ctx.field(fields, "explicit_entries") {
                Some(v) => {
                    let mut out = Vec::new();
                    for elem in free_monoid_elems(v, ctx)? {
                        let efields = match elem {
                            Value::Record { fields, .. } => fields,
                            Value::Variant { fields, .. } => fields,
                            other => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch.explicit_entries element is {}, not a record",
                                    other.type_label_public()
                                ))
                            }
                        };
                        out.push((
                            str_field(efields, "entry", "explicit_entries", ctx)?,
                            str_field(efields, "function", "explicit_entries", ctx)?,
                        ));
                    }
                    out
                }
                None => Vec::new(),
            };
            let skip_unaffected_node_frontier =
                match ctx.field(fields, "skip_unaffected_node_frontier") {
                    Some(v) => match v {
                        Value::Bool(b) => *b,
                        other => {
                            return Err(format!(
                        "RunnableDiscoveryBatch.skip_unaffected_node_frontier must be Bool, got {}",
                        other.type_label_public()
                    ))
                        }
                    },
                    None => false,
                };
            let exclude_substrings = match ctx.field(fields, "exclude_substrings") {
                Some(v) => str_list_from_value(v, ctx)?,
                // Field absent means the plan author specified no exclusions — default is empty,
                // not the Rust constant (the model is the sole authority on the plan path).
                None => Vec::new(),
            };
            let discovery_scope_dirs = match ctx.field(fields, "discovery_scope_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => Vec::new(),
            };
            let spawn_width_cap = match ctx.field(fields, "spawn_width_cap") {
                Some(Value::Int(n)) => (*n).max(0) as usize,
                Some(_) | None => 0,
            };
            Ok(Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                skip_unaffected_node_frontier,
                exclude_substrings,
                discovery_scope_dirs,
                spawn_width_cap,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableCompileCleanShardBatch") => {
            let entry_paths = match ctx.field(fields, "entry_paths") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => {
                    return Err(
                        "RunnableCompileCleanShardBatch missing field `entry_paths`".to_string()
                    )
                }
            };
            let spawn_width_cap = match ctx.field(fields, "spawn_width_cap") {
                Some(Value::Int(n)) => (*n).max(0) as usize,
                Some(_) | None => 0,
            };
            Ok(Runnable::CompileCleanShardBatch {
                entry_paths,
                spawn_width_cap,
            })
        }
        other => Err(format!(
            "expected a ClaimRef record or Runnable variant, got {}",
            ctx.format_value(other)
        )),
    }
}

fn batches_from_plan(plan: &Value, ctx: &InterpContext) -> Result<Vec<Vec<Runnable>>, String> {
    let mut batches = Vec::new();
    for batch_val in free_monoid_elems(plan, ctx)? {
        let mut batch = Vec::new();
        for elem in free_monoid_elems(batch_val, ctx)? {
            batch.push(runnable_from_value(elem, ctx)?);
        }
        batches.push(batch);
    }
    Ok(batches)
}

struct ClaimResult {
    function: String,
    ok: bool,
    detail: String,
    /// Wall-clock eval time for this single claim (0 for discovery aggregate).
    wall_nanos: u128,
    /// Resolve time charged to this result; non-zero only on the first claim in a
    /// SharedClaims group (the group resolves once, cost attributed to first claim).
    resolve_nanos: u128,
    /// For discovery batch nodes: sum of per-file resolve times (serial sum, not wall).
    corpus_resolve_nanos: u128,
    /// For discovery batch nodes: sum of per-witness eval times (serial sum, not wall).
    corpus_eval_nanos: u128,
    /// Number of discovery witnesses (non-zero only for discovery batch nodes).
    corpus_witnesses: usize,
}

/// A batch is partitioned into resolve-groups before scheduling. SingleClaims that share one
/// `entry` collapse into a single `SharedClaims` group: the entry's import closure is resolved
/// (and typechecked) EXACTLY ONCE and every claim runs on that one shared interpreter context,
/// instead of each claim re-resolving the identical graph on its own thread. This is the floor's
/// dominant footprint win — batch-2's gate witnesses all live in one file
/// (`dsl/tools/floor_effect_gate_witness.dag`, ~0.9 GiB / 106 modules per resolve), so the
/// per-thread-resolve scheme held that graph ~6x concurrently (~4.5 GiB of pure duplication,
/// roughly half the self-RSS). Resolve is a pure function of `(source_roots, entry)`, so sharing
/// the graph across same-entry claims is semantically identical — correctness by construction
/// (DESIGN §2: duplicated work removed; §4: realization may share what the pure spec models apart).
enum BatchUnit {
    SharedClaims {
        entry: String,
        functions: Vec<String>,
        use_walk_memo: bool,
    },
    UnrunnableSentinel {
        function: String,
    },
    Discovery {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        skip_unaffected_node_frontier: bool,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        spawn_width_cap: usize,
    },
    CompileCleanShardBatch {
        entry_paths: Vec<String>,
        spawn_width_cap: usize,
    },
}

/// Partition a batch's runnables into resolve-groups, preserving first-appearance order so the
/// PASS/FAIL log stays stable. SingleClaims with a non-empty `entry` coalesce by `entry`;
/// empty-entry sentinels and DiscoveryBatch nodes stay their own units (each resolves apart).
fn group_batch_units(batch: &[Runnable]) -> Vec<BatchUnit> {
    let mut units: Vec<BatchUnit> = Vec::new();
    let mut entry_to_unit: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for runnable in batch {
        match runnable {
            Runnable::SingleClaim {
                entry, function, ..
            } if entry.is_empty() => {
                units.push(BatchUnit::UnrunnableSentinel {
                    function: function.clone(),
                });
            }
            Runnable::SingleClaim {
                entry,
                function,
                use_walk_memo,
            } => {
                if let Some(&idx) = entry_to_unit.get(entry) {
                    if let BatchUnit::SharedClaims {
                        functions,
                        use_walk_memo: existing_memo,
                        ..
                    } = &mut units[idx]
                    {
                        functions.push(function.clone());
                        // A memo claim (heavy whole-tree resolve) may merge into a
                        // group created from a non-memo claim first; OR here so the
                        // group gets the memo path if any member is heavy.
                        *existing_memo |= use_walk_memo;
                    }
                } else {
                    entry_to_unit.insert(entry.clone(), units.len());
                    units.push(BatchUnit::SharedClaims {
                        entry: entry.clone(),
                        functions: vec![function.clone()],
                        use_walk_memo: *use_walk_memo,
                    });
                }
            }
            Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                skip_unaffected_node_frontier,
                exclude_substrings,
                discovery_scope_dirs,
                spawn_width_cap,
            } => units.push(BatchUnit::Discovery {
                source_roots: source_roots.clone(),
                scan_dirs: scan_dirs.clone(),
                explicit_entries: explicit_entries.clone(),
                skip_unaffected_node_frontier: *skip_unaffected_node_frontier,
                exclude_substrings: exclude_substrings.clone(),
                discovery_scope_dirs: discovery_scope_dirs.clone(),
                spawn_width_cap: *spawn_width_cap,
            }),
            Runnable::CompileCleanShardBatch {
                entry_paths,
                spawn_width_cap,
            } => units.push(BatchUnit::CompileCleanShardBatch {
                entry_paths: entry_paths.clone(),
                spawn_width_cap: *spawn_width_cap,
            }),
        }
    }
    units
}

fn claim_result_for_outcome(
    function: String,
    outcome: ClaimOutcome,
    wall_nanos: u128,
    resolve_nanos: u128,
) -> ClaimResult {
    match outcome {
        ClaimOutcome::Pass => ClaimResult {
            function,
            ok: true,
            detail: String::new(),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::Fail => ClaimResult {
            function,
            ok: false,
            detail: "returned Bool(false)".to_string(),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::NotBool { got } => ClaimResult {
            function,
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::RuntimeError { message } => ClaimResult {
            function,
            ok: false,
            detail: format!("runtime error: {}", message),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
    }
}

fn run_batch_unit(
    source_roots: Vec<String>,
    unit: BatchUnit,
    spawn_width: usize,
) -> Vec<ClaimResult> {
    match unit {
        BatchUnit::UnrunnableSentinel { function } => vec![ClaimResult {
            function,
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        }],
        BatchUnit::Discovery {
            source_roots: roots,
            scan_dirs,
            explicit_entries,
            skip_unaffected_node_frontier,
            exclude_substrings,
            discovery_scope_dirs,
            spawn_width_cap,
        } => vec![run_discovery_batch_node(
            roots,
            scan_dirs,
            explicit_entries,
            skip_unaffected_node_frontier,
            exclude_substrings,
            discovery_scope_dirs,
            spawn_width,
            spawn_width_cap,
        )],
        BatchUnit::CompileCleanShardBatch {
            entry_paths,
            spawn_width_cap,
        } => vec![run_compile_clean_shard_batch_node(
            &source_roots,
            entry_paths,
            spawn_width,
            spawn_width_cap,
        )],
        BatchUnit::SharedClaims {
            entry, functions, ..
        } => run_shared_entry_claims(&source_roots, &entry, &functions),
    }
}

fn run_shared_entry_claims(
    source_roots: &[String],
    entry: &str,
    functions: &[String],
) -> Vec<ClaimResult> {
    let resolve_start = Instant::now();
    let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return functions
                .iter()
                .map(|function| ClaimResult {
                    function: function.clone(),
                    ok: false,
                    detail: format!("resolve failed for {}: {}", entry, msg),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                })
                .collect();
        }
    };
    let resolve_nanos = resolve_start.elapsed().as_nanos();
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
    let mut first = true;
    functions
        .iter()
        .map(|function| {
            let claim_start = Instant::now();
            let outcome = run_claim(&ctx, function);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let rn = if first {
                first = false;
                resolve_nanos
            } else {
                0
            };
            claim_result_for_outcome(function.clone(), outcome, wall_nanos, rn)
        })
        .collect()
}

/// Run claims whose entry-graph is already resolved and cached in `memo`, resolving on first miss.
/// Runs on the main thread so `InterpContext` (which contains `Rc` fields) never crosses thread
/// boundaries. Subsequent callers for the same entry share the cached context — resolve runs once.
fn run_memo_shared_claims(
    source_roots: &[String],
    entry: &str,
    functions: &[String],
    memo: &mut std::collections::HashMap<String, InterpContext>,
) -> Vec<ClaimResult> {
    let resolve_start = Instant::now();
    let mut fresh_resolve = false;
    if !memo.contains_key(entry) {
        let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
            Ok(pair) => pair,
            Err(msg) => {
                return functions
                    .iter()
                    .map(|function| ClaimResult {
                        function: function.clone(),
                        ok: false,
                        detail: format!("resolve failed for {}: {}", entry, msg),
                        wall_nanos: 0,
                        resolve_nanos: 0,
                        corpus_resolve_nanos: 0,
                        corpus_eval_nanos: 0,
                        corpus_witnesses: 0,
                    })
                    .collect();
            }
        };
        memo.insert(
            entry.to_string(),
            make_eval_context(&graph, source_indices, ExecutionMode::Wet),
        );
        fresh_resolve = true;
    }
    let resolve_nanos = if fresh_resolve {
        resolve_start.elapsed().as_nanos()
    } else {
        0
    };
    let ctx = memo.get(entry).expect("memo populated above");
    let mut first = fresh_resolve;
    functions
        .iter()
        .map(|function| {
            let claim_start = Instant::now();
            let outcome = run_claim(ctx, function);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let rn = if first {
                first = false;
                resolve_nanos
            } else {
                0
            };
            claim_result_for_outcome(function.clone(), outcome, wall_nanos, rn)
        })
        .collect()
}

/// The medium's `Viewport.width` for boxed output. The seed sources the runtime value from the host
/// (`COLUMNS` when present) and passes it into the `.dag` render model; the model owns how width
/// shapes the box. Conservative 88-col default fits common CI-log and chat viewers; clamped so a
/// hostile `COLUMNS` cannot produce a degenerate box.
fn histogram_output_width() -> i64 {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|w| *w >= 48)
        .unwrap_or(88)
        .min(120)
}

fn clamp_nanos_to_i64(n: u128) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// Render the timing histogram boxes through `dsl/gunbc/ci_render.dag` (`render_percentile_box`),
/// the single authority for boxed-Frame width. The seed only supplies measured data + the host
/// viewport width; all layout (borders, padding, duration formatting) lives in `.dag`.
fn render_timing_histogram(
    source_roots: &[String],
    data: &HistogramData,
) -> Result<String, String> {
    let entry = "dsl/gunbc/ci_render.dag";
    let (graph, indices) = resolve_entry_graph(source_roots, entry)
        .map_err(|m| format!("resolve failed for {entry}:\n{m}"))?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
    let width = histogram_output_width();
    let color = color_enabled();

    let render_box = |title: &str, p: &TimingPercentiles| -> Result<String, String> {
        let value = run_in_context_with_args(
            &ctx,
            "render_percentile_box",
            &[
                (Some("title".to_string()), Value::Str(title.to_string())),
                (
                    Some("p50".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p50)),
                ),
                (
                    Some("p90".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p90)),
                ),
                (
                    Some("p95".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p95)),
                ),
                (
                    Some("p99".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p99)),
                ),
                (
                    Some("p100".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p100)),
                ),
                (Some("width".to_string()), Value::Int(width)),
                (Some("color".to_string()), Value::Bool(color)),
            ],
            false,
        )
        .map_err(|e| format!("render_percentile_box eval failed: {e}"))?;
        match value {
            Value::Str(s) => Ok(s),
            other => Err(format!(
                "render_percentile_box returned non-string: {other}"
            )),
        }
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Total witnesses: {} (included in histogram); {} skipped (no entry-resolve timing)\n",
        data.included, data.skipped
    ));
    out.push_str(
        "Note: Resolve times are per-entry-amortized (witnesses in an entry share its resolve cost); eval times are per-witness.\n\n",
    );
    out.push_str(&render_box("TOTAL TIME (Resolve + Eval)", &data.total)?);
    out.push('\n');
    out.push_str(&render_box("RESOLVE TIME", &data.resolve)?);
    out.push('\n');
    out.push_str(&render_box("EVAL TIME", &data.eval)?);
    Ok(out)
}

fn slowest_witness_attribution_n() -> usize {
    std::env::var("GUNBC_FLOOR_SLOWEST_N")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N)
}

fn str_list_value(lines: &[String]) -> Value {
    Value::List(Rc::new(
        lines
            .iter()
            .cloned()
            .map(Value::Str)
            .collect::<Vec<_>>()
            .into(),
    ))
}

/// Render the top-N slowest witnesses through `dsl/gunbc/ci_render.dag`.
fn render_slowest_witnesses(
    source_roots: &[String],
    rows: &[WitnessTimingRow],
) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let entry = "dsl/gunbc/ci_render.dag";
    let (graph, indices) = resolve_entry_graph(source_roots, entry)
        .map_err(|m| format!("resolve failed for {entry}:\n{m}"))?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
    let width = histogram_output_width();
    let color = color_enabled();

    let mut body_lines: Vec<String> = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let line = run_in_context_with_args(
            &ctx,
            "slowest_witness_row",
            &[
                (Some("rank".to_string()), Value::Int((i + 1) as i64)),
                (
                    Some("function".to_string()),
                    Value::Str(row.function.clone()),
                ),
                (Some("entry".to_string()), Value::Str(row.entry.clone())),
                (
                    Some("eval_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.eval_nanos)),
                ),
                (
                    Some("resolve_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.resolve_nanos)),
                ),
                (
                    Some("total_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.total_nanos)),
                ),
            ],
            false,
        )
        .map_err(|e| format!("slowest_witness_row eval failed: {e}"))?;
        match line {
            Value::Str(s) => body_lines.push(s),
            other => return Err(format!("slowest_witness_row returned non-string: {other}")),
        }
    }

    let value = run_in_context_with_args(
        &ctx,
        "render_slowest_witnesses_box",
        &[
            (Some("body".to_string()), str_list_value(&body_lines)),
            (Some("width".to_string()), Value::Int(width)),
            (Some("color".to_string()), Value::Bool(color)),
        ],
        false,
    )
    .map_err(|e| format!("render_slowest_witnesses_box eval failed: {e}"))?;
    match value {
        Value::Str(s) => Ok(s),
        other => Err(format!(
            "render_slowest_witnesses_box returned non-string: {other}"
        )),
    }
}

fn emit_slowest_witness_attribution(source_roots: &[String], summary: &DiscoverySummary) {
    match compute_witness_timing_rows(summary) {
        Ok(rows) => {
            let n = slowest_witness_attribution_n().min(rows.len());
            if n == 0 {
                return;
            }
            let top = top_n_slowest_witnesses(&rows, n);
            match render_slowest_witnesses(source_roots, &top) {
                Ok(boxed) => {
                    eprintln!("{boxed}");
                    let tail_eval_ms: u128 =
                        top.iter().map(|r| r.eval_nanos).sum::<u128>() / 1_000_000;
                    let total_eval_ms = summary.total_measured_nanos / 1_000_000;
                    let pct = if total_eval_ms == 0 {
                        0.0
                    } else {
                        100.0 * tail_eval_ms as f64 / total_eval_ms as f64
                    };
                    eprintln!(
                        "[attribution] top-{n} slowest witnesses: eval serial-sum {tail_eval_ms}ms ({pct:.1}% of corpus eval)"
                    );
                }
                Err(e) => eprintln!("[attribution] render failed (timings unaffected): {e}"),
            }
        }
        Err(msg) => eprintln!("{msg}"),
    }
}

const COMPILE_CLEAN_SHARD_TRANSPORT: &str = "dsl/tools/dsl_compile_clean_shard_transport.dag";
const PERTURB_SHARD_BROKEN_SOURCE_DATA: &str = "broken_shard_module_source";

fn perturb_shard_broken_module_source(source_roots: &[String]) -> Result<String, String> {
    let (graph, source_indices) = resolve_entry_graph(source_roots, COMPILE_CLEAN_SHARD_TRANSPORT)
        .map_err(|msg| format!("resolve {COMPILE_CLEAN_SHARD_TRANSPORT}: {msg}"))?;
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Hermetic);
    let value = eval_data_item_value(&ctx, PERTURB_SHARD_BROKEN_SOURCE_DATA).map_err(|e| {
        format!("eval {PERTURB_SHARD_BROKEN_SOURCE_DATA} from {COMPILE_CLEAN_SHARD_TRANSPORT}: {e}")
    })?;
    match value {
        Some(Value::Str(s)) => Ok(s),
        Some(other) => Err(format!(
            "{PERTURB_SHARD_BROKEN_SOURCE_DATA} is {}, not String",
            ctx.format_value(&other)
        )),
        None => Err(format!(
            "{PERTURB_SHARD_BROKEN_SOURCE_DATA} not found in {COMPILE_CLEAN_SHARD_TRANSPORT}"
        )),
    }
}

fn run_single_shard_compile(ctx: &InterpContext, entry_path: &str) -> Result<bool, String> {
    let value = run_in_context_with_args(
        ctx,
        "run_shard_compile",
        &[(
            Some("entry_rel_path".to_string()),
            Value::Str(entry_path.to_string()),
        )],
        false,
    )
    .map_err(|e| format!("run_shard_compile({entry_path}): {e}"))?;
    match value {
        Value::Bool(b) => Ok(b),
        other => Err(format!(
            "run_shard_compile({entry_path}) returned {}, not Bool",
            ctx.format_value(&other)
        )),
    }
}

fn run_compile_clean_shard_batch_node(
    source_roots: &[String],
    entry_paths: Vec<String>,
    spawn_width: usize,
    spawn_width_cap: usize,
) -> ClaimResult {
    let width = spawn_width.max(1).min(spawn_width_cap.max(1));
    let shard_count = entry_paths.len();
    let label = format!("compile-clean-shard-batch[{shard_count} modules, width={width}]");
    let batch_start = Instant::now();

    if entry_paths.is_empty() {
        return ClaimResult {
            function: label,
            ok: false,
            detail: "compile-clean-shard:empty entry_paths — fail-closed".to_string(),
            wall_nanos: batch_start.elapsed().as_nanos(),
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        };
    }

    let mut shards: Vec<Vec<String>> = vec![Vec::new(); width];
    for (i, path) in entry_paths.iter().enumerate() {
        shards[i % width].push(path.clone());
    }

    let mut handles = Vec::new();
    for shard in shards {
        if shard.is_empty() {
            continue;
        }
        let roots = source_roots.to_vec();
        handles.push(thread::spawn(move || {
            let (graph, source_indices) =
                match resolve_entry_graph(&roots, COMPILE_CLEAN_SHARD_TRANSPORT) {
                    Ok(pair) => pair,
                    Err(msg) => {
                        return vec![format!(
                            "resolve failed for {COMPILE_CLEAN_SHARD_TRANSPORT}: {msg}"
                        )];
                    }
                };
            let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
            let mut failures = Vec::new();
            for path in shard {
                match run_single_shard_compile(&ctx, &path) {
                    Ok(true) => {}
                    Ok(false) => failures.push(format!("{path}: compile returned false")),
                    Err(msg) => failures.push(msg),
                }
            }
            failures
        }));
    }

    let mut all_failures = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(failures) => all_failures.extend(failures),
            Err(_) => all_failures.push("compile-clean shard thread panicked".to_string()),
        }
    }

    let wall_nanos = batch_start.elapsed().as_nanos();
    let wall_ms = wall_nanos / 1_000_000;
    eprintln!(
        "[measurement] compile-clean shard batch: {shard_count} module shard(s), width={width}, wall {wall_ms}ms"
    );

    if all_failures.is_empty() {
        ClaimResult {
            function: label,
            ok: true,
            detail: String::new(),
            wall_nanos,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: shard_count,
        }
    } else {
        ClaimResult {
            function: label,
            ok: false,
            detail: format!(
                "{} of {} compile-clean shard(s) failed: {}",
                all_failures.len(),
                shard_count,
                all_failures.join("; ")
            ),
            wall_nanos,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: shard_count,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_discovery_batch_node(
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    explicit_entries: Vec<(String, String)>,
    skip_unaffected_node_frontier: bool,
    exclude_substrings: Vec<String>,
    discovery_scope_dirs: Vec<String>,
    spawn_width: usize,
    spawn_width_cap: usize,
) -> ClaimResult {
    let label = format!(
        "discovery-corpus[{} root(s)+{} explicit, width={}]",
        source_roots.len(),
        explicit_entries.len(),
        spawn_width.max(1),
    );
    match run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit_entries,
        ExecutionMode::Wet,
        spawn_width,
        DiscoveryCorpusOptions {
            skip_unaffected_node_frontier,
            explicit_roster_only: false,
            exclude_substrings,
            discovery_scope_dirs,
            spawn_width_cap,
        },
    ) {
        Ok(summary) if summary.failures.is_empty() => {
            eprintln!(
                "[measurement] discovery corpus: {} witness(es) ({} skipped), resolve {:.3}ms, evalu {:.3}ms, CostAccount.time basis=Measured {}ns",
                summary.total,
                summary.skipped,
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
                summary.total_measured_nanos,
            );
            match compute_histogram_data(&summary) {
                Ok(data) => match render_timing_histogram(&source_roots, &data) {
                    Ok(histogram) => eprintln!("{histogram}"),
                    Err(e) => eprintln!("[histogram] render failed (timings unaffected): {e}"),
                },
                Err(msg) => eprintln!("{msg}"),
            }
            emit_slowest_witness_attribution(&source_roots, &summary);
            ClaimResult {
                function: format!("{label} ({} witnesses)", summary.total),
                ok: true,
                detail: String::new(),
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: summary.total_resolve_nanos,
                corpus_eval_nanos: summary.total_measured_nanos,
                corpus_witnesses: summary.total,
            }
        }
        Ok(summary) => ClaimResult {
            function: label,
            ok: false,
            detail: format!(
                "{} of {} discovery witness(es) failed: {}",
                summary.failures.len(),
                summary.total,
                summary.failures.join("; ")
            ),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: summary.total_resolve_nanos,
            corpus_eval_nanos: summary.total_measured_nanos,
            corpus_witnesses: summary.total,
        },
        Err(msg) => ClaimResult {
            function: label,
            ok: false,
            detail: format!("discovery corpus failed: {msg}"),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
    }
}

fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<Runnable>>, String> {
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for plan {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Hermetic);
    let plan_value = run_value(&plan_ctx, plan_function).map_err(|msg| {
        format!(
            "plan eval failed ({}::{}): {}",
            plan_entry, plan_function, msg
        )
    })?;
    let batches = batches_from_plan(&plan_value, &plan_ctx)
        .map_err(|msg| format!("malformed plan value: {}", msg))?;
    drop(plan_value);
    drop(plan_graph);
    Ok(batches)
}

fn spawn_width_function_name(plan_function: &str) -> Option<String> {
    match plan_function {
        "gunbc_ci_floor_batches" => Some("gunbc_ci_plan_spawn_width".to_string()),
        other => other
            .strip_suffix("_batches")
            .map(|prefix| format!("{prefix}_plan_spawn_width")),
    }
}

fn hardware_thread_count_from_value(value: &Value, ctx: &InterpContext) -> Result<usize, String> {
    match value {
        Value::Record { fields, .. } => match ctx.field(fields, "count") {
            Some(Value::Int(n)) => Ok((*n).max(1) as usize),
            Some(other) => Err(format!(
                "HardwareThreadCount.count is {}, not Int",
                ctx.format_value(other)
            )),
            None => Err("HardwareThreadCount missing `count` field".to_string()),
        },
        other => Err(format!(
            "expected HardwareThreadCount record, got {}",
            other.type_label_public()
        )),
    }
}

fn read_host_memory_budget_bytes() -> Option<u64> {
    if let Ok(self_cg) = fs::read_to_string("/proc/self/cgroup") {
        if let Some(rel) = self_cg
            .lines()
            .find_map(|l| l.strip_prefix("0::"))
            .map(|p| p.trim().trim_start_matches('/').to_string())
        {
            let root = Path::new("/sys/fs/cgroup");
            let mut dir = root.join(&rel);
            let mut effective: Option<u64> = None;
            loop {
                if let Ok(s) = fs::read_to_string(dir.join("memory.max")) {
                    let s = s.trim();
                    if s != "max" {
                        if let Ok(v) = s.parse::<u64>() {
                            effective = Some(effective.map_or(v, |cur| cur.min(v)));
                        }
                    }
                }
                if dir == root || !dir.pop() {
                    break;
                }
            }
            if let Some(v) = effective {
                return Some(v);
            }
        }
    }
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for key in ["MemAvailable", "MemTotal"] {
            if let Some(kb) = meminfo
                .lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
            {
                return Some(kb.saturating_mul(1024));
            }
        }
    }
    None
}

fn eval_spawn_width(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<usize, String> {
    let Some(width_fn) = spawn_width_function_name(plan_function) else {
        return Ok(1);
    };
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for spawn width {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Wet);
    let budget_bytes = read_host_memory_budget_bytes().unwrap_or(0);
    match budget_bytes {
        0 => eprintln!(
            "claim_executor: live memory budget unavailable — width uses the .dag conservative fallback"
        ),
        b => eprintln!("claim_executor: live memory budget {b} bytes (cgroup memory.max / meminfo)"),
    }
    let budget_arg = i64::try_from(budget_bytes).unwrap_or(i64::MAX);
    let width_value = run_in_context_with_args(
        &plan_ctx,
        &width_fn,
        &[(
            Some("memory_budget_bytes".to_string()),
            Value::Int(budget_arg),
        )],
        false,
    )
    .map_err(|e| {
        format!(
            "spawn width eval failed ({}::{}): {}",
            plan_entry, width_fn, e
        )
    })?;
    hardware_thread_count_from_value(&width_value, &plan_ctx)
}

struct WalkOutcome {
    any_failed: bool,
    batches_run: usize,
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// The cgroup directory whose `memory.max` is the TIGHTEST along the `/proc/self/cgroup`
/// leaf→root walk — the EFFECTIVE budget the OOM-killer enforces (the same ancestor
/// `read_host_memory_budget_bytes` reduces to by `min`, returned here so the peak can be
/// read at the SAME level the budget binds). `None` when `/proc/self/cgroup` is unreadable
/// or no ancestor sets a numeric cap.
fn binding_cap_cgroup_dir() -> Option<std::path::PathBuf> {
    let self_cg = fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = self_cg
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    let root = Path::new("/sys/fs/cgroup");
    let mut dir = root.join(&rel);
    let mut best: Option<(u64, std::path::PathBuf)> = None;
    loop {
        if let Ok(s) = fs::read_to_string(dir.join("memory.max")) {
            let s = s.trim();
            if s != "max" {
                if let Ok(v) = s.parse::<u64>() {
                    let take = best.as_ref().map(|(cur, _)| v < *cur).unwrap_or(true);
                    if take {
                        best = Some((v, dir.clone()));
                    }
                }
            }
        }
        if dir == root || !dir.pop() {
            break;
        }
    }
    best.map(|(_, d)| d)
}

/// The process's own deepest (leaf) cgroup from `/proc/self/cgroup`. On the ephemeral GitHub
/// runners this is `actions-runner@srv1-NN.service` — fresh per job — so its hierarchical cgroup-v2
/// `memory.peak` isolates ONE job's whole-tree footprint. We read the peak HERE, not at a shared
/// parent slice, so the placement divisor is per-job and not an aggregate over co-resident runners
/// (measured 2026-06-22: the runner units run `MemoryMax=infinity`, so there is NO binding numeric
/// cap and `binding_cap_cgroup_dir` returns `None` on the real fleet; the leaf is always present).
fn leaf_cgroup_dir() -> Option<std::path::PathBuf> {
    let self_cg = fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = self_cg
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    Some(Path::new("/sys/fs/cgroup").join(rel))
}

/// Total physical RAM in bytes (`/proc/meminfo` `MemTotal`, kB→bytes) — the EFFECTIVE memory budget
/// when the cgroup is uncapped, which is the real CI fleet (runner units `MemoryMax=infinity`, so
/// the OOM bound is physical RAM, not a cgroup `memory.max`).
fn mem_total_bytes() -> Option<u64> {
    let s = fs::read_to_string("/proc/meminfo").ok()?;
    let line = s.lines().find(|l| l.starts_with("MemTotal"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// One read of the whole-tree job footprint and its budget context, taken in a single cgroup walk so
/// a consumer (placement divisor, compile-jobs derive) can parse usage AND budget from one emitted
/// line without re-walking. `cap_bytes` is the tightest numeric `memory.max` on the leaf→root walk
/// (`None` = uncapped / RAM-bound); `host_ram` is the physical-RAM budget that binds when uncapped;
/// `sccache_under_leaf` classifies the sccache server's cgroup as a descendant (already counted in
/// `leaf_peak`) vs a sibling (must be subtracted as `host_fixed_overhead`) — the "accounted exactly
/// once" decision, read from paths rather than guessed.
struct CgroupMeasurement {
    leaf_peak: u64,
    leaf_rel: String,
    cap_bytes: Option<u64>,
    host_ram: Option<u64>,
    pids_current: u64,
    pids_max: String,
    sccache_rel: Option<String>,
    sccache_under_leaf: bool,
}

fn cgroup_job_measurement() -> Option<CgroupMeasurement> {
    let leaf = leaf_cgroup_dir()?;
    let leaf_peak = fs::read_to_string(leaf.join("memory.peak"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let leaf_rel = leaf
        .strip_prefix("/sys/fs/cgroup")
        .map(|p| format!("/{}", p.to_string_lossy().trim_start_matches('/')))
        .unwrap_or_else(|_| leaf.to_string_lossy().into_owned());
    let cap_bytes = binding_cap_cgroup_dir()
        .and_then(|d| fs::read_to_string(d.join("memory.max")).ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let pids_current = fs::read_to_string(leaf.join("pids.current"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let pids_max = fs::read_to_string(leaf.join("pids.max"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let sccache_rel = sccache_server_cgroup_rel();
    // Descendant iff sccache's cgroup is the leaf or strictly under it — a PATH-COMPONENT prefix,
    // not a bare string prefix, so a sibling like `<leaf>-other.service` is NOT misclassified as a
    // descendant (that would under-count host overhead — the fail-OPEN direction). When the leaf is
    // the cgroup root (`leaf_r` empty — e.g. a single-cgroup container) everything is a descendant.
    // On the real fleet the leaf is the runner-service cgroup and sccache is a sibling service
    // cgroup, so this is `false` (subtract as host_fixed_overhead).
    let leaf_r = leaf_rel.trim_start_matches('/').to_string();
    let sccache_under_leaf = sccache_rel
        .as_deref()
        .map(|r| {
            let r = r.trim_start_matches('/');
            leaf_r.is_empty() || r == leaf_r || r.starts_with(&format!("{leaf_r}/"))
        })
        .unwrap_or(false);
    Some(CgroupMeasurement {
        leaf_peak,
        leaf_rel,
        cap_bytes,
        host_ram: mem_total_bytes(),
        pids_current,
        pids_max,
        sccache_rel,
        sccache_under_leaf,
    })
}

/// Verify each declared release artifact exists, is executable, and is non-empty — failing CLOSED
/// (exit 1) with the GitHub-Actions `::error::` annotation on the first violation. This is the
/// in-binary home of what was previously inline `[ -x ]` / `[ -s ]` shell in the generated ci.yml
/// (DESIGN §5 fail-open guard: an sccache-served truncated/empty cached artifact after a
/// `successful` build). The artifact paths are authored by the .dag spec and passed positionally.
fn verify_build_artifacts(paths: &[String]) -> Result<ExitCode, ExitCode> {
    use std::os::unix::fs::PermissionsExt;
    if paths.is_empty() {
        eprintln!("claim_executor: --verify-build-artifacts requires at least one artifact path");
        return Err(ExitCode::from(2));
    }
    for path in paths {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path.as_str());
        match std::fs::metadata(path) {
            Ok(meta) => {
                let executable = meta.permissions().mode() & 0o111 != 0;
                if !meta.is_file() || !executable {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' absent or not \
                         executable after a 'successful' build (sccache/cache corruption — DESIGN \
                         §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
                if meta.len() == 0 {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' is zero-byte after \
                         a 'successful' build (sccache served a truncated/empty cached artifact — \
                         DESIGN §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
            }
            Err(_) => {
                eprintln!(
                    "::error::build verification: declared artifact '{name}' absent or not \
                     executable after a 'successful' build (sccache/cache corruption — DESIGN §5 \
                     fail-open); failing closed: {path}"
                );
                return Err(ExitCode::from(1));
            }
        }
    }
    eprintln!(
        "claim_executor: build-artifact verification passed ({} declared release binar{} present \
         + non-empty)",
        paths.len(),
        if paths.len() == 1 { "y" } else { "ies" }
    );
    Ok(ExitCode::SUCCESS)
}

/// Single authority for the one-line whole-tree cgroup measurement, shared by the floor run and the
/// standalone `--measure-cgroup-peak` mode so the `ci` and `rust_tests` jobs report an
/// identically-shaped line. `context` distinguishes the call site.
fn emit_cgroup_measurement(context: &str) {
    match cgroup_job_measurement() {
        Some(m) => {
            let cap = match m.cap_bytes {
                Some(b) => format!("{b} bytes"),
                None => "uncapped(RAM-bound)".to_string(),
            };
            let host_ram = m
                .host_ram
                .map(|b| format!("{b} bytes"))
                .unwrap_or_else(|| "unknown".to_string());
            let sccache = match (&m.sccache_rel, m.sccache_under_leaf) {
                (Some(r), true) => format!("{r} (descendant: counted in memory.peak)"),
                (Some(r), false) => format!("{r} (sibling: subtract as host_fixed_overhead)"),
                (None, _) => "not-found (treat as fixed host overhead)".to_string(),
            };
            eprintln!(
                "[measurement] cgroup peak: {peak} bytes (memory.peak @ {rel}) memory.max={cap} host_ram={host_ram} pids.current={pc} pids.max={pm} sccache-server-cgroup={sccache} context={context}",
                peak = m.leaf_peak,
                rel = m.leaf_rel,
                pc = m.pids_current,
                pm = m.pids_max
            );
        }
        None => eprintln!(
            "[measurement] cgroup peak: unavailable (no leaf cgroup or memory.peak unreadable; kernel < 5.19?) context={context}"
        ),
    }
}

/// Best-effort cgroup-v2 relative path of the sccache SERVER daemon (a `sccache` comm), scanned
/// from `/proc`. Emitted beside the binding-cap ancestor path so the analysis can classify the
/// "accounted exactly once" case: a path UNDER the ancestor → sccache is inside `memory.peak`
/// (don't subtract); a SIBLING path → subtract it as `host_fixed_overhead`. Returns `None` when
/// no sccache server process is found (then it's fixed host overhead by default, fail-closed).
fn sccache_server_cgroup_rel() -> Option<String> {
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if fs::read_to_string(p.join("comm"))
            .unwrap_or_default()
            .trim()
            != "sccache"
        {
            continue;
        }
        if let Some(rel) = fs::read_to_string(p.join("cgroup")).ok().and_then(|cg| {
            cg.lines()
                .find_map(|l| l.strip_prefix("0::"))
                .map(|s| s.trim().to_string())
        }) {
            return Some(rel);
        }
    }
    None
}

/// Per-batch timing record collected during run_walk for Gantt emission.
struct BatchRecord {
    batch_index: usize,
    wall_nanos: u128,
    /// Flattened results from all units in this batch (order: unit by unit).
    results: Vec<ClaimResult>,
}

/// Emit a fractal Gantt tree to stderr when GUNBC_FLOOR_GANTT=1.
fn emit_gantt(batch_records: &[BatchRecord], total_wall_nanos: u128) {
    let gantt_enabled = std::env::var("GUNBC_FLOOR_GANTT")
        .map(|v| v == "1")
        .unwrap_or(false);
    if !gantt_enabled {
        return;
    }
    let total_ms = total_wall_nanos / 1_000_000;
    eprintln!("[gantt] claim_executor wall: {}ms", total_ms);
    for rec in batch_records {
        let batch_ms = rec.wall_nanos / 1_000_000;
        let pct = if total_ms == 0 {
            0.0
        } else {
            100.0 * batch_ms as f64 / total_ms as f64
        };
        eprintln!(
            "[gantt]   batch {} wall: {}ms ({:.1}%)",
            rec.batch_index + 1,
            batch_ms,
            pct,
        );
        for result in &rec.results {
            let batch_pct = |ns: u128| -> f64 {
                if rec.wall_nanos == 0 {
                    0.0
                } else {
                    100.0 * ns as f64 / rec.wall_nanos as f64
                }
            };
            if result.corpus_witnesses > 0 {
                // Discovery batch: show serial-sum breakdown.
                let corpus_resolve_ms = result.corpus_resolve_nanos / 1_000_000;
                let corpus_eval_ms = result.corpus_eval_nanos / 1_000_000;
                eprintln!(
                    "[gantt]     {} ({} witnesses)",
                    result.function, result.corpus_witnesses
                );
                eprintln!(
                    "[gantt]       resolve (serial sum): {}ms  ({:.1}% of batch wall)",
                    corpus_resolve_ms,
                    batch_pct(result.corpus_resolve_nanos),
                );
                eprintln!(
                    "[gantt]       eval    (serial sum): {}ms  ({:.1}% of batch wall)",
                    corpus_eval_ms,
                    batch_pct(result.corpus_eval_nanos),
                );
            } else {
                // Single claim: show resolve (if charged) + eval.
                if result.resolve_nanos > 0 {
                    let resolve_ms = result.resolve_nanos / 1_000_000;
                    eprintln!(
                        "[gantt]     resolve (entry): {}ms  ({:.1}% of batch wall)",
                        resolve_ms,
                        batch_pct(result.resolve_nanos),
                    );
                }
                let wall_ms = result.wall_nanos / 1_000_000;
                let ok = if result.ok { "PASS" } else { "FAIL" };
                eprintln!(
                    "[gantt]     {}: {}ms  [{ok}]  ({:.1}% of batch wall)",
                    result.function,
                    wall_ms,
                    batch_pct(result.wall_nanos),
                );
            }
        }
    }
}

fn run_walk(source_roots: &[String], batches: &[Vec<Runnable>], spawn_width: usize) -> WalkOutcome {
    let width = spawn_width.max(1);
    let mut any_failed = false;
    let mut batches_run = 0usize;
    let walk_start = Instant::now();
    let mut batch_records: Vec<BatchRecord> = Vec::new();
    // Cross-batch resolve memo: SharedClaims whose runnable does a heavy whole-tree resolve
    // run on the main thread and share a single resolved InterpContext per entry across all batches.
    // Rc<ResolvedGraph> is !Send so these units cannot run on spawned threads; they
    // run sequentially here after the spawned (non-memo) threads in each batch are joined.
    // Key invariant: source_roots is constant for the lifetime of a run_walk call, so
    // keying the memo by entry alone is sufficient — a given entry always resolves against
    // the same source_roots here. If this function ever accepts multiple source_root sets,
    // the key must become (source_roots_hash, entry).
    let mut walk_memo: std::collections::HashMap<String, InterpContext> =
        std::collections::HashMap::new();
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        let units = group_batch_units(batch);
        eprintln!(
            "claim_executor: batch {} — {} node(s) in {} resolve-group(s), spawn_width={}",
            bi + 1,
            batch.len(),
            units.len(),
            width
        );
        let batch_start = Instant::now();
        // Partition units: memo units stay on the main thread; others spawn.
        // A unit goes to the memo path if (a) its profile declares heavy whole-tree
        // resolve, or (b) its entry is already in walk_memo from a prior batch — in
        // which case re-resolving would be redundant regardless of profile.
        let mut memo_units: Vec<BatchUnit> = Vec::new();
        let mut thread_units: Vec<BatchUnit> = Vec::new();
        for unit in units {
            match &unit {
                BatchUnit::SharedClaims {
                    use_walk_memo: true,
                    ..
                } => memo_units.push(unit),
                BatchUnit::SharedClaims { entry, .. } if walk_memo.contains_key(entry) => {
                    memo_units.push(unit)
                }
                _ => thread_units.push(unit),
            }
        }
        // Bracket the parallel walk in a host-effect group: the `[file]`/`[rest]`/
        // `[shell]` trace lines stream to stderr from the worker threads INSIDE the
        // group, while the scannable PASS/FAIL summary is deferred to AFTER the group
        // closes (below) so it stays outside the collapsed section. GitHub Actions
        // renders this as a collapsible `::group::`; a plain terminal as a header.
        // Threads can't interleave group markers (one open/close on the main thread
        // spans the whole batch), so it is sound under `spawn_width > 1`.
        let grouped = v1_compiler::v1_interpreter::host_trace_grouping_active();
        if grouped {
            v1_compiler::v1_interpreter::group_begin(&format!("batch {} host-effects", bi + 1));
        }
        let handles: Vec<_> = thread_units
            .into_iter()
            .map(|unit| {
                let roots = source_roots.to_vec();
                thread::spawn(move || run_batch_unit(roots, unit, width))
            })
            .collect();
        // Run memo units on the main thread while spawned threads are working.
        let mut memo_results: Vec<ClaimResult> = Vec::new();
        for unit in memo_units {
            if let BatchUnit::SharedClaims {
                entry, functions, ..
            } = unit
            {
                let results =
                    run_memo_shared_claims(source_roots, &entry, &functions, &mut walk_memo);
                memo_results.extend(results);
            }
        }
        // Collect all results before printing any PASS/FAIL — the prints must land
        // after `group_end`, and a thread panic still has to close the group.
        let mut batch_results: Vec<ClaimResult> = memo_results;
        let mut thread_panicked = false;
        for handle in handles {
            match handle.join() {
                Ok(results) => batch_results.extend(results),
                Err(_) => thread_panicked = true,
            }
        }
        if grouped {
            v1_compiler::v1_interpreter::group_end();
        }
        for result in &batch_results {
            if result.ok {
                println!(
                    "{}",
                    paint(
                        &format!("✓ PASS [batch {}] {}", bi + 1, result.function),
                        sgr::SUCCESS
                    )
                );
            } else {
                println!(
                    "{}",
                    paint(
                        &format!(
                            "✗ FAIL [batch {}] {} ({})",
                            bi + 1,
                            result.function,
                            result.detail
                        ),
                        sgr::ERROR
                    )
                );
                any_failed = true;
            }
        }
        if thread_panicked {
            println!(
                "{}",
                paint(
                    &format!("✗ FAIL [batch {}] <claim thread panicked>", bi + 1),
                    sgr::ERROR
                )
            );
            any_failed = true;
        }
        let batch_wall_nanos = batch_start.elapsed().as_nanos();
        batch_records.push(BatchRecord {
            batch_index: bi,
            wall_nanos: batch_wall_nanos,
            results: batch_results,
        });
        if any_failed {
            eprintln!(
                "claim_executor: batch {} had failures — stopping before dependent batches",
                bi + 1
            );
            break;
        }
    }
    let total_wall_nanos = walk_start.elapsed().as_nanos();
    emit_gantt(&batch_records, total_wall_nanos);
    WalkOutcome {
        any_failed,
        batches_run,
    }
}

fn remap_entry_for_temp(source_root: &str, temp_src: &Path, entry: &str) -> PathBuf {
    let prefix = format!("{source_root}/");
    if let Some(suffix) = entry.strip_prefix(&prefix) {
        temp_src.join(suffix)
    } else if let Some(suffix) = entry.strip_prefix("src/v2/") {
        temp_src.join(suffix)
    } else {
        PathBuf::from(entry)
    }
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Err(format!("{} is not a directory", from.display()));
    }
    fs::create_dir_all(to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("read_dir {}: {e}", from.display()))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let ft = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        let dest = to.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest).map_err(|e| {
                format!("copy {} -> {}: {e}", entry.path().display(), dest.display())
            })?;
        }
    }
    Ok(())
}

fn perturb_function_to_false(path: &Path, function: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let needle_fn = format!("fn {function}(");
    let needle_func = format!("func {function}(");
    let start = match (text.find(&needle_func), text.find(&needle_fn)) {
        (Some(f), _) => f,
        (None, Some(f)) => f,
        (None, None) => return Err(format!("{}: missing function {function}", path.display())),
    };
    let brace = start
        + text[start..]
            .find('{')
            .ok_or_else(|| format!("{}: missing body for {function}", path.display()))?;
    let mut depth = 0;
    let mut end = None;
    for (i, ch) in text[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.ok_or_else(|| format!("{}: unterminated body for {function}", path.display()))?;
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..brace]);
    out.push_str("{\n  false\n}");
    out.push_str(&text[end..]);
    fs::write(path, out).map_err(|e| format!("write {}: {e}", path.display()))
}

const PERTURB_SHARD_BROKEN_ENTRY: &str = "_perturb_compile_clean_shard_red.dag";

#[derive(Clone, Debug, PartialEq, Eq)]
enum PerturbPlant {
    SingleClaim { entry: String, function: String },
    CompileCleanShardBrokenEntry { rel_path: String },
}

fn perturb_plant_target(batch: &[Runnable]) -> Result<PerturbPlant, String> {
    match batch.first() {
        Some(Runnable::CompileCleanShardBatch { .. }) => {
            Ok(PerturbPlant::CompileCleanShardBrokenEntry {
                rel_path: PERTURB_SHARD_BROKEN_ENTRY.to_string(),
            })
        }
        Some(Runnable::SingleClaim {
            entry, function, ..
        }) if !entry.is_empty() => Ok(PerturbPlant::SingleClaim {
            entry: entry.clone(),
            function: function.clone(),
        }),
        _ => Err(
            "batch 1 has no plantable gating node (SingleClaim or CompileCleanShardBatch)"
                .to_string(),
        ),
    }
}

fn perturb_plant_label(plant: &PerturbPlant) -> String {
    match plant {
        PerturbPlant::SingleClaim { function, .. } => function.clone(),
        PerturbPlant::CompileCleanShardBrokenEntry { rel_path } => {
            format!("compile-clean-shard-batch[{rel_path}]")
        }
    }
}

fn run_perturb_check(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<ExitCode, ExitCode> {
    let batches = match eval_plan(source_roots, plan_entry, plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: --perturb-check: {msg}");
            return Err(ExitCode::from(2));
        }
    };
    if batches.len() < 2 {
        eprintln!(
            "claim_executor: --perturb-check needs a plan with >= 2 batches to witness the \
             walk halt (got {})",
            batches.len()
        );
        return Err(ExitCode::from(2));
    }
    let plant = match perturb_plant_target(&batches[0]) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("claim_executor: --perturb-check: {msg}");
            return Err(ExitCode::from(2));
        }
    };

    let primary = &source_roots[0];
    let tmp = std::env::temp_dir().join(format!("claim-executor-perturb-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let temp_src = tmp.join("src");
    if let Err(e) = copy_dir_all(Path::new(primary), &temp_src) {
        eprintln!("claim_executor: --perturb-check: {e}");
        return Err(ExitCode::from(2));
    }

    match &plant {
        PerturbPlant::SingleClaim { entry, function } => {
            let gating_path = remap_entry_for_temp(primary, &temp_src, entry);
            if let Err(e) = perturb_function_to_false(&gating_path, function) {
                let _ = fs::remove_dir_all(&tmp);
                eprintln!("claim_executor: --perturb-check: plant gating->false failed: {e}");
                return Err(ExitCode::from(2));
            }
        }
        PerturbPlant::CompileCleanShardBrokenEntry { rel_path } => {
            let broken_path = temp_src.join(rel_path);
            let module_source = match perturb_shard_broken_module_source(source_roots) {
                Ok(s) => s,
                Err(e) => {
                    let _ = fs::remove_dir_all(&tmp);
                    eprintln!("claim_executor: --perturb-check: {e}");
                    return Err(ExitCode::from(2));
                }
            };
            if let Err(e) = fs::write(&broken_path, module_source) {
                let _ = fs::remove_dir_all(&tmp);
                eprintln!("claim_executor: --perturb-check: plant broken shard entry failed: {e}");
                return Err(ExitCode::from(2));
            }
        }
    }

    let temp_root = temp_src.to_string_lossy().into_owned();
    let remap_root = |root: &str| -> String {
        if root == primary.as_str() {
            temp_root.clone()
        } else {
            root.to_string()
        }
    };
    let remapped: Vec<Vec<Runnable>> = batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .map(|r| match r {
                    Runnable::SingleClaim {
                        entry,
                        function,
                        use_walk_memo,
                    } => Runnable::SingleClaim {
                        entry: if entry.is_empty() {
                            entry.clone()
                        } else {
                            remap_entry_for_temp(primary, &temp_src, entry)
                                .to_string_lossy()
                                .into_owned()
                        },
                        function: function.clone(),
                        use_walk_memo: *use_walk_memo,
                    },
                    Runnable::DiscoveryBatch {
                        source_roots: roots,
                        scan_dirs,
                        explicit_entries,
                        skip_unaffected_node_frontier,
                        exclude_substrings,
                        discovery_scope_dirs,
                        spawn_width_cap,
                    } => Runnable::DiscoveryBatch {
                        source_roots: roots.iter().map(|r| remap_root(r)).collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        explicit_entries: explicit_entries.clone(),
                        skip_unaffected_node_frontier: *skip_unaffected_node_frontier,
                        exclude_substrings: exclude_substrings.clone(),
                        discovery_scope_dirs: discovery_scope_dirs.clone(),
                        spawn_width_cap: *spawn_width_cap,
                    },
                    Runnable::CompileCleanShardBatch {
                        entry_paths,
                        spawn_width_cap,
                    } => match &plant {
                        PerturbPlant::CompileCleanShardBrokenEntry { rel_path } => {
                            Runnable::CompileCleanShardBatch {
                                entry_paths: vec![rel_path.clone()],
                                spawn_width_cap: *spawn_width_cap,
                            }
                        }
                        _ => Runnable::CompileCleanShardBatch {
                            entry_paths: entry_paths.clone(),
                            spawn_width_cap: *spawn_width_cap,
                        },
                    },
                })
                .collect()
        })
        .collect();

    eprintln!(
        "claim_executor: --perturb-check: planted batch-1 gating witness `{}` -> false; re-walking",
        perturb_plant_label(&plant)
    );
    let outcome = run_walk(&[temp_root], &remapped, 1);
    let _ = fs::remove_dir_all(&tmp);

    if outcome.any_failed && outcome.batches_run == 1 {
        eprintln!(
            "claim_executor: --perturb-check OK: gating batch-1 false -> run failed closed AND \
             walk halted before batch 2 (batches_run=1 of {})",
            batches.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "claim_executor: --perturb-check FAIL: expected fail-closed + halt-at-batch-1, got \
             any_failed={} batches_run={} (of {})",
            outcome.any_failed,
            outcome.batches_run,
            batches.len()
        );
        Ok(ExitCode::from(1))
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut plan_entry: Option<String> = None;
    let mut plan_function = "bre_claim_batches".to_string();
    let mut notice_title: Option<String> = None;
    let mut perturb_check = false;
    let mut measure_cgroup_peak = false;
    let mut verify_artifacts: Vec<String> = Vec::new();
    let mut verify_artifacts_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--verify-build-artifacts" => {
                // All remaining positional args are declared release-artifact paths to verify.
                verify_artifacts_mode = true;
                i += 1;
                while i < args.len() {
                    verify_artifacts.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--plan-entry" => {
                i += 1;
                plan_entry = Some(require_value(&args, i, "--plan-entry")?);
            }
            "--plan-function" => {
                i += 1;
                plan_function = require_value(&args, i, "--plan-function")?;
            }
            "--notice-title" => {
                i += 1;
                notice_title = Some(require_value(&args, i, "--notice-title")?);
            }
            "--perturb-check" => perturb_check = true,
            "--measure-cgroup-peak" => measure_cgroup_peak = true,
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    // Build-artifact verification (no plan run): the floor's bootstrap `cargo build` is followed by
    // this check so a `successful` build that nonetheless produced a missing/zero-byte binary
    // (sccache serving a truncated/empty cached artifact — DESIGN §5 fail-open) fails CLOSED before
    // the floor runs. The LOGIC lives here (the floor binary) instead of inline shell in ci.yml; the
    // declared artifact paths are still authored by the .dag spec and passed as positional args.
    // Short-circuits before the plan-arg requirements so it needs no `--plan-entry`.
    if verify_artifacts_mode {
        return verify_build_artifacts(&verify_artifacts);
    }

    // Standalone whole-tree cgroup measurement (no plan run): the `rust_tests` job invokes this
    // after its gate to emit ITS leaf cgroup peak (a separate ephemeral runner cgroup from the `ci`
    // job's), reusing the same single-authority walk/emit. Short-circuits before the plan-arg
    // requirements so it needs no `--source-root`/`--plan-entry`.
    if measure_cgroup_peak {
        let job = std::env::var("GITHUB_JOB").unwrap_or_else(|_| "standalone".to_string());
        emit_cgroup_measurement(&format!("job={job} (--measure-cgroup-peak)"));
        return Ok(ExitCode::SUCCESS);
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let plan_entry = match plan_entry {
        Some(e) => e,
        None => {
            eprintln!("claim_executor: --plan-entry <file.dag> is required");
            return Err(ExitCode::from(2));
        }
    };

    // Install the host-effect trace policy from the .dag authority once, before
    // discovery threads spawn, so `[file] read` / `[rest]` / `[hermetic:mock]` etc.
    // are funnelled per `gunbc.output_policy` instead of flooding the floor log.
    v1_compiler::cli_run::install_output_policy(&source_roots);
    // Install the per-target group-marker syntax (GitHub Actions `::group::` vs a
    // plain-terminal header) from the .dag authority, so the parallel walk folds each
    // batch's host-effect traces into a collapsible group.
    v1_compiler::cli_run::install_group_syntax(&source_roots);

    if perturb_check {
        return run_perturb_check(&source_roots, &plan_entry, &plan_function);
    }

    let batches = match eval_plan(&source_roots, &plan_entry, &plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };

    eprintln!(
        "claim_executor: [{}] executor plan = {} batch(es) from {}::{}",
        notice_title.as_deref().unwrap_or("ci floor"),
        batches.len(),
        plan_entry,
        plan_function
    );

    if batches.is_empty() {
        eprintln!("claim_executor: executor plan produced 0 batches — failing closed");
        return Err(ExitCode::from(1));
    }

    let spawn_width = match eval_spawn_width(&source_roots, &plan_entry, &plan_function) {
        Ok(w) => w,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    eprintln!(
        "claim_executor: spawn_width={} from {}::{}",
        spawn_width,
        plan_entry,
        spawn_width_function_name(&plan_function)
            .as_deref()
            .unwrap_or("<serial>")
    );

    let outcome = run_walk(&source_roots, &batches, spawn_width);
    match peak_rss_bytes() {
        Some(bytes) => {
            eprintln!(
                "[measurement] floor peak RSS: {bytes} bytes (VmHWM) at spawn_width={spawn_width}"
            );
            let width = spawn_width.max(1) as u64;
            let per_shard = bytes.div_ceil(width);
            eprintln!(
                "[calibration] max-per-shard-peak-rss: {per_shard} bytes at spawn_width={spawn_width}"
            );
        }
        None => eprintln!(
            "[measurement] floor peak RSS: unavailable (no /proc/self/status) at spawn_width={spawn_width}"
        ),
    }
    // [measurement] WHOLE-TREE cgroup peak — the SOUND placement divisor input (SELF-RSS above omits
    // child rustc/sccache PIDs; cgroup-v2 `memory.peak` at the leaf job cgroup is hierarchical and
    // captures them). Single authority `emit_cgroup_measurement` so the `ci` and `rust_tests` jobs
    // report an identically-shaped line. Runtime-harmless read-only.
    emit_cgroup_measurement(&format!("floor spawn_width={spawn_width}"));
    if outcome.any_failed {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn single(entry: &str, function: &str) -> Runnable {
        Runnable::SingleClaim {
            entry: entry.to_string(),
            function: function.to_string(),
            use_walk_memo: false,
        }
    }

    fn discovery() -> Runnable {
        Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string()],
            scan_dirs: vec![],
            explicit_entries: vec![],
            skip_unaffected_node_frontier: true,
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            spawn_width_cap: 0,
        }
    }

    fn compile_clean_shard_batch() -> Runnable {
        Runnable::CompileCleanShardBatch {
            entry_paths: vec!["dsl/std/logic.dag".to_string()],
            spawn_width_cap: 4,
        }
    }

    #[test]
    fn perturb_plant_target_prefers_shard_batch_when_first() {
        let batch = vec![compile_clean_shard_batch(), single("gate.dag", "g1")];
        assert_eq!(
            perturb_plant_target(&batch),
            Ok(PerturbPlant::CompileCleanShardBrokenEntry {
                rel_path: PERTURB_SHARD_BROKEN_ENTRY.to_string(),
            })
        );
    }

    #[test]
    fn perturb_plant_target_uses_single_claim_for_monolith_batch() {
        let batch = vec![single(
            "dsl/tools/dsl_compile_clean_gate.dag",
            "dsl_compile_clean_gate_passes",
        )];
        assert_eq!(
            perturb_plant_target(&batch),
            Ok(PerturbPlant::SingleClaim {
                entry: "dsl/tools/dsl_compile_clean_gate.dag".to_string(),
                function: "dsl_compile_clean_gate_passes".to_string(),
            })
        );
    }

    /// Flatten the grouped units back to the (entry, function) claims they will execute, in the
    /// order each unit runs them. `Discovery` contributes no SingleClaim pairs; an empty-entry
    /// sentinel surfaces as `("", function)`. This is the verdict-preservation oracle: grouping is
    /// only allowed to change scheduling, never which claims run.
    fn executed_claims(units: &[BatchUnit]) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for unit in units {
            match unit {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } => {
                    for f in functions {
                        out.push((entry.clone(), f.clone()));
                    }
                }
                BatchUnit::UnrunnableSentinel { function } => {
                    out.push((String::new(), function.clone()));
                }
                BatchUnit::Discovery { .. } => {}
                BatchUnit::CompileCleanShardBatch { .. } => {}
            }
        }
        out
    }

    #[test]
    fn same_entry_claims_coalesce_into_one_resolve_group() {
        // The floor's batch-2 shape: many gate witnesses share one file. They must collapse to a
        // single resolve-group (one resolve) carrying every function, in input order.
        let batch = vec![
            single("gate.dag", "rust_gate"),
            single("gate.dag", "emit_gate"),
            single("gate.dag", "layering_gate"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(
            units.len(),
            1,
            "three same-entry claims => one resolve-group"
        );
        match &units[0] {
            BatchUnit::SharedClaims {
                entry, functions, ..
            } => {
                assert_eq!(entry, "gate.dag");
                assert_eq!(functions, &["rust_gate", "emit_gate", "layering_gate"]);
            }
            _ => panic!("expected a SharedClaims unit"),
        }
    }

    #[test]
    fn grouping_preserves_every_claim_exactly_once() {
        // Verdict preservation: no claim dropped, duplicated, or invented — grouping only reorders
        // by coalescing same-entry claims (keeping their relative order). Mixed entries interleave.
        let batch = vec![
            single("a.dag", "a1"),
            single("b.dag", "b1"),
            single("a.dag", "a2"),
            discovery(),
            single("b.dag", "b2"),
            single("", "__unmapped__"),
        ];
        let units = group_batch_units(&batch);

        // a.dag coalesces (a1 before a2), b.dag coalesces (b1 before b2), discovery + sentinel stay.
        let mut got = executed_claims(&units);
        let mut want = vec![
            ("a.dag".to_string(), "a1".to_string()),
            ("a.dag".to_string(), "a2".to_string()),
            ("b.dag".to_string(), "b1".to_string()),
            ("b.dag".to_string(), "b2".to_string()),
            (String::new(), "__unmapped__".to_string()),
        ];
        got.sort();
        want.sort();
        assert_eq!(got, want, "exact same claim set runs after grouping");

        // Same-entry relative order is preserved within each coalesced group.
        let a = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "a.dag" => Some(functions),
                _ => None,
            })
            .expect("a.dag group present");
        assert_eq!(a, &["a1", "a2"]);
    }

    #[test]
    fn empty_entry_sentinel_is_its_own_failing_unit() {
        // The unmapped-node sentinel (empty entry) must never coalesce into a real resolve-group;
        // it is a stand-alone fail-closed unit so a non-complete plan stays red (DESIGN §5).
        let batch = vec![single("", "__gunbc_ci_floor_unmapped__")];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 1);
        match &units[0] {
            BatchUnit::UnrunnableSentinel { function } => {
                assert_eq!(function, "__gunbc_ci_floor_unmapped__");
            }
            _ => panic!("empty-entry claim must be an UnrunnableSentinel, not a resolve-group"),
        }

        // And it fails closed when run.
        let unit = units.into_iter().next().unwrap();
        let results = run_batch_unit(vec!["src/v2".to_string()], unit, 1);
        assert_eq!(results.len(), 1);
        assert!(!results[0].ok, "unmapped sentinel must fail closed");
    }

    #[test]
    fn discovery_batch_stays_an_isolated_unit() {
        // A discovery corpus node never merges with SingleClaims — it keeps its own shard-parallel
        // resolve path.
        let batch = vec![
            single("gate.dag", "g1"),
            discovery(),
            single("gate.dag", "g2"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 2, "gate group + discovery");
        assert!(units
            .iter()
            .any(|u| matches!(u, BatchUnit::Discovery { .. })));
        let gate = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "gate.dag" => Some(functions),
                _ => None,
            })
            .expect("gate group present");
        assert_eq!(gate, &["g1", "g2"], "both gate claims kept in one group");
    }

    // --- build-artifact verification teeth (DESIGN §5 fail-open guard) ---

    fn write_exec(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, bytes).expect("write artifact");
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&p, perms).expect("chmod");
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn verify_build_artifacts_accepts_nonempty_executables() {
        let dir = std::env::temp_dir().join(format!("cev-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"\x7fELF-not-really-but-nonempty");
        let b = write_exec(&dir, "gunbc", b"binary");
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Ok(c) if c == ExitCode::SUCCESS),
            "real bins pass"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_zero_byte() {
        let dir = std::env::temp_dir().join(format!("cev-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"ok");
        let b = write_exec(&dir, "gunbc", b""); // sccache served a truncated/empty cached artifact
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "zero-byte artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_missing() {
        let missing = std::env::temp_dir()
            .join(format!("cev-missing-{}/gunbc", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let r = verify_build_artifacts(&[missing]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "absent artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_empty_arglist() {
        let r = verify_build_artifacts(&[]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(2)),
            "no declared artifacts is a usage error, fail closed"
        );
    }

    /// Warm==cold purity oracle for the cross-batch resolve memo (DESIGN §5).
    ///
    /// Proves that `run_memo_shared_claims` (warm path: 2nd+ calls share the cached
    /// `InterpContext`) produces byte-identical `(function, ok, detail)` results vs
    /// `run_shared_entry_claims` (cold path: fresh resolve each time). Goes RED if
    /// the shared `InterpContext` has interior mutability that contaminates results
    /// across claims — the failure mode the declaration-validity lenses cannot catch.
    #[test]
    fn memo_warm_cold_results_are_identical() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dsl").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dsl/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let functions: &[String] = &[
            "witness_negligible_profile_is_not_heavy".to_string(),
            "witness_substantial_memory_forbids_corpus_co_residence".to_string(),
        ];

        // Cold path: two independent resolves, one function each.
        let cold: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_shared_entry_claims(&source_roots, &entry, std::slice::from_ref(f))
                    .into_iter()
                    .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        // Warm path: single memo, first call resolves fresh, second hits the cache.
        let mut memo = std::collections::HashMap::new();
        let warm: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_memo_shared_claims(&source_roots, &entry, std::slice::from_ref(f), &mut memo)
                    .into_iter()
                    .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        assert_eq!(
            warm, cold,
            "memo warm path must be byte-identical to cold path — shared InterpContext is pure"
        );
    }

    /// Resolve-count oracle: proves the memo fires resolve_entry_graph exactly once
    /// per distinct entry per walk (DESIGN §2 — no redundant resolve).
    ///
    /// Goes RED if the memo is bypassed: both calls would have resolve_nanos > 0,
    /// meaning resolve_entry_graph fired twice for the same entry instead of once.
    #[test]
    fn memo_deduplicates_resolve_count() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dsl").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dsl/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();

        let mut memo = std::collections::HashMap::new();

        // First call for this entry: must resolve (resolve_nanos > 0).
        let first = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_negligible_profile_is_not_heavy".to_string()],
            &mut memo,
        );
        assert!(
            first[0].resolve_nanos > 0,
            "first call must pay the resolve cost (resolve_entry_graph fires)"
        );

        // Second call for the same entry: must cache-hit (resolve_nanos == 0).
        let second = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_substantial_memory_forbids_corpus_co_residence".to_string()],
            &mut memo,
        );
        assert_eq!(
            second[0].resolve_nanos, 0,
            "second call must cache-hit — resolve_entry_graph must NOT fire again"
        );
    }

    #[test]
    fn perturb_shard_broken_module_source_reads_dag_authority() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("dsl").to_string_lossy().into_owned(),
            root.join("src/v2").to_string_lossy().into_owned(),
        ];
        std::env::set_current_dir(&root).expect("chdir to workspace root");
        let source = perturb_shard_broken_module_source(&source_roots)
            .expect("read broken_shard_module_source from shard transport");
        assert!(
            source.contains("perturb_compile_clean_red"),
            "expected authority module body"
        );
        assert!(
            source.contains("Some { value: s }"),
            "expected broken Optional skew from perturb_module_source authority"
        );
    }
}
