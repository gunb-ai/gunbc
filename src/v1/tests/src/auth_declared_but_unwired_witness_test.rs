use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, AuthResolution, ExecutionMode, InterpError};

use crate::helpers::resolve_imports_transitively;

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

fn resolve(src: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

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

#[test]
fn auth_declared_no_source_fails_closed_pre_send() {
    let resolved = resolve(SERVICE_AUTH_BEARER_NO_SOURCE);
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

#[test]
fn auth_input_empty_fails_closed_pre_send() {
    let resolved = resolve(SERVICE_AUTH_INPUT_NOT_PROVIDED);
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

#[test]
fn no_auth_declared_does_not_fire_guard() {
    let resolved = resolve(SERVICE_NO_AUTH);
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

#[test]
fn resolve_auth_three_way_split_matches_dispatch_behavior() {
    let cases: &[(&str, &str, bool)] = &[
        ("Bearer + no source", SERVICE_AUTH_BEARER_NO_SOURCE, true),
        ("auth_input empty", SERVICE_AUTH_INPUT_NOT_PROVIDED, true),
        ("no auth declared", SERVICE_NO_AUTH, false),
    ];
    for (label, src, expect_unwired) in cases {
        let resolved = resolve(src);
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

#[test]
fn auth_resolution_enum_is_pub_and_discriminable() {
    let _ = AuthResolution::NoAuthDeclared;
    let _ = AuthResolution::Resolved {
        header: "Authorization".to_string(),
        token: "tok".to_string(),
    };
    let _ = AuthResolution::DeclaredButUnwired {
        reason: "test".to_string(),
    };
}
