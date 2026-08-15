//! EvalCallMemo A/B receipt for `parse_json` (parse-only — no `json_unescape` isolation).
//!
//! Operator redirect (eager-koi-458, session sunny-lark-465): run this A/B **before** any
//! `json_unescape_from` source edit. Same binary, tree, fixture, host, cap, harness; vary only
//! `GUNBC_EVAL_MEMO` (`1` default vs `0` diagnostic recompute).
//!
//! Hypothesis under test: `EvalCallMemo` retains full argument+result `Value`s per miss; a
//! recursive parse threads `(full_source, changing_index, growing_accumulator)` so keys are
//! almost unique — near-zero hit rate with large miss retention inflates peak RSS.
//!
//! Receipt fields: peak RSS, `memo_hits` / `memo_misses` / `memo_overflow`, wall time,
//! typed in-process termination (`Completed`). Out-of-process termination (OOM SIGKILL=137,
//! abort 134, command-not-found 127) is recorded by `docs/probes/json_parse_eval_memo_ab_probe.sh`.
//!
//! Run deliberately:
//!   GUNBC_EVAL_MEMO=1 cargo test -p v1-compiler --release --test json_parse_eval_memo_ab_receipt -- --ignored --nocapture
//!   GUNBC_EVAL_MEMO=0 cargo test -p v1-compiler --release --test json_parse_eval_memo_ab_receipt -- --ignored --nocapture

use std::path::PathBuf;
use std::time::Instant;

use v1_compiler::cli_run::{self, make_eval_context, peak_rss_vhwm_bytes};
use v1_compiler::v1_interpreter::{self, eval_call_memo_counters, ExecutionMode, Value};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

fn json_parse_ctx() -> v1_interpreter::InterpContext {
    let _ = std::env::set_current_dir(workspace_root());
    let entry = "dag/extdeps/languages/json/parse.dag".to_string();
    let index = cli_run::build_multi_entry_index(&source_roots());
    let (graph, source_indices) =
        cli_run::resolve_entry_with_index(&index, &entry).expect("resolve parse.dag");
    make_eval_context(&graph, source_indices, ExecutionMode::Hermetic)
}

fn json_document_literal(decoded_len: usize) -> String {
    let mut out = String::with_capacity(decoded_len + 16);
    out.push_str("{\"x\":\"");
    out.push_str(&"a".repeat(decoded_len));
    out.push_str("\"}");
    out
}

fn call_parse_json(ctx: &v1_interpreter::InterpContext, text: &str) -> bool {
    let args = [(Some("s".to_string()), Value::Str(text.to_string()))];
    match v1_interpreter::run_in_context_with_args(ctx, "parse_json", &args, false) {
        Ok(Value::Variant { type_name, .. }) => ctx.sym_eq(type_name, "Optional"),
        other => panic!("parse_json returned {other:?}"),
    }
}

fn decoded_len_from_env() -> usize {
    std::env::var("JSON_PARSE_AB_DECODED_LEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50 * 1024)
}

fn eval_memo_env_label() -> String {
    std::env::var("GUNBC_EVAL_MEMO").unwrap_or_else(|_| "1".to_string())
}

fn eval_memo_enabled_label() -> &'static str {
    match eval_memo_env_label().as_str() {
        "0" => "false",
        _ => "true",
    }
}

#[test]
fn json_parse_eval_memo_ab_decode_sanity() {
    let ctx = json_parse_ctx();
    assert!(call_parse_json(&ctx, &json_document_literal(32)));
}

#[test]
#[ignore = "eval-memo A/B probe: run with GUNBC_EVAL_MEMO=0|1 via docs/probes/json_parse_eval_memo_ab_probe.sh"]
fn json_parse_eval_memo_ab_probe() {
    let decoded_len = decoded_len_from_env();
    let memo_env = eval_memo_env_label();
    let memo_enabled = eval_memo_enabled_label();
    let doc = json_document_literal(decoded_len);

    let ctx = json_parse_ctx();
    let rss_before = peak_rss_vhwm_bytes();
    let t0 = Instant::now();
    let parse_ok = call_parse_json(&ctx, &doc);
    let wall = t0.elapsed();
    let (memo_hits, memo_misses, memo_overflow) = eval_call_memo_counters(&ctx);
    let rss_after = peak_rss_vhwm_bytes();
    let peak_rss = rss_after.unwrap_or(0);
    let rss_delta = match (rss_before, rss_after) {
        (Some(b), Some(a)) => Some(a as i64 - b as i64),
        _ => None,
    };

    assert!(
        parse_ok,
        "parse_json must accept synthetic literal document"
    );

    println!("eval_memo_ab_receipt:");
    println!("  GUNBC_EVAL_MEMO={memo_env}");
    println!("  eval_call_memo_enabled={memo_enabled}");
    println!("  decoded_len={decoded_len}");
    println!("  input_bytes={}", doc.len());
    println!("  wall={wall:?}");
    println!("  peak_rss_bytes={peak_rss}");
    if let Some(delta) = rss_delta {
        println!("  rss_delta_bytes={delta:+}");
    } else {
        println!("  rss_delta_bytes=unavailable");
    }
    println!("  memo_hits={memo_hits}");
    println!("  memo_misses={memo_misses}");
    println!("  memo_overflow={memo_overflow}");
    println!("  in_process_termination=CompletedExit0");
}
