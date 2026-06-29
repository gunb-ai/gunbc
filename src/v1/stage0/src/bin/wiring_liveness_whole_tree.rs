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
//!
//! SCAFFOLD / not floor-enrolled yet (DESIGN §5 "wall after grounding"). The
//! whole-tree wiring GATE over the real corpus is BLOCKED on a single named
//! authority: the `v2.lens.resolved_imports` whole-tree-resolve grounding — i.e.
//! `front_end_sources` no longer short-circuiting the whole graph to `None` on an
//! unresolved import. The `src/v2` corpus does not whole-tree-resolve today
//! (test scaffolds and even non-test modules like `v2.lens.testgen` import
//! modules that only resolve inside a scoped closure), so this bin can run the
//! lens only over a source-root set that fully resolves. The enumeration
//! SUBSTRATE it stands on (`whole_tree_resolved_ctx` + the reflection accessor)
//! is proven green-by-execution by `v1-compiler-tests`
//! `whole_tree_wiring_enum_test`. DISSOLVES INTO a live floor gate (flip the
//! existing per-entry `wiring_liveness_corpus_is_clean` to whole-tree) the moment
//! that resolve grounding lands and `--source-root src/v2 --source-root dsl`
//! resolves clean.

use std::process::ExitCode;

use v1_compiler::cli_run::{peak_rss_vhwm_bytes, whole_tree_resolved_ctx, WholeTreeCtx};
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
    // Intentionally-malformed scanner fixture inputs (test DATA referenced by string
    // path, not live code) declare imports of nonexistent modules and so cannot be
    // part of a Strict whole-tree resolve. Excluded by default; extendable via flag.
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
    } = whole_tree_resolved_ctx(&source_roots, &exclude_subpaths, ExecutionMode::Wet).map_err(
        |e| {
            eprintln!("wiring_liveness_whole_tree: whole-tree resolve failed:\n{e}");
            ExitCode::from(2)
        },
    )?;
    eprintln!(
        "wiring_liveness_whole_tree: resolved {} module(s) over {} source root(s) \
         ({} excluded by subpath: {:?})",
        modules_resolved,
        source_roots.len(),
        modules_excluded,
        exclude_subpaths
    );
    match peak_rss_vhwm_bytes() {
        Some(bytes) => eprintln!(
            "[measurement] whole-tree resolve peak RSS: {bytes} bytes (VmHWM) modules={modules_resolved}"
        ),
        None => eprintln!(
            "[measurement] whole-tree resolve peak RSS: unavailable (no /proc/self/status) modules={modules_resolved}"
        ),
    }

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
