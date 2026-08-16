#![allow(clippy::disallowed_macros)]

//! STR-RC-0 acceptance probe: measure wall-clock scaling of `parse_json` over
//! synthetic object documents whose member count (and byte length) grow linearly.
//! Resolve happens once outside the timed loop; each row is one hermetic
//! `parse_json` call through the v1 interpreter on the JSON parse entry closure.

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, workspace_root};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

fn str_value(s: impl AsRef<str>) -> Value {
    Value::Str(std::rc::Rc::from(s.as_ref()))
}

const ENTRY: &str = "dag/extdeps/languages/json/parse.dag";
const ITERS_PER_SIZE: usize = 500;

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

fn parse_json_ok(ctx: &v1_interpreter::InterpContext, json: &str) -> bool {
    let args = [(Some("s".to_string()), str_value(json))];
    match v1_interpreter::run_in_context_with_args(ctx, "parse_json", &args, false) {
        Ok(Value::Variant {
            variant_name,
            fields,
            ..
        }) => {
            let variant = ctx.resolve(variant_name);
            variant == "Present" && fields.iter().any(|(sym, _)| ctx.resolve(*sym) == "value")
        }
        _ => false,
    }
}

fn run() -> Result<(), String> {
    let ws = workspace_root();
    let roots = vec![ws.join("dag").to_string_lossy().into_owned()];

    eprintln!("json_parse_scaling_probe: resolving {ENTRY} ...");
    let resolve_start = Instant::now();
    let (graph, indices) = resolve_entry_graph(&roots, ENTRY)?;
    eprintln!(
        "json_parse_scaling_probe: resolve_ms={}",
        resolve_start.elapsed().as_millis()
    );

    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);

    let member_counts = [50usize, 100, 200, 400, 800, 1600, 3200];
    println!("member_count\tbytes\telapsed_ms_per_call\tok");

    for &n in &member_counts {
        let json = make_json_object(n);
        let args = [(Some("s".to_string()), str_value(&json))];

        // One warmup call per size (not printed).
        let _ = v1_interpreter::run_in_context_with_args(&ctx, "parse_json", &args, false);

        let start = Instant::now();
        let mut ok = true;
        for _ in 0..ITERS_PER_SIZE {
            if !parse_json_ok(&ctx, &json) {
                ok = false;
                break;
            }
        }
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0 / ITERS_PER_SIZE as f64;
        println!("{n}\t{}\t{elapsed_ms:.4}\t{ok}", json.len());
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
