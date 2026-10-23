//! Green-by-execution anchor for `fn_arrow_decl_facts_live`'s CALLER-NAMED SUBJECT
//! (gunbc#10156 clause 2, the reflection SOURCE side).
//!
//! WHAT THIS FILE USED TO PROVE, AND WHY THAT CLAIM NO LONGER EXISTS. The accessor
//! walked `ctx.modules`, so its population was whatever module closure the calling
//! context had resolved. This test's subject was that dependence: it resolved the
//! fixture whole-tree, resolved `wt.a` alone, and asserted the first enumeration
//! strictly exceeded the second. Both arms now return the same rows, because the
//! population is a function of the declared `pool_roots` and of nothing else — so the
//! old assertion is not merely stale, its premise is deleted. Rewritten rather than
//! patched: a widening test kept alive over a producer that no longer widens would be
//! green for a reason it does not state.
//!
//! WHAT IT PROVES NOW, in the same fixture and with the same discriminating shape. The
//! two leaf modules (`wt.a`, `wt.b`) do NOT import each other, and they sit in
//! sibling directories, so:
//!   * the SAME named subject yields the SAME rows from a whole-tree context and from
//!     `wt.a`'s own single-entry closure — the row that goes red if `ctx.modules` is
//!     still an input;
//!   * a root that contributes no `.dag` file REFUSES rather than answering with a
//!     silently smaller population.
//!
//! The exclusion half — naming one root and NOT seeing a sibling root's declaration,
//! the row that goes red if the argument is ignored — needs two directories, which this
//! fixture does not have; it lives in the `.dag` witness named below, over a fixture
//! built for it.
//!
//! This target is compiled by `repo_self_clippy_command` and run by no CI step (see
//! DESIGN, "Building & checks"). The executing evidence on the required floor is
//! `dag/test/claim/fn_arrow_decl_facts_live_subject_witness_test.dag`; this file is
//! the host-side control over the refusal arm, which `.dag` cannot author.

use std::rc::Rc;

use v1_compiler::cli_run::{self, whole_tree_resolved_ctx, WholeTreeCtx};
use v1_compiler::coproduct_reflection::eval_fn_arrow_decl_facts_live;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{str_value, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const FIXTURE_REL: &str = "src/v1/tests/fixtures/whole_tree_wiring_enum";
const MOD_A_REL: &str = "src/v1/tests/fixtures/whole_tree_wiring_enum/mod_a.dag";

fn fixture_root() -> String {
    workspace_root()
        .join(FIXTURE_REL)
        .to_string_lossy()
        .into_owned()
}

fn pool_roots_arg(roots: &[&str]) -> Vec<(Option<String>, Value)> {
    let items: im::Vector<Value> = roots.iter().copied().map(str_value).collect();
    vec![(Some("pool_roots".to_string()), Value::List(Rc::new(items)))]
}

/// The `qualified_name` of every `FnArrowDecl` row the accessor yields for `roots`.
fn enumerated_qualified_names(ctx: &InterpContext, roots: &[&str]) -> Vec<String> {
    let val = eval_fn_arrow_decl_facts_live(ctx, &pool_roots_arg(roots))
        .expect("eval_fn_arrow_decl_facts_live");
    let items = match &val {
        Value::List(items) => items,
        other => panic!("expected List of FnArrowDecl, got {other:?}"),
    };
    items
        .iter()
        .map(|row| {
            match ctx.field(
                match row {
                    Value::Record { fields, .. } => fields,
                    other => panic!("expected FnArrowDecl Record, got {other:?}"),
                },
                "qualified_name",
            ) {
                Some(Value::Str(s)) => s.to_string(),
                other => panic!("expected qualified_name Str, got {other:?}"),
            }
        })
        .collect()
}

/// A context holding ONLY `wt.a`'s import closure — `wt.b` is unreachable from it.
fn single_entry_closure_ctx() -> InterpContext {
    let entry_content = std::fs::read_to_string(workspace_root().join(MOD_A_REL))
        .unwrap_or_else(|e| panic!("read {MOD_A_REL}: {e}"));
    let sources: Vec<Rc<SourceFile>> = resolve_imports_transitively_with_source_roots(
        MOD_A_REL,
        &entry_content,
        &[workspace_root().join(FIXTURE_REL)],
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved
        .graph
        .as_ref()
        .expect("fixture entry closure resolves to a graph");
    cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet)
}

#[test]
fn the_named_subject_decides_the_population_and_the_context_does_not() {
    let WholeTreeCtx {
        ctx,
        modules_resolved,
        modules_excluded,
    } = whole_tree_resolved_ctx(&[fixture_root()], &[], ExecutionMode::Wet)
        .expect("whole-tree resolve of the self-contained fixture");
    assert_eq!(modules_excluded, 0, "fixture excludes nothing");
    assert_eq!(
        modules_resolved, 3,
        "fixture is wt.common + wt.a + wt.b = 3 modules"
    );

    let both = enumerated_qualified_names(&ctx, &[FIXTURE_REL]);
    assert!(
        both.iter().any(|q| q == "wt.a.wt_a_wired") && both.iter().any(|q| q == "wt.b.wt_b_wired"),
        "naming the fixture root must yield both leaf fns, got {both:?}"
    );

    // THE CONTEXT IS NOT AN INPUT. `wt.a`'s own closure cannot reach `wt.b` at all, yet
    // the same named subject yields the same rows from it. Under the previous accessor
    // this assertion was false by construction — that delta IS the climb.
    let from_closure = enumerated_qualified_names(&single_entry_closure_ctx(), &[FIXTURE_REL]);
    let mut a = both.clone();
    let mut b = from_closure.clone();
    a.sort();
    b.sort();
    assert_eq!(
        a, b,
        "the same named subject must yield the same population from any context"
    );
}

#[test]
fn a_root_that_contributes_nothing_refuses_rather_than_narrowing_silently() {
    let WholeTreeCtx { ctx, .. } =
        whole_tree_resolved_ctx(&[fixture_root()], &[], ExecutionMode::Wet)
            .expect("whole-tree resolve of the self-contained fixture");
    let refused = eval_fn_arrow_decl_facts_live(
        &ctx,
        &pool_roots_arg(&["src/v1/tests/fixtures/does_not_exist"]),
    );
    assert!(
        refused.is_err(),
        "a pool root contributing no .dag file must refuse, got {refused:?}"
    );
}
