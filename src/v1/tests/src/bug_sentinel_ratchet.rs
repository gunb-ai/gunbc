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
    // Host-physics oracle (51 tests); not the thin floor .dag wrapper.
    let infer_semantics_oracle = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../stage0/src/bin/infer_semantics_witness.rs"
    ));
    assert!(
        !infer_semantics_oracle.contains(SENTINEL),
        "infer_semantics_witness.rs must not contain fabrication fallback"
    );
}
