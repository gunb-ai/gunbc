//! Green-by-execution anchor for the `ResolveTypecheckGate::DiagnosticsCollector`
//! carrier (north-star, gunbc#5760 + #5772 advisory whole-tree typecheck).
//!
//! The carrier exists so a whole-tree resolve can COLLECT every resolve/typecheck
//! diagnostic and continue, rather than fail-closed on the first blocking one — the
//! data the advisory `whole_tree_typecheck_advisory` bin counts and classifies. The
//! discriminating property that makes the carrier real (DESIGN §5/§6 — not an inert
//! field that merely type-checks) is the DELTA between the two gates over the SAME
//! sources: a module with an unresolved reference must BLOCK under `Strict` and be
//! COLLECTED (non-blocking, surfaced in `resolve_diagnostics`) under
//! `DiagnosticsCollector`.
//!
//! Fixture: two modules — `wtd.clean` resolves; `wtd.broken` imports `wtd.clean`
//! and calls a function that does not exist anywhere. That single dangling call is
//! the discriminating input: flip it to a real call and BOTH gates go green, which
//! is the red-witness control this test pins by execution.

use v1_compiler::cli_run::{whole_tree_resolved_ctx, ResolveTypecheckGate, WholeTreeCtx};
use v1_compiler::v1_interpreter::ExecutionMode;
use v1_compiler::v1_std_core::diagnostic_to_message;

use crate::helpers::workspace_root;

const FIXTURE_REL: &str = "src/v1/tests/fixtures/whole_tree_diag_collector";

fn fixture_root() -> String {
    workspace_root()
        .join(FIXTURE_REL)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn strict_blocks_what_diagnostics_collector_collects() {
    // Strict gate: the dangling call in `wtd.broken` is BLOCKING — the whole-tree
    // resolve fails closed (DESIGN §5). This is the "wall" half.
    let strict = whole_tree_resolved_ctx(
        &[fixture_root()],
        &[],
        ExecutionMode::Wet,
        ResolveTypecheckGate::Strict,
    );
    assert!(
        strict.is_err(),
        "Strict gate must fail closed on wtd.broken's dangling call, got Ok"
    );

    // DiagnosticsCollector gate: the SAME dangling call is COLLECTED, not blocking.
    // The resolve succeeds, both modules are resolved, and the diagnostic is carried
    // in `resolve_diagnostics` naming the offending module + symbol.
    let WholeTreeCtx {
        ctx: _,
        modules_resolved,
        modules_excluded,
        resolve_diagnostics,
    } = whole_tree_resolved_ctx(
        &[fixture_root()],
        &[],
        ExecutionMode::Wet,
        ResolveTypecheckGate::DiagnosticsCollector,
    )
    .expect("DiagnosticsCollector must collect-and-continue, not fail closed");

    assert_eq!(modules_excluded, 0, "fixture excludes nothing");
    assert_eq!(
        modules_resolved, 2,
        "fixture is wtd.clean + wtd.broken = 2 modules, both resolved despite the diagnostic"
    );

    assert!(
        !resolve_diagnostics.is_empty(),
        "DiagnosticsCollector must surface at least one diagnostic, got none"
    );
    assert!(
        resolve_diagnostics
            .iter()
            .any(|d| d.module_name == "wtd.broken"),
        "the collected diagnostic must name the offending module wtd.broken, got {:?}",
        resolve_diagnostics
            .iter()
            .map(|d| d.module_name.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        resolve_diagnostics.iter().any(|d| {
            diagnostic_to_message(d.diagnostic.clone()).contains("wtd_nonexistent_function")
        }),
        "the collected diagnostic must name the dangling symbol, got {:?}",
        resolve_diagnostics
            .iter()
            .map(|d| diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
}

/// Control: the clean module ALONE collects ZERO diagnostics under
/// DiagnosticsCollector — the gate does not fabricate diagnostics, so a non-empty
/// `resolve_diagnostics` above is caused by the broken module, not by the gate.
#[test]
fn diagnostics_collector_is_empty_on_a_clean_subtree() {
    // Exclude the broken module by subpath, leaving only the clean one.
    let WholeTreeCtx {
        resolve_diagnostics,
        modules_resolved,
        ..
    } = whole_tree_resolved_ctx(
        &[fixture_root()],
        &["broken.dag".to_string()],
        ExecutionMode::Wet,
        ResolveTypecheckGate::DiagnosticsCollector,
    )
    .expect("clean module resolves under DiagnosticsCollector");
    assert_eq!(modules_resolved, 1, "only wtd.clean is in this source root");
    assert!(
        resolve_diagnostics.is_empty(),
        "a clean subtree must collect ZERO diagnostics (no fabrication), got {:?}",
        resolve_diagnostics
            .iter()
            .map(|d| d.module_name.clone())
            .collect::<Vec<_>>()
    );
}
