//! Fail if `__BUG_NO_PROFILE_` fabrication sentinel is reintroduced (P0-C).

#[test]
fn no_bug_no_profile_sentinel_in_tracked_sources() {
    let types_dag = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../dsl/std/types.dag"
    ));
    assert!(
        !types_dag.contains("__BUG_NO_PROFILE_"),
        "dsl/std/types.dag must not contain __BUG_NO_PROFILE_ sentinel"
    );
    let infer_semantics = include_str!("infer_semantics.rs");
    assert!(
        !infer_semantics.contains("__BUG_NO_PROFILE_"),
        "infer_semantics.rs must not contain __BUG_NO_PROFILE_ fallback"
    );
}
