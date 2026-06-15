//! Batch claim executor: the host transport for the v4.workflow.executor .dag.
//!
//! The `.dag` is the batching AUTHORITY. A plan function (default
//! `bre_claim_batches` in `src/v4/test/claim/workflow/batch_runner.dag`) folds
//! the dependency frontier through `v4.workflow.executor` and returns
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

use v2_compiler::cli_run::{
    make_eval_context, resolve_entry_graph, run_claim, run_value, ClaimOutcome,
};
use v2_compiler::v2_interpreter::{InterpContext, Value};

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
    let ctx = make_eval_context(&graph, source_indices);
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

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut plan_entry: Option<String> = None;
    let mut plan_function = "bre_claim_batches".to_string();

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

    // 1. Evaluate the executor-decided plan.
    let (plan_graph, plan_indices) = match resolve_entry_graph(&source_roots, &plan_entry) {
        Ok(pair) => pair,
        Err(msg) => {
            eprintln!(
                "claim_executor: resolve failed for plan {}:\n{}",
                plan_entry, msg
            );
            return Err(ExitCode::from(1));
        }
    };
    let plan_ctx = make_eval_context(&plan_graph, plan_indices);
    let plan_value = match run_value(&plan_ctx, &plan_function) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!(
                "claim_executor: plan eval failed ({}::{}): {}",
                plan_entry, plan_function, msg
            );
            return Err(ExitCode::from(1));
        }
    };
    // Drop the plan graph before running claims (keeps no `!Send` state alive).
    drop(plan_graph);

    let batches = match batches_from_plan(&plan_value, &plan_ctx) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: malformed plan value: {}", msg);
            return Err(ExitCode::from(1));
        }
    };
    drop(plan_value);

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

    // 2. Run batch by batch (executor ordering); claims within a batch in
    //    parallel. The batch boundary is a barrier: batch N+1 starts only after
    //    every claim in batch N has reported.
    let mut any_failed = false;
    for (bi, batch) in batches.iter().enumerate() {
        eprintln!(
            "claim_executor: batch {} — {} claim(s)",
            bi + 1,
            batch.len()
        );
        let handles: Vec<_> = batch
            .iter()
            .map(|claim| {
                let roots = source_roots.clone();
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

    if any_failed {
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
