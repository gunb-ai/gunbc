#![allow(clippy::disallowed_macros)]

use im::HashMap;
use std::process::ExitCode;
use std::sync::Arc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, AuthResolution, ExecutionMode, InterpError};

type ModuleIndex = HashMap<String, std::path::PathBuf>;
type WitnessCase = (&'static str, fn(&ModuleIndex));

const SERVICE_AUTH_BEARER_NO_SOURCE: &str = r#"module auth_unwired_t1

service test.Svc {
  config {
    endpoint: "https://unreachable.invalid.example"
    auth: Bearer
  }
  operation GetData {
    output { data: String }
    transport rest { method: GET, path: "/data" }
    response {
      200 => String
    }
    mock_response {
      200 => { data: "ok" }
    }
  }
}

fn probe() -> String {
  let r = test.Svc.GetData()
  r.data
}
"#;

const SERVICE_AUTH_INPUT_NOT_PROVIDED: &str = r#"module auth_unwired_t2

service test.Svc {
  config {
    endpoint: "https://unreachable.invalid.example"
    auth: Bearer
    auth_input: token
  }
  operation GetData {
    input { token: String }
    output { data: String }
    transport rest { method: GET, path: "/data" }
    response {
      200 => String
    }
    mock_response {
      200 => { data: "ok" }
    }
  }
}

fn probe() -> String {
  let r = test.Svc.GetData(token: "")
  r.data
}
"#;

// Dual-declare: both auth_input (caller-supplied) and auth_source (env-var fallback) declared.
// Used by the two fallback-regression witnesses below.
const SERVICE_DUAL_DECLARE: &str = r#"module auth_unwired_t4

service test.Svc {
  config {
    endpoint: "https://unreachable.invalid.example"
    auth: Bearer
    auth_input: api_key
    auth_source: "TEST_AUTH_GUARD_DUAL_FALLBACK_VAR"
  }
  operation GetData {
    input { api_key: String }
    output { data: String }
    transport rest { method: GET, path: "/data" }
    response {
      200 => String
    }
    mock_response {
      200 => { data: "ok" }
    }
  }
}

fn probe() -> String {
  let r = test.Svc.GetData(api_key: "")
  r.data
}
"#;

const SERVICE_NO_AUTH: &str = r#"module auth_unwired_t3

service test.Svc {
  config {
    endpoint: "https://unreachable.invalid.example"
  }
  operation GetData {
    output { data: String }
    transport rest { method: GET, path: "/data" }
    response {
      200 => String
    }
    mock_response {
      200 => { data: "ok" }
    }
  }
}

fn probe() -> String {
  let r = test.Svc.GetData()
  r.data
}
"#;

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("auth_declared_but_unwired_witness: {msg}");
    ExitCode::from(1)
}

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
}

fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        return trimmed
            .strip_prefix("module ")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }
    None
}

fn scan_dag_files(dir: &std::path::Path, index: &mut ModuleIndex) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                index.insert(module_path, path);
            }
        }
    }
}

fn build_module_index() -> ModuleIndex {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn extract_imports(source: &str) -> Vec<String> {
    let tokens =
        v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string());
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    let result = v1_compiler::v1_compiler_parse::parse(tokens, Arc::new(source_indices));
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
    module_index: &ModuleIndex,
) -> Vec<Arc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Arc<SourceFile>> = HashMap::new();
    let mut queue = vec![(entry_path.to_string(), entry_content.to_string())];

    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_imports(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    seen.insert(
                        module_path.clone(),
                        Arc::new(SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push((rel_path, file_content));
                }
            }
        }
    }

    let mut sources: Vec<Arc<SourceFile>> = seen.into_iter().map(|(_, v)| v).collect();
    sources.push(Arc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?}",
        msgs,
    );
}

fn resolve(module_index: &ModuleIndex, src: &str) -> Arc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src, module_index);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

fn auth_declared_no_source_fails_closed_pre_send(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_AUTH_BEARER_NO_SOURCE);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::AuthDeclaredButUnwired { service, reason }) => {
            assert!(
                service.contains("unreachable.invalid.example"),
                "expected service endpoint in error, got service='{service}' reason='{reason}'"
            );
        }
        other => panic!(
            "expected AuthDeclaredButUnwired pre-send, got {other:?} — \
             guard did not fire before dispatch"
        ),
    }
}

fn auth_input_empty_fails_closed_pre_send(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_AUTH_INPUT_NOT_PROVIDED);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::AuthDeclaredButUnwired { .. }) => {}
        other => panic!("expected AuthDeclaredButUnwired for empty auth_input, got {other:?}"),
    }
}

fn no_auth_declared_does_not_fire_guard(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_NO_AUTH);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    if let Err(InterpError::AuthDeclaredButUnwired { service, reason }) =
        v1_interpreter::run_in_context(&ctx, "probe", false)
    {
        panic!(
            "guard must NOT fire for a service with no auth declaration; \
             got AuthDeclaredButUnwired service='{service}' reason='{reason}'"
        );
    }
}

fn resolve_auth_three_way_split_matches_dispatch_behavior(module_index: &ModuleIndex) {
    let cases: &[(&str, &str, bool)] = &[
        ("Bearer + no source", SERVICE_AUTH_BEARER_NO_SOURCE, true),
        ("auth_input empty", SERVICE_AUTH_INPUT_NOT_PROVIDED, true),
        ("no auth declared", SERVICE_NO_AUTH, false),
    ];
    for (label, src, expect_unwired) in cases {
        let resolved = resolve(module_index, src);
        let graph = resolved.graph.as_ref().expect("graph");
        let ctx = v1_interpreter::InterpContext::new(
            graph,
            resolved.source_indices.clone(),
            ExecutionMode::Wet,
        );
        let result = v1_interpreter::run_in_context(&ctx, "probe", false);
        let is_unwired = matches!(&result, Err(InterpError::AuthDeclaredButUnwired { .. }));
        assert_eq!(
            is_unwired, *expect_unwired,
            "{label}: expect_unwired={expect_unwired} but got {result:?}"
        );
    }
}

// Regression guard: dual-declare (auth_input + auth_source), api_key empty but env var present →
// must fall through to auth_source and NOT raise AuthDeclaredButUnwired.
fn dual_declare_env_var_fallback_resolves_when_input_empty(module_index: &ModuleIndex) {
    // Set a synthetic env var the service fixture reads.
    std::env::set_var("TEST_AUTH_GUARD_DUAL_FALLBACK_VAR", "test-token-sentinel");
    let resolved = resolve(module_index, SERVICE_DUAL_DECLARE);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let result = v1_interpreter::run_in_context(&ctx, "probe", false);
    std::env::remove_var("TEST_AUTH_GUARD_DUAL_FALLBACK_VAR");
    // Auth resolved via env-var fallback → guard must NOT fire; a network error is acceptable
    // (the endpoint is unreachable) but AuthDeclaredButUnwired is the regression.
    if let Err(InterpError::AuthDeclaredButUnwired { service, reason }) = result {
        panic!(
            "regression: guard fired on dual-declare with env-var present; \
             auth_input→auth_source fallback broken. service='{service}' reason='{reason}'"
        );
    }
}

// Dual-declare, api_key empty AND env var absent → guard must still fire (fail-closed).
fn dual_declare_both_empty_fails_closed(module_index: &ModuleIndex) {
    std::env::remove_var("TEST_AUTH_GUARD_DUAL_FALLBACK_VAR");
    let resolved = resolve(module_index, SERVICE_DUAL_DECLARE);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::AuthDeclaredButUnwired { .. }) => {}
        other => panic!(
            "expected AuthDeclaredButUnwired when both auth_input and auth_source fail, \
             got {other:?}"
        ),
    }
}

// Pub-API smoke: confirms the 3 variants are reachable from outside v1_compiler.
// Execution discrimination lives in the wet-dispatch tests above.
fn auth_resolution_enum_is_pub_and_discriminable(_module_index: &ModuleIndex) {
    let _ = AuthResolution::NoAuthDeclared;
    let _ = AuthResolution::Resolved {
        header: "Authorization".to_string(),
        token: "tok".to_string(),
    };
    let _ = AuthResolution::DeclaredButUnwired {
        reason: "test".to_string(),
    };
}

fn main() -> ExitCode {
    let module_index = build_module_index();

    let tests: Vec<WitnessCase> = vec![
        (
            "auth_declared_no_source_fails_closed_pre_send",
            auth_declared_no_source_fails_closed_pre_send,
        ),
        (
            "auth_input_empty_fails_closed_pre_send",
            auth_input_empty_fails_closed_pre_send,
        ),
        (
            "no_auth_declared_does_not_fire_guard",
            no_auth_declared_does_not_fire_guard,
        ),
        (
            "resolve_auth_three_way_split_matches_dispatch_behavior",
            resolve_auth_three_way_split_matches_dispatch_behavior,
        ),
        (
            "dual_declare_env_var_fallback_resolves_when_input_empty",
            dual_declare_env_var_fallback_resolves_when_input_empty,
        ),
        (
            "dual_declare_both_empty_fails_closed",
            dual_declare_both_empty_fails_closed,
        ),
        (
            "auth_resolution_enum_is_pub_and_discriminable",
            auth_resolution_enum_is_pub_and_discriminable,
        ),
    ];

    for (name, test) in tests {
        let index = module_index.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&index)));
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }

    ExitCode::SUCCESS
}
