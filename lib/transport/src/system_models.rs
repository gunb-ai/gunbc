//! Transport system model definitions registered via inventory.

use gunbc_ir::system_model::{
    Behavior, BehaviorInput, BehaviorOutput, InputType, Invocation, OutputType, Property,
    SystemKind, SystemModel,
};
use gunbc_ir::TypeId;

fn ty(id: &str) -> InputType {
    InputType::TypeId(TypeId::from(id))
}

fn out_ty(id: &str) -> OutputType {
    OutputType::TypeId(TypeId::from(id))
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
            Invocation::Protocol {
                protocol: "file".to_string(),
                docs: "gunbc file transport".to_string(),
            },
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
            Invocation::Protocol {
                protocol: "file".to_string(),
                docs: "gunbc file transport".to_string(),
            },
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
            Invocation::Protocol {
                protocol: "file".to_string(),
                docs: "gunbc file transport".to_string(),
            },
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
            Invocation::Protocol {
                protocol: "file".to_string(),
                docs: "gunbc file transport".to_string(),
            },
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
        Invocation::Protocol {
            protocol: "shell".to_string(),
            docs: "gunbc shell transport".to_string(),
        },
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

pub fn build_transport_http_rest_model() -> SystemModel {
    SystemModel::new(
        "transport.http_rest",
        "HTTP/REST Transport",
        SystemKind::Transport,
        "v1",
        "HTTP+REST request behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "http_get",
            "HTTP GET request",
            Invocation::Protocol {
                protocol: "http".to_string(),
                docs: "gunbc http transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::ReadOnly]),
        Behavior::new(
            "rest_post",
            "REST POST request",
            Invocation::Protocol {
                protocol: "rest".to_string(),
                docs: "gunbc rest transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[Property::WritesWorld]),
        Behavior::new(
            "http_post",
            "HTTP POST request",
            Invocation::Protocol {
                protocol: "http".to_string(),
                docs: "gunbc http transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld]),
        Behavior::new(
            "http_put",
            "HTTP PUT request",
            Invocation::Protocol {
                protocol: "http".to_string(),
                docs: "gunbc http transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "http_delete",
            "HTTP DELETE request",
            Invocation::Protocol {
                protocol: "http".to_string(),
                docs: "gunbc http transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("HttpRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("HttpResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "rest_get",
            "REST GET request",
            Invocation::Protocol {
                protocol: "rest".to_string(),
                docs: "gunbc rest transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[Property::ReadOnly]),
        Behavior::new(
            "rest_put",
            "REST PUT request",
            Invocation::Protocol {
                protocol: "rest".to_string(),
                docs: "gunbc rest transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "rest_delete",
            "REST DELETE request",
            Invocation::Protocol {
                protocol: "rest".to_string(),
                docs: "gunbc rest transport".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("request", ty("RestRequest"))])
        .with_outputs(vec![BehaviorOutput::new(
            "response",
            out_ty("RestResponse"),
        )])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
}

gunbc_ir::submit_system_model!(build_transport_http_rest_model);

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::system_model::validate_system_model;
    use std::collections::BTreeSet;

    #[test]
    fn transport_models_validate() {
        validate_system_model(&build_transport_file_model())
            .expect("transport file model should validate");
        validate_system_model(&build_transport_shell_model())
            .expect("transport shell model should validate");
        validate_system_model(&build_transport_http_rest_model())
            .expect("transport http_rest model should validate");
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

        let http_rest_model = build_transport_http_rest_model();
        let ops: BTreeSet<_> = http_rest_model
            .behaviors
            .iter()
            .map(|b| b.id.as_str())
            .collect();
        for op in [
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "rest_get",
            "rest_post",
            "rest_put",
            "rest_delete",
        ] {
            assert!(ops.contains(op), "missing http/rest operation {op}");
        }
    }
}
