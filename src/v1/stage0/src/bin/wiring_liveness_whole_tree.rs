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
//! The liveness check is implemented directly in Rust by traversing the body
//! skeleton `Value` tree produced by `eval_fn_arrow_decl_facts_live`, rather than
//! running `wiring_liveness_corpus_is_clean` through the DAG interpreter. The
//! interpreter's fixpoint-saturating fold over the whole corpus is O(n²) in
//! node count and blows up memory; the Rust DFS is O(n_nodes_per_fn) per param
//! per fn and stays bounded in RSS.
//!
//! A param is "wired" iff there exists an Atom node with `identity == param_name`
//! anywhere in the output skeleton — exactly the invariant the DAG lens checks via
//! `wiring_reach_saturate` over `ready_set`. The Atom identity is a `Value::Str`
//! in the interpreter representation (see `atom_connective_variant` in
//! `coproduct_reflection.rs`), so the search is a string equality check at each
//! Atom node in the DFS.
//!
//! Live CI floor gate (gunbc#5364 + #5760). Enrolled as
//! `WiringLivenessWholeTreeGate` in `gunbc_ci_floor_gates`; invoked via
//! `tools.wiring_liveness_transport` / `tools.wiring_liveness_gate`.

use std::process::ExitCode;

use v1_compiler::cli_run::{whole_tree_resolved_ctx, ResolveTypecheckGate, WholeTreeCtx};
use v1_compiler::coproduct_reflection::eval_fn_arrow_decl_facts_live;
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("wiring_liveness_whole_tree: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

// --- Rust-side liveness check (avoids DAG interpreter fold) ---

/// True iff the body skeleton rooted at `node` contains an Atom with the given
/// identity anywhere in its subtree. This is the Rust analog of
/// `wiring_reach_contains(ready_set(output), declared_input)`.
fn skeleton_contains_param(ctx: &InterpContext, node: &Value, param_name: &str) -> bool {
    match node {
        Value::Record { type_name, fields } => {
            if ctx.sym_eq(*type_name, "Node") {
                // Is this an Atom node with the target identity?
                if let Some(kind) = ctx.field(fields, "kind") {
                    if is_atom_kind_with_identity(ctx, kind, param_name) {
                        return true;
                    }
                }
                // Recurse into children (each child is an Edge whose "target" is a Node)
                if let Some(children) = ctx.field(fields, "children") {
                    return children_contain_param(ctx, children, param_name);
                }
                return false;
            }
            if ctx.sym_eq(*type_name, "Edge") {
                if let Some(target) = ctx.field(fields, "target") {
                    return skeleton_contains_param(ctx, target, param_name);
                }
                return false;
            }
            false
        }
        Value::List(items) => items.iter().any(|v| skeleton_contains_param(ctx, v, param_name)),
        _ => false,
    }
}

fn children_contain_param(ctx: &InterpContext, children: &Value, param_name: &str) -> bool {
    let Value::List(edges) = children else {
        return false;
    };
    edges
        .iter()
        .any(|e| skeleton_contains_param(ctx, e, param_name))
}

/// True iff `kind` is `TypeNode { connective: Atom { identity: target_identity } }`.
fn is_atom_kind_with_identity(ctx: &InterpContext, kind: &Value, target_identity: &str) -> bool {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = kind
    else {
        return false;
    };
    if !ctx.sym_eq(*variant_name, "TypeNode") {
        return false;
    }
    let Some(connective) = ctx.field(fields, "connective") else {
        return false;
    };
    let Value::Variant {
        variant_name: cn_name,
        fields: cn_fields,
        ..
    } = connective
    else {
        return false;
    };
    if !ctx.sym_eq(*cn_name, "Atom") {
        return false;
    }
    let Some(identity) = ctx.field(cn_fields, "identity") else {
        return false;
    };
    match identity {
        Value::Str(s) => s == target_identity,
        _ => false,
    }
}

// --- Binary entry point ---

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

    let all_decls = eval_fn_arrow_decl_facts_live(&ctx, &[]).map_err(|e| {
        eprintln!(
            "wiring_liveness_whole_tree: fn_arrow_decl_facts_live failed: {e}"
        );
        ExitCode::from(2)
    })?;

    let Value::List(decl_list) = &all_decls else {
        eprintln!("wiring_liveness_whole_tree: fn_arrow_decl_facts_live returned non-list");
        return Err(ExitCode::from(2));
    };

    eprintln!(
        "wiring_liveness_whole_tree: checking liveness for {} fn declaration(s)",
        decl_list.len()
    );

    let mut dead_count = 0usize;
    for decl in decl_list.iter() {
        let Value::Record {
            fields: decl_fields,
            ..
        } = decl
        else {
            continue;
        };
        let Some(Value::Str(qualified_name)) = ctx.field(decl_fields, "qualified_name") else {
            continue;
        };
        let Some(output_val) = ctx.field(decl_fields, "output") else {
            continue;
        };
        let Some(Value::List(params)) = ctx.field(decl_fields, "params") else {
            continue;
        };

        for param in params.iter() {
            let Value::Record {
                fields: param_fields,
                ..
            } = param
            else {
                continue;
            };
            let Some(Value::Str(param_name)) = ctx.field(param_fields, "name") else {
                continue;
            };

            if !skeleton_contains_param(&ctx, output_val, param_name) {
                dead_count += 1;
                eprintln!("  dead wire: {qualified_name}:{param_name}");
            }
        }
    }

    if dead_count == 0 {
        eprintln!(
            "wiring_liveness_whole_tree: CLEAN — 0 dead wires across {} fn(s)",
            decl_list.len()
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
