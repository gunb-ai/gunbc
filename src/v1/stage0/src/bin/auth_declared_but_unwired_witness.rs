#![allow(clippy::disallowed_macros)]

use im::HashMap;
use std::process::ExitCode;
use std::sync::Arc as Rc;

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

// A service endpoint spelled as a DATA REFERENCE rather than a literal. This is the
// shape thirteen dag/extdeps services use (production_api_base, default_api_base,
// docker_default_endpoint, edgar_data_api_base) and none of them could execute: the
// config read returned a value only for a string literal and otherwise fell through
// to the identifier's own SOURCE TEXT, so the base URL became the string "svc_base".
//
// The auth guard is the observation point because it fires pre-send and reports the
// endpoint it was about to use, so this needs no network. On the pre-fix seed the
// reported service is "svc_base"; after the fix it is the resolved host. That
// difference is the whole discrimination — a literal-endpoint case cannot express it,
// which is why every existing case here passed while the class was broken.
const SERVICE_ENDPOINT_BY_REFERENCE: &str = r#"module auth_unwired_t8

data svc_base: String = "https://unreachable.invalid.example"

service test.Svc {
  config {
    endpoint: svc_base
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

// The refusal arm. An endpoint that resolves to nothing must STOP, not proceed with an
// empty base — an empty base and a garbage base produce the identical downstream
// RelativeUrlWithoutBase, which is precisely how the reference defect stayed invisible.
const SERVICE_ENDPOINT_RESOLVES_EMPTY: &str = r#"module auth_unwired_t9

data svc_base_empty: String = ""

service test.Svc {
  config {
    endpoint: svc_base_empty
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

// The second slip path, found in review of the first fix (review 45550). The empty check
// above runs on a Display rendering, and Display is total over Value: Null renders as the
// non-empty "null" and an Int renders as its digits, so a non-string endpoint would clear
// the emptiness test and be sent as a base URL. An Int is the fixture because it makes the
// rendering visibly plausible — "8080" looks like configuration, not like a defect.
const SERVICE_ENDPOINT_RESOLVES_NON_STRING: &str = r#"module auth_unwired_t10

data svc_base_port: Int = 8080

service test.Svc {
  config {
    endpoint: svc_base_port
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

// The third slip path (review 45552). The two above cover a key that is PRESENT and
// unreadable; this covers a key that is absent entirely, which reached `String::new()`
// and sent the bare path as a relative URL. It gets its own error rather than sharing
// ServiceConfigUnresolved: "declared nothing" and "declared something unreadable" are
// different authoring mistakes whose fixes name different edits.
const SERVICE_ENDPOINT_ABSENT: &str = r#"module auth_unwired_t11

service test.Svc {
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
    let result = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
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
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
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
                        Rc::new(SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push((rel_path, file_content));
                }
            }
        }
    }

    let mut sources: Vec<Rc<SourceFile>> = seen.into_iter().map(|(_, v)| v).collect();
    sources.push(Rc::new(SourceFile {
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

fn resolve(module_index: &ModuleIndex, src: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src, module_index);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
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

fn endpoint_by_reference_resolves_to_its_value(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_ENDPOINT_BY_REFERENCE);
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
                "a data-reference endpoint must resolve to its VALUE; got service='{service}' \
                 reason='{reason}' — a value of 'svc_base' means the config read returned the \
                 identifier's source text and every reference-endpoint service is unrunnable"
            );
            assert!(
                !service.contains("svc_base"),
                "endpoint resolved to the identifier spelling, not its value: service='{service}'"
            );
        }
        other => panic!("expected AuthDeclaredButUnwired pre-send, got {other:?}"),
    }
}

fn endpoint_resolving_empty_refuses(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_ENDPOINT_RESOLVES_EMPTY);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::ServiceConfigUnresolved { key, spelled }) => {
            assert_eq!(
                key, "endpoint",
                "refusal must name the config key it could not read"
            );
            assert!(
                spelled.contains("svc_base_empty"),
                "refusal must carry the spelling that failed, got '{spelled}'"
            );
        }
        other => panic!(
            "an endpoint resolving to empty must REFUSE, not proceed with an empty base \
             (which fails downstream as an indistinguishable URL parse error); got {other:?}"
        ),
    }
}

fn endpoint_resolving_non_string_refuses(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_ENDPOINT_RESOLVES_NON_STRING);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::ServiceConfigUnresolved { key, spelled }) => {
            assert_eq!(
                key, "endpoint",
                "refusal must name the config key it could not read"
            );
            assert!(
                spelled.contains("svc_base_port"),
                "refusal must carry the spelling that failed, got '{spelled}'"
            );
        }
        other => panic!(
            "an endpoint resolving to a non-string must REFUSE. Display is total over Value, \
             so a rendering-based read would have sent \"8080\" as the base URL and passed \
             the emptiness check; got {other:?}"
        ),
    }
}

fn endpoint_absent_refuses(module_index: &ModuleIndex) {
    let resolved = resolve(module_index, SERVICE_ENDPOINT_ABSENT);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    match v1_interpreter::run_in_context(&ctx, "probe", false) {
        Err(InterpError::ServiceConfigMissing { key, service }) => {
            assert_eq!(
                key, "endpoint",
                "refusal must name the config key that was never declared"
            );
            assert!(
                !service.is_empty(),
                "refusal must name the service, or it cannot be located"
            );
        }
        other => panic!(
            "a service declaring no endpoint must REFUSE, not send the bare path as a \
             relative URL; got {other:?}"
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
            "endpoint_by_reference_resolves_to_its_value",
            endpoint_by_reference_resolves_to_its_value,
        ),
        (
            "endpoint_resolving_empty_refuses",
            endpoint_resolving_empty_refuses,
        ),
        (
            "endpoint_resolving_non_string_refuses",
            endpoint_resolving_non_string_refuses,
        ),
        ("endpoint_absent_refuses", endpoint_absent_refuses),
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
