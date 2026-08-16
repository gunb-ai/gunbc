#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / STR-RC-0) — host transport for
//! `parse_json` scaling and large-document survival measurements on the v1 interpreter.
//!
//! NOT floor-enrolled — run standalone via `cargo run --release -p v1-compiler --bin
//! json_parse_scaling_probe`. Modes (`JSON_PARSE_PROBE_MODE`):
//! - `scaling` (default): sub-40KB member grid (fixed-cost dominated; not discriminating).
//! - `large`: 100KB / 200KB / 507KB targets — the regime where pre-change `Value::Str(String)`
//!   binding was reported to OOM.
//!
//! CHECKABLE RECEIPT: large-regime row records `ok` vs process death (OOM) for each target;
//! acceptance is categorical (post survives sizes pre could not), not sub-40KB exponent fit.
//!
//! DISSOLUTION: delete this bin when STR-RC-0 acceptance is floor-enrolled with a modeled
//! witness, or when large-regime measurement refutes the hypothesis and the branch reverts.
//! Receipt: `rg JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER src/v1/stage0` until deletion.

/// Grep receipt for scaffold dissolution (`rg JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER`).
pub const JSON_PARSE_SCALING_PROBE_SCAFFOLD_MARKER: &str =
    "STR-RC-0 json_parse_scaling_probe measurement transport (not floor-enrolled)";

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, workspace_root};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn str_value(s: impl AsRef<str>) -> Value {
    Value::Str(std::rc::Rc::from(s.as_ref()))
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

#[derive(Debug)]
enum ParseOutcome {
    Ok { elapsed_ms: f64 },
    Refused,
}

fn parse_json_once(ctx: &v1_interpreter::InterpContext, json: &str) -> ParseOutcome {
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
            if variant == "Present" && fields.iter().any(|(sym, _)| ctx.resolve(*sym) == "value") {
                ParseOutcome::Ok { elapsed_ms }
            } else {
                ParseOutcome::Refused
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

fn run_scaling(ctx: &v1_interpreter::InterpContext) {
    const ITERS_PER_SIZE: usize = 500;
    let member_counts = [50usize, 100, 200, 400, 800, 1600, 3200];
    println!("mode\tscaling");
    println!("member_count\tbytes\telapsed_ms_per_call\tok");

    for &n in &member_counts {
        let json = make_json_object(n);
        let args = [(Some("s".to_string()), str_value(&json))];
        let _ = v1_interpreter::run_in_context_with_args(ctx, "parse_json", &args, false);

        let start = Instant::now();
        let mut ok = true;
        for _ in 0..ITERS_PER_SIZE {
            if !matches!(parse_json_once(ctx, &json), ParseOutcome::Ok { .. }) {
                ok = false;
                break;
            }
        }
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0 / ITERS_PER_SIZE as f64;
        println!("{n}\t{}\t{elapsed_ms:.4}\t{ok}", json.len());
    }
}

fn run_large(ctx: &v1_interpreter::InterpContext) {
    const TARGETS: [usize; 3] = [100_000, 200_000, 507_000];
    const ITERS: usize = 3;
    println!("mode\tlarge");
    println!("target_bytes\tmember_count\tactual_bytes\toutcome\telapsed_ms");

    for &target in &TARGETS {
        let (member_count, json) = json_object_at_least_bytes(target);
        eprintln!(
            "json_parse_scaling_probe: large target={target} members={member_count} bytes={}",
            json.len()
        );

        // Warmup once.
        let _ = parse_json_once(ctx, &json);

        let mut last_ok_ms = None;
        let mut refused = false;
        for _ in 0..ITERS {
            match parse_json_once(ctx, &json) {
                ParseOutcome::Ok { elapsed_ms } => last_ok_ms = Some(elapsed_ms),
                ParseOutcome::Refused => {
                    refused = true;
                    break;
                }
            }
        }

        let (outcome, elapsed_ms) = if refused {
            ("refused", 0.0)
        } else {
            ("ok", last_ok_ms.unwrap_or(0.0))
        };
        println!(
            "{target}\t{member_count}\t{}\t{outcome}\t{elapsed_ms:.2}",
            json.len()
        );
    }
}

fn run() -> Result<(), String> {
    let ctx = resolve_ctx()?;
    match std::env::var("JSON_PARSE_PROBE_MODE")
        .unwrap_or_else(|_| "scaling".to_string())
        .as_str()
    {
        "large" => run_large(&ctx),
        "scaling" | "" => run_scaling(&ctx),
        other => return Err(format!("unknown JSON_PARSE_PROBE_MODE={other:?}")),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("json_parse_scaling_probe: {err}");
            ExitCode::FAILURE
        }
    }
}
