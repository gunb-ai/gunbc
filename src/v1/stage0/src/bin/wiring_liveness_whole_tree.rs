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
//! runs the clean check IN that context — so coverage is whole-tree-in-one-pass
//! and a declared input with no path to its output ANYWHERE in the corpus fails.
//!
//! Marshaling runs in the whole-tree context's own interner (the same `ctx` that
//! holds the modules), so reflected `Node` values are self-consistent — no
//! cross-context `Symbol` mismatch.

use std::process::ExitCode;

use v1_compiler::cli_run::whole_tree_resolved_ctx;
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

const DEAD_WIRES_FN: &str = "wiring_liveness_corpus_dead_wires";

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

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
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

    let ctx = whole_tree_resolved_ctx(&source_roots, ExecutionMode::Wet).map_err(|e| {
        eprintln!("wiring_liveness_whole_tree: whole-tree resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    eprintln!(
        "wiring_liveness_whole_tree: resolved {} module(s) over {} source root(s)",
        ctx.modules.len(),
        source_roots.len()
    );

    let dead = v1_interpreter::run_in_context(&ctx, DEAD_WIRES_FN, false).map_err(|e| {
        eprintln!("wiring_liveness_whole_tree: interpreter error running {DEAD_WIRES_FN}: {e}");
        ExitCode::from(2)
    })?;

    let items = match &dead {
        Value::List(items) => items,
        other => {
            eprintln!(
                "wiring_liveness_whole_tree: {DEAD_WIRES_FN} returned {}, not a List",
                ctx.format_value(other)
            );
            return Err(ExitCode::from(2));
        }
    };

    if items.is_empty() {
        eprintln!("wiring_liveness_whole_tree: CLEAN — 0 dead wires across the whole corpus");
        return Ok(ExitCode::SUCCESS);
    }

    eprintln!(
        "wiring_liveness_whole_tree: {} dead wire(s) across the whole corpus:",
        items.len()
    );
    for item in items.iter() {
        eprintln!("  {}", ctx.format_value(item));
    }
    Ok(ExitCode::from(1))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
