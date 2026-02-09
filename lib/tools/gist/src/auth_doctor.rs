//! Auth doctor graph for gist credential setup diagnostics.
//!
//! Delegates to the provider-neutral cloud auth doctor in cloud-ops,
//! parameterized with the gist credential intent.

use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::{BuilderError, Dag};
use gunbc_lib_cloud_ops::auth_doctor::{build_cloud_auth_doctor_graph, CloudAuthDoctorOp};

pub fn build_gist_auth_doctor_graph(
    runtime_hint: Option<String>,
) -> Result<Dag<CloudAuthDoctorOp>, BuilderError> {
    let intent = GistRequest::new().credential_intent();
    build_cloud_auth_doctor_graph(intent, runtime_hint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::{execute_with_mode, ExecutionMode};
    use gunbc_ir::Value;

    #[test]
    fn test_auth_doctor_graph_runs() {
        let dag = build_gist_auth_doctor_graph(None).expect("auth doctor graph should build");
        let log = execute_with_mode(&dag, ExecutionMode::DryRun(Default::default()))
            .expect("auth doctor graph should execute");
        let report = log.get("auth_report").expect("auth_report should run");
        assert!(report.outputs.contains_key("required_scopes"));
        assert_eq!(
            report.outputs.get("service"),
            Some(&Value::Str("github".to_string()))
        );
    }
}
