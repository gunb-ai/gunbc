//! Transport system model definitions registered via inventory.
//!
//! Models are layered by protocol stack:
//! - `transport.tcp` — raw TCP connectivity (Convention layer)
//! - `transport.http` — HTTP protocol on top of TCP (Protocol layer)
//! - `transport.rest` — REST conventions on top of HTTP (RestApi layer)
//! - `transport.file` — filesystem I/O (Transport layer)
//! - `transport.shell` — shell command execution (Transport layer)

use gunbc_ir::system_model::{
    Behavior, BehaviorInput, BehaviorOutput, Dependency, InputType, Invocation, OutputType,
    Property, SystemKind, SystemModel,
};
use gunbc_ir::transport::{default_transport_behaviors, TransportKind};
use gunbc_ir::TypeId;

fn ty(id: &str) -> InputType {
    InputType::TypeId(TypeId::from(id))
}

fn out_ty(id: &str) -> OutputType {
    OutputType::TypeId(TypeId::from(id))
}

fn invocation_for_transport_kind(kind: TransportKind) -> Invocation {
    default_transport_behaviors()
        .into_iter()
        .find(|behavior| behavior.transport == kind)
        .map(|behavior| behavior.invocation_contract())
        .unwrap_or_else(|| {
            panic!("missing default transport behavior contract for kind '{kind:?}'")
        })
}

pub fn build_transport_file_model() -> SystemModel {
    SystemModel::new(
        "transport.file",
        "File Transport",
        SystemKind::Transport,
        "v1",
        "File read/write/exists/delete transport behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "read",
            "Read a file path",
            invocation_for_transport_kind(TransportKind::File),
        )
        .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("FileResponse"),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "write",
            "Write file content",
            invocation_for_transport_kind(TransportKind::File),
        )
        .with_inputs(vec![
            BehaviorInput::required("path", ty("String")),
            BehaviorInput::required("content", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("FileResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "exists",
            "Check if a file exists",
            invocation_for_transport_kind(TransportKind::File),
        )
        .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("FileResponse"),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "delete",
            "Delete file path",
            invocation_for_transport_kind(TransportKind::File),
        )
        .with_inputs(vec![BehaviorInput::required("path", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("FileResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
}

gunbc_ir::submit_system_model!(build_transport_file_model);

pub fn build_transport_shell_model() -> SystemModel {
    SystemModel::new(
        "transport.shell",
        "Shell Transport",
        SystemKind::Transport,
        "v1",
        "Shell execution transport behavior",
    )
    .with_behaviors(vec![Behavior::new(
        "exec",
        "Execute shell command",
        invocation_for_transport_kind(TransportKind::Shell),
    )
    .with_inputs(vec![
        BehaviorInput::required("command", ty("String")),
        BehaviorInput::optional("args", ty("StringList")),
        BehaviorInput::optional("cwd", ty("OptionalString")),
        BehaviorInput::optional("env", ty("Json")),
        BehaviorInput::optional("timeout_ms", ty("OptionalInt")),
    ])
    .with_outputs(vec![BehaviorOutput::new(
        "response",
        out_ty("ShellResponse"),
    )])
    .with_properties(&[Property::WritesWorld])])
}

gunbc_ir::submit_system_model!(build_transport_shell_model);

// ---------------------------------------------------------------------------
// Layered protocol stack: TCP -> HTTP -> REST
// ---------------------------------------------------------------------------

/// TCP connectivity layer (bottom of the protocol stack).
///
/// No dependencies — this is the foundation layer.
pub fn build_transport_tcp_model() -> SystemModel {
    SystemModel::new(
        "transport.tcp",
        "TCP Transport",
        SystemKind::Convention,
        "v1",
        "Raw TCP connection behaviors (connect, send, receive)",
    )
    .with_behaviors(vec![
        Behavior::new(
            "connect",
            "Establish TCP connection",
            invocation_for_transport_kind(TransportKind::Tcp),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("TcpRequest"))])
        .with_outputs(vec![BehaviorOutput::new("response", out_ty("TcpResponse"))])
        .with_properties(&[Property::WritesWorld]),
        Behavior::new(
            "send",
            "Send data over TCP connection",
            invocation_for_transport_kind(TransportKind::Tcp),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("TcpRequest"))])
        .with_outputs(vec![BehaviorOutput::new("response", out_ty("TcpResponse"))])
        .with_properties(&[Property::WritesWorld]),
        Behavior::new(
            "receive",
            "Receive data over TCP connection",
            invocation_for_transport_kind(TransportKind::Tcp),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("TcpRequest"))])
        .with_outputs(vec![BehaviorOutput::new("response", out_ty("TcpResponse"))])
        .with_properties(&[Property::WritesWorld]),
    ])
}

gunbc_ir::submit_system_model!(build_transport_tcp_model);

/// HTTP protocol layer (sits on top of TCP).
///
/// Depends on `transport.tcp`.
pub fn build_transport_http_model() -> SystemModel {
    SystemModel::new(
        "transport.http",
        "HTTP Transport",
        SystemKind::Protocol,
        "v1",
        "HTTP request behaviors (RFC 9110 method semantics)",
    )
    .with_behaviors(vec![
        Behavior::new(
            "get",
            "HTTP GET request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "post",
            "HTTP POST request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "put",
            "HTTP PUT request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "delete",
            "HTTP DELETE request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "head",
            "HTTP HEAD request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::ReadOnly]),
        Behavior::new(
            "options",
            "HTTP OPTIONS request",
            invocation_for_transport_kind(TransportKind::Http),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::ReadOnly]),
    ])
    .with_dependencies(vec![Dependency::system("transport.tcp")])
    .with_depends_on(&["transport.tcp"])
}

gunbc_ir::submit_system_model!(build_transport_http_model);

/// REST convention layer (sits on top of HTTP).
///
/// Depends on `transport.http`. Inherits HTTP behavioral properties and adds
/// the `JsonContentType` property to all behaviors.
pub fn build_transport_rest_model() -> SystemModel {
    SystemModel::new(
        "transport.rest",
        "REST Transport",
        SystemKind::RestApi,
        "v1",
        "REST API behaviors with JSON content-type convention",
    )
    .with_behaviors(vec![
        Behavior::new(
            "get",
            "REST GET request",
            invocation_for_transport_kind(TransportKind::Rest),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[
            Property::ReadOnly,
            Property::Deterministic,
            Property::JsonContentType,
        ]),
        Behavior::new(
            "post",
            "REST POST request",
            invocation_for_transport_kind(TransportKind::Rest),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[
            Property::WritesWorld,
            Property::Idempotent,
            Property::JsonContentType,
        ]),
        Behavior::new(
            "put",
            "REST PUT request",
            invocation_for_transport_kind(TransportKind::Rest),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[
            Property::WritesWorld,
            Property::Idempotent,
            Property::JsonContentType,
        ]),
        Behavior::new(
            "patch",
            "REST PATCH request",
            invocation_for_transport_kind(TransportKind::Rest),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::JsonContentType]),
        Behavior::new(
            "delete",
            "REST DELETE request",
            invocation_for_transport_kind(TransportKind::Rest),
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[
            Property::WritesWorld,
            Property::Idempotent,
            Property::JsonContentType,
        ]),
    ])
    .with_dependencies(vec![Dependency::system("transport.http")])
    .with_depends_on(&["transport.http"])
}

gunbc_ir::submit_system_model!(build_transport_rest_model);

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::system_model::{validate_dependency_graph_acyclic, validate_system_model};
    use gunbc_ir::transport::default_transport_behaviors;
    use std::collections::BTreeSet;

    #[test]
    fn transport_models_validate() {
        validate_system_model(&build_transport_file_model())
            .expect("transport file model should validate");
        validate_system_model(&build_transport_shell_model())
            .expect("transport shell model should validate");
        validate_system_model(&build_transport_tcp_model())
            .expect("transport tcp model should validate");
        validate_system_model(&build_transport_http_model())
            .expect("transport http model should validate");
        validate_system_model(&build_transport_rest_model())
            .expect("transport rest model should validate");
    }

    #[test]
    fn transport_models_expose_expected_behavior_sets() {
        let file_model = build_transport_file_model();
        let file_ops: BTreeSet<_> = file_model.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            file_ops,
            BTreeSet::from(["read", "write", "exists", "delete"])
        );

        let shell_model = build_transport_shell_model();
        let exec = shell_model
            .behaviors
            .iter()
            .find(|b| b.id == "exec")
            .expect("shell exec behavior should exist");
        let shell_inputs: BTreeSet<_> = exec.inputs.iter().map(|i| i.name.as_str()).collect();
        assert!(shell_inputs.contains("command"));
        assert!(shell_inputs.contains("args"));
        assert!(shell_inputs.contains("env"));
        assert!(shell_inputs.contains("cwd"));
        assert!(shell_inputs.contains("timeout_ms"));

        let tcp_model = build_transport_tcp_model();
        let tcp_ops: BTreeSet<_> = tcp_model.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(tcp_ops, BTreeSet::from(["connect", "send", "receive"]));

        let http_model = build_transport_http_model();
        let http_ops: BTreeSet<_> = http_model.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            http_ops,
            BTreeSet::from(["get", "post", "put", "delete", "head", "options"])
        );

        let rest_model = build_transport_rest_model();
        let rest_ops: BTreeSet<_> = rest_model.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(
            rest_ops,
            BTreeSet::from(["get", "post", "put", "patch", "delete"])
        );
    }

    #[test]
    fn tcp_model_has_no_dependencies() {
        let tcp_model = build_transport_tcp_model();
        assert!(tcp_model.dependencies.is_empty(), "TCP is the base layer");
    }

    #[test]
    fn http_model_depends_on_tcp() {
        let http_model = build_transport_http_model();
        assert_eq!(http_model.dependencies.len(), 1);
        assert!(matches!(
            &http_model.dependencies[0].kind,
            gunbc_ir::system_model::DependencyKind::System(id) if id.0 == "transport.tcp"
        ));
    }

    #[test]
    fn rest_model_depends_on_http() {
        let rest_model = build_transport_rest_model();
        assert_eq!(rest_model.dependencies.len(), 1);
        assert!(matches!(
            &rest_model.dependencies[0].kind,
            gunbc_ir::system_model::DependencyKind::System(id) if id.0 == "transport.http"
        ));
    }

    #[test]
    fn protocol_stack_is_acyclic() {
        let models = vec![
            build_transport_tcp_model(),
            build_transport_http_model(),
            build_transport_rest_model(),
        ];
        validate_dependency_graph_acyclic(&models)
            .expect("TCP -> HTTP -> REST dependency chain must be acyclic");
    }

    #[test]
    fn tcp_behaviors_all_write_world() {
        let tcp_model = build_transport_tcp_model();
        for behavior in &tcp_model.behaviors {
            assert!(
                behavior.properties.contains(&Property::WritesWorld),
                "TCP behavior '{}' should have WritesWorld property",
                behavior.id
            );
        }
    }

    #[test]
    fn http_get_is_readonly_deterministic() {
        let http_model = build_transport_http_model();
        let get = http_model
            .behaviors
            .iter()
            .find(|b| b.id == "get")
            .expect("HTTP get behavior should exist");
        assert!(get.properties.contains(&Property::ReadOnly));
        assert!(get.properties.contains(&Property::Deterministic));
    }

    #[test]
    fn http_head_and_options_are_readonly() {
        let http_model = build_transport_http_model();
        for method in ["head", "options"] {
            let behavior = http_model
                .behaviors
                .iter()
                .find(|b| b.id == method)
                .unwrap_or_else(|| panic!("HTTP {method} behavior should exist"));
            assert!(
                behavior.properties.contains(&Property::ReadOnly),
                "HTTP {method} should be ReadOnly"
            );
        }
    }

    #[test]
    fn http_write_methods_are_idempotent() {
        let http_model = build_transport_http_model();
        for method in ["post", "put", "delete"] {
            let behavior = http_model
                .behaviors
                .iter()
                .find(|b| b.id == method)
                .unwrap_or_else(|| panic!("HTTP {method} behavior should exist"));
            assert!(
                behavior.properties.contains(&Property::WritesWorld),
                "HTTP {method} should have WritesWorld"
            );
            assert!(
                behavior.properties.contains(&Property::Idempotent),
                "HTTP {method} should be Idempotent"
            );
        }
    }

    #[test]
    fn rest_behaviors_all_have_json_content_type() {
        let rest_model = build_transport_rest_model();
        for behavior in &rest_model.behaviors {
            assert!(
                behavior.properties.contains(&Property::JsonContentType),
                "REST behavior '{}' should have JsonContentType property",
                behavior.id
            );
        }
    }

    #[test]
    fn rest_get_inherits_readonly_from_http() {
        let rest_model = build_transport_rest_model();
        let get = rest_model
            .behaviors
            .iter()
            .find(|b| b.id == "get")
            .expect("REST get behavior should exist");
        assert!(get.properties.contains(&Property::ReadOnly));
        assert!(get.properties.contains(&Property::Deterministic));
        assert!(get.properties.contains(&Property::JsonContentType));
    }

    #[test]
    fn rest_patch_is_not_idempotent() {
        let rest_model = build_transport_rest_model();
        let patch = rest_model
            .behaviors
            .iter()
            .find(|b| b.id == "patch")
            .expect("REST patch behavior should exist");
        assert!(patch.properties.contains(&Property::WritesWorld));
        assert!(
            !patch.properties.contains(&Property::Idempotent),
            "PATCH is not idempotent by default"
        );
    }

    #[test]
    fn system_model_invocations_align_with_transport_behavior_contracts() {
        let behavior_contracts = default_transport_behaviors();
        let expected = behavior_contracts
            .iter()
            .map(|behavior| (behavior.transport, behavior.invocation_contract()))
            .collect::<std::collections::BTreeMap<_, _>>();

        let file_model = build_transport_file_model();
        for behavior in &file_model.behaviors {
            assert_eq!(
                behavior.invocation,
                expected
                    .get(&TransportKind::File)
                    .expect("file invocation contract")
                    .clone()
            );
        }

        let shell_model = build_transport_shell_model();
        for behavior in &shell_model.behaviors {
            assert_eq!(
                behavior.invocation,
                expected
                    .get(&TransportKind::Shell)
                    .expect("shell invocation contract")
                    .clone()
            );
        }

        let tcp_model = build_transport_tcp_model();
        for behavior in &tcp_model.behaviors {
            assert_eq!(
                behavior.invocation,
                expected
                    .get(&TransportKind::Tcp)
                    .expect("tcp invocation contract")
                    .clone()
            );
        }

        let http_model = build_transport_http_model();
        for behavior in &http_model.behaviors {
            assert_eq!(
                behavior.invocation,
                expected
                    .get(&TransportKind::Http)
                    .expect("http invocation contract")
                    .clone()
            );
        }

        let rest_model = build_transport_rest_model();
        for behavior in &rest_model.behaviors {
            assert_eq!(
                behavior.invocation,
                expected
                    .get(&TransportKind::Rest)
                    .expect("rest invocation contract")
                    .clone()
            );
        }
    }

    #[test]
    fn system_kind_matches_protocol_layer() {
        assert_eq!(build_transport_tcp_model().kind, SystemKind::Convention);
        assert_eq!(build_transport_http_model().kind, SystemKind::Protocol);
        assert_eq!(build_transport_rest_model().kind, SystemKind::RestApi);
        assert_eq!(build_transport_file_model().kind, SystemKind::Transport);
        assert_eq!(build_transport_shell_model().kind, SystemKind::Transport);
    }

    #[test]
    fn tcp_model_depends_on_field_is_empty() {
        let tcp_model = build_transport_tcp_model();
        assert!(tcp_model.depends_on.is_empty(), "TCP is the base layer");
    }

    #[test]
    fn http_model_depends_on_field_contains_tcp() {
        let http_model = build_transport_http_model();
        assert_eq!(http_model.depends_on, vec!["transport.tcp".to_string()]);
    }

    #[test]
    fn rest_model_depends_on_field_contains_http() {
        let rest_model = build_transport_rest_model();
        assert_eq!(rest_model.depends_on, vec!["transport.http".to_string()]);
    }

    #[test]
    fn depends_on_references_are_valid() {
        use gunbc_ir::system_model::validate_depends_on_references;
        let models = vec![
            build_transport_tcp_model(),
            build_transport_http_model(),
            build_transport_rest_model(),
            build_transport_file_model(),
            build_transport_shell_model(),
        ];
        validate_depends_on_references(&models)
            .expect("all transport depends_on references should be valid");
    }

    #[test]
    fn inherited_properties_flow_through_protocol_stack() {
        use gunbc_ir::system_model::collect_inherited_properties;
        let models = vec![
            build_transport_tcp_model(),
            build_transport_http_model(),
            build_transport_rest_model(),
        ];

        // REST inherits from HTTP which inherits from TCP
        let rest_props = collect_inherited_properties("transport.rest", &models);

        // From TCP: WritesWorld
        assert!(rest_props.contains(&Property::WritesWorld));
        // From HTTP: ReadOnly, Deterministic, Idempotent
        assert!(rest_props.contains(&Property::ReadOnly));
        assert!(rest_props.contains(&Property::Deterministic));
        assert!(rest_props.contains(&Property::Idempotent));
        // From REST itself: JsonContentType
        assert!(rest_props.contains(&Property::JsonContentType));
    }
}
