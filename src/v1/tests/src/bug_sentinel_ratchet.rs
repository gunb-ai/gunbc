#[test]
fn no_bug_no_profile_sentinel_in_tracked_sources() {
    const SENTINEL: &str = concat!("__BUG", "_NO_PROFILE_");
    let types_dag = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../dsl/std/types.dag"
    ));
    assert!(
        !types_dag.contains(SENTINEL),
        "dsl/std/types.dag must not contain fabrication sentinel"
    );
    let infer_semantics = include_str!("infer_semantics.rs");
    assert!(
        !infer_semantics.contains(SENTINEL),
        "infer_semantics.rs must not contain fabrication fallback"
    );
}
