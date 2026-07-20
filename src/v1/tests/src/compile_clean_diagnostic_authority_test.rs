//! The compile-clean gate must decide "is this diagnostic hard?" from the single
//! `00_core.dag` authority, never a hand-rolled restatement (DESIGN.md §3/§7).
//!
//! History this pins: `compile_clean_pipeline_has_hard_errors` hand-rolled
//! `!matches!(ComplexityUnknown)`. That predicate predated `UnlistedImportUse`'s
//! demotion to advisory, so the seed called it HARD while the model called it
//! advisory — the two authorities disagreed and the namespace import strip (#6640)
//! reded on 20+ `unlisted import use` diagnostics that the model does not consider
//! errors at all.
//!
//! RED control: revert the gate to any predicate that treats `UnlistedImportUse` as
//! hard and `unlisted_import_use_alone_is_not_hard` fails. The genuine-error and
//! mixed cases keep it from being satisfied by a blanket `false`.

use std::sync::Arc;

use v1_compiler::cli_run::compile_clean_pipeline_has_hard_errors;
use v1_compiler::std_types::SourceSpan;
use v1_compiler::v1_std_core::{CompilerDiagnostic, ErrorNode};

fn span() -> Arc<SourceSpan> {
    Arc::new(SourceSpan {
        file: "test.dag".to_string(),
        start: 0,
        end: 0,
    })
}

fn node(d: CompilerDiagnostic) -> Arc<ErrorNode> {
    Arc::new(ErrorNode {
        diagnostic: Arc::new(d),
        module_name: "test".to_string(),
    })
}

fn unlisted_import_use() -> Arc<ErrorNode> {
    node(CompilerDiagnostic::UnlistedImportUse {
        name: "NonEmptyStr".to_string(),
        span: span(),
    })
}

fn complexity_unknown() -> Arc<ErrorNode> {
    node(CompilerDiagnostic::ComplexityUnknown {
        func_name: "f".to_string(),
        reason: "unclassifiable".to_string(),
        span: span(),
    })
}

/// A diagnostic the model genuinely calls an error.
fn genuine_error() -> Arc<ErrorNode> {
    node(CompilerDiagnostic::UnresolvedType {
        name: "NoSuchType".to_string(),
        span: span(),
    })
}

fn has_hard(nodes: Vec<Arc<ErrorNode>>) -> bool {
    let v: im::Vector<Arc<ErrorNode>> = nodes.into_iter().collect();
    compile_clean_pipeline_has_hard_errors(&v)
}

#[test]
fn empty_is_clean() {
    assert!(!has_hard(vec![]), "no diagnostics must be clean");
}

/// THE discriminating case — this is what #6640 tripped on.
#[test]
fn unlisted_import_use_alone_is_not_hard() {
    assert!(
        !has_hard(vec![unlisted_import_use()]),
        "UnlistedImportUse is advisory per 00_core.dag is_error_diagnostic; the gate \
         must not red on it (the seed-vs-model fork that reded the #6640 import strip)"
    );
}

#[test]
fn complexity_unknown_alone_is_not_hard() {
    assert!(
        !has_hard(vec![complexity_unknown()]),
        "ComplexityUnknown was already tolerated; consolidating onto the .dag \
         authority must not regress that"
    );
}

/// Keeps the gate from being satisfied by a blanket `false`.
#[test]
fn genuine_error_is_hard() {
    assert!(
        has_hard(vec![genuine_error()]),
        "a real error must still red the gate"
    );
}

/// Advisories must not mask a real error sharing the run.
#[test]
fn genuine_error_is_hard_even_beside_advisories() {
    assert!(
        has_hard(vec![
            unlisted_import_use(),
            complexity_unknown(),
            genuine_error(),
        ]),
        "advisories must not suppress a real error"
    );
}

/// Pins the *delegation* rather than the verdicts: the gate must return whatever
/// `00_core.dag` says, so a future change to the authority carries the gate with it
/// instead of drifting from it again. Sampled over the three classes that matter
/// (advisory / tolerated / genuine); it is not an exhaustive variant sweep.
#[test]
fn gate_agrees_with_dag_authority() {
    for d in [unlisted_import_use(), complexity_unknown(), genuine_error()] {
        let authority =
            v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone());
        assert_eq!(
            has_hard(vec![d.clone()]),
            authority,
            "gate disagreed with 00_core.dag authority on {:?}",
            d.diagnostic
        );
    }
}
