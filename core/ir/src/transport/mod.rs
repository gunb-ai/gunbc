//! Transport layer types for I/O abstraction.
//!
//! This module provides request/response types for different transport mechanisms:
//! - REST/HTTP for web APIs
//! - File operations for filesystem I/O
//! - TCP for raw network connections
//! - Shell for command execution
//! - GitHub platform (API + CLI)
//! - GitHub Actions for CI/CD integration
//! - CI provider abstraction for workflow commands
//! - Cloud resource management (GCP, AWS)
//!
//! The key insight is that all world I/O can be modeled as request/response pairs,
//! allowing business logic to remain pure while transport execution happens at
//! well-defined boundaries.
//!
//! # GitHub Architecture
//!
//! ```text
//! github/                     ← Platform layer
//! ├── mod.rs (auth, versions)
//! ├── api.rs (REST API)
//! └── cli.rs (gh CLI)
//!
//! gist.rs                     ← Service layer (uses github/)
//! github_actions.rs           ← Service layer (uses github/)
//!
//! ci/                         ← CI provider abstraction
//! ├── command.rs (WorkflowCommand enum)
//! ├── provider.rs (CiProvider trait)
//! ├── runner.rs (Runner trait)
//! └── providers/ (GitHub, GitLab, Plain)
//!
//! cloud/                      ← Cloud resource management
//! ├── mod.rs (CloudProvider, ResourceHandle, etc.)
//! ├── gcp/ (Service Account, Secret Manager, Workload Identity)
//! ├── aws/ (IAM Role, Secrets Manager, Parameter Store)
//! └── secrets/ (Secret references, Workload Identity Federation)
//! ```

pub mod ci;
pub mod cli;
pub mod cloud;
pub mod file;
pub mod gist;
pub mod github;
pub mod github_actions;
pub mod http;
pub mod rest;
pub mod tcp;
pub mod tool;

pub use file::{FileOp, FileRequest, FileResponse};
pub use gist::GistRequest;
pub use github::{
    api::{github_rest_request, GitHubApi, GITHUB_API},
    cli::{gh_cli_commands, gh_cli_request, is_gh_installed, GHCommand, GitHubCLI, GH_CLI},
    GitHubAuth, GITHUB_API_VERSION, GITHUB_CONTRACT_VERSION, GH_CLI_MIN_VERSION,
};
pub use github_actions::{
    merge_permissions, Integration, PermissionLevel, PermissionScope, Permissions, RunnerImage,
    WorkflowConfig,
};
pub use http::{HttpMethod, HttpRequest, HttpResponse};
pub use rest::{AuthMethod, RestRequest, RestResponse};
pub use tcp::{TcpRequest, TcpResponse};
pub use cli::{
    build_cli_ensure, build_cli_upsert, CliToolDef, CliToolError, CliToolOp, ToolHandle,
    // CLI Tool definitions (use these with node.requires())
    CLIPPY as CLI_CLIPPY, RUSTFMT as CLI_RUSTFMT, CARGO as CLI_CARGO, GIT as CLI_GIT, GH as CLI_GH,
};
pub use tool::{
    check_all_satisfiable, default_platform_registry, default_tool_registry, is_satisfiable,
    plan_installation, InstallInputs, InstallOption, InstallPlan,
    PlatformDef, PlatformRegistry, ToolDef, ToolRegistry, UnsatisfiableError,
    // Package managers
    APK, APT, BREW, CARGO,
    // Tools
    CLIPPY, GIT, RUST, RUSTFMT,
    // Platforms
    ALPINE, DEBIAN, LINUX, MACOS, UBUNTU,
};
pub use github::cli::GH_TOOL;
pub use ci::{
    detect_provider, is_ci, AnnotationLevel, CiProvider, FileLocation, GitHubActionsProvider,
    GitLabCiProvider, GitLabRunner, PlainTextProvider, Runner, WorkflowCommand,
};
pub use cloud::{
    // Core types
    CloudCredential, CloudProvider, CheckResult, CreateResult, ResourceHandle, ResourceOp, ResourceState,
    // GCP
    gcp::{
        GcpCredential, GcpLocation, GcpResourceType, ResourceName as GcpResourceName,
        ServiceAccountDef, SecretDef as GcpSecretDef, WorkloadIdentityPoolDef, WorkloadIdentityProviderDef,
        IamBinding, IamMember, IamResource, RoleDef as GcpRoleDef,
    },
    // AWS
    aws::{
        AwsCredential, AwsRegion, AwsResourceType, Arn,
        IamRoleDef, IamPolicyDef, TrustPolicy, ManagedPolicies,
        AwsSecretDef, ParameterDef,
    },
    // Secrets
    secrets::{
        SecretRef, SecretSource, SecretVersion, WorkloadIdentityConfig,
        GcpWorkloadIdentity, AwsWebIdentity, WebIdentityTokenSource,
        GitHubSecretsRequirements, GitHubSecretDef, SecretScope,
    },
    // CLI tools
    GCLOUD, AWS_CLI, AZ_CLI,
};

use serde::{Deserialize, Serialize};

/// Unified transport request enum.
///
/// All I/O operations are represented as one of these request types,
/// allowing uniform handling at transport boundaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportRequest {
    /// REST API request
    Rest(RestRequest),
    /// Raw HTTP request
    Http(HttpRequest),
    /// File operation request
    File(FileRequest),
    /// TCP connection request
    Tcp(TcpRequest),
    /// Shell command request
    Shell(ShellRequest),
}

/// Unified transport response enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportResponse {
    /// REST API response
    Rest(RestResponse),
    /// Raw HTTP response
    Http(HttpResponse),
    /// File operation response
    File(FileResponse),
    /// TCP connection response
    Tcp(TcpResponse),
    /// Shell command response
    Shell(ShellResponse),
}

/// Shell command request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellRequest {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory (optional)
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Standard input to pipe to the command
    pub stdin: Option<String>,
}

/// Shell command response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellResponse {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

impl ShellRequest {
    /// Create a new shell request.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env: std::collections::HashMap::new(),
            stdin: None,
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set standard input.
    pub fn stdin(mut self, stdin: impl Into<String>) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

impl ShellResponse {
    /// Check if the command succeeded (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_request_builder() {
        let req = ShellRequest::new("gh")
            .args(["gist", "create"])
            .arg("-f")
            .arg("test.md")
            .cwd("/tmp")
            .stdin("# Test");

        assert_eq!(req.command, "gh");
        assert_eq!(req.args, vec!["gist", "create", "-f", "test.md"]);
        assert_eq!(req.cwd, Some("/tmp".to_string()));
        assert_eq!(req.stdin, Some("# Test".to_string()));
    }
}
