#![allow(clippy::disallowed_macros)]

//! Closure-overlap probe over the production-selected entry set
//! (entry-graph-union slice 1 / lane ci-cost, subject `entry-graph-union-construction`).
//!
//! Answers, for one real `(base, head)`: how many entries the production affected-set
//! selector selects, how large each selected entry's production import closure is, and how
//! much those closures overlap — `N`, `sum_closure_memberships`, `union_modules`,
//! `duplication_factor`, `membership_upper_bound`, plus per-module selected-entry fanout.
//!
//! Measurement only. It resolves and typechecks nothing and implements no union: the
//! result is an UPPER BOUND on repeated module membership, never a wall-time saving.
//!
//! The base ref comes from the production `GUNBC_CI_DIFF_BASE` operator-override channel
//! (`gunbc.diff_baseline`), so a subject is reproducible by naming the same base.
//!
//! Same dissolution trigger as the rest of the host-side measurement family: a `.dag`
//! `PerformanceReceipt` carrier consumed by a floor witness replaces this probe.
//!
//! ```text
//! GUNBC_CI_DIFF_BASE=<sha> measure_selected_closure_overlap \
//!   --source-root dag --source-root src/v2 --scan-dir dag/test/claim
//! ```

use std::process::ExitCode;

use v1_compiler::cli_run::{
    measure_selected_entry_closure_overlap, render_selected_entry_closure_overlap_json,
    witness_exclusion_substrings,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_selected_closure_overlap: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut discovery_scope_dirs: Vec<String> = Vec::new();
    // THIS MEASUREMENT's discovery-exclusion authority
    // (`gunbc.ci_layer_roots.witness_exclusion_substrings`), not
    // `whole_tree_probe_exclusion_substrings` — the probe list is that list UNION
    //
    // NOT THE REQUIRED FLOOR'S EXCLUSION AUTHORITY, and this comment said it was until
    // 2026-08-22. `run_required_floor` consults `floor_prepared_subject_exclusions` in
    // `cli_run.rs` and nothing else; that function's own comment records a change that added
    // rows to `gunbc.ci_layer_roots` and measured NO effect. The sentence was true about this
    // binary's own subject — which corpus to measure over — and false about the floor, so a
    // reader grepping for the floor's exclusion authority found the wrong answer here first and
    // the right one three files away. Naming the subject is the whole of the fix; do NOT
    // re-point this binary at the floor's list, which would make it measure a corpus it is not
    // about.
    // the whole-tree strict-resolve exclusions, so defaulting to it measured a corpus far
    // narrower than the one production selects over (measured: 45 roster entries against
    // 579 `*_test.dag` files under the same scan dirs). A subject drawn from 8% of the
    // corpus cannot answer a question about the corpus.
    let mut exclude_substrings = witness_exclusion_substrings();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                scan_dirs.push(require_value(&args, i, "--scan-dir")?);
            }
            "--discovery-scope-dir" => {
                i += 1;
                discovery_scope_dirs.push(require_value(&args, i, "--discovery-scope-dir")?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_substrings.push(require_value(&args, i, "--exclude-subpath")?);
            }
            other => {
                eprintln!("measure_selected_closure_overlap: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("measure_selected_closure_overlap: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }
    if scan_dirs.is_empty() {
        eprintln!("measure_selected_closure_overlap: at least one --scan-dir is required");
        return Err(ExitCode::from(2));
    }

    // A refusal from the selector or the diff observation is the receipt: it propagates
    // rather than falling back to "measure every entry", which would be the absorbing
    // fallback DESIGN §5 forbids and would silently answer a different question.
    let measured = measure_selected_entry_closure_overlap(
        &source_roots,
        &scan_dirs,
        &exclude_substrings,
        &discovery_scope_dirs,
    )
    .map_err(|e| {
        eprintln!("measure_selected_closure_overlap: REFUSED — {e}");
        ExitCode::from(2)
    })?;

    println!(
        "[closure-overlap] {}",
        render_selected_entry_closure_overlap_json(&measured)
    );
    eprintln!(
        "measure_selected_closure_overlap: N={} sum_closure_memberships={} union_modules={} \
         duplication_factor={} membership_upper_bound={}",
        measured.selected_count(),
        measured.sum_closure_memberships,
        measured.union_modules,
        measured
            .duplication_factor()
            .map(|f| format!("{f:.3}"))
            .unwrap_or_else(|| "n/a".to_string()),
        measured.membership_upper_bound(),
    );

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
