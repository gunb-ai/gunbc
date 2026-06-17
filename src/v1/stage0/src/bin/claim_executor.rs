//! Batch claim executor: the host transport for the v2.workflow.executor .dag.
//!
//! The `.dag` is the batching AUTHORITY. A plan function (default
//! `bre_claim_batches` in `src/v2/workflow/batch_runner.dag`) folds
//! the dependency frontier through `v2.workflow.executor` and returns
//! `List<List<ClaimRef>>` — the outer list is batches in execution order, the
//! inner list is the claims runnable in parallel within that batch. This binary
//! evaluates that plan, walks the returned value, and RUNS it: batch by batch
//! (respecting the executor's ordering), claims within a batch concurrently.
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
    make_eval_context, resolve_entry_graph, run_claim, run_value, ClaimOutcome,
};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

/// One runnable claim, projected from a `ClaimRef` record in the plan value.
#[derive(Clone)]
struct ClaimRef {
    entry: String,
    function: String,
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

fn claim_ref_from_value(value: &Value, ctx: &InterpContext) -> Result<ClaimRef, String> {
    let fields = match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => fields,
        other => {
            return Err(format!(
                "expected a ClaimRef record, got {}",
                ctx.format_value(other)
            ))
        }
    };
    let str_field = |name: &str| -> Result<String, String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Ok(s.clone()),
            Some(other) => Err(format!(
                "ClaimRef.{} is {}, not String",
                name,
                ctx.format_value(other)
            )),
            None => Err(format!("ClaimRef missing field `{}`", name)),
        }
    };
    Ok(ClaimRef {
        entry: str_field("entry")?,
        function: str_field("function")?,
    })
}

/// Parse the plan value `List<List<ClaimRef>>` into ordered batches of claims.
fn batches_from_plan(plan: &Value, ctx: &InterpContext) -> Result<Vec<Vec<ClaimRef>>, String> {
    let mut batches = Vec::new();
    for batch_val in free_monoid_elems(plan, ctx)? {
        let mut batch = Vec::new();
        for claim_val in free_monoid_elems(batch_val, ctx)? {
            batch.push(claim_ref_from_value(claim_val, ctx)?);
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

fn run_one_claim(source_roots: Vec<String>, claim: ClaimRef) -> ClaimResult {
    // Fail-closed sentinel: the plan projects an empty-`entry` ClaimRef for any
    // unmapped suite node or non-complete executor plan (see batch_runner.dag).
    // It carries no resolvable witness, so it is a hard error — never a vacuous
    // pass.
    if claim.entry.is_empty() {
        return ClaimResult {
            function: claim.function,
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
        };
    }
    let (graph, source_indices) = match resolve_entry_graph(&source_roots, &claim.entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return ClaimResult {
                function: claim.function,
                ok: false,
                detail: format!("resolve failed for {}: {}", claim.entry, msg),
            }
        }
    };
    // Context scoped to this claim's graph: its `data` cache drops with it.
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
    match run_claim(&ctx, &claim.function) {
        ClaimOutcome::Pass => ClaimResult {
            function: claim.function,
            ok: true,
            detail: String::new(),
        },
        ClaimOutcome::Fail => ClaimResult {
            function: claim.function,
            ok: false,
            detail: "returned Bool(false)".to_string(),
        },
        ClaimOutcome::NotBool { got } => ClaimResult {
            function: claim.function,
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
        },
        ClaimOutcome::RuntimeError { message } => ClaimResult {
            function: claim.function,
            ok: false,
            detail: format!("runtime error: {}", message),
        },
    }
}

/// Evaluate the executor-decided plan into ordered batches. The `.dag` is the
/// batching authority: this reads the `List<List<ClaimRef>>` the plan function
/// returns and parses it — it never groups or orders anything itself.
fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<ClaimRef>>, String> {
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
fn run_walk(source_roots: &[String], batches: &[Vec<ClaimRef>]) -> WalkOutcome {
    let mut any_failed = false;
    let mut batches_run = 0usize;
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        eprintln!(
            "claim_executor: batch {} — {} claim(s)",
            bi + 1,
            batch.len()
        );
        let handles: Vec<_> = batch
            .iter()
            .map(|claim| {
                let roots = source_roots.to_vec();
                let claim = claim.clone();
                thread::spawn(move || run_one_claim(roots, claim))
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
    let needle = format!("fn {function}(");
    let start = text
        .find(&needle)
        .ok_or_else(|| format!("{}: missing function {function}", path.display()))?;
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
    let gating = match batches[0].first() {
        Some(c) if !c.entry.is_empty() => c.clone(),
        _ => {
            eprintln!("claim_executor: --perturb-check: batch 1 has no runnable gating claim");
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
    let gating_path = remap_entry_for_temp(primary, &temp_src, &gating.entry);
    if let Err(e) = perturb_function_to_false(&gating_path, &gating.function) {
        let _ = fs::remove_dir_all(&tmp);
        eprintln!("claim_executor: --perturb-check: plant gating->false failed: {e}");
        return Err(ExitCode::from(2));
    }

    // Remap every claim's entry onto the temp tree (pure path rewrite; batch
    // membership and order are unchanged — that is the .dag plan's), then re-walk.
    // Only the gating witness body differs from the green run.
    let temp_root = temp_src.to_string_lossy().into_owned();
    let remapped: Vec<Vec<ClaimRef>> = batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .map(|c| ClaimRef {
                    entry: if c.entry.is_empty() {
                        c.entry.clone()
                    } else {
                        remap_entry_for_temp(primary, &temp_src, &c.entry)
                            .to_string_lossy()
                            .into_owned()
                    },
                    function: c.function.clone(),
                })
                .collect()
        })
        .collect();

    eprintln!(
        "claim_executor: --perturb-check: planted batch-1 gating witness `{}` -> false; re-walking",
        gating.function
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
        "claim_executor: executor plan = {} batch(es) from {}::{}",
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
