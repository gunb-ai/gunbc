#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / CHARAT-0) — host transport for
//! `parse_json` survival and mechanism measurements on the v1 interpreter.
//!
//! NOT floor-enrolled — run standalone via `cargo run --release -p v1-compiler --bin
//! json_parse_scaling_probe`. Modes (`JSON_PARSE_PROBE_MODE`):
//! - `survival`: one `JSON_PARSE_TARGET_BYTES`, exactly one `parse_json` call (fresh process).
//! - `memo_receipt`: one target; cold parse + first repeat + average of subsequent memo hits.
//! - `scaling` / `large`: legacy grids (memo-contaminated; not used for acceptance receipts).
//!
//! CHECKABLE RECEIPT: survival mode records Present parse + member count vs process death.
//!
//! DISSOLUTION: delete this bin when CHARAT-0 acceptance is floor-enrolled with a modeled
//! witness, or when large-regime measurement refutes the hypothesis and the branch reverts.
//! Receipt: `rg JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER src/v1/stage0` until deletion.

/// Grep receipt for scaffold dissolution (`rg JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER`).
pub const JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER: &str =
    "CHARAT-0 json_parse_scaling_probe measurement transport (not floor-enrolled)";

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, workspace_root};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn str_value(s: impl AsRef<str>) -> Value {
    Value::Str(s.as_ref().to_string())
}

const ENTRY: &str = "dag/extdeps/languages/json/parse.dag";

fn make_json_object(member_count: usize) -> String {
    let mut s = String::from("{");
    for i in 0..member_count {
        if i > 0 {
            s.push(',');
        }
        use std::fmt::Write;
        write!(&mut s, "\"k{i}\":{i}").expect("fmt");
    }
    s.push('}');
    s
}

/// Grow member count until the synthetic object is at least `target_bytes`.
fn json_object_at_least_bytes(target_bytes: usize) -> (usize, String) {
    let mut n = 1usize;
    loop {
        let json = make_json_object(n);
        if json.len() >= target_bytes {
            return (n, json);
        }
        n = n.saturating_mul(2).max(n + 1);
        if n > 10_000_000 {
            panic!(
                "could not reach {target_bytes} bytes (stuck at {})",
                json.len()
            );
        }
    }
}

fn target_bytes_from_env() -> Result<usize, String> {
    std::env::var("JSON_PARSE_TARGET_BYTES")
        .map_err(|_| "JSON_PARSE_TARGET_BYTES is required".to_string())
        .and_then(|s| {
            s.parse()
                .map_err(|_| format!("invalid JSON_PARSE_TARGET_BYTES={s:?}"))
        })
}

#[derive(Debug)]
enum ParseOutcome {
    Parsed {
        elapsed_ms: f64,
        members_found: usize,
    },
    MemberMismatch {
        members_found: usize,
        expected: usize,
    },
    Refused,
}

fn field_by_name<'a>(
    ctx: &v1_interpreter::InterpContext,
    fields: &'a [(v1_interpreter::Symbol, Value)],
    name: &str,
) -> Option<&'a Value> {
    let key = ctx.sym(name);
    fields.iter().find(|(sym, _)| *sym == key).map(|(_, v)| v)
}

fn json_object_member_count(ctx: &v1_interpreter::InterpContext, val: &Value) -> Option<usize> {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = val
    else {
        return None;
    };
    if ctx.resolve(*variant_name) != "JsonObject" {
        return None;
    }
    match field_by_name(ctx, fields.as_ref(), "members") {
        Some(Value::List(items)) => Some(items.len()),
        _ => None,
    }
}

fn parse_json_once(
    ctx: &v1_interpreter::InterpContext,
    json: &str,
    expected_members: usize,
) -> ParseOutcome {
    let args = [(Some("s".to_string()), str_value(json))];
    let start = Instant::now();
    let result = v1_interpreter::run_in_context_with_args(ctx, "parse_json", &args, false);
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(Value::Variant {
            variant_name,
            fields,
            ..
        }) => {
            let variant = ctx.resolve(variant_name);
            if variant != "Present" {
                return ParseOutcome::Refused;
            }
            let Some(value) = field_by_name(ctx, fields.as_ref(), "value") else {
                return ParseOutcome::Refused;
            };
            match json_object_member_count(ctx, value) {
                Some(found) if found == expected_members => ParseOutcome::Parsed {
                    elapsed_ms,
                    members_found: found,
                },
                Some(found) => ParseOutcome::MemberMismatch {
                    members_found: found,
                    expected: expected_members,
                },
                None => ParseOutcome::Refused,
            }
        }
        _ => ParseOutcome::Refused,
    }
}

fn resolve_ctx() -> Result<v1_interpreter::InterpContext, String> {
    let ws = workspace_root();
    let roots = vec![ws.join("dag").to_string_lossy().into_owned()];
    eprintln!("json_parse_scaling_probe: resolving {ENTRY} ...");
    let resolve_start = Instant::now();
    let (graph, indices) = resolve_entry_graph(&roots, ENTRY)?;
    eprintln!(
        "json_parse_scaling_probe: resolve_ms={}",
        resolve_start.elapsed().as_millis()
    );
    Ok(make_eval_context(&graph, indices, ExecutionMode::Hermetic))
}

fn eval_memo_label() -> String {
    std::env::var("GUNBC_EVAL_MEMO").unwrap_or_else(|_| "default".to_string())
}

fn outcome_label(outcome: &ParseOutcome) -> &'static str {
    match outcome {
        ParseOutcome::Parsed { .. } => "parsed",
        ParseOutcome::MemberMismatch { .. } => "member_mismatch",
        ParseOutcome::Refused => "refused",
    }
}

fn print_survival_row(
    target: usize,
    member_count: usize,
    actual_bytes: usize,
    outcome: &ParseOutcome,
) {
    match outcome {
        ParseOutcome::Parsed {
            elapsed_ms,
            members_found,
        } => println!(
            "{target}\t{member_count}\t{actual_bytes}\tparsed\t{members_found}\t{elapsed_ms:.2}"
        ),
        ParseOutcome::MemberMismatch {
            members_found,
            expected,
        } => println!(
            "{target}\t{member_count}\t{actual_bytes}\tmember_mismatch\t{members_found}/expected={expected}\t0.00"
        ),
        ParseOutcome::Refused => {
            println!("{target}\t{member_count}\t{actual_bytes}\trefused\t0\t0.00")
        }
    }
}

fn survival_succeeded(outcome: &ParseOutcome) -> bool {
    matches!(outcome, ParseOutcome::Parsed { .. })
}

/// One target, one parse — intended for a fresh process per invocation.
fn run_survival(ctx: &v1_interpreter::InterpContext) -> Result<ParseOutcome, String> {
    let target = target_bytes_from_env()?;
    let (member_count, json) = json_object_at_least_bytes(target);
    eprintln!(
        "json_parse_scaling_probe: survival target={target} members={member_count} bytes={}",
        json.len()
    );
    let memo = eval_memo_label();
    println!("mode\tsurvival\teval_memo={memo}");
    println!("target_bytes\tmember_count\tactual_bytes\toutcome\tmembers_found\telapsed_ms");
    let outcome = parse_json_once(ctx, &json, member_count);
    print_survival_row(target, member_count, json.len(), &outcome);
    Ok(outcome)
}

/// Cold parse + first repeat + average of subsequent hits (memo diagnostic only).
fn run_memo_receipt(ctx: &v1_interpreter::InterpContext) -> Result<(), String> {
    const SUBSEQUENT_HITS: usize = 5;
    let target = target_bytes_from_env()?;
    let (member_count, json) = json_object_at_least_bytes(target);
    let memo = eval_memo_label();
    println!("mode\tmemo_receipt\teval_memo={memo}");
    println!("target_bytes\tmember_count\tactual_bytes\tcold_ms\tfirst_repeat_ms\tavg_subsequent_hit_ms\toutcome");

    let cold = parse_json_once(ctx, &json, member_count);
    if !survival_succeeded(&cold) {
        println!(
            "{target}\t{member_count}\t{}\t0.00\t0.00\t0.00\t{}",
            json.len(),
            outcome_label(&cold)
        );
        return Ok(());
    }
    let cold_ms = match &cold {
        ParseOutcome::Parsed { elapsed_ms, .. } => *elapsed_ms,
        _ => 0.0,
    };

    let first_repeat = parse_json_once(ctx, &json, member_count);
    let first_repeat_ms = match &first_repeat {
        ParseOutcome::Parsed { elapsed_ms, .. } => *elapsed_ms,
        _ => 0.0,
    };

    let start = Instant::now();
    let mut hits = 0usize;
    for _ in 0..SUBSEQUENT_HITS {
        if !survival_succeeded(&parse_json_once(ctx, &json, member_count)) {
            break;
        }
        hits += 1;
    }
    let avg_subsequent_ms = if hits > 0 {
        start.elapsed().as_secs_f64() * 1000.0 / hits as f64
    } else {
        0.0
    };

    println!(
        "{target}\t{member_count}\t{}\t{cold_ms:.2}\t{first_repeat_ms:.2}\t{avg_subsequent_ms:.2}\tparsed",
        json.len()
    );
    Ok(())
}

fn run_scaling(ctx: &v1_interpreter::InterpContext) {
    const ITERS_PER_SIZE: usize = 500;
    let member_counts = [50usize, 100, 200, 400, 800, 1600, 3200];
    let memo = eval_memo_label();
    println!("mode\tscaling\teval_memo={memo}");
    println!("member_count\tbytes\tfirst_call_ms\tavg_ms_per_call\toutcome\tmembers_found");

    for &n in &member_counts {
        let json = make_json_object(n);

        let first = parse_json_once(ctx, &json, n);
        let (first_ms, members_found, _ok) = match &first {
            ParseOutcome::Parsed {
                elapsed_ms,
                members_found,
            } => (*elapsed_ms, *members_found, true),
            ParseOutcome::MemberMismatch { members_found, .. } => (0.0, *members_found, false),
            ParseOutcome::Refused => (0.0, 0, false),
        };

        let start = Instant::now();
        for _ in 1..ITERS_PER_SIZE {
            if !matches!(parse_json_once(ctx, &json, n), ParseOutcome::Parsed { .. }) {
                break;
            }
        }
        let avg_ms = if ITERS_PER_SIZE > 1 {
            start.elapsed().as_secs_f64() * 1000.0 / (ITERS_PER_SIZE - 1) as f64
        } else {
            first_ms
        };

        println!(
            "{n}\t{}\t{first_ms:.4}\t{avg_ms:.4}\t{}\t{members_found}",
            json.len(),
            outcome_label(&first)
        );
    }
}

fn run_large(ctx: &v1_interpreter::InterpContext) {
    const DEFAULT_TARGETS: [usize; 3] = [100_000, 200_000, 507_000];
    let targets: Vec<usize> = std::env::var("JSON_PARSE_PROBE_TARGETS")
        .ok()
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .filter(|v: &Vec<usize>| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_TARGETS.to_vec());

    let memo = eval_memo_label();
    println!("mode\tlarge\teval_memo={memo}");
    println!("target_bytes\tmember_count\tactual_bytes\toutcome\tmembers_found\tfirst_call_ms");

    for &target in &targets {
        let (member_count, json) = json_object_at_least_bytes(target);
        eprintln!(
            "json_parse_scaling_probe: large target={target} members={member_count} bytes={}",
            json.len()
        );

        let outcome = parse_json_once(ctx, &json, member_count);
        print_survival_row(target, member_count, json.len(), &outcome);
    }
}

fn run() -> Result<bool, String> {
    eprintln!(
        "json_parse_scaling_probe: GUNBC_EVAL_MEMO={}",
        eval_memo_label()
    );
    let ctx = resolve_ctx()?;
    let mode = std::env::var("JSON_PARSE_PROBE_MODE").unwrap_or_else(|_| "scaling".to_string());
    let success = match mode.as_str() {
        "survival" => survival_succeeded(&run_survival(&ctx)?),
        "memo_receipt" => {
            run_memo_receipt(&ctx)?;
            true
        }
        "large" => {
            run_large(&ctx);
            true
        }
        "scaling" | "" => {
            run_scaling(&ctx);
            true
        }
        other => return Err(format!("unknown JSON_PARSE_PROBE_MODE={other:?}")),
    };
    Ok(success)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(err) => {
            eprintln!("json_parse_scaling_probe: {err}");
            ExitCode::FAILURE
        }
    }
}
