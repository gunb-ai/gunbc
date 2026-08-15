//! EvalCallMemo A/B receipt for `parse_json` (parse-only — no `json_unescape` isolation).
//!
//! Operator redirect (eager-koi-458, session sunny-lark-465): run this A/B **before** any
//! `json_unescape_from` source edit. Same binary, tree, fixture, host, cap, harness; vary only
//! `GUNBC_EVAL_MEMO` (`1` default vs `0` diagnostic recompute).
//!
//! **Falsifiable prediction (state before looking):** if `EvalCallMemo` retention drives peak
//! RSS, the memo-ON arm shows **near-zero `memo_hits` with large `memo_misses`** (recursive parse
//! keys are almost unique). A **high hit rate refutes** the hypothesis — report plainly, do not
//! reinterpret. If `memo_overflow` is 0, the 1M **entry** cap never bound the run; cost is byte-
//! denominated while admission is entry-denominated (finding in itself).
//!
//! **VmHWM guard:** `peak_rss_vhwm_bytes()` is process-wide and monotone. Setup (fixture build,
//! module resolve, ctx construction) can peak above the parse; comparing only end-of-process peak
//! or `rss_after - rss_before` around parse would false-negative into "memo-off does not help".
//! The receipt samples VmHWM at fixture-built, parse-entry (post-setup), and parse-exit, and
//! reports `parse_phase_vhwm_increase_bytes` for cross-arm comparison. If setup dominates,
//! `setup_dominates_discriminator=true` — shrink setup or do not trust peak-RSS A/B.
//!
//! Run deliberately (srv1 capped): `docs/probes/json_parse_eval_memo_ab_probe.sh`

use std::path::PathBuf;
use std::time::Instant;

use v1_compiler::cli_run::{self, make_eval_context, peak_rss_vhwm_bytes};
use v1_compiler::v1_interpreter::{self, eval_call_memo_counters, ExecutionMode, Value};

/// Parse-phase increase must exceed this fraction of process peak to trust RSS A/B.
const PARSE_PHASE_MIN_SHARE_OF_PEAK: f64 = 0.10;

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

fn print_rss_line(label: &str, rss: Option<u64>) {
    match rss {
        Some(v) => println!("  {label}={v}"),
        None => println!("  {label}=unavailable"),
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

    let rss_process_start = peak_rss_vhwm_bytes();

    let doc = json_document_literal(decoded_len);
    let rss_after_fixture = peak_rss_vhwm_bytes();

    let ctx = json_parse_ctx();
    let rss_at_parse_entry = peak_rss_vhwm_bytes();

    let t0 = Instant::now();
    let parse_ok = call_parse_json(&ctx, &doc);
    let wall = t0.elapsed();

    let (memo_hits, memo_misses, memo_overflow) = eval_call_memo_counters(&ctx);
    let rss_at_parse_exit = peak_rss_vhwm_bytes();

    assert!(
        parse_ok,
        "parse_json must accept synthetic literal document"
    );

    let parse_phase_increase = match (rss_at_parse_entry, rss_at_parse_exit) {
        (Some(entry), Some(exit)) => Some(exit.saturating_sub(entry)),
        _ => None,
    };
    let setup_dominates = match (parse_phase_increase, rss_at_parse_exit) {
        (Some(inc), Some(peak)) if peak > 0 => {
            (inc as f64) < (peak as f64) * PARSE_PHASE_MIN_SHARE_OF_PEAK
        }
        _ => true,
    };
    let hit_rate = if memo_hits + memo_misses > 0 {
        Some(memo_hits as f64 / (memo_hits + memo_misses) as f64)
    } else {
        None
    };
    let hypothesis_refuted = hit_rate.is_some_and(|r| r > 0.05);

    println!("eval_memo_ab_receipt:");
    println!("  prediction=memo_on_near_zero_hits_large_misses; high_hit_rate_refutes_hypothesis");
    println!("  GUNBC_EVAL_MEMO={memo_env}");
    println!("  eval_call_memo_enabled={memo_enabled}");
    println!("  decoded_len={decoded_len}");
    println!("  input_bytes={}", doc.len());
    print_rss_line("rss_process_start_bytes", rss_process_start);
    print_rss_line("rss_after_fixture_bytes", rss_after_fixture);
    print_rss_line("rss_at_parse_entry_bytes", rss_at_parse_entry);
    print_rss_line("rss_at_parse_exit_bytes", rss_at_parse_exit);
    if let Some(inc) = parse_phase_increase {
        println!("  parse_phase_vhwm_increase_bytes={inc}");
    } else {
        println!("  parse_phase_vhwm_increase_bytes=unavailable");
    }
    println!(
        "  setup_dominates_discriminator={setup_dominates} (parse_phase_share_min={PARSE_PHASE_MIN_SHARE_OF_PEAK})"
    );
    println!("  wall={wall:?}");
    println!("  memo_hits={memo_hits}");
    println!("  memo_misses={memo_misses}");
    println!("  memo_overflow={memo_overflow}");
    if memo_overflow == 0 {
        println!("  memo_overflow_note=entry_cap_1M_never_bound_run_cost_is_byte_denominated");
    }
    if let Some(r) = hit_rate {
        println!("  memo_hit_rate={r:.6}");
    }
    println!("  hypothesis_refuted_by_hit_rate={hypothesis_refuted}");
    println!("  in_process_termination=CompletedExit0");
}
