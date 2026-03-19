// Non-hermetic integration test: exercises the target-file provider-binding
// contract through `check_from_context` with synthetic `.dag` fixtures written
// to a temp directory. Lives in `tests/` (integration test harness) per the
// testing invariant that non-hermetic tests must not reside in `src/` unit test
// modules.
//
// Proves that service calls in a target file require the corresponding service
// module imports to be present — removing the imports causes an unresolved
// service call error at the typecheck stage.
#![allow(clippy::disallowed_methods)]

use daglang_cli::compile::{check_from_context, CompileError};
use daglang_cli::pipeline::PipelineContext;
use gunbc_test::unique_temp_dir;

fn unique_temp_root(name: &str) -> std::path::PathBuf {
    let root = unique_temp_dir(name);
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    root
}

#[test]
fn check_target_file_requires_imported_service_modules_for_service_calls() {
    let root = unique_temp_root("target_file_imported_service_modules");
    let write_source = |relative: &str, content: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture file should have parent"))
            .expect("failed to create fixture parent directory");
        std::fs::write(path, content).expect("failed to write fixture source");
    };

    write_source(
        "providers/identity.dag",
        r#"module providers.identity
service acme.Identity {
  operation IssueToken(audience: String) -> { token: String }
}
"#,
    );
    write_source(
        "providers/snippets.dag",
        r#"module providers.snippets
service contoso.Snippets {
  operation Create(description: String, body: String, auth_token: String) -> { url: String }
}
"#,
    );
    write_source(
        "sample/main.dag",
        r#"module sample.main

import providers.identity
import providers.snippets

func run() -> { token: String, url: String } {
  issued = acme.Identity.IssueToken(audience: "dag")
  created = contoso.Snippets.Create(description: "snapshot", body: "body", auth_token: issued.token)
  return { token: issued.token, url: created.url }
}
"#,
    );

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: Some(root.join("sample/main.dag")),
    };

    let output =
        check_from_context(&context).expect("check should succeed with imported service modules");
    assert_eq!(
        output.parsed_files, 3,
        "expected target file plus imported service modules"
    );

    write_source(
        "sample/main.dag",
        r#"module sample.main

func run() -> { token: String, url: String } {
  issued = acme.Identity.IssueToken(audience: "dag")
  created = contoso.Snippets.Create(description: "snapshot", body: "body", auth_token: issued.token)
  return { token: issued.token, url: created.url }
}
"#,
    );

    let error =
        check_from_context(&context).expect_err("check should fail without service modules");
    assert!(
        matches!(error, CompileError::Diagnostics(_)),
        "expected CompileError::Diagnostics, got: {error}"
    );
    assert!(error.contains("unresolved service call"));
    for service_call in ["acme.Identity.IssueToken", "contoso.Snippets.Create"] {
        assert!(
            error.contains(service_call),
            "expected missing import error to mention {service_call}: {error}"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}
