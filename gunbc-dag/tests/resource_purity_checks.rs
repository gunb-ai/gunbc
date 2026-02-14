//! Registry-wide resource purity checks.
//!
//! This test links all DAG crates that currently participate in workflow
//! execution and enforces:
//! - resource access derivation succeeds
//! - no conflicting resource access pairs in the same DAG
//! - all `res:*` resource ports are wired

use gunbc_ir::{
    derive_resource_accesses, detect_resource_conflicts, validate_resource_wiring_recursive,
};
use gunbc_testgen_registry::iter_resource_tests;

// Force-link crates with `#[resource_test_target]` registrations used by CI/tooling.
use gunbc_clippy as _;
use gunbc_deps as _;
use gunbc_gist as _;
use gunbc_lib_cloud_ops as _;
use gunbc_lib_gcp_ops as _;
use gunbc_lib_llm_ops as _;
use gunbc_lib_review as _;

#[test]
fn resource_purity_registry_wide() {
    // Touch representative symbols so linker keeps object files that contain
    // inventory submissions from graph + graph_mock modules.
    let _: fn() -> gunbc_test::MockSpec = gunbc_clippy::graph_mock::clippy_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_deps::graph_mock::deps_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_snapshot_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_diff_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_recent_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_gcp_ops::graph_mock::gcp_github_mock_spec;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_lib_gcp_ops::graph_mock::gcp_github_upsert_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_llm_ops::graph_mock::openai_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::inline_review_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::diff_review_mock_spec;

    let mut defs: Vec<_> = iter_resource_tests().collect();
    defs.sort_by(|a, b| {
        (a.origin_crate, a.name)
            .cmp(&(b.origin_crate, b.name))
            .then_with(|| a.name.cmp(b.name))
    });

    assert!(
        !defs.is_empty(),
        "no resource test targets were registered in this test binary"
    );

    let mut failures = Vec::new();

    for def in defs {
        let dag = (def.build)();

        if let Err(err) = derive_resource_accesses(&dag) {
            failures.push(format!(
                "{} ({}): derive_resource_accesses failed: {:?}",
                def.name, def.origin_crate, err
            ));
            continue;
        }

        match detect_resource_conflicts(&dag) {
            Ok(conflicts) => {
                if !conflicts.is_empty() {
                    failures.push(format!(
                        "{} ({}): {} resource conflict(s): {:?}",
                        def.name,
                        def.origin_crate,
                        conflicts.len(),
                        conflicts
                    ));
                }
            }
            Err(err) => failures.push(format!(
                "{} ({}): detect_resource_conflicts failed: {:?}",
                def.name, def.origin_crate, err
            )),
        }

        let unwired = validate_resource_wiring_recursive(&dag);
        if !unwired.is_empty() {
            failures.push(format!(
                "{} ({}): {} unwired resource port(s): {:?}",
                def.name,
                def.origin_crate,
                unwired.len(),
                unwired
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "registry-wide resource purity checks failed:\n{}",
        failures.join("\n")
    );
}
