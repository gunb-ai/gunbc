use gunbc_exec::{execute_with_mode, ExecutionMode};
use gunbc_ir::Value;
use gunbc_lib_cloud_ops::build_github_credential_graph;
use gunbc_test::{guard_test, FermiCost, TestClass};

#[test]
fn test_github_live_rate_limit() {
    if !guard_test(
        "test_github_live_rate_limit",
        TestClass::Integration,
        FermiCost::M,
        &["http"],
        &[
            "GCP_WIF_PROVIDER",
            "GCP_SECRETS_PROJECT",
            "GCP_SECRETS_PREFIX",
            "ACTIONS_ID_TOKEN_REQUEST_URL",
            "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
        ],
    ) {
        return;
    }

    if std::env::var("GCP_SECRETS_SA").is_err()
        && std::env::var("GCP_SECRETS_IMPERSONATE_SA").is_err()
    {
        return;
    }

    let dag = build_github_credential_graph();
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
