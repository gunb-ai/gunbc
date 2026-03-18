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

fn assert_missing_provider_binding_fails_check(
    import_line: &str,
    replacement: &str,
    missing_service_call: &str,
) {
    let tmp = copy_dsl_to_temp();
    let patterns = tmp.join("gunbc/auth/patterns.dag");

    let original = std::fs::read_to_string(&patterns).expect("read patterns.dag");
    let modified = original.replacen(import_line, replacement, 1);
    assert_ne!(
        original, modified,
        "replacement did not match — patterns.dag import line may have changed"
    );
    std::fs::write(&patterns, &modified).expect("write modified patterns.dag");

    let context = DriverContext {
        roots: vec![tmp.clone()],
        target_file: Some(patterns),
    };

    let result = check_from_context(&context);
    let _ = std::fs::remove_dir_all(&tmp);
    let error = result.expect_err("check should fail when a required provider binding is removed");

    assert!(
        error.contains("unresolved service call"),
        "expected unresolved service call error, got: {error}"
    );
    assert!(
        error.contains(missing_service_call),
        "expected missing binding error to mention {missing_service_call}: {error}"
    );
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

/// Proves that each explicit provider binding in the real auth-patterns file is
/// compiler-enforced: removing any one binding must fail the check/typecheck
/// pipeline on the corresponding service call.
#[test]
fn removing_required_provider_bindings_from_auth_patterns_fails_check() {
    for (import_line, replacement, missing_service_call) in [
        (
            "import extdeps.cloud.gcp.gcp { shell.OAuth2, shell.GCloud }\n",
            "import extdeps.cloud.gcp.gcp { shell.GCloud }\n",
            "shell.OAuth2.RefreshToken",
        ),
        (
            "import extdeps.cloud.gcp.gcp { shell.OAuth2, shell.GCloud }\n",
            "import extdeps.cloud.gcp.gcp { shell.OAuth2 }\n",
            "shell.GCloud.AuthPrintAccessToken",
        ),
        (
            "import extdeps.cloud.gcp.secret_manager { gcp.SecretManager }\n",
            "",
            "gcp.SecretManager.AccessVersion",
        ),
        (
            "import extdeps.cloud.gcp.iam { gcp.IAM }\n",
            "",
            "gcp.IAM.GenerateAccessToken",
        ),
        (
            "import extdeps.cloud.gcp.sts { gcp.STS, github.OIDC, gcp.Metadata }\n",
            "import extdeps.cloud.gcp.sts { github.OIDC, gcp.Metadata }\n",
            "gcp.STS.Exchange",
        ),
        (
            "import extdeps.cloud.gcp.sts { gcp.STS, github.OIDC, gcp.Metadata }\n",
            "import extdeps.cloud.gcp.sts { gcp.STS, gcp.Metadata }\n",
            "github.OIDC.GetToken",
        ),
        (
            "import extdeps.cloud.gcp.sts { gcp.STS, github.OIDC, gcp.Metadata }\n",
            "import extdeps.cloud.gcp.sts { gcp.STS, github.OIDC }\n",
            "gcp.Metadata.GetIdentityToken",
        ),
        (
            "import extdeps.shell { shell.Env }\n",
            "",
            "shell.Env.Get",
        ),
    ] {
        assert_missing_provider_binding_fails_check(
            import_line,
            replacement,
            missing_service_call,
        );
    }
}
