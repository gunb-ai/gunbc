use gunbc_exec::{execute_with_mode, ExecutionMode};
use gunbc_ir::transport::cloud::CloudProviderKind;
use gunbc_ir::Value;
use gunbc_lib_cloud_ops::{build_github_credential_graph, detect_cloud_env_requirements};
use gunbc_test::{guard_test_with_env, FermiCost, TestClass};

#[test]
fn test_github_live_rate_limit() {
    let env_req = match detect_cloud_env_requirements() {
        Ok(req) => req,
        Err(_) => return,
    };
    if env_req.provider != CloudProviderKind::Gcp {
        return;
    }
    if !guard_test_with_env(
        "test_github_live_rate_limit",
        TestClass::Integration,
        FermiCost::M,
        &["http"],
        env_req.required,
        env_req.required_any_of,
    ) {
        return;
    }

    let dag = build_github_credential_graph().unwrap();
    let log = execute_with_mode(&dag, ExecutionMode::Real).expect("live GitHub request should run");

    let parse = log
        .get("parse_status")
        .expect("parse_status node should run");
    let ok = parse
        .outputs
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    assert!(ok, "expected GitHub /rate_limit to succeed");

    let status = parse
        .outputs
        .get("status")
        .and_then(Value::as_int)
        .unwrap_or(0);
    assert!(
        (200_i64..300_i64).contains(&status),
        "expected 2xx status, got {}",
        status
    );
}
