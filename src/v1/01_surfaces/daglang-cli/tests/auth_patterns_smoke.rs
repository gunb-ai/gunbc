// Non-hermetic corpus test: exercises the real `dsl/gunbc/auth/patterns.dag` through
// module discovery and import resolution. Lives in `tests/` (integration test harness)
// per the testing invariant that non-hermetic tests must not reside in `src/` unit
// test modules.
//
// Replaces the synthetic-fixture unit tests that previously lived in
// `src/compile/tests.rs` (check_target_file_gunbc_auth_patterns_requires_*).
// The generic imported-service-module contract is covered by the hermetic
// `check_target_file_requires_imported_service_modules_for_service_calls` test;
// this test proves the real file's imports resolve against the real dsl/ tree.
#![allow(clippy::disallowed_methods)]

use std::path::PathBuf;

use daglang_resolve::ModuleGraph;

fn dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl")
}

/// Verifies that the real `dsl/gunbc/auth/patterns.dag` is discoverable and its
/// imports resolve against the real `dsl/` source tree. Catches regressions where
/// auth patterns imports drift out of sync with their provider modules.
#[test]
fn real_gunbc_auth_patterns_imports_resolve() {
    let root = dsl_root();
    let graph = ModuleGraph::discover(std::slice::from_ref(&root))
        .expect("module graph discovery should succeed on real dsl/ tree");

    let patterns_module = graph
        .modules
        .iter()
        .find(|m| m.module_path.as_dotted() == "gunbc.auth.patterns")
        .expect("expected gunbc.auth.patterns to be discovered in dsl/");

    // The real file imports from these modules; verify each resolved as a dependency.
    let expected_imports = [
        "std.resources",
        "std.types",
        "extdeps.cloud.gcp.gcp",
        "extdeps.cloud.gcp.secret_manager",
        "extdeps.cloud.gcp.iam",
        "extdeps.cloud.gcp.sts",
        "extdeps.shell",
    ];

    let resolved_dep_paths: Vec<String> = patterns_module
        .dependencies
        .iter()
        .map(|&idx| graph.modules[idx].module_path.as_dotted())
        .collect();

    for expected in &expected_imports {
        assert!(
            resolved_dep_paths.contains(&expected.to_string()),
            "expected gunbc.auth.patterns to depend on {expected}, \
             resolved deps: {resolved_dep_paths:?}"
        );
    }
}
