//! Refusal witness for the resolved-graph materialization-provider consumer seam.
//!
//! The live route-A helper still refuses faithfully when the resolved-graph
//! disk probe is unavailable, and that refusal must not boot the provider
//! context. This module owns that witness so the consumer seam has a floor
//! anchor instead of living as an unbound stub.
//!
//! Checkable receipt: `dag/test/retirement/materialization_provider_resolved_graph_consumer_retained.dag`.

use v1_compiler::cli_run::{
    materialization_provider_ctx_build_count_for_test, reset_materialization_provider_ctx_for_test,
    serve_resolved_graph_v1_disk_probe_for_test,
};

#[test]
fn faithful_probe_refusal_does_not_build_provider_ctx() {
    reset_materialization_provider_ctx_for_test();
    let builds_before = materialization_provider_ctx_build_count_for_test();

    let err = serve_resolved_graph_v1_disk_probe_for_test("closure-digest", "compiler-digest")
        .expect_err("unavailable faithful probe must refuse");
    assert!(
        err.contains("provider refused faithful probe"),
        "refusal should name the faithful-probe gap: {err}"
    );
    assert_eq!(
        materialization_provider_ctx_build_count_for_test(),
        builds_before,
        "faithful-probe refusal must not build provider ctx"
    );
}
