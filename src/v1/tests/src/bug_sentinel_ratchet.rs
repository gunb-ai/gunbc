#[test]
fn no_bug_no_profile_sentinel_in_tracked_sources() {
    const SENTINEL: &str = concat!("__BUG", "_NO_PROFILE_");
    let types_dag = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../dag/std/types.dag"
    ));
    assert!(
        !types_dag.contains(SENTINEL),
        "dag/std/types.dag must not contain fabrication sentinel"
    );
    let witness_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../src/v2/test/claim/infer_semantics_witness_test.dag"
    ));
    assert!(
        !witness_source.contains(SENTINEL),
        "infer_semantics_witness_test.dag must not contain fabrication fallback"
    );
}
