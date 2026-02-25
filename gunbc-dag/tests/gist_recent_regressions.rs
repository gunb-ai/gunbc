#![allow(clippy::disallowed_methods)]

use std::sync::{Arc, Mutex};

use gunbc_dag::build_gist_recent_graph_dsl;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{
    HttpMethod, RestResponse, ShellResponse, TransportRequest, TransportResponse,
};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::{executor::TransportError, TransportBackend, TransportBackendGuard};

#[derive(Debug)]
struct GistRecentBackend {
    requests: Arc<Mutex<Vec<TransportRequest>>>,
}

impl TransportBackend for GistRecentBackend {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
        self.requests
            .lock()
            .expect("capture lock")
            .push(request.clone());

        match request {
            TransportRequest::Shell(shell) if shell.command == "git" => {
                let args = shell.args.as_slice();
                if args == ["rev-parse", "--abbrev-ref", "HEAD"] {
                    return Ok(TransportResponse::Shell(ShellResponse::ok("feature/mock\n")));
                }
                if args == ["diff", "oldest-commit...HEAD"] {
                    return Ok(TransportResponse::Shell(ShellResponse::ok(
                        "diff --git a/a b/a\n+line\n",
                    )));
                }
                Err(TransportError::new(format!(
                    "unexpected git invocation: {:?}",
                    shell.args
                )))
            }
            TransportRequest::Shell(shell)
                if shell.command == "bash"
                    && shell.args.len() >= 2
                    && shell.args[0] == "-lc"
                    && shell.args[1].contains("git rev-list") =>
            {
                Ok(TransportResponse::Shell(ShellResponse::ok(
                    "oldest-commit\n",
                )))
            }
            TransportRequest::Shell(shell) if shell.command == "gcloud" => Ok(
                TransportResponse::Shell(ShellResponse::ok("ghp_mock_token\n")),
            ),
            TransportRequest::Rest(rest) => {
                if rest.method != HttpMethod::Post || !rest.url.ends_with("/gists") {
                    return Err(TransportError::new(format!(
                        "unexpected REST request: method={:?} url={}",
                        rest.method, rest.url
                    )));
                }
                Ok(TransportResponse::Rest(RestResponse::new(
                    201,
                    serde_json::json!({
                        "id": "mock-gist-id",
                        "html_url": "https://gist.github.com/mock-gist-id"
                    }),
                )))
            }
            other => Err(TransportError::new(format!(
                "unexpected request variant: {other:?}"
            ))),
        }
    }
}

#[test]
fn gist_recent_graph_wires_diff_base_input() {
    let dag = build_gist_recent_graph_dsl().expect("gist-recent graph should build");
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_base_edge = lowered.dag.edges.iter().any(|edge| {
        edge.to_node.0 == "prepare_transport_services_git_git_Core_Diff" && edge.to_port.0 == "base"
    });
    assert!(
        has_base_edge,
        "gist-recent must wire a base ref into git diff prepare node"
    );
}

#[test]
fn gist_recent_end_to_end_emits_gist_url() {
    let dag = build_gist_recent_graph_dsl().expect("gist-recent graph should build");

    let requests = Arc::new(Mutex::new(Vec::new()));
    let backend = Arc::new(GistRecentBackend {
        requests: requests.clone(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "since" => input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Str("3.days.ago".into())),
            "public" => input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Bool(false)),
            _ => {}
        }
    }

    let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks))
        .expect("gist-recent execution should succeed with mocked backend");

    let gist_parse = log
        .entries
        .iter()
        .find(|entry| entry.node_id == "parse_transport_services_github_gist_github_Gist_Create")
        .expect("gist parse node should be present");
    let gist_prepare = log
        .entries
        .iter()
        .find(|entry| entry.node_id == "prepare_transport_services_github_gist_github_Gist_Create")
        .expect("gist prepare node should be present");
    let gist_execute = log
        .entries
        .iter()
        .find(|entry| entry.node_id == "execute_transport_services_github_gist_github_Gist_Create")
        .expect("gist execute node should be present");
    let diff_parse = log
        .entries
        .iter()
        .find(|entry| entry.node_id == "parse_transport_services_git_git_Core_Diff")
        .expect("diff parse node should be present");
    let render = log
        .entries
        .iter()
        .find(|entry| entry.node_id == "tools.gist_recent::render_diff_markdown")
        .expect("render node should be present");

    assert!(
        matches!(diff_parse.outputs.get("diff"), Some(Value::Str(_))),
        "diff parse should produce a diff string: {:?}",
        diff_parse.outputs
    );
    assert!(
        matches!(render.outputs.get("return"), Some(Value::Str(_))),
        "render node should produce markdown content: {:?}",
        render.outputs
    );

    assert!(
        matches!(gist_prepare.outputs.get("request"), Some(Value::Request(_))),
        "gist prepare must produce a concrete request: inputs={:?} outputs={:?}",
        gist_prepare.inputs,
        gist_prepare.outputs
    );
    assert!(
        matches!(
            gist_execute.outputs.get("response"),
            Some(Value::Response(TransportResponse::Rest(_)))
        ),
        "gist execute must produce a REST response: {:?}",
        gist_execute.outputs
    );

    assert_eq!(
        gist_parse.outputs.get("html_url").and_then(Value::as_str),
        Some("https://gist.github.com/mock-gist-id"),
        "gist parse output should expose html_url"
    );

    let seen_diff_request = requests.lock().expect("capture lock").iter().any(|request| {
        matches!(
            request,
            TransportRequest::Shell(shell)
            if shell.command == "git" && shell.args.as_slice() == ["diff", "oldest-commit...HEAD"]
        )
    });
    assert!(
        seen_diff_request,
        "gist-recent should diff from the oldest commit in the recent window"
    );
}
