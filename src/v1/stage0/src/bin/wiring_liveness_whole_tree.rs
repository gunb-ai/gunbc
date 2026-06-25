#![allow(clippy::disallowed_macros)]

//! Whole-tree wiring-liveness scan (gunbc#5364 widening).
//!
//! `v2.lens.wiring_liveness.wiring_liveness_corpus_is_clean` folds the wave-1
//! reachability over `fn_arrow_decl_facts_live()`, which enumerates one
//! `FnArrowDecl` per declared fn across `ctx.modules`. Run as a per-entry witness
//! (`--claim-run --entry <file>`), `ctx.modules` is only that entry's import
//! closure, so a dead wire in a fn OUTSIDE the closure is invisible. This bin
//! builds a context over the WHOLE source-root corpus in one pass (the same
//! whole-tree resolve `precompute_whole_tree_published_mock_keys` performs) and
//! checks liveness IN that context — so coverage is whole-tree-in-one-pass and a
//! declared input with no path to its output ANYWHERE in the corpus fails.
//!
//! The liveness check is implemented via `check_wiring_liveness_streaming` in
//! `coproduct_reflection`, which processes one fn at a time so peak memory is
//! O(max_skeleton_per_fn) rather than O(sum_of_all_skeletons). The interpreter's
//! fixpoint-saturating fold over the whole corpus is O(n²) in node count and
//! blows up memory; the streaming approach stays bounded in RSS.
//!
//! Live CI floor gate (gunbc#5364 + #5760). Enrolled as
//! `WiringLivenessWholeTreeGate` in `gunbc_ci_floor_gates`; invoked via
//! `tools.wiring_liveness_transport` / `tools.wiring_liveness_gate`.

use std::process::ExitCode;

use v1_compiler::cli_run::{whole_tree_resolved_ctx, ResolveTypecheckGate, WholeTreeCtx};
use v1_compiler::coproduct_reflection::check_wiring_liveness_streaming;
use v1_compiler::v1_interpreter::ExecutionMode;

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("wiring_liveness_whole_tree: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    // Intentionally-malformed scanner fixture inputs declare imports of nonexistent
    // modules and cannot be part of a whole-tree resolve. Excluded by default;
    // extendable via flag.
    let mut exclude_subpaths: Vec<String> = vec!["test/fixture/".to_string()];

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(require_value(&args, i, "--exclude-subpath")?);
            }
            other => {
                eprintln!("wiring_liveness_whole_tree: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("wiring_liveness_whole_tree: at least one --source-root is required");
        return Err(ExitCode::from(2));
    }

    let WholeTreeCtx {
        ctx,
        modules_resolved,
        modules_excluded,
    } = whole_tree_resolved_ctx(
        &source_roots,
        &exclude_subpaths,
        ExecutionMode::Wet,
        ResolveTypecheckGate::WholeLivenessCorpus,
    )
    .map_err(|e| {
        eprintln!("wiring_liveness_whole_tree: whole-tree resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    eprintln!(
        "wiring_liveness_whole_tree: resolved {} module(s) over {} source root(s) \
         ({} excluded by subpath: {:?})",
        modules_resolved,
        source_roots.len(),
        modules_excluded,
        exclude_subpaths
    );

    let (fn_count, dead_count) =
        check_wiring_liveness_streaming(&ctx, |qualified_name, param_name| {
            eprintln!("  dead wire: {qualified_name}:{param_name}");
        });

    if dead_count == 0 {
        eprintln!(
            "wiring_liveness_whole_tree: CLEAN — 0 dead wires across {} fn(s)",
            fn_count
        );
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "wiring_liveness_whole_tree: FAIL — {dead_count} dead wire(s) across the whole corpus"
        );
        Ok(ExitCode::from(1))
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
