#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / CHARAT-0) — isolated `char_at` scaling
//! receipt: many repeated calls at varying position and varying string length, uncontaminated
//! by the JSON parser's independent `value_to_list_carrier` / `free_monoid_to_vec`
//! materialization cost (a different, separately scoped defect — eager-koi-458's 2026-08-17
//! scope correction on this work item names it and rules it out of this instrument).
//!
//! `json_parse_scaling_probe`'s end-to-end parse measures whichever quadratic dominates and
//! cannot by itself distinguish "char_at is O(1)" from "the list carrier is O(m^2)". This bin
//! isolates the one thing STRING-INDEX-0 changed: `char_at` no longer rescans `is_ascii` per
//! call, because ascii-ness is now a precomputed fact carried on `RcStr`. On an ASCII input,
//! per-call cost should therefore be flat against BOTH position and string length.
//!
//! NOT floor-enrolled — run standalone via `cargo run --release -p v1-compiler --bin
//! char_at_scaling_probe`. Entry: `dag/gunbc/char_at_scaling_probe_support.dag`'s
//! `char_at_probe(s, pos)`, a one-line wrapper around the `char_at` free call.
//!
//! CHECKABLE RECEIPT: for each (length, position) pair, mean per-call elapsed time over
//! `CHAR_AT_PROBE_REPS` repeated calls through the interpreter — printed as TSV.
//!
//! DISSOLUTION (own trigger, independent of `json_parse_scaling_probe.rs`'s — see
//! `char_at_scaling_probe_dissolution` in `dag/gunbc/char_at_scaling_probe_support.dag`,
//! DESIGN §5's same-unit rule): delete this bin when CHARAT-0's `char_at` O(1) property is
//! floor-enrolled with a modeled witness, or when a fresh run's own printed TSV (the
//! CHECKABLE RECEIPT below) shows `mean_call_us` growing with `string_len` across the
//! `CHAR_AT_PROBE_LENGTHS` range instead of staying flat.
//! Receipt: `rg CHAR_AT_SCALING_PROBE_SCAFFOLD_MARKER src/v1/stage0` until deletion.

/// Grep receipt for scaffold dissolution (`rg CHAR_AT_SCALING_PROBE_SCAFFOLD_MARKER`).
pub const CHAR_AT_SCALING_PROBE_SCAFFOLD_MARKER: &str =
    "CHARAT-0 char_at_scaling_probe measurement transport (not floor-enrolled)";

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, workspace_root};
use v1_compiler::v1_interpreter::{self, str_value, ExecutionMode, Value};

const ENTRY: &str = "dag/gunbc/char_at_scaling_probe_support.dag";

/// One repeated call per (length, position) pair; small enough to keep the whole
/// grid under a minute, large enough to average out interpreter dispatch noise.
const DEFAULT_REPS: usize = 2000;

fn reps_from_env() -> usize {
    std::env::var("CHAR_AT_PROBE_REPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_REPS)
}

/// Pure-ASCII fill so the fast path (byte-offset indexing, no `.chars().nth` walk) is
/// what's being timed — this is the branch STRING-INDEX-0 made O(1).
fn make_ascii_string(len: usize) -> String {
    (0..len).map(|i| (b'a' + (i % 26) as u8) as char).collect()
}

fn resolve_ctx() -> Result<v1_interpreter::InterpContext, String> {
    let ws = workspace_root();
    let roots = vec![ws.join("dag").to_string_lossy().into_owned()];
    eprintln!("char_at_scaling_probe: resolving {ENTRY} ...");
    let resolve_start = Instant::now();
    let (graph, indices) = resolve_entry_graph(&roots, ENTRY)?;
    eprintln!(
        "char_at_scaling_probe: resolve_ms={}",
        resolve_start.elapsed().as_millis()
    );
    Ok(make_eval_context(&graph, indices, ExecutionMode::Hermetic))
}

fn call_char_at_once(
    ctx: &v1_interpreter::InterpContext,
    s: &Value,
    pos: i64,
) -> Result<(), String> {
    let args = [
        (Some("s".to_string()), s.clone()),
        (Some("pos".to_string()), Value::Int(pos)),
    ];
    v1_interpreter::run_in_context_with_args(ctx, "char_at_probe", &args, false)
        .map(|_| ())
        .map_err(|e| format!("char_at_probe: {e}"))
}

/// Mean per-call elapsed microseconds over `reps` calls at one fixed (s, pos).
fn mean_call_us(
    ctx: &v1_interpreter::InterpContext,
    s: &Value,
    pos: i64,
    reps: usize,
) -> Result<f64, String> {
    // One untimed warmup call so any first-call setup (e.g. lazy memo frame init)
    // doesn't bias the measured mean.
    call_char_at_once(ctx, s, pos)?;
    let start = Instant::now();
    for _ in 0..reps {
        call_char_at_once(ctx, s, pos)?;
    }
    let elapsed = start.elapsed();
    Ok(elapsed.as_secs_f64() * 1_000_000.0 / reps as f64)
}

fn run() -> Result<(), String> {
    let ctx = resolve_ctx()?;
    let reps = reps_from_env();
    let lengths: Vec<usize> = std::env::var("CHAR_AT_PROBE_LENGTHS")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|tok| tok.trim().parse().ok())
                .collect()
        })
        .filter(|v: &Vec<usize>| !v.is_empty())
        .unwrap_or_else(|| vec![10_000, 100_000, 500_000, 2_000_000]);
    let position_fractions: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 0.99];

    eprintln!("char_at_scaling_probe: reps={reps} lengths={lengths:?}");
    println!("mode\tchar_at_scaling\treps={reps}");
    println!("string_len\tposition\tposition_frac\tmean_call_us");

    for &len in &lengths {
        let s = make_ascii_string(len);
        let value = str_value(&s);
        for frac in position_fractions {
            let pos = ((len.saturating_sub(1)) as f64 * frac).round() as i64;
            let mean_us = mean_call_us(&ctx, &value, pos, reps)?;
            println!("{len}\t{pos}\t{frac:.2}\t{mean_us:.3}");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("char_at_scaling_probe: ERROR: {e}");
            ExitCode::FAILURE
        }
    }
}
