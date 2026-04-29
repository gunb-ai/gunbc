//! **Layer:** integration
//!
//! Acceptance for Prereq-3a (`workflow_root_port` accessor) per the
//! merged audit at `docs/design-lens-fold-prerequisites.md`. Director-
//! locked α implementation: last topological `Bind` in `d.nodes`.
//!
//! Three claims pin the α partition over `WorkflowRoot`:
//! - `workflow_root_single_bind_returns_single_root`
//! - `workflow_root_zero_bind_returns_no_root`
//! - `workflow_root_multi_bind_returns_single_under_alpha`
//!   (renamed from the audit's `_returns_ambiguous` since linear
//!    `d.nodes` cannot produce ambiguity under α; the test pins
//!    that α picks the LAST Bind even with multiple Binds present).
//!
//! `WorkflowRoot::AmbiguousRoot` cannot be exercised by the α
//! implementation today; `workflow_root_ambiguous_unreachable_under_alpha`
//! pins that claim explicitly so a future enumerate-all-eligible-entries
//! rule landing under the same accessor causes the test to fail loudly,
//! forcing the rule's behavior to grow its own coverage.
//!
//! The fourth audit claim — `workflow_root_consumed_by_runtime_entry_point`
//! — is deferred to the R2-Evaluator integration PR; this slice
//! only authors the substrate accessor.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId, WorkflowRoot};

fn last_bind_result_port(dag: &Dag) -> PortId {
    for behavior in dag.nodes().iter().rev() {
        if let Behavior::Bind(b) = behavior {
            return b.result_port();
        }
    }
    panic!("test fixture has no Bind — adjust source");
}

#[test]
fn workflow_root_single_bind_returns_single_root() {
    let dag = cached_compile_to_dag("let x = 1 + 2", "workflow_root_single.v3");
    let expected = last_bind_result_port(&dag);
    let root = dag.workflow_root_port();
    assert_eq!(
        root,
        WorkflowRoot::SingleRoot(expected),
        "single Bind must return SingleRoot pointing at its result_port"
    );
}

#[test]
fn workflow_root_multi_bind_returns_single_under_alpha() {
    // Two top-level Binds in source order. Under α (last topological
    // Bind), the second Bind's result_port is the workflow root.
    // AmbiguousRoot is intentionally NOT emitted — it's reserved for
    // the future enumerate-all-eligible-entries rule.
    let dag = cached_compile_to_dag("let x = 1\nlet y = x + 2", "workflow_root_multi.v3");
    let expected = last_bind_result_port(&dag);
    let root = dag.workflow_root_port();
    assert_eq!(
        root,
        WorkflowRoot::SingleRoot(expected),
        "α picks the last topological Bind even with multiple Binds; \
         AmbiguousRoot reserved for the enumerate-all rule"
    );
}

#[test]
fn workflow_root_ambiguous_unreachable_under_alpha() {
    // Drift trigger: under α with linear d.nodes, AmbiguousRoot is
    // structurally unreachable. If a future commit makes
    // workflow_root_port emit AmbiguousRoot, this test fails and
    // forces the change to grow its own enumerate-all-rule coverage
    // rather than silently inheriting α's tests.
    let fixtures = [
        ("let x = 1", "wf_root_amb_a.v3"),
        ("let x = 1\nlet y = 2\nlet z = x + y", "wf_root_amb_b.v3"),
    ];
    for (src, file) in fixtures.iter() {
        let dag = cached_compile_to_dag(src, file);
        let root = dag.workflow_root_port();
        assert!(
            !matches!(root, WorkflowRoot::AmbiguousRoot { .. }),
            "fixture `{file}` produced AmbiguousRoot under α — α is a \
             single-pick rule over linear d.nodes and must never tie. \
             If this fires, an enumerate-all-eligible-entries rule has \
             been wired and the audit's ambiguous-acceptance must move \
             to its own consumer test."
        );
    }
}

// `workflow_root_zero_bind_returns_no_root` lives as a `#[cfg(test)]`
// unit test inside `src/v3/compiler/src/dag.rs` next to
// `Dag::workflow_root_port` itself, because constructing a truly
// empty `Dag` requires the crate-private `Dag::empty()` constructor.
// V3 surface syntax always lowers each top-level decl to a `Bind`, so
// the zero-Bind case is structurally unreachable from `compile_to_dag`
// fixtures and the `NoRoot` arm is defensive-only at the substrate
// boundary.
//
// See: dag.rs `mod tests::workflow_root_zero_bind_returns_no_root`.
