//! Git pre-push hook — thin transport over `gunbc.githooks_pre_push_plan`.
//!
//! Policy (which gates fire, trigger globs, entry/fn recipes) lives in the typed plan
//! projected from `commit_gate_roster`. This module: stdin parse, git host effects,
//! plan evaluation, step dispatch.

use std::collections::BTreeSet;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use super::{make_eval_context, resolve_entry_graph_shared, witness_layer_roots};
use crate::v1_interpreter::{self, ExecutionMode, Value};

const PLAN_ENTRY: &str = "dag/gunbc/githooks_pre_push_plan.dag";

struct PrePushStdinRow {
    local_ref: String,
    local_sha: String,
    remote_sha: String,
}

enum ActiveGate {
    DocWitness {
        entry: String,
        function: String,
    },
    CargoFmt {
        fail_recipe: String,
    },
    WitnessCorpus {
        fail_recipe: String,
    },
}

struct PlanCtx {
    eval_ctx: v1_interpreter::InterpContext,
}

pub fn run() -> ExitCode {
    match run_inner() {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("[pre-push] {msg}");
            ExitCode::from(1)
        }
    }
}

fn run_inner() -> Result<ExitCode, String> {
    let root = git_toplevel()?;
    std::env::set_current_dir(&root).map_err(|e| format!("cd to repo root: {e}"))?;

    let plan = load_plan_ctx()?;
    let zero_sha = load_zero_sha(&plan)?;

    let stdin_rows = read_pre_push_stdin()?;
    if !pushes_content(&stdin_rows, &zero_sha) {
        return Ok(ExitCode::SUCCESS);
    }

    let changed = collect_push_changed(&stdin_rows, &zero_sha)?;
    let head_ref = git_head_ref()?;
    let head_branch_in_push = head_branch_in_push(&stdin_rows, &head_ref, &zero_sha);

    if head_branch_in_push && working_tree_dirty()? {
        eprintln!("[pre-push] uncommitted changes — commit or stash before pushing.");
        return Ok(ExitCode::from(1));
    }

    let claim_batch = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let active = load_active_gates(&plan, &changed)?;

    if active.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    for gate in &active {
        execute_gate(gate, &claim_batch, &plan, &root)?;
    }

    Ok(ExitCode::SUCCESS)
}

fn load_zero_sha(plan: &PlanCtx) -> Result<String, String> {
    match eval_fn(plan, "pre_push_zero_sha_authority")? {
        Value::Str(s) => Ok(s),
        other => Err(format!(
            "pre_push_zero_sha_authority not a String: {other:?}"
        )),
    }
}

fn load_plan_ctx() -> Result<PlanCtx, String> {
    let roots = witness_layer_roots();
    let (graph, indices) =
        resolve_entry_graph_shared(&roots, PLAN_ENTRY).map_err(|e| format!("resolve plan: {e}"))?;
    Ok(PlanCtx {
        eval_ctx: make_eval_context(&graph, indices, ExecutionMode::Hermetic),
    })
}

fn eval_fn(plan: &PlanCtx, function: &str) -> Result<Value, String> {
    v1_interpreter::run_in_context(&plan.eval_ctx, function, false)
        .map_err(|e| format!("{function}: {e}"))
}

fn load_source_roots(plan: &PlanCtx) -> Result<Vec<String>, String> {
    let val = eval_fn(plan, "pre_push_source_roots")?;
    string_list_from_value(&val, "pre_push_source_roots")
}

fn load_active_gates(plan: &PlanCtx, changed: &[String]) -> Result<Vec<ActiveGate>, String> {
    let changed_val = string_list_to_value(changed);
    let args = [(Some("changed_paths".to_string()), changed_val)];
    let result = v1_interpreter::run_in_context_with_args(
        &plan.eval_ctx,
        "pre_push_active_kinds",
        &args,
        false,
    )
    .map_err(|e| format!("pre_push_active_kinds: {e}"))?;
    parse_active_gates(&plan.eval_ctx, &result)
}

fn string_list_to_value(items: &[String]) -> Value {
    Value::List(std::rc::Rc::new(
        items.iter().cloned().map(Value::Str).collect(),
    ))
}

fn string_list_from_value(val: &Value, field: &str) -> Result<Vec<String>, String> {
    match val {
        Value::List(items) => items
            .iter()
            .map(|v| match v {
                Value::Str(s) => Ok(s.clone()),
                other => Err(format!("{field} entry not a String: {other:?}")),
            })
            .collect(),
        other => Err(format!("{field} not a List: {other:?}")),
    }
}

fn parse_active_gates(
    ctx: &v1_interpreter::InterpContext,
    val: &Value,
) -> Result<Vec<ActiveGate>, String> {
    let Value::List(items) = val else {
        return Err(format!("pre_push_active_kinds not a List: {val:?}"));
    };
    items.iter().map(|v| parse_gate_kind(ctx, v)).collect()
}

fn parse_gate_kind(ctx: &v1_interpreter::InterpContext, val: &Value) -> Result<ActiveGate, String> {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = val
    else {
        return Err(format!("gate kind not a Variant: {val:?}"));
    };
    if ctx.sym_eq(*variant_name, "DocWitnessRun") {
        return Ok(ActiveGate::DocWitness {
            entry: field_str(ctx, fields, "entry")?,
            function: field_str(ctx, fields, "function")?,
        });
    }
    if ctx.sym_eq(*variant_name, "CargoFmtCheck") {
        return Ok(ActiveGate::CargoFmt {
            fail_recipe: field_str(ctx, fields, "fail_recipe")?,
        });
    }
    if ctx.sym_eq(*variant_name, "WitnessCorpusRun") {
        return Ok(ActiveGate::WitnessCorpus {
            fail_recipe: field_str(ctx, fields, "fail_recipe")?,
        });
    }
    Err("unknown PrePushGateKind variant".to_string())
}

fn field_str(
    ctx: &v1_interpreter::InterpContext,
    fields: &[(v1_interpreter::Symbol, Value)],
    name: &str,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.clone()),
        other => Err(format!("{name} not a String: {other:?}")),
    }
}

fn execute_gate(
    gate: &ActiveGate,
    claim_batch: &Path,
    plan: &PlanCtx,
    root: &Path,
) -> Result<(), String> {
    match gate {
        ActiveGate::DocWitness { entry, function } => {
            eprintln!("[pre-push] doc reachability: {function}");
            run_claim_batch(claim_batch, plan, entry, function, &["--claim-run"])
        }
        ActiveGate::CargoFmt { fail_recipe } => {
            eprintln!("[pre-push] cargo fmt --all --check");
            let ok = Command::new("cargo")
                .args(["fmt", "--all", "--check"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| format!("cargo fmt --all --check: {e}"))?
                .success();
            if ok {
                Ok(())
            } else {
                eprintln!("[pre-push] {fail_recipe}");
                Err("fmt drift".to_string())
            }
        }
        ActiveGate::WitnessCorpus { fail_recipe } => run_witness_corpus(root, plan, fail_recipe),
    }
}

fn run_claim_batch(
    claim_batch: &Path,
    plan: &PlanCtx,
    entry: &str,
    function: &str,
    suffix_args: &[&str],
) -> Result<(), String> {
    let roots = load_source_roots(plan)?;
    let mut cmd = Command::new(claim_batch);
    for root in roots {
        cmd.arg("--source-root").arg(root);
    }
    cmd.arg("--entry")
        .arg(entry)
        .arg("--function")
        .arg(function);
    for arg in suffix_args {
        cmd.arg(arg);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("claim_batch {function}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("claim_batch {function} failed"))
    }
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
        && std::fs::metadata(path)
            .map(|meta| {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                }
                #[cfg(not(unix))]
                {
                    let _ = meta;
                    true
                }
            })
            .unwrap_or(false)
}

fn resolve_claim_executor(root: &Path) -> Result<PathBuf, String> {
    let release = root.join("target/release/claim_executor");
    if is_executable(&release) {
        return Ok(release);
    }
    let debug = root.join("target/debug/claim_executor");
    if is_executable(&debug) {
        return Ok(debug);
    }
    eprintln!("[pre-push] claim_executor not built; compiling release (one-time, minutes)...");
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "v1-compiler",
            "--release",
            "--bin",
            "claim_executor",
        ])
        .env("CTRL_BUILD_WRAP_CARGO", "0")
        .current_dir(root)
        .status()
        .map_err(|e| format!("cargo build claim_executor: {e}"))?;
    if !status.success() {
        return Err("cargo build claim_executor failed".to_string());
    }
    let built = root.join("target/release/claim_executor");
    if is_executable(&built) {
        Ok(built)
    } else {
        Err("claim_executor missing after build".to_string())
    }
}

fn run_witness_corpus(root: &Path, plan: &PlanCtx, fail_recipe: &str) -> Result<(), String> {
    let executor = resolve_claim_executor(root)?;
    let entry = match eval_fn(plan, "pre_push_witness_corpus_plan_entry_authority")? {
        Value::Str(s) => s,
        other => {
            return Err(format!(
                "pre_push_witness_corpus_plan_entry_authority not a String: {other:?}"
            ));
        }
    };
    let function = match eval_fn(plan, "pre_push_witness_corpus_plan_fn_authority")? {
        Value::Str(s) => s,
        other => {
            return Err(format!(
                "pre_push_witness_corpus_plan_fn_authority not a String: {other:?}"
            ));
        }
    };
    eprintln!(
        "[pre-push] affected-set witness corpus (claim_executor; scoped to origin/main...HEAD)"
    );
    let roots = load_source_roots(plan)?;
    let mut cmd = Command::new(executor);
    for rel_root in roots {
        cmd.arg("--source-root").arg(root.join(rel_root));
    }
    cmd.arg("--plan-entry")
        .arg(&entry)
        .arg("--plan-function")
        .arg(&function);
    let status = cmd
        .status()
        .map_err(|e| format!("claim_executor {function}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        eprintln!("[pre-push] {fail_recipe}");
        Err("witness corpus failed".to_string())
    }
}

fn git_toplevel() -> Result<std::path::PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("git rev-parse: {e}"))?;
    if !output.status.success() {
        return Err("git rev-parse --show-toplevel failed".to_string());
    }
    Ok(std::path::PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn read_pre_push_stdin() -> Result<Vec<PrePushStdinRow>, String> {
    let stdin = io::stdin();
    let mut rows = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("read pre-push stdin: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let local_ref = parts
            .next()
            .ok_or_else(|| format!("malformed pre-push stdin row: {line:?}"))?
            .to_string();
        let local_sha = parts
            .next()
            .ok_or_else(|| format!("malformed pre-push stdin row: {line:?}"))?
            .to_string();
        let _remote_ref = parts
            .next()
            .ok_or_else(|| format!("malformed pre-push stdin row: {line:?}"))?;
        let remote_sha = parts
            .next()
            .ok_or_else(|| format!("malformed pre-push stdin row: {line:?}"))?
            .to_string();
        rows.push(PrePushStdinRow {
            local_ref,
            local_sha,
            remote_sha,
        });
    }
    Ok(rows)
}

fn pushes_content(rows: &[PrePushStdinRow], zero_sha: &str) -> bool {
    rows.iter().any(|row| row.local_sha != zero_sha)
}

fn head_branch_in_push(rows: &[PrePushStdinRow], head_ref: &str, zero_sha: &str) -> bool {
    if head_ref.is_empty() {
        return false;
    }
    // Parity with the old emitted hook's stdin loop: `[[ "$local_sha" == "$ZERO_SHA" ]] && continue`
    // runs before the `local_ref == HEAD_REF` test, so branch-delete rows never set the flag.
    rows.iter()
        .any(|row| row.local_sha != zero_sha && row.local_ref == head_ref)
}

fn git_head_ref() -> Result<String, String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "HEAD"])
        .output()
        .map_err(|e| format!("git symbolic-ref: {e}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    Ok(String::new())
}

fn working_tree_dirty() -> Result<bool, String> {
    let diff = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map_err(|e| format!("git diff --quiet: {e}"))?;
    if !diff.success() {
        return Ok(true);
    }
    let cached = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(|e| format!("git diff --cached --quiet: {e}"))?;
    Ok(!cached.success())
}

fn empty_tree_hash() -> Result<String, String> {
    let output = Command::new("git")
        .args(["hash-object", "-t", "tree", "/dev/null"])
        .output()
        .map_err(|e| format!("git hash-object: {e}"))?;
    if !output.status.success() {
        return Err("git hash-object -t tree /dev/null failed".to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn collect_push_changed(rows: &[PrePushStdinRow], zero_sha: &str) -> Result<Vec<String>, String> {
    let empty_tree = empty_tree_hash()?;
    let mut changed = BTreeSet::new();
    for row in rows {
        if row.local_sha == zero_sha {
            continue;
        }
        let base = if row.remote_sha == zero_sha {
            empty_tree.as_str()
        } else {
            row.remote_sha.as_str()
        };
        let paths = git_diff_name_only(base, &row.local_sha)?;
        changed.extend(paths);
    }
    Ok(changed.into_iter().collect())
}

fn git_diff_name_only(base: &str, head: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=ACMR", base, head])
        .output()
        .map_err(|e| format!("git diff --name-only: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git diff --name-only --diff-filter=ACMR {base} {head} failed"
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}
