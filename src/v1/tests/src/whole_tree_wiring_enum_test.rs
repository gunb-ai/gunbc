//! Green-by-execution anchor for the gunbc#5364 whole-tree wiring-liveness
//! ENUMERATION widening (the reflection SOURCE side).
//!
//! `cli_run::whole_tree_resolved_ctx` resolves every `.dag` module under the
//! given source roots in one pass, so `eval_fn_arrow_decl_facts_live` (which
//! walks `ctx.modules`) enumerates fn-arrow decls across the WHOLE tree rather
//! than a single entry's import closure. This test exercises that accessor on a
//! self-contained 3-module fixture whose two leaf modules (`wt.a`, `wt.b`) do
//! NOT import each other — so a per-entry resolve of `wt.a` can never see
//! `wt.b`'s fn, but the whole-tree resolve does. That delta IS the widening.
//!
//! WHY A FIXTURE, NOT THE REAL CORPUS: the wiring lens itself (`v2.lens.
//! wiring_liveness`) lives under `src/v2`, and the `src/v2` corpus does not
//! whole-tree-resolve today — many test scaffolds (and non-test modules like
//! `v2.lens.testgen`) carry imports that only resolve inside a scoped closure,
//! and the pipeline short-circuits the whole graph to `None` on any unresolved
//! import (the `v2.lens.resolved_imports` open thread). So the whole-tree GATE
//! over the real corpus is deferred (DESIGN §5 "wall after grounding") until
//! that whole-tree-resolve grounding lands; this test proves the enumeration
//! SUBSTRATE works wherever resolve succeeds.

use std::rc::Rc;

use v1_compiler::cli_run::{self, whole_tree_resolved_ctx, ResolveTypecheckGate, WholeTreeCtx};
use v1_compiler::coproduct_reflection::eval_fn_arrow_decl_facts_live;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const FIXTURE_REL: &str = "src/v1/tests/fixtures/whole_tree_wiring_enum";
const MOD_A_REL: &str = "src/v1/tests/fixtures/whole_tree_wiring_enum/mod_a.dag";

fn fixture_root() -> String {
    workspace_root()
        .join(FIXTURE_REL)
        .to_string_lossy()
        .into_owned()
}

/// The `qualified_name` of every `FnArrowDecl` row the accessor yields in `ctx`.
fn enumerated_qualified_names(ctx: &InterpContext) -> Vec<String> {
    let val = eval_fn_arrow_decl_facts_live(ctx, &[]).expect("eval_fn_arrow_decl_facts_live");
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
                Some(Value::Str(s)) => s.clone(),
                other => panic!("expected qualified_name Str, got {other:?}"),
            }
        })
        .collect()
}

/// Resolve a single entry's import closure and enumerate within it — the
/// per-entry mechanism the widening transcends (and the no-whole-tree control).
fn closure_enumerated_qualified_names(entry_rel: &str) -> Vec<String> {
    let entry_content = std::fs::read_to_string(workspace_root().join(entry_rel))
        .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
    let sources: Vec<Rc<SourceFile>> = resolve_imports_transitively_with_source_roots(
        entry_rel,
        &entry_content,
        &[workspace_root().join(FIXTURE_REL)],
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    let graph = resolved
        .graph
        .as_ref()
        .expect("fixture entry closure resolves to a graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    enumerated_qualified_names(&ctx)
}

#[test]
fn whole_tree_enumeration_sees_fns_outside_a_per_entry_closure() {
    // Whole-tree pass over the fixture: every module resolved in one pass.
    let WholeTreeCtx {
        ctx,
        modules_resolved,
        modules_excluded,
        resolve_diagnostics: _,
    } = whole_tree_resolved_ctx(
        &[fixture_root()],
        &[],
        ExecutionMode::Wet,
        ResolveTypecheckGate::Strict,
    )
    .expect("whole-tree resolve of the self-contained fixture");
    assert_eq!(modules_excluded, 0, "fixture excludes nothing");
    assert_eq!(
        modules_resolved, 3,
        "fixture is wt.common + wt.a + wt.b = 3 modules"
    );

    let whole = enumerated_qualified_names(&ctx);
    assert!(
        whole.iter().any(|q| q == "wt.a.wt_a_wired"),
        "whole-tree enumeration must include wt.a's fn, got {whole:?}"
    );
    assert!(
        whole.iter().any(|q| q == "wt.b.wt_b_wired"),
        "whole-tree enumeration must include wt.b's fn, got {whole:?}"
    );

    // Control: `wt.a` and `wt.b` do not import each other, so wt.a's per-entry
    // closure can NEVER enumerate wt.b's fn. This is the discriminating delta —
    // the exact coverage the host SOURCE half gains by going whole-tree.
    let closure = closure_enumerated_qualified_names(MOD_A_REL);
    assert!(
        closure.iter().any(|q| q == "wt.a.wt_a_wired"),
        "wt.a's own closure must enumerate its own fn, got {closure:?}"
    );
    assert!(
        !closure.iter().any(|q| q == "wt.b.wt_b_wired"),
        "wt.a's closure must NOT see wt.b's fn (the gap whole-tree closes), got {closure:?}"
    );

    // The widening, stated as a set relation proven by execution.
    assert!(
        whole.len() > closure.len(),
        "whole-tree enumeration ({}) must strictly exceed the per-entry closure ({})",
        whole.len(),
        closure.len()
    );
}
