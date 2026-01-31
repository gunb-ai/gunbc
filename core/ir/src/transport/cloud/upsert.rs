//! Cloud resource upsert builder.
//!
//! This module provides a builder for creating cloud resource upsert patterns
//! that integrate with the existing DAG pattern system.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::{gcp::ServiceAccountDef, CloudResourceUpsert};
//!
//! let sa = ServiceAccountDef::new("my-project", "github-actions")
//!     .with_display_name("GitHub Actions SA");
//!
//! // Build the upsert DAG node
//! let upsert_node = CloudResourceUpsert::new("ensure_sa")
//!     .gcp_service_account(sa)
//!     .build();
//! ```

use crate::node::Node;
use crate::patterns::UpsertBuilder;
use crate::transport::ShellRequest;
use serde::{Deserialize, Serialize};

use super::aws::iam::IamRoleDef;
use super::aws::secrets::{AwsSecretDef, ParameterDef};
use super::gcp::secret_manager::SecretDef as GcpSecretDef;
use super::gcp::{ServiceAccountDef, WorkloadIdentityPoolDef, WorkloadIdentityProviderDef};
use super::CloudProvider;

// ============================================================================
// Cloud Resource Operation
// ============================================================================

/// Operation type for cloud resource DAG nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CloudResourceOp {
    // GCP Operations
    /// Check if GCP service account exists
    GcpServiceAccountCheck(ServiceAccountDef),
    /// Create GCP service account
    GcpServiceAccountCreate(ServiceAccountDef),
    /// Resolve GCP service account
    GcpServiceAccountResolve(ServiceAccountDef),

    /// Check if GCP secret exists
    GcpSecretCheck(GcpSecretDef),
    /// Create GCP secret
    GcpSecretCreate(GcpSecretDef),
    /// Resolve GCP secret
    GcpSecretResolve(GcpSecretDef),

    /// Check if workload identity pool exists
    GcpWipCheck(WorkloadIdentityPoolDef),
    /// Create workload identity pool
    GcpWipCreate(WorkloadIdentityPoolDef),
    /// Resolve workload identity pool
    GcpWipResolve(WorkloadIdentityPoolDef),

    /// Check if workload identity provider exists
    GcpWipProviderCheck(WorkloadIdentityProviderDef),
    /// Create workload identity provider
    GcpWipProviderCreate(WorkloadIdentityProviderDef),
    /// Resolve workload identity provider
    GcpWipProviderResolve(WorkloadIdentityProviderDef),

    // AWS Operations
    /// Check if AWS IAM role exists
    AwsIamRoleCheck(IamRoleDef),
    /// Create AWS IAM role
    AwsIamRoleCreate(IamRoleDef),
    /// Resolve AWS IAM role
    AwsIamRoleResolve(IamRoleDef),

    /// Check if AWS secret exists
    AwsSecretCheck(AwsSecretDef),
    /// Create AWS secret
    AwsSecretCreate(AwsSecretDef),
    /// Resolve AWS secret
    AwsSecretResolve(AwsSecretDef),

    /// Check if AWS SSM parameter exists
    AwsParameterCheck(ParameterDef),
    /// Create/update AWS SSM parameter
    AwsParameterPut(ParameterDef),
    /// Resolve AWS SSM parameter
    AwsParameterResolve(ParameterDef),
}

impl CloudResourceOp {
    /// Get the shell request for this operation.
    pub fn to_shell_request(&self) -> ShellRequest {
        match self {
            // GCP Service Account
            Self::GcpServiceAccountCheck(def) => def.check_command(),
            Self::GcpServiceAccountCreate(def) => def.create_command(),
            Self::GcpServiceAccountResolve(def) => def.check_command(),

            // GCP Secret
            Self::GcpSecretCheck(def) => def.check_command(),
            Self::GcpSecretCreate(def) => def.create_command(),
            Self::GcpSecretResolve(def) => def.check_command(),

            // GCP Workload Identity Pool
            Self::GcpWipCheck(def) => def.check_command(),
            Self::GcpWipCreate(def) => def.create_command(),
            Self::GcpWipResolve(def) => def.check_command(),

            // GCP Workload Identity Provider
            Self::GcpWipProviderCheck(def) => def.check_command(),
            Self::GcpWipProviderCreate(def) => def.create_command(),
            Self::GcpWipProviderResolve(def) => def.check_command(),

            // AWS IAM Role
            Self::AwsIamRoleCheck(def) => def.check_command(),
            Self::AwsIamRoleCreate(def) => def.create_command(),
            Self::AwsIamRoleResolve(def) => def.check_command(),

            // AWS Secret
            Self::AwsSecretCheck(def) => def.check_command(),
            Self::AwsSecretCreate(def) => def.create_command(),
            Self::AwsSecretResolve(def) => def.check_command(),

            // AWS Parameter
            Self::AwsParameterCheck(def) => def.check_command(),
            Self::AwsParameterPut(def) => def.put_command("", true), // Value provided separately
            Self::AwsParameterResolve(def) => def.get_command(false),
        }
    }

    /// Get the cloud provider for this operation.
    pub fn provider(&self) -> CloudProvider {
        match self {
            Self::GcpServiceAccountCheck(_)
            | Self::GcpServiceAccountCreate(_)
            | Self::GcpServiceAccountResolve(_)
            | Self::GcpSecretCheck(_)
            | Self::GcpSecretCreate(_)
            | Self::GcpSecretResolve(_)
            | Self::GcpWipCheck(_)
            | Self::GcpWipCreate(_)
            | Self::GcpWipResolve(_)
            | Self::GcpWipProviderCheck(_)
            | Self::GcpWipProviderCreate(_)
            | Self::GcpWipProviderResolve(_) => CloudProvider::Gcp,

            Self::AwsIamRoleCheck(_)
            | Self::AwsIamRoleCreate(_)
            | Self::AwsIamRoleResolve(_)
            | Self::AwsSecretCheck(_)
            | Self::AwsSecretCreate(_)
            | Self::AwsSecretResolve(_)
            | Self::AwsParameterCheck(_)
            | Self::AwsParameterPut(_)
            | Self::AwsParameterResolve(_) => CloudProvider::Aws,
        }
    }

    /// Get the required CLI tool for this operation.
    pub fn required_tool(&self) -> &'static str {
        match self.provider() {
            CloudProvider::Gcp => "gcloud",
            CloudProvider::Aws => "aws",
            CloudProvider::Azure => "az",
        }
    }
}

// ============================================================================
// Cloud Resource Upsert Builder
// ============================================================================

/// Builder for cloud resource upsert patterns.
///
/// This creates a 3-node SubDag pattern (Check → Create → Resolve) that
/// integrates with the standard gunbc upsert pattern.
pub struct CloudResourceUpsertBuilder {
    name: String,
    resource_type: Option<CloudResourceType>,
}

/// Type of cloud resource being managed.
#[derive(Debug, Clone)]
enum CloudResourceType {
    GcpServiceAccount(ServiceAccountDef),
    GcpSecret(GcpSecretDef),
    GcpWorkloadIdentityPool(WorkloadIdentityPoolDef),
    GcpWorkloadIdentityProvider(WorkloadIdentityProviderDef),
    AwsIamRole(IamRoleDef),
    AwsSecret(AwsSecretDef),
    AwsParameter(ParameterDef),
}

impl CloudResourceUpsertBuilder {
    /// Create a new cloud resource upsert builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            resource_type: None,
        }
    }

    /// Set the resource to a GCP service account.
    pub fn gcp_service_account(mut self, def: ServiceAccountDef) -> Self {
        self.resource_type = Some(CloudResourceType::GcpServiceAccount(def));
        self
    }

    /// Set the resource to a GCP secret.
    pub fn gcp_secret(mut self, def: GcpSecretDef) -> Self {
        self.resource_type = Some(CloudResourceType::GcpSecret(def));
        self
    }

    /// Set the resource to a GCP workload identity pool.
    pub fn gcp_workload_identity_pool(mut self, def: WorkloadIdentityPoolDef) -> Self {
        self.resource_type = Some(CloudResourceType::GcpWorkloadIdentityPool(def));
        self
    }

    /// Set the resource to a GCP workload identity provider.
    pub fn gcp_workload_identity_provider(mut self, def: WorkloadIdentityProviderDef) -> Self {
        self.resource_type = Some(CloudResourceType::GcpWorkloadIdentityProvider(def));
        self
    }

    /// Set the resource to an AWS IAM role.
    pub fn aws_iam_role(mut self, def: IamRoleDef) -> Self {
        self.resource_type = Some(CloudResourceType::AwsIamRole(def));
        self
    }

    /// Set the resource to an AWS secret.
    pub fn aws_secret(mut self, def: AwsSecretDef) -> Self {
        self.resource_type = Some(CloudResourceType::AwsSecret(def));
        self
    }

    /// Set the resource to an AWS SSM parameter.
    pub fn aws_parameter(mut self, def: ParameterDef) -> Self {
        self.resource_type = Some(CloudResourceType::AwsParameter(def));
        self
    }

    /// Build the upsert pattern as a SubDag node.
    pub fn build(self) -> Node<CloudResourceOp> {
        let resource_type = self
            .resource_type
            .expect("resource type is required");

        let (check_op, create_op, resolve_op, resource_type_str) = match resource_type {
            CloudResourceType::GcpServiceAccount(def) => (
                CloudResourceOp::GcpServiceAccountCheck(def.clone()),
                CloudResourceOp::GcpServiceAccountCreate(def.clone()),
                CloudResourceOp::GcpServiceAccountResolve(def),
                "gcp:serviceAccount",
            ),
            CloudResourceType::GcpSecret(def) => (
                CloudResourceOp::GcpSecretCheck(def.clone()),
                CloudResourceOp::GcpSecretCreate(def.clone()),
                CloudResourceOp::GcpSecretResolve(def),
                "gcp:secret",
            ),
            CloudResourceType::GcpWorkloadIdentityPool(def) => (
                CloudResourceOp::GcpWipCheck(def.clone()),
                CloudResourceOp::GcpWipCreate(def.clone()),
                CloudResourceOp::GcpWipResolve(def),
                "gcp:workloadIdentityPool",
            ),
            CloudResourceType::GcpWorkloadIdentityProvider(def) => (
                CloudResourceOp::GcpWipProviderCheck(def.clone()),
                CloudResourceOp::GcpWipProviderCreate(def.clone()),
                CloudResourceOp::GcpWipProviderResolve(def),
                "gcp:workloadIdentityProvider",
            ),
            CloudResourceType::AwsIamRole(def) => (
                CloudResourceOp::AwsIamRoleCheck(def.clone()),
                CloudResourceOp::AwsIamRoleCreate(def.clone()),
                CloudResourceOp::AwsIamRoleResolve(def),
                "aws:iamRole",
            ),
            CloudResourceType::AwsSecret(def) => (
                CloudResourceOp::AwsSecretCheck(def.clone()),
                CloudResourceOp::AwsSecretCreate(def.clone()),
                CloudResourceOp::AwsSecretResolve(def),
                "aws:secret",
            ),
            CloudResourceType::AwsParameter(def) => (
                CloudResourceOp::AwsParameterCheck(def.clone()),
                CloudResourceOp::AwsParameterPut(def.clone()),
                CloudResourceOp::AwsParameterResolve(def),
                "aws:parameter",
            ),
        };

        // Use the existing UpsertBuilder pattern
        UpsertBuilder::new(&self.name)
            .with_check(check_op)
            .with_create(create_op)
            .with_resolve(resolve_op)
            .with_input_port("config", resource_type_str)
            .with_output_port("handle", "ResourceHandle")
            .build()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_gcp_service_account_upsert() {
        let sa = ServiceAccountDef::new("my-project", "test-sa");
        let node = CloudResourceUpsertBuilder::new("ensure_sa")
            .gcp_service_account(sa)
            .build();

        assert_eq!(node.id.0, "ensure_sa");
        assert!(node.is_subdag());

        // Check it has check, create, resolve nodes
        if let NodeBody::SubDag(dag) = &node.body {
            assert_eq!(dag.nodes.len(), 3);
            let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
            assert!(names.contains(&"check"));
            assert!(names.contains(&"create"));
            assert!(names.contains(&"resolve"));
        } else {
            panic!("Expected SubDag");
        }
    }

    #[test]
    fn test_aws_iam_role_upsert() {
        let role = IamRoleDef::for_github_actions("123456789012", "test-role", "owner/repo");
        let node = CloudResourceUpsertBuilder::new("ensure_role")
            .aws_iam_role(role)
            .build();

        assert_eq!(node.id.0, "ensure_role");
        assert!(node.is_subdag());
    }

    #[test]
    fn test_cloud_resource_op_provider() {
        let sa = ServiceAccountDef::new("my-project", "test-sa");
        let op = CloudResourceOp::GcpServiceAccountCheck(sa);
        assert_eq!(op.provider(), CloudProvider::Gcp);
        assert_eq!(op.required_tool(), "gcloud");

        let role = IamRoleDef::for_github_actions("123456789012", "test-role", "owner/repo");
        let op = CloudResourceOp::AwsIamRoleCheck(role);
        assert_eq!(op.provider(), CloudProvider::Aws);
        assert_eq!(op.required_tool(), "aws");
    }

    #[test]
    fn test_cloud_resource_op_to_shell_request() {
        let sa = ServiceAccountDef::new("my-project", "test-sa");
        let op = CloudResourceOp::GcpServiceAccountCheck(sa);
        let req = op.to_shell_request();

        assert_eq!(req.command, "gcloud");
        assert!(req.args.contains(&"describe".to_string()));
    }
}
