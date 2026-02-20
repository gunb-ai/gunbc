//! GCP system model definitions registered via inventory.

use gunbc_ir::system_model::{
    Behavior, BehaviorInput, BehaviorOutput, Dependency, InputType, Invocation, OutputType,
    Property, SystemKind, SystemModel,
};
use gunbc_ir::TypeId;

fn ty(id: &str) -> InputType {
    InputType::TypeId(TypeId::from(id))
}

fn out_ty(id: &str) -> OutputType {
    OutputType::TypeId(TypeId::from(id))
}

pub fn build_gcp_secret_manager_model() -> SystemModel {
    SystemModel::new(
        "gcp.secret_manager",
        "GCP Secret Manager",
        SystemKind::SecretProvider,
        "v1",
        "Secret Manager access/list/upsert behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "access_secret_version",
            "Access one secret version payload",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/{project_id}/secrets/{secret_id}/versions/{version}:access"
                    .to_string(),
                docs: "https://cloud.google.com/secret-manager/docs".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("secret_id", ty("String")),
            BehaviorInput::required("version", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new(
            "payload",
            out_ty("GcpSecretPayload"),
        )])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "list_secrets",
            "List available secrets in a project",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/v1/projects/{project_id}/secrets".to_string(),
                docs: "https://cloud.google.com/secret-manager/docs".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("project_id", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new("secrets", out_ty("JsonList"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "upsert_secret",
            "Create or update a secret payload",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/v1/projects/{project_id}/secrets/{secret_id}".to_string(),
                docs: "https://cloud.google.com/secret-manager/docs".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("secret_id", ty("String")),
            BehaviorInput::required("payload", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("written", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
    .with_dependencies(vec![Dependency::secret(
        "secret:GOOGLE_APPLICATION_CREDENTIALS",
    )])
}

gunbc_ir::submit_system_model!(build_gcp_secret_manager_model);

pub fn build_gcp_iam_model() -> SystemModel {
    SystemModel::new(
        "gcp.iam",
        "GCP IAM",
        SystemKind::IdentityProvider,
        "v1",
        "Service account and IAM binding management",
    )
    .with_behaviors(vec![
        Behavior::new(
            "service_account_upsert",
            "Create/update service account",
            Invocation::Cli {
                command: "gcloud iam service-accounts".to_string(),
                docs: "https://cloud.google.com/iam/docs/service-accounts".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("account_id", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("service_account", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "binding_upsert",
            "Create/update IAM binding",
            Invocation::Cli {
                command: "gcloud projects add-iam-policy-binding".to_string(),
                docs: "https://cloud.google.com/iam/docs/granting-changing-revoking-access"
                    .to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("member", ty("String")),
            BehaviorInput::required("role", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("binding", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "service_account_delete",
            "Delete service account",
            Invocation::Cli {
                command: "gcloud iam service-accounts delete".to_string(),
                docs: "https://cloud.google.com/iam/docs/service-accounts-delete".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("account_email", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "binding_remove",
            "Remove IAM binding",
            Invocation::Cli {
                command: "gcloud projects remove-iam-policy-binding".to_string(),
                docs: "https://cloud.google.com/iam/docs/granting-changing-revoking-access"
                    .to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("member", ty("String")),
            BehaviorInput::required("role", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("binding_removed", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "wif_pool_upsert",
            "Create or update WIF pool",
            Invocation::Cli {
                command: "gcloud iam workload-identity-pools".to_string(),
                docs: "https://cloud.google.com/iam/docs/workload-identity-federation".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("pool_id", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("pool", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "wif_provider_upsert",
            "Create or update WIF provider",
            Invocation::Cli {
                command: "gcloud iam workload-identity-pools providers".to_string(),
                docs: "https://cloud.google.com/iam/docs/workload-identity-federation".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("project_id", ty("String")),
            BehaviorInput::required("pool_id", ty("String")),
            BehaviorInput::required("provider_id", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("provider", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
    .with_dependencies(vec![Dependency::secret(
        "secret:GOOGLE_APPLICATION_CREDENTIALS",
    )])
}

gunbc_ir::submit_system_model!(build_gcp_iam_model);

pub fn build_gcp_gcs_model() -> SystemModel {
    SystemModel::new(
        "gcp.gcs",
        "GCP Cloud Storage",
        SystemKind::StorageProvider,
        "v1",
        "Object get/put/list/delete behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "get_object",
            "Read one object from bucket",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/storage/v1/b/{bucket}/o/{object}".to_string(),
                docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("bucket", ty("String")),
            BehaviorInput::required("object", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("content", out_ty("String"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "put_object",
            "Write object to bucket",
            Invocation::Rest {
                method: "PUT".to_string(),
                path: "/upload/storage/v1/b/{bucket}/o/{object}".to_string(),
                docs: "https://cloud.google.com/storage/docs/json_api/v1/how-tos/upload"
                    .to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("bucket", ty("String")),
            BehaviorInput::required("object", ty("String")),
            BehaviorInput::required("content", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("written", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "list_objects",
            "List bucket objects",
            Invocation::Rest {
                method: "GET".to_string(),
                path: "/storage/v1/b/{bucket}/o".to_string(),
                docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("bucket", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new("objects", out_ty("JsonList"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "delete_object",
            "Delete object from bucket",
            Invocation::Rest {
                method: "DELETE".to_string(),
                path: "/storage/v1/b/{bucket}/o/{object}".to_string(),
                docs: "https://cloud.google.com/storage/docs/json_api".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("bucket", ty("String")),
            BehaviorInput::required("object", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
    .with_dependencies(vec![Dependency::secret(
        "secret:GOOGLE_APPLICATION_CREDENTIALS",
    )])
}

gunbc_ir::submit_system_model!(build_gcp_gcs_model);

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::system_model::{validate_system_model, DependencyKind, SecretDependencyId};
    use std::collections::BTreeSet;

    #[test]
    fn gcp_secret_manager_model_validates() {
        validate_system_model(&build_gcp_secret_manager_model())
            .expect("gcp secret manager model should validate");
    }

    #[test]
    fn gcp_secret_manager_model_declares_adc_secret_dependency() {
        let model = build_gcp_secret_manager_model();
        assert!(model.dependencies.iter().any(|dep| {
            dep.kind
                == DependencyKind::Secret(SecretDependencyId::new(
                    "secret:GOOGLE_APPLICATION_CREDENTIALS",
                ))
        }));
    }

    #[test]
    fn gcp_models_expose_expected_behavior_sets() {
        let secret_manager = build_gcp_secret_manager_model();
        let secret_ops: BTreeSet<_> = secret_manager
            .behaviors
            .iter()
            .map(|b| b.id.as_str())
            .collect();
        assert!(secret_ops.contains("access_secret_version"));
        assert!(secret_ops.contains("list_secrets"));
        assert!(secret_ops.contains("upsert_secret"));

        let iam = build_gcp_iam_model();
        let iam_ops: BTreeSet<_> = iam.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert!(iam_ops.contains("service_account_upsert"));
        assert!(iam_ops.contains("service_account_delete"));
        assert!(iam_ops.contains("binding_upsert"));
        assert!(iam_ops.contains("binding_remove"));
        assert!(iam_ops.contains("wif_pool_upsert"));
        assert!(iam_ops.contains("wif_provider_upsert"));

        let gcs = build_gcp_gcs_model();
        let gcs_ops: BTreeSet<_> = gcs.behaviors.iter().map(|b| b.id.as_str()).collect();
        assert!(gcs_ops.contains("get_object"));
        assert!(gcs_ops.contains("put_object"));
        assert!(gcs_ops.contains("list_objects"));
        assert!(gcs_ops.contains("delete_object"));
    }
}
