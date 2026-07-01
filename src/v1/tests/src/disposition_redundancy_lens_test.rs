// Green-by-execution proof for the disposition redundancy lens (DESIGN §3 / plan §3 of
// docs/plans/disposition-carrier.md). The lens (src/v2/lens/disposition_redundancy.dag) reads
// std.disposition (dsl) and the region-1 marks (extdeps.llm.anthropic, dsl), so resolution needs
// BOTH source roots.
//
// The §5 non-vacuity requirement: a green-only test proves nothing. This asserts a DISCRIMINATING
// pair — a synthetic Scaffold whose successor is present fires RED (count == 1), and the SAME
// Scaffold with the successor absent flips GREEN (count == 0) — plus the region-1 marks staying
// green against a non-matching present-set, and being non-empty (red-on-revert: reverting the
// anthropic migration to List<String> breaks the import).

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn cross_tree_source_roots() -> Vec<std::path::PathBuf> {
    vec![
        workspace_root().join("dsl"),
        workspace_root().join("src/v2"),
    ]
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn assert_witness_true(entry: &str, witness_fn: &str) {
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    let sources: Vec<std::rc::Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots(entry, &content, &cross_tree_source_roots());
    let resolved = compile_to_resolved(std::rc::Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), witness_fn)
        .unwrap_or_else(|e| panic!("run {witness_fn}: {e:?}"))
    {
        Value::Bool(true) => {}
        other => panic!("expected {witness_fn} true, got {other:?}"),
    }
}

#[test]
fn disposition_redundancy_lens_discriminates_by_execution() {
    let test_entry = "src/v2/lens/disposition_redundancy_test.dag";
    // Red control: a Scaffold whose successor IS present fires.
    assert_witness_true(
        test_entry,
        "redundancy_red_control_fires_when_successor_present",
    );
    // Flip: the SAME Scaffold with the successor absent goes green.
    assert_witness_true(
        test_entry,
        "redundancy_red_control_flips_green_when_successor_absent",
    );
    // Region-1 marks: green against a non-matching present-set (no false positive).
    assert_witness_true(
        test_entry,
        "redundancy_region1_anthropic_green_against_nonmatching_present",
    );
    // Region-1 wiring / red-on-revert: the migrated marks are non-empty and typed.
    assert_witness_true(test_entry, "redundancy_region1_anthropic_marks_nonempty");
    // Discrimination on REAL region-1 marks (not synthetic): a real mark's successor present fires.
    assert_witness_true(
        test_entry,
        "redundancy_region1_fires_when_a_real_mark_successor_present",
    );
    // Slice-2: bytes/encoding markers non-empty and two Scaffolds fire on synthetic present.
    assert_witness_true(test_entry, "redundancy_region_bytes_marks_nonempty");
    assert_witness_true(
        test_entry,
        "redundancy_region_bytes_fires_on_builtin_registry_present",
    );
    // Slice-2: budget-tree markers non-empty and fire on job_peaks locator.
    assert_witness_true(test_entry, "redundancy_region_budget_marks_nonempty");
    assert_witness_true(
        test_entry,
        "redundancy_region_budget_fires_on_job_peaks_present",
    );
    // Slice-2: rust_stage0_gates markers non-empty and fire on synthetic present.
    assert_witness_true(test_entry, "redundancy_region_rust_gates_marks_nonempty");
    assert_witness_true(
        test_entry,
        "redundancy_region_rust_gates_fires_on_convergence_target_present",
    );
}
