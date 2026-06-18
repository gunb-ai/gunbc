//! Batch claim executor: the host transport for the v2.workflow.executor .dag.
//!
//! The `.dag` is the batching AUTHORITY. A plan function (default
//! `bre_claim_batches` in `src/v2/workflow/batch_runner.dag`; the CI floor uses
//! `gunbc_ci_floor_batches` in `src/v2/workflow/ci_floor_plan.dag`, spec-derived
//! from `gunbc.ci_spec`) folds the
//! dependency frontier through `v2.workflow.executor` and returns
//! `List<List<Runnable>>` — the outer list is batches in execution order, the
//! inner list is the runnables (a `SingleClaim` witness/gate, or the whole
//! `DiscoveryBatch` corpus) runnable in parallel within that batch. This binary
//! evaluates that plan, walks the returned value, and RUNS it: batch by batch
//! (respecting the executor's ordering), nodes within a batch concurrently.
//!
//! It does NOT decide grouping or ordering — add a node or a dependency in the
//! `.dag` and the batches change with zero edit here. That is the dogfood: the
//! `.dag` earns CI authority by being consumed to drive real behavior, not by
//! mirroring a hand-written schedule.
//!
//! Like `claim_batch`/`regen_stage0`, this is a hand-written CLI bin — NOT routed
//! through the generated `main.rs`/emit stage — reusing the same resolve/run
//! primitives as `gunbc run` (`cli_run::resolve_entry_graph`,
//! `cli_run::run_value`, `cli_run::run_claim`).
//!
//! Usage:
//!   claim_executor --source-root <dir> [--source-root <dir> ...] \
//!                  --plan-entry <file.dag> [--plan-function <fn>]
//!
//! Exit codes: 0 = every claim in every batch passed; 1 = any claim failed,
//! returned non-Bool, raised a runtime error, or a resolve/plan eval failed;
//! 2 = usage error.

// Binary entrypoint: reports results directly on stdout/stderr.
#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;

use v1_compiler::cli_run::{
    make_eval_context, resolve_entry_graph, run_claim, run_discovery_corpus, run_value,
    ClaimOutcome,
};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

/// One runnable plan node, projected from the plan value. A `SingleClaim` is one
/// `(entry, function)` Bool witness (the demo suite + the floor's per-gate
/// nodes); a `DiscoveryBatch` is the whole `--roster-from-discovery` corpus as a
/// single node — it REUSES `cli_run::run_discovery_corpus` (the shared roster
/// authority), it does NOT re-coin the ~199-row roster as explicit plan nodes
/// (that would duplicate the discovery scan — DESIGN §3 — and cost a cold
/// resolve per row).
///
/// 🟡 SCAFFOLD — feature:floor-discovery-batch-node — owner:merry-owl —
/// dissolve-on: the "actually correct" floor model lands (per-job typed verdicts
/// reified into .dag / affected-set→scheduler-frontier fusion). Until then this
/// coproduct is the pragmatic interim that lets the WHOLE floor (gates + corpus)
/// run dependency-ordered through one host. See merry-owl's lane.
#[derive(Clone)]
enum Runnable {
    SingleClaim {
        entry: String,
        function: String,
    },
    DiscoveryBatch {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
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

/// Walk the std `List` representation (the `FreeMonoid` Cons/Empty coproduct)
/// into a borrowed Vec of element values. `gunbc` renders `List<T>` as
/// `Cons { head, tail } | Empty`, not `Value::List`.
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
            // Tolerate an eager `Value::List` too, in case the representation
            // ever changes; keeps the walker honest rather than silently wrong.
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

/// Read a `List<String>` (FreeMonoid Cons/Empty) into a `Vec<String>`.
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

/// Read a required String record/variant field.
fn str_field(
    fields: &std::collections::HashMap<v1_compiler::v1_interpreter::Symbol, Value>,
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

/// Project one plan element into a `Runnable`. Accepts (fail-closed on anything
/// else):
///   - a bare `ClaimRef { entry, function }` record (back-compat: the demo suite
///     and any SingleClaim authored as a record) → `SingleClaim`;
///   - a `RunnableSingleClaim { entry, function }` variant → `SingleClaim`;
///   - a `RunnableDiscoveryBatch { source_roots, scan_dirs }` variant → the
///     discovery-corpus node.
fn runnable_from_value(value: &Value, ctx: &InterpContext) -> Result<Runnable, String> {
    match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => {
            Ok(Runnable::SingleClaim {
                entry: str_field(fields, "entry", "ClaimRef", ctx)?,
                function: str_field(fields, "function", "ClaimRef", ctx)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableSingleClaim") => Ok(Runnable::SingleClaim {
            entry: str_field(fields, "entry", "RunnableSingleClaim", ctx)?,
            function: str_field(fields, "function", "RunnableSingleClaim", ctx)?,
        }),
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
            // explicit_entries: a List of records with `entry`/`function` String
            // fields (the CiSpec witness_entries appended to the discovery roster).
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
            Ok(Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
            })
        }
        other => Err(format!(
            "expected a ClaimRef record or Runnable variant, got {}",
            ctx.format_value(other)
        )),
    }
}

/// Parse the plan value `List<List<Runnable>>` into ordered batches.
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

/// Result of running one claim, in a thread-safe (Send) form. The resolved graph
/// is `Rc`-based (`!Send`), so each claim resolves and runs entirely within its
/// own thread and reports back only this plain summary.
struct ClaimResult {
    function: String,
    ok: bool,
    detail: String,
}

fn run_one_runnable(source_roots: Vec<String>, runnable: Runnable) -> ClaimResult {
    match runnable {
        Runnable::SingleClaim { entry, function } => {
            run_single_claim(&source_roots, entry, function)
        }
        Runnable::DiscoveryBatch {
            source_roots: roots,
            scan_dirs,
            explicit_entries,
        } => run_discovery_batch_node(roots, scan_dirs, explicit_entries),
    }
}

fn run_single_claim(source_roots: &[String], entry: String, function: String) -> ClaimResult {
    // Fail-closed sentinel: the plan projects an empty-`entry` ClaimRef for any
    // unmapped suite node or non-complete executor plan (see batch_runner.dag).
    // It carries no resolvable witness, so it is a hard error — never a vacuous
    // pass.
    if entry.is_empty() {
        return ClaimResult {
            function,
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
        };
    }
    let (graph, source_indices) = match resolve_entry_graph(source_roots, &entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return ClaimResult {
                function,
                ok: false,
                detail: format!("resolve failed for {}: {}", entry, msg),
            }
        }
    };
    // Context scoped to this claim's graph: its `data` cache drops with it.
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
    match run_claim(&ctx, &function) {
        ClaimOutcome::Pass => ClaimResult {
            function,
            ok: true,
            detail: String::new(),
        },
        ClaimOutcome::Fail => ClaimResult {
            function,
            ok: false,
            detail: "returned Bool(false)".to_string(),
        },
        ClaimOutcome::NotBool { got } => ClaimResult {
            function,
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
        },
        ClaimOutcome::RuntimeError { message } => ClaimResult {
            function,
            ok: false,
            detail: format!("runtime error: {}", message),
        },
    }
}

/// Run the whole discovery corpus as one plan node, reusing the shared roster +
/// run loop (`cli_run::run_discovery_corpus`). Fail-closed: an empty roster, a
/// resolve failure, or any failing witness fails the node.
fn run_discovery_batch_node(
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    explicit_entries: Vec<(String, String)>,
) -> ClaimResult {
    let label = format!(
        "discovery-corpus[{} root(s)+{} explicit]",
        source_roots.len(),
        explicit_entries.len()
    );
    match run_discovery_corpus(
        &source_roots,
        &scan_dirs,
        &explicit_entries,
        ExecutionMode::Wet,
    ) {
        Ok(summary) if summary.failures.is_empty() => {
            eprintln!(
                "[measurement] discovery corpus: {} witness(es), {:.3}ms total wall",
                summary.total,
                summary.total_measured_nanos as f64 / 1.0e6,
            );
            ClaimResult {
                function: format!("{label} ({} witnesses)", summary.total),
                ok: true,
                detail: String::new(),
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
        },
        Err(msg) => ClaimResult {
            function: label,
            ok: false,
            detail: format!("discovery corpus failed: {msg}"),
        },
    }
}

/// Evaluate the executor-decided plan into ordered batches. The `.dag` is the
/// batching authority: this reads the `List<List<Runnable>>` the plan function
/// returns and parses it — it never groups or orders anything itself.
fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<Runnable>>, String> {
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for plan {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Wet);
    let plan_value = run_value(&plan_ctx, plan_function).map_err(|msg| {
        format!(
            "plan eval failed ({}::{}): {}",
            plan_entry, plan_function, msg
        )
    })?;
    let batches = batches_from_plan(&plan_value, &plan_ctx)
        .map_err(|msg| format!("malformed plan value: {}", msg))?;
    // Plan graph/value are `Rc`-based (`!Send`); drop before spawning claim threads.
    drop(plan_value);
    drop(plan_graph);
    Ok(batches)
}

/// Outcome of walking the executor-decided batches: whether any claim failed and
/// how many batches actually started executing (the walk halts at a failed batch,
/// so `batches_run < batches.len()` witnesses that the halt fired).
struct WalkOutcome {
    any_failed: bool,
    batches_run: usize,
}

/// Run batch by batch (executor ordering); claims within a batch in parallel. The
/// batch boundary is a barrier: batch N+1 starts only after every claim in batch N
/// has reported. A failed batch halts the walk before its dependents. The batch
/// MEMBERSHIP and ORDER are the `.dag` plan's — this only walks them.
fn run_walk(source_roots: &[String], batches: &[Vec<Runnable>]) -> WalkOutcome {
    let mut any_failed = false;
    let mut batches_run = 0usize;
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        eprintln!("claim_executor: batch {} — {} node(s)", bi + 1, batch.len());
        let handles: Vec<_> = batch
            .iter()
            .map(|runnable| {
                let roots = source_roots.to_vec();
                let runnable = runnable.clone();
                thread::spawn(move || run_one_runnable(roots, runnable))
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(result) => {
                    if result.ok {
                        println!("PASS [batch {}] {}", bi + 1, result.function);
                    } else {
                        println!(
                            "FAIL [batch {}] {} ({})",
                            bi + 1,
                            result.function,
                            result.detail
                        );
                        any_failed = true;
                    }
                }
                Err(_) => {
                    println!("FAIL [batch {}] <claim thread panicked>", bi + 1);
                    any_failed = true;
                }
            }
        }
        // Fail closed at the barrier: if a gating batch failed, do not run the
        // dependent batches that the executor placed behind it.
        if any_failed {
            eprintln!(
                "claim_executor: batch {} had failures — stopping before dependent batches",
                bi + 1
            );
            break;
        }
    }
    WalkOutcome {
        any_failed,
        batches_run,
    }
}

/// Map a repo-relative entry path onto the temp copy of `source_root` (same scheme
/// as `ci-claim-gate`): strip the root prefix and rejoin under the temp `src` dir.
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

/// Rewrite a witness function's body to `{ false }` in place (same brace-matched
/// transform `ci-claim-gate` uses) so the planted witness evaluates false.
fn perturb_function_to_false(path: &Path, function: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    // Match both `fn NAME(` and `func NAME(` (effectful gates are `func`). Prefer
    // whichever appears; `func ` ends in `c ` so it won't be confused with `fn `.
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

/// `--perturb-check` receipt for the executor's run-loop WALK (CI orchestration).
///
/// Reads the `.dag`-decided plan (structure/ordering is the model's — see the
/// `bre_*_yields_*_batches` witnesses in batch_runner.dag), plants the batch-1
/// gating witness body -> `false` in a temp copy, and re-walks. The receipt
/// asserts BOTH halves of the walk-halt: the run fails closed (exit != 0) AND the
/// walk stops before the dependent batches (only batch 1 executed). This tests
/// ONLY the run-loop walk — it derives no grouping or ordering itself.
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
    // The walk-halt is only observable with a dependent batch behind the gate.
    if batches.len() < 2 {
        eprintln!(
            "claim_executor: --perturb-check needs a plan with >= 2 batches to witness the \
             walk halt (got {})",
            batches.len()
        );
        return Err(ExitCode::from(2));
    }
    // The gating node must be a SingleClaim with a plantable witness body; a
    // DiscoveryBatch in batch 1 has no single function to perturb (the floor plan
    // places compile-clean — a SingleClaim — at batch 1 precisely so this holds).
    let (gating_entry, gating_function) = match batches[0].first() {
        Some(Runnable::SingleClaim { entry, function }) if !entry.is_empty() => {
            (entry.clone(), function.clone())
        }
        _ => {
            eprintln!(
                "claim_executor: --perturb-check: batch 1 has no plantable SingleClaim gating node"
            );
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

    // Plant the gating (batch-1) witness body -> false in the temp tree.
    let gating_path = remap_entry_for_temp(primary, &temp_src, &gating_entry);
    if let Err(e) = perturb_function_to_false(&gating_path, &gating_function) {
        let _ = fs::remove_dir_all(&tmp);
        eprintln!("claim_executor: --perturb-check: plant gating->false failed: {e}");
        return Err(ExitCode::from(2));
    }

    // Remap every node's entry onto the temp tree (pure path rewrite; batch
    // membership and order are unchanged — that is the .dag plan's), then re-walk.
    // Only the gating witness body differs from the green run. The walk halts at
    // batch 1 (planted false), so batch-2 DiscoveryBatch nodes never execute, but
    // we remap their source_roots too so the perturb tree is self-consistent.
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
                    Runnable::SingleClaim { entry, function } => Runnable::SingleClaim {
                        entry: if entry.is_empty() {
                            entry.clone()
                        } else {
                            remap_entry_for_temp(primary, &temp_src, entry)
                                .to_string_lossy()
                                .into_owned()
                        },
                        function: function.clone(),
                    },
                    Runnable::DiscoveryBatch {
                        source_roots: roots,
                        scan_dirs,
                        explicit_entries,
                    } => Runnable::DiscoveryBatch {
                        source_roots: roots.iter().map(|r| remap_root(r)).collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        explicit_entries: explicit_entries.clone(),
                    },
                })
                .collect()
        })
        .collect();

    eprintln!(
        "claim_executor: --perturb-check: planted batch-1 gating witness `{}` -> false; re-walking",
        gating_function
    );
    let outcome = run_walk(&[temp_root], &remapped);
    let _ = fs::remove_dir_all(&tmp);

    // Receipt: the planted gating failure must fail the run closed AND halt the
    // walk before batch 2 (exactly one batch executed).
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

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
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

    // --perturb-check: the run-loop walk-halt receipt (CI orchestration), kept
    // entirely separate from the green dogfood run below.
    if perturb_check {
        return run_perturb_check(&source_roots, &plan_entry, &plan_function);
    }

    // 1. Evaluate the executor-decided plan (the `.dag` is the batching authority).
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

    // Fail closed on a zero-batch plan: an empty run is never a successful run.
    // A non-complete executor state is projected as a sentinel batch (not []),
    // but guard here too so no plan shape can become a vacuous exit-0.
    if batches.is_empty() {
        eprintln!("claim_executor: executor plan produced 0 batches — failing closed");
        return Err(ExitCode::from(1));
    }

    // 2. Run batch by batch (executor ordering), claims within a batch in parallel.
    let outcome = run_walk(&source_roots, &batches);
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
