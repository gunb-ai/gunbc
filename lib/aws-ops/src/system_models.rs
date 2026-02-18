//! AWS system model definitions registered via inventory.

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

pub fn build_aws_secrets_manager_model() -> SystemModel {
    SystemModel::new(
        "aws.secrets_manager",
        "AWS Secrets Manager",
        SystemKind::SecretProvider,
        "v1",
        "Secrets Manager get/create/destroy behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "get_secret_value",
            "Get secret value payload",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/".to_string(),
                docs: "https://docs.aws.amazon.com/secretsmanager/latest/apireference/".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("secret_id", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new("payload", out_ty("String"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "create_secret",
            "Create or update secret",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/".to_string(),
                docs: "https://docs.aws.amazon.com/secretsmanager/latest/apireference/".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("secret_id", ty("String")),
            BehaviorInput::required("payload", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("written", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "put_secret_value",
            "Put new secret value version",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/".to_string(),
                docs: "https://docs.aws.amazon.com/secretsmanager/latest/apireference/".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("secret_id", ty("String")),
            BehaviorInput::required("payload", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("version", out_ty("String"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "describe_secret",
            "Describe secret metadata",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/".to_string(),
                docs: "https://docs.aws.amazon.com/secretsmanager/latest/apireference/".to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("secret_id", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new("metadata", out_ty("Json"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "destroy_secret_version",
            "Delete secret/version",
            Invocation::Rest {
                method: "POST".to_string(),
                path: "/".to_string(),
                docs: "https://docs.aws.amazon.com/secretsmanager/latest/apireference/".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("secret_id", ty("String")),
            BehaviorInput::optional("version_id", ty("OptionalString")),
        ])
        .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
    .with_dependencies(vec![
        Dependency::secret("secret:AWS_ACCESS_KEY_ID"),
        Dependency::secret("secret:AWS_SECRET_ACCESS_KEY"),
    ])
}

gunbc_ir::submit_system_model!(build_aws_secrets_manager_model);

pub fn build_aws_iam_model() -> SystemModel {
    SystemModel::new(
        "aws.iam",
        "AWS IAM",
        SystemKind::IdentityProvider,
        "v1",
        "Role/policy/assume-role behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "role_upsert",
            "Create or update IAM role",
            Invocation::Cli {
                command: "aws iam create-role".to_string(),
                docs: "https://docs.aws.amazon.com/iam/".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("role_name", ty("String")),
            BehaviorInput::required("trust_policy", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("role", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "policy_attach",
            "Attach policy to role",
            Invocation::Cli {
                command: "aws iam attach-role-policy".to_string(),
                docs: "https://docs.aws.amazon.com/iam/".to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("role_name", ty("String")),
            BehaviorInput::required("policy_arn", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("attached", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
        Behavior::new(
            "assume_role",
            "Assume role and mint session creds",
            Invocation::Cli {
                command: "aws sts assume-role".to_string(),
                docs: "https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html"
                    .to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("role_arn", ty("String")),
            BehaviorInput::required("session_name", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("session", out_ty("Json"))])
        .with_properties(&[Property::WritesWorld]),
    ])
    .with_dependencies(vec![
        Dependency::secret("secret:AWS_ACCESS_KEY_ID"),
        Dependency::secret("secret:AWS_SECRET_ACCESS_KEY"),
    ])
}

gunbc_ir::submit_system_model!(build_aws_iam_model);

pub fn build_aws_s3_model() -> SystemModel {
    SystemModel::new(
        "aws.s3",
        "AWS S3",
        SystemKind::StorageProvider,
        "v1",
        "S3 object get/put/list/delete behaviors",
    )
    .with_behaviors(vec![
        Behavior::new(
            "get_object",
            "Read object from S3 bucket",
            Invocation::Cli {
                command: "aws s3api get-object".to_string(),
                docs: "https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html"
                    .to_string(),
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
            "Write object into S3 bucket",
            Invocation::Cli {
                command: "aws s3api put-object".to_string(),
                docs: "https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html"
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
            "List objects in S3 bucket",
            Invocation::Cli {
                command: "aws s3api list-objects-v2".to_string(),
                docs: "https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html"
                    .to_string(),
            },
        )
        .with_inputs(vec![BehaviorInput::required("bucket", ty("String"))])
        .with_outputs(vec![BehaviorOutput::new("objects", out_ty("JsonList"))])
        .with_properties(&[Property::ReadOnly, Property::Deterministic]),
        Behavior::new(
            "delete_object",
            "Delete object from S3 bucket",
            Invocation::Cli {
                command: "aws s3api delete-object".to_string(),
                docs: "https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html"
                    .to_string(),
            },
        )
        .with_inputs(vec![
            BehaviorInput::required("bucket", ty("String")),
            BehaviorInput::required("object", ty("String")),
        ])
        .with_outputs(vec![BehaviorOutput::new("deleted", out_ty("Bool"))])
        .with_properties(&[Property::WritesWorld, Property::Idempotent]),
    ])
    .with_dependencies(vec![
        Dependency::secret("secret:AWS_ACCESS_KEY_ID"),
        Dependency::secret("secret:AWS_SECRET_ACCESS_KEY"),
    ])
}

gunbc_ir::submit_system_model!(build_aws_s3_model);

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::system_model::validate_system_model;
    use std::collections::BTreeSet;

    #[test]
    fn aws_models_validate() {
        validate_system_model(&build_aws_secrets_manager_model())
            .expect("aws secrets manager model should validate");
        validate_system_model(&build_aws_iam_model()).expect("aws iam model should validate");
        validate_system_model(&build_aws_s3_model()).expect("aws s3 model should validate");
    }

    #[test]
    fn aws_models_expose_expected_behavior_sets() {
        let secrets = build_aws_secrets_manager_model();
        let secret_ops: BTreeSet<_> = secrets.behaviors.iter().map(|b| b.id.as_str()).collect();
        for op in [
            "get_secret_value",
            "create_secret",
            "put_secret_value",
            "describe_secret",
            "destroy_secret_version",
        ] {
            assert!(
                secret_ops.contains(op),
                "missing aws secrets manager op {op}"
            );
        }

        let iam = build_aws_iam_model();
        let iam_ops: BTreeSet<_> = iam.behaviors.iter().map(|b| b.id.as_str()).collect();
        for op in ["role_upsert", "policy_attach", "assume_role"] {
            assert!(iam_ops.contains(op), "missing aws iam op {op}");
        }

        let s3 = build_aws_s3_model();
        let s3_ops: BTreeSet<_> = s3.behaviors.iter().map(|b| b.id.as_str()).collect();
        for op in ["get_object", "put_object", "list_objects", "delete_object"] {
            assert!(s3_ops.contains(op), "missing aws s3 op {op}");
        }
    }
}
