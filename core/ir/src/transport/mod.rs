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
//! - LLM provider integration (OpenAI, Anthropic)
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
//! git.rs                      ← Service layer (deterministic git commands)
//! github_actions.rs           ← Service layer (uses github/)
//!
//! ci/                         ← CI provider abstraction
//! ├── command.rs (WorkflowCommand enum)
//! ├── provider.rs (CiProvider trait)
//! ├── runner.rs (Runner trait)
//! └── providers/ (GitHub, GitLab, Plain)
//!
//! llm/                         ← LLM provider abstraction
//! ├── chat.rs (ChatMessage, ChatRequest, ChatResponse)
//! ├── provider.rs (LlmProvider, provider registry)
//! ├── openai.rs (OpenAI conversions)
//! └── anthropic.rs (Anthropic conversions)
//! ```

pub mod agent;
pub mod agent_adapter;
pub mod behavior;
pub mod ci;
pub mod cli;
pub mod cloud;
pub mod credential;
pub mod credential_policy;
pub mod file;
pub mod gcp;
pub mod git;
pub mod github;
pub mod github_actions;
pub mod http;
pub mod infra_scope;
pub mod llm;
pub mod rest;
pub mod review;
pub mod scope;
pub mod tcp;
pub mod tool;

pub use agent::{
    AgentConstraints, AgentError, AgentHandle, AgentStatus, DesignArtifact, HandoffSpec,
    PrValidationResult, PullRequestResult, PullRequestSpec,
};
pub use agent_adapter::{AgentAdapter, StubAgentAdapter};
pub use behavior::{default_transport_behaviors, FieldRouteSpec, TransportBehavior, TransportKind};
pub use ci::{
    detect_provider, detect_provider_strict, is_ci, AnnotationLevel, CiProvider, FileLocation,
    GitHubActionsProvider, GitLabCiProvider, GitLabRunner, PlainTextProvider, Runner,
    WorkflowCommand,
};
pub use cli::{
    build_cli_ensure,
    build_cli_upsert,
    CliToolDef,
    CliToolError,
    CliToolOp,
    MockResolver,
    ToolHandle,
    // Tool path resolution trait + implementations
    ToolPathResolver,
    CARGO as CLI_CARGO,
    // CLI Tool definitions (use these with node.requires())
    CLIPPY as CLI_CLIPPY,
    GH as CLI_GH,
    GIT as CLI_GIT,
    RUSTFMT as CLI_RUSTFMT,
};
pub use cloud::{
    CloudConfigSpec, CloudNamespace, CloudProviderKind, CloudRuntimeKind, CloudSecretConfig,
    CloudSecretRef,
};
pub use credential::{AuthScheme, Credential, CredentialError, Secret, SecretSource};
pub use credential_policy::{
    CredentialIntentPolicy, CredentialPolicyDefaults, CredentialPolicyError,
    CredentialPolicyProfile, CredentialPolicySpec, ImpersonationPolicy,
    ResolvedCredentialIntentPolicy, ResolvedCredentialPolicyProfile, ScopeMergeMode, SecretBinding,
    VersionSelector,
};
pub use file::{FileOp, FileRequest, FileResponse};
pub use gcp::{
    GcpBucket, GcpIamBinding, GcpIamCondition, GcpIamPolicy, GcpInfraSpec, GcpProject, GcpSecret,
    GcpSecretPayload, GcpServiceAccount, GcpWifPool, GcpWifProvider,
};
pub use git::GitRequest;
pub use github::cli::GH_TOOL;
pub use github::{
    api::{github_rest_request, GitHubApi, GITHUB_API},
    cli::{gh_cli_commands, gh_cli_request, GHCommand},
    GH_CLI_MIN_VERSION, GITHUB_API_VERSION, GITHUB_CONTRACT_VERSION, GITHUB_SECRET_ID,
};
pub use github_actions::{
    merge_permissions, Integration, PermissionLevel, PermissionScope, Permissions, RunnerImage,
    WorkflowConfig,
};
pub use http::{HttpMethod, HttpRequest, HttpResponse};
pub use infra_scope::{GcpScope, InfraAccessLevel, InfraScope, InfraScopeType};
pub use llm::{
    anthropic_provider, build_chat_request, builtin_provider_ids, openai_provider,
    parse_chat_response, provider_by_id, ChatMessage, ChatRequest, ChatResponse, FinishReason,
    LlmProvider, Role, Usage,
};
pub use rest::{RestRequest, RestResponse};
pub use scope::{CredentialIntent, ScopeContract, ScopeContractError};
pub use tcp::{TcpRequest, TcpResponse};
pub use tool::{
    check_all_satisfiable,
    default_platform_registry,
    default_tool_registry,
    is_satisfiable,
    plan_installation,
    InstallInputs,
    InstallOption,
    InstallPlan,
    PlatformDef,
    PlatformRegistry,
    ToolDef,
    ToolRegistry,
    UnsatisfiableError,
    // Platforms
    ALPINE,
    // Package managers
    APK,
    APT,
    BREW,
    CARGO,
    // Tools
    CLIPPY,
    DEBIAN,
    GIT,
    LINUX,
    MACOS,
    RUST,
    RUSTFMT,
    UBUNTU,
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
    /// Timeout in milliseconds. If the command exceeds this, it is killed
    /// and an error is returned. `None` means no timeout (wait forever).
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Stream stdout/stderr directly to the terminal instead of capturing.
    ///
    /// Use this for interactive auth flows (e.g., `gcloud auth login`) that
    /// require live prompts and browser URLs.
    #[serde(default, skip_serializing_if = "is_false")]
    pub passthrough: bool,
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
            timeout_ms: None,
            passthrough: false,
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

    /// Set a timeout in milliseconds. The command is killed if it exceeds this.
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Enable or disable stdio passthrough for interactive commands.
    pub fn passthrough(mut self, enabled: bool) -> Self {
        self.passthrough = enabled;
        self
    }

    /// Wrap this request in a [`TransportRequest::Shell`].
    pub fn into_transport_request(self) -> TransportRequest {
        TransportRequest::Shell(self)
    }
}

impl ShellResponse {
    /// Create a successful shell response (exit code 0).
    pub fn ok(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    /// Create a failed shell response with the given exit code and stderr.
    pub fn failed(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    /// Check if the command succeeded (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Wrap this response in a [`TransportResponse::Shell`].
    pub fn into_transport_response(self) -> TransportResponse {
        TransportResponse::Shell(self)
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

// ---------------------------------------------------------------------------
// From impls for transport request/response types
// ---------------------------------------------------------------------------

impl From<ShellRequest> for TransportRequest {
    fn from(req: ShellRequest) -> Self {
        TransportRequest::Shell(req)
    }
}

impl From<RestRequest> for TransportRequest {
    fn from(req: RestRequest) -> Self {
        TransportRequest::Rest(req)
    }
}

impl From<HttpRequest> for TransportRequest {
    fn from(req: HttpRequest) -> Self {
        TransportRequest::Http(req)
    }
}

impl From<FileRequest> for TransportRequest {
    fn from(req: FileRequest) -> Self {
        TransportRequest::File(req)
    }
}

impl From<TcpRequest> for TransportRequest {
    fn from(req: TcpRequest) -> Self {
        TransportRequest::Tcp(req)
    }
}

impl From<ShellResponse> for TransportResponse {
    fn from(resp: ShellResponse) -> Self {
        TransportResponse::Shell(resp)
    }
}

impl From<RestResponse> for TransportResponse {
    fn from(resp: RestResponse) -> Self {
        TransportResponse::Rest(resp)
    }
}

impl From<HttpResponse> for TransportResponse {
    fn from(resp: HttpResponse) -> Self {
        TransportResponse::Http(resp)
    }
}

impl From<FileResponse> for TransportResponse {
    fn from(resp: FileResponse) -> Self {
        TransportResponse::File(resp)
    }
}

impl From<TcpResponse> for TransportResponse {
    fn from(resp: TcpResponse) -> Self {
        TransportResponse::Tcp(resp)
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
            .stdin("# Test")
            .passthrough(true);

        assert_eq!(req.command, "gh");
        assert_eq!(req.args, vec!["gist", "create", "-f", "test.md"]);
        assert_eq!(req.cwd, Some("/tmp".to_string()));
        assert_eq!(req.stdin, Some("# Test".to_string()));
        assert!(req.passthrough);
    }

    #[test]
    fn test_shell_response_ok() {
        let resp = ShellResponse::ok("hello");
        assert!(resp.success());
        assert_eq!(resp.stdout, "hello");
        assert_eq!(resp.stderr, "");
        assert_eq!(resp.exit_code, 0);
    }

    #[test]
    fn test_shell_response_failed() {
        let resp = ShellResponse::failed(1, "error");
        assert!(!resp.success());
        assert_eq!(resp.stdout, "");
        assert_eq!(resp.stderr, "error");
        assert_eq!(resp.exit_code, 1);
    }

    #[test]
    fn test_from_shell_request() {
        let req = ShellRequest::new("ls");
        let transport: TransportRequest = req.into();
        assert!(matches!(transport, TransportRequest::Shell(_)));
    }

    #[test]
    fn test_from_shell_response() {
        let resp = ShellResponse::ok("ok");
        let transport: TransportResponse = resp.into();
        assert!(matches!(transport, TransportResponse::Shell(_)));
    }
}
