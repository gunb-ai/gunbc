// Non-hermetic corpus test: exercises the real `dsl/gunbc/auth/patterns.dag` through
// the full check pipeline (discovery + typecheck with strict imports). Lives in
// `tests/` (integration test harness) per the testing invariant that non-hermetic
// tests must not reside in `src/` unit test modules.
//
// Replaces the synthetic-fixture unit tests that previously lived in
// `src/compile/tests.rs` (check_target_file_gunbc_auth_patterns_requires_*).
// The generic imported-service-module contract is covered by the non-hermetic
// `check_target_file_requires_imported_service_modules_for_service_calls` test
// in `src/v1/01_surfaces/daglang-cli/tests/provider_binding_contract.rs` (writes
// synthetic fixtures to a temp dir);
// this test proves the real file's imports resolve and typecheck against the real
// dsl/ tree, catching drift in provider bindings and service call signatures.
#![allow(clippy::disallowed_methods)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use daglang_contract::DiagnosticContext;
use daglang_driver::{check_from_context, CompileError, DriverContext};
use daglang_syntax::{ast::SourceFile, parser::parse};

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

fn parse_source_file(path: &std::path::Path) -> (String, SourceFile) {
    let source = std::fs::read_to_string(path).expect("read dag source");
    let ast = parse(&source)
        .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:#?}", path.display()));
    (source, ast)
}

fn parsed_import_binding_sets(path: &std::path::Path) -> BTreeMap<String, BTreeSet<String>> {
    let (_, ast) = parse_source_file(path);
    let mut imports = BTreeMap::new();

    for import in ast.imports {
        let import = import.node;
        let Some(bindings) = import.bindings else {
            continue;
        };
        let module = import.path.as_dotted();
        let previous = imports.insert(module.clone(), bindings.into_iter().collect());
        assert!(
            previous.is_none(),
            "expected a single import entry for {module}"
        );
    }

    imports
}

fn source_with_removed_import_binding(
    path: &std::path::Path,
    module: &str,
    binding: &str,
) -> String {
    let (source, ast) = parse_source_file(path);
    let mut matching_imports = ast
        .imports
        .iter()
        .filter(|import| import.node.path.as_dotted() == module);
    let import = matching_imports
        .next()
        .unwrap_or_else(|| panic!("expected auth patterns to import {module}"));
    assert!(
        matching_imports.next().is_none(),
        "expected a single import entry for {module}"
    );

    let bindings = import
        .node
        .bindings
        .as_ref()
        .unwrap_or_else(|| panic!("expected {module} to use explicit bindings"));
    let remaining: Vec<_> = bindings
        .iter()
        .filter(|candidate| candidate.as_str() != binding)
        .cloned()
        .collect();
    assert_ne!(
        remaining.len(),
        bindings.len(),
        "expected {module} to import binding `{binding}`"
    );

    let replacement = if remaining.is_empty() {
        String::new()
    } else {
        format!("import {module} {{ {} }}", remaining.join(", "))
    };

    let mut modified = String::with_capacity(
        source.len() - (import.span.end - import.span.start) + replacement.len(),
    );
    modified.push_str(&source[..import.span.start]);
    modified.push_str(&replacement);
    modified.push_str(&source[import.span.end..]);
    modified
}

fn assert_missing_provider_binding_fails_check(
    module: &str,
    binding: &str,
    missing_service_call: &str,
) {
    let tmp = copy_dsl_to_temp();
    let patterns = tmp.join("gunbc/auth/patterns.dag");

    let modified = source_with_removed_import_binding(&patterns, module, binding);
    std::fs::write(&patterns, &modified).expect("write modified patterns.dag");

    let context = DriverContext {
        roots: vec![tmp.clone()],
        target_file: Some(patterns),
    };

    let result = check_from_context(&context);
    let _ = std::fs::remove_dir_all(&tmp);
    let error = result.expect_err("check should fail when a required provider binding is removed");

    let diagnostics = match &error {
        CompileError::Diagnostics(d) => &d.errors,
        other => panic!("expected Diagnostics error, got: {other}"),
    };

    let has_unresolved = diagnostics.iter().any(|d| {
        d.code == "TC026"
            && matches!(
                &d.context,
                DiagnosticContext::Missing { name, .. } if name == missing_service_call
            )
    });
    assert!(
        has_unresolved,
        "expected a TC026 (UnresolvedServiceCall) diagnostic for `{missing_service_call}`, \
         got diagnostics: {diagnostics:#?}"
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
        roots: vec![root.clone()],
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

    // Assert explicit imported provider symbols for multi-provider modules.
    // The parsed_files count above is module-level; these parsed-import assertions
    // pin the exact provider bindings that the compiler sees without depending on
    // source formatting. Combined with the per-symbol removal test below, this
    // proves the complete symbol set is both present and compiler-enforced.
    let import_bindings = parsed_import_binding_sets(&root.join("gunbc/auth/patterns.dag"));
    assert!(
        import_bindings.get("extdeps.cloud.gcp.gcp")
            == Some(&BTreeSet::from([
                "shell.OAuth2".to_string(),
                "shell.GCloud".to_string()
            ])),
        "extdeps.cloud.gcp.gcp must import exactly shell.OAuth2 and shell.GCloud"
    );
    assert!(
        import_bindings.get("extdeps.cloud.gcp.sts")
            == Some(&BTreeSet::from([
                "gcp.STS".to_string(),
                "github.OIDC".to_string(),
                "gcp.Metadata".to_string(),
            ])),
        "extdeps.cloud.gcp.sts must import exactly gcp.STS, github.OIDC, and gcp.Metadata"
    );
}

/// Proves that each explicit provider binding in the real auth-patterns file is
/// compiler-enforced: removing any one binding must fail the check/typecheck
/// pipeline on the corresponding service call.
#[test]
fn removing_required_provider_bindings_from_auth_patterns_fails_check() {
    for (module, binding, missing_service_call) in [
        (
            "extdeps.cloud.gcp.gcp",
            "shell.OAuth2",
            "shell.OAuth2.RefreshToken",
        ),
        (
            "extdeps.cloud.gcp.gcp",
            "shell.GCloud",
            "shell.GCloud.AuthPrintAccessToken",
        ),
        (
            "extdeps.cloud.gcp.secret_manager",
            "gcp.SecretManager",
            "gcp.SecretManager.AccessVersion",
        ),
        (
            "extdeps.cloud.gcp.iam",
            "gcp.IAM",
            "gcp.IAM.GenerateAccessToken",
        ),
        (
            "extdeps.cloud.gcp.sts",
            "gcp.STS",
            "gcp.STS.Exchange",
        ),
        (
            "extdeps.cloud.gcp.sts",
            "github.OIDC",
            "github.OIDC.GetToken",
        ),
        (
            "extdeps.cloud.gcp.sts",
            "gcp.Metadata",
            "gcp.Metadata.GetIdentityToken",
        ),
        ("extdeps.shell", "shell.Env", "shell.Env.Get"),
    ] {
        assert_missing_provider_binding_fails_check(module, binding, missing_service_call);
    }
}
