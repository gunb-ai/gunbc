// Non-hermetic corpus test: exercises the real `dsl/gunbc/auth/patterns.dag` through
// the full check pipeline (discovery + typecheck with strict imports). Lives in
// `tests/` (integration test harness) per the testing invariant that non-hermetic
// tests must not reside in `src/` unit test modules.
//
// Replaces the synthetic-fixture unit tests that previously lived in
// `src/compile/tests.rs` (check_target_file_gunbc_auth_patterns_requires_*).
// The generic imported-service-module contract is covered by the hermetic
// `check_target_file_requires_imported_service_modules_for_service_calls` test;
// this test proves the real file's imports resolve and typecheck against the real
// dsl/ tree, catching drift in provider bindings and service call signatures.
#![allow(clippy::disallowed_methods)]

use std::path::PathBuf;

use daglang_driver::{check_from_context, DriverContext};

fn dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl")
}

/// Verifies that the real `dsl/gunbc/auth/patterns.dag` passes the full check
/// pipeline (discovery + typecheck) against the real `dsl/` source tree with
/// `target_file` set and strict import resolution.
///
/// Catches regressions where auth pattern service calls drift out of sync with
/// their provider definitions in `extdeps.cloud.gcp.sts`, `extdeps.cloud.gcp.gcp`,
/// `extdeps.cloud.gcp.iam`, `extdeps.cloud.gcp.secret_manager`, or `extdeps.shell`.
#[test]
fn real_gunbc_auth_patterns_check_typechecks() {
    let root = dsl_root();
    let target = root.join("gunbc/auth/patterns.dag");

    let context = DriverContext {
        roots: vec![root],
        target_file: Some(target),
    };

    let output = check_from_context(&context)
        .expect("check/typecheck should succeed on real dsl/gunbc/auth/patterns.dag");

    // The target file plus its transitive imports (at least std.resources, std.types,
    // extdeps.cloud.gcp.gcp, extdeps.cloud.gcp.secret_manager,
    // extdeps.cloud.gcp.iam, extdeps.cloud.gcp.sts, extdeps.shell).
    assert!(
        output.parsed_files >= 8,
        "expected at least 8 parsed files (target + 7 imports), got {}",
        output.parsed_files
    );
}
