//! Receipt for `extdeps.languages.json.parse` `json_unescape_from` cost shape.
//!
//! **Measure before fix** (operator brief, session sunny-lark-465). The candidate is the
//! recursive unescaper in `dag/extdeps/languages/json/parse.dag`:
//!
//! ```dag
//! fn json_unescape_from(s: String, i: Int, acc: String) -> String {
//!   ...
//!   json_unescape_from(s: s, i: i + 1, acc: concat(acc, c))
//! }
//! ```
//!
//! Two quadratic terms are visible once `Value::Str` is known to be an owned `String`
//! (full buffer copy on every `clone()` at the interpreter binding site):
//!
//! - **Parameter binding**: each of the ~n recursive calls re-clones the full `s` buffer.
//!   Modeled copy volume: n × n = n² bytes.
//! - **Accumulator concat**: `concat(acc, c)` copies the growing prefix each step.
//!   Modeled copy volume: 0 + 1 + … + (n−1) = n(n−1)/2 ≈ n²/2 bytes.
//!
//! Measurements (all synthetic — do not use the Codex schema):
//!
//! - **A — length sweep, zero escapes**: one long literal string inside a JSON document at
//!   10 KiB, 50 KiB, and 200 KiB decoded lengths. Reports wall time and RSS delta for both
//!   `json_unescape` (candidate isolated) and `parse_json` (full document path).
//! - **B — escape-density control**: fixed 50 KiB decoded length; 0% vs 100% `\n` escapes.
//! - **C — term separation**: modeled byte-copy accounting plus a linear Rust shadow walk.
//!
//! **Executed receipt (BuildBuddy remote, release, 2026-08-15):**
//!
//! - `json_unescape_modeled_copy_terms_scale_quadratically`: PASS — doubling n quadruples
//!   modeled total copy bytes (~4.0×); parameter-binding term is ~2× the concat term.
//! - Length sweep 10 KiB: `json_unescape` wall reported 7.721µs on BuildBuddy remote
//!   (suspiciously fast — re-verify on srv1; output length verified in-test). Peak RSS
//!   ~1.6 GiB is dominated by one-time `parse.dag` resolve, not the timed call alone.
//! - Length sweep 50 KiB and escape-density 50 KiB: **SIGKILL (OOM)** on BuildBuddy remote
//!   before completing the timed section — consistent with superlinear allocation, not a
//!   declared MemoryMax cap (that host path had no scope limit).
//!
//! srv1 authoritative RSS runs: `docs/probes/json_unescape_cost_probe.sh`.
//!
//! Wall-clock and RSS benchmarks are **`#[ignore]`d — not gates** (tokenize_escape_receipt
//! precedent, review 45416). Run deliberately:
//!
//!   cargo test -p v1-compiler --release --test json_unescape_cost_receipt -- --ignored --nocapture

use std::path::PathBuf;
use std::time::{Duration, Instant};

use v1_compiler::cli_run::{self, make_eval_context, peak_rss_vhwm_bytes};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

const LENGTH_SWEEP_DECODED_BYTES: [usize; 3] = [10 * 1024, 50 * 1024, 200 * 1024];
const DENSITY_CONTROL_DECODED_BYTES: usize = 50 * 1024;

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

fn call_json_unescape(ctx: &v1_interpreter::InterpContext, input: &str) -> String {
    let args = [(Some("s".to_string()), Value::Str(input.to_string()))];
    match v1_interpreter::run_in_context_with_args(ctx, "json_unescape", &args, false) {
        Ok(Value::Str(out)) => out,
        other => panic!("json_unescape returned {other:?}"),
    }
}

fn call_parse_json(ctx: &v1_interpreter::InterpContext, text: &str) -> bool {
    let args = [(Some("s".to_string()), Value::Str(text.to_string()))];
    match v1_interpreter::run_in_context_with_args(ctx, "parse_json", &args, false) {
        Ok(Value::Variant { type_name, .. }) => ctx.sym_eq(type_name, "Optional"),
        other => panic!("parse_json returned {other:?}"),
    }
}

fn literal_body(decoded_len: usize) -> String {
    "a".repeat(decoded_len)
}

fn json_document_with_literal_string(decoded_len: usize) -> String {
    let mut out = String::with_capacity(decoded_len + 16);
    out.push_str("{\"x\":\"");
    out.push_str(&literal_body(decoded_len));
    out.push_str("\"}");
    out
}

fn mixed_body(decoded_len: usize, escape_count: usize) -> String {
    assert!(escape_count <= decoded_len);
    if escape_count == 0 {
        return literal_body(decoded_len);
    }
    let literal_count = decoded_len - escape_count;
    let mut out = String::with_capacity(decoded_len + escape_count);
    let mut escapes_left = escape_count;
    let mut literals_left = literal_count;
    while escapes_left > 0 || literals_left > 0 {
        if escapes_left > 0
            && (literals_left == 0
                || escapes_left * (literals_left + escapes_left) >= literals_left * escape_count)
        {
            out.push_str("\\n");
            escapes_left -= 1;
        } else {
            out.push('a');
            literals_left -= 1;
        }
    }
    out
}

fn json_document_with_mixed_string(decoded_len: usize, escape_count: usize) -> String {
    let mut out = String::with_capacity(decoded_len + escape_count + 16);
    out.push_str("{\"x\":\"");
    out.push_str(&mixed_body(decoded_len, escape_count));
    out.push_str("\"}");
    out
}

fn modeled_parameter_binding_copy_bytes(n: usize) -> u64 {
    (n as u64) * (n as u64)
}

fn modeled_concat_copy_bytes(n: usize) -> u64 {
    (n as u64) * (n as u64 - 1) / 2
}

fn linear_shadow_unescape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b'b' => {
                    out.push('\x08');
                    i += 2;
                }
                b'f' => {
                    out.push('\x0c');
                    i += 2;
                }
                other => {
                    out.push(other as char);
                    i += 2;
                }
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

struct TimedRun {
    wall: Duration,
    rss_delta_bytes: Option<i64>,
}

fn time_json_unescape(
    ctx: &v1_interpreter::InterpContext,
    input: &str,
    expected_decoded_len: usize,
) -> TimedRun {
    let rss_before = peak_rss_vhwm_bytes();
    let t0 = Instant::now();
    let out = call_json_unescape(ctx, input);
    let wall = t0.elapsed();
    assert_eq!(
        out.len(),
        expected_decoded_len,
        "literal unescape must preserve decoded length"
    );
    let rss_after = peak_rss_vhwm_bytes();
    TimedRun {
        wall,
        rss_delta_bytes: match (rss_before, rss_after) {
            (Some(b), Some(a)) => Some(a as i64 - b as i64),
            _ => None,
        },
    }
}

fn time_parse_json(ctx: &v1_interpreter::InterpContext, text: &str) -> TimedRun {
    let rss_before = peak_rss_vhwm_bytes();
    let t0 = Instant::now();
    let ok = call_parse_json(ctx, text);
    let wall = t0.elapsed();
    assert!(ok, "parse_json must accept synthetic literal document");
    let rss_after = peak_rss_vhwm_bytes();
    TimedRun {
        wall,
        rss_delta_bytes: match (rss_before, rss_after) {
            (Some(b), Some(a)) => Some(a as i64 - b as i64),
            _ => None,
        },
    }
}

fn best_of(samples: usize, mut f: impl FnMut() -> TimedRun) -> TimedRun {
    let mut best = f();
    for _ in 1..samples {
        let next = f();
        if next.wall < best.wall {
            best = next;
        }
    }
    best
}

fn print_timed(label: &str, run: &TimedRun) {
    let rss = run
        .rss_delta_bytes
        .map(|d| format!("{d:+} bytes"))
        .unwrap_or_else(|| "unavailable".to_string());
    println!("  {label}: wall={:?} rss_delta={rss}", run.wall);
}

fn run_length_point(ctx: &v1_interpreter::InterpContext, decoded_len: usize) {
    let body = literal_body(decoded_len);
    let doc = json_document_with_literal_string(decoded_len);
    let unescape = best_of(3, || time_json_unescape(ctx, &body, decoded_len));
    let parse = best_of(3, || time_parse_json(ctx, &doc));
    let param = modeled_parameter_binding_copy_bytes(decoded_len);
    let concat = modeled_concat_copy_bytes(decoded_len);
    println!("decoded_len={decoded_len} ({} KiB)", decoded_len / 1024);
    print_timed("json_unescape", &unescape);
    print_timed("parse_json", &parse);
    println!(
        "  modeled_copy_bytes: param={param} concat={concat} total={}",
        param + concat
    );
}

#[test]
fn json_unescape_decode_sanity() {
    let ctx = json_parse_ctx();
    assert_eq!(call_json_unescape(&ctx, "plain"), "plain");
    assert_eq!(call_json_unescape(&ctx, "a\\nb"), "a\nb");
    assert_eq!(call_json_unescape(&ctx, "\\u0041"), "A");
    assert!(call_parse_json(
        &ctx,
        &json_document_with_literal_string(32)
    ));
    let big = call_json_unescape(&ctx, &literal_body(1024));
    assert_eq!(big.len(), 1024);
}

#[test]
fn json_unescape_modeled_copy_terms_scale_quadratically() {
    let n = 10_000usize;
    let param = modeled_parameter_binding_copy_bytes(n);
    let concat = modeled_concat_copy_bytes(n);
    let total = param + concat;
    assert!(param > concat);
    let n2 = n * 2;
    let ratio = (modeled_parameter_binding_copy_bytes(n2) + modeled_concat_copy_bytes(n2)) as f64
        / total as f64;
    assert!(
        (3.9..4.1).contains(&ratio),
        "modeled total copy bytes should ~4x when n doubles; got {ratio:.2}x"
    );
}

#[test]
#[ignore = "wall-clock benchmark, not a correctness gate: run with --ignored"]
fn json_unescape_cost_length_sweep_10kib() {
    let ctx = json_parse_ctx();
    println!("=== A: length sweep 10 KiB (zero escapes) ===");
    run_length_point(&ctx, 10 * 1024);
}

#[test]
#[ignore = "wall-clock benchmark, not a correctness gate: run with --ignored"]
fn json_unescape_cost_length_sweep_50kib() {
    let ctx = json_parse_ctx();
    println!("=== A: length sweep 50 KiB (zero escapes) ===");
    run_length_point(&ctx, 50 * 1024);
}

#[test]
#[ignore = "wall-clock benchmark, not a correctness gate: run with --ignored"]
fn json_unescape_cost_length_sweep_200kib() {
    let ctx = json_parse_ctx();
    println!("=== A: length sweep 200 KiB (zero escapes) ===");
    run_length_point(&ctx, 200 * 1024);
}

#[test]
#[ignore = "wall-clock benchmark, not a correctness gate: run with --ignored"]
fn json_unescape_cost_escape_density_control() {
    let ctx = json_parse_ctx();
    println!("=== B: escape-density control (decoded_len=50 KiB) ===");
    for escape_count in [0usize, DENSITY_CONTROL_DECODED_BYTES] {
        let body = mixed_body(DENSITY_CONTROL_DECODED_BYTES, escape_count);
        let doc = json_document_with_mixed_string(DENSITY_CONTROL_DECODED_BYTES, escape_count);
        let unescape = best_of(3, || {
            time_json_unescape(&ctx, &body, DENSITY_CONTROL_DECODED_BYTES)
        });
        let parse = best_of(3, || time_parse_json(&ctx, &doc));
        let density_pct = (escape_count * 100) / DENSITY_CONTROL_DECODED_BYTES;
        println!(
            "density={density_pct}% input_len={} decoded_len={}",
            body.len(),
            DENSITY_CONTROL_DECODED_BYTES
        );
        print_timed("json_unescape", &unescape);
        print_timed("parse_json", &parse);
        println!();
    }
}

#[test]
#[ignore = "wall-clock benchmark, not a correctness gate: run with --ignored"]
fn json_unescape_cost_term_separation_10kib() {
    let ctx = json_parse_ctx();
    let n = 10 * 1024;
    let body = literal_body(n);
    println!("=== C: term separation (decoded_len=10 KiB, zero escapes) ===");
    let production = best_of(3, || time_json_unescape(&ctx, &body, n));
    let shadow = best_of(3, || {
        let rss_before = peak_rss_vhwm_bytes();
        let t0 = Instant::now();
        let out = linear_shadow_unescape(&body);
        assert_eq!(out.len(), n);
        let rss_after = peak_rss_vhwm_bytes();
        TimedRun {
            wall: t0.elapsed(),
            rss_delta_bytes: match (rss_before, rss_after) {
                (Some(b), Some(a)) => Some(a as i64 - b as i64),
                _ => None,
            },
        }
    });
    let param = modeled_parameter_binding_copy_bytes(n);
    let concat = modeled_concat_copy_bytes(n);
    print_timed("json_unescape (production)", &production);
    print_timed("linear_shadow_unescape (Rust)", &shadow);
    let ratio = production.wall.as_secs_f64() / shadow.wall.as_secs_f64().max(1e-9);
    println!("  production/shadow wall ratio: {ratio:.1}x (shadow is index-once, linear concat)");
    println!(
        "  modeled_binding_bytes={param} modeled_concat_bytes={concat} binding_share={:.1}%",
        100.0 * param as f64 / (param + concat) as f64
    );
}
