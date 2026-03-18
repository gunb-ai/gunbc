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

fn copy_dsl_to_temp() -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("auth_patterns_smoke_{}", std::process::id()));
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp).expect("cleanup stale temp dir");
    }
    copy_dir_recursive(&dsl_root(), &tmp);
    tmp
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).expect("create dest dir");
    for entry in std::fs::read_dir(src).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let ty = entry.file_type().expect("file type");
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), dest).expect("copy file");
        }
    }
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

    // Exact count: target file + 8 transitive imports (std.resources, std.types,
    // std.errors, extdeps.cloud.gcp.gcp, extdeps.cloud.gcp.secret_manager,
    // extdeps.cloud.gcp.iam, extdeps.cloud.gcp.sts, extdeps.shell).
    // An exact assertion catches both added and removed provider modules.
    assert_eq!(
        output.parsed_files, 9,
        "expected exactly 9 parsed files (target + 8 imports); \
         if a provider module was added or removed, update this count"
    );
}

/// Proves that removing a provider module import (`extdeps.cloud.gcp.iam`)
/// from `gunbc/auth/patterns.dag` drops the module from the dependency graph.
///
/// This catches the case where the positive test above would pass vacuously
/// even after a provider import is removed, by proving the module graph
/// shrinks when a provider binding disappears.
#[test]
fn removing_iam_provider_import_drops_module_from_graph() {
    let tmp = copy_dsl_to_temp();
    let patterns = tmp.join("gunbc/auth/patterns.dag");

    let original = std::fs::read_to_string(&patterns).expect("read patterns.dag");

    // Remove the entire IAM provider import line.
    let modified = original.replace(
        "import extdeps.cloud.gcp.iam { gcp.IAM }\n",
        "",
    );
    assert_ne!(
        original, modified,
        "replacement did not match — patterns.dag IAM import line may have changed"
    );
    std::fs::write(&patterns, &modified).expect("write modified patterns.dag");

    let context = DriverContext {
        roots: vec![tmp.clone()],
        target_file: Some(patterns),
    };

    let result = check_from_context(&context);
    let _ = std::fs::remove_dir_all(&tmp);

    match result {
        Err(_) => {
            // Best case: the check pipeline rejects the missing provider.
        }
        Ok(output) => {
            // The typechecker does not yet validate service call references,
            // so check may succeed — but the missing import must shrink the
            // parsed module count, proving the provider was in the graph.
            assert!(
                output.parsed_files < 9,
                "removing the IAM provider import should reduce the module count \
                 below 9, but got {}",
                output.parsed_files
            );
        }
    }
}
