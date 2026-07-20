//! Resolution divergence census (namespace-resolution-design.md §12.4).
//!
//! Compares `lookup_resolved_sig` (first-hit over `func_env.parents`) against the
//! landed SymbolIndex containment walk. Read-only — no policy flip.
//!
//! Run: `cargo nextest run -p v1-compiler-tests -- --ignored resolution_divergence_census_whole_tree --nocapture`

use v1_compiler::cli_run::{
    format_resolution_divergence_census, resolution_divergence_census_live,
    resolution_divergence_census_source_roots, whole_tree_resolve_exclusion_substrings,
};

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
#[ignore = "CI witness opt-in inversion: whole-tree resolve over dag+src/v1+src/v2 exceeds per-PR floor budget. Run explicitly: cargo nextest run -p v1-compiler-tests -- --ignored resolution_divergence_census_whole_tree --nocapture. Re-enroll when affected-set selection lands (namespace lane §12.4 census)."]
fn resolution_divergence_census_whole_tree() {
    let ws = workspace_root();
    let roots = resolution_divergence_census_source_roots(&ws);
    let exclude = whole_tree_resolve_exclusion_substrings();
    let census =
        resolution_divergence_census_live(&roots, &exclude).expect("whole-tree census resolve");
    let report = format_resolution_divergence_census(&census);
    println!("{report}");
    assert!(
        census.sites_checked > 0,
        "expected bare call sites in the resolved corpus"
    );
}
