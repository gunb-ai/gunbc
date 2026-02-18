//! Gist operations.
//!
//! Operations for working with GitHub Gists.
//!
//! All operations are PURE (no I/O). I/O happens through TransportOps::Execute nodes.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_gist_ops::prepare_gist_request;
//!
//! let request = prepare_gist_request("# My Content", true, "My gist");
//! // request is now a TransportRequest ready to be executed via TransportOps::Execute
//! ```

#![deny(dead_code)]
use gunbc_exec::{
    optional_int_strict, optional_str_list_strict, optional_str_strict, propagate_skipped,
    require_response, require_str, ExecError, Executable, IntoExecResult, OutputMap,
};
use gunbc_ir::build::{list, optional, port, resource, scalar, AccessMode};
use gunbc_ir::builder::BuilderError;
use gunbc_ir::dag::{Dag, Edge};
use gunbc_ir::node::Node;
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::{
    validate_authenticate_bindings, AuthenticatePhase, AuthenticatePhaseBinding, Timestamp, Value,
};
use gunbc_lib_cloud_ops::{
    bind_credential_intent_policy, build_cloud_secret_manager_credential_graph_from_config,
    policy_allows_impersonation, CloudOps, CloudSecretManagerGraphOp,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::filename;
use gunbc_primitives::{ClockEnv, FsEnv};
use std::collections::HashMap;
use std::time::SystemTime;

/// Gist operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use TransportOps::Execute for actual I/O.
#[derive(Debug, Clone)]
pub enum GistOps {
    /// Prepare a gist creation request (PURE - no I/O)
    PrepareRequest { public: bool },
    /// Parse gist response to extract URL (PURE - no I/O)
    ParseGistResponse,
}

impl Executable for GistOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistOps::PrepareRequest { public } => {
                if let Some(result) = propagate_skipped(&inputs, "markdown", &["request"]) {
                    return result;
                }
                let markdown = require_str(&inputs, "markdown")?;
                let branch = optional_str_strict(&inputs, "branch")?;
                let remote_branch = optional_str_strict(&inputs, "remote_branch")?;
                let base_ref = optional_str_strict(&inputs, "base_ref")?;
                // These contract metadata inputs are currently informational for gist request
                // preparation, but still validated for type safety.
                let _credential_expires_in = optional_int_strict(&inputs, "credential_expires_in")?;
                let _required_scopes = optional_str_list_strict(&inputs, "required_scopes")?;

                // Acquire system resources at the DAG boundary (not inline)
                let fs = require_filesystem_handle(&inputs, "res:file")?;
                let now = require_timestamp(&inputs, "res:clock")?;

                // Explicit priority: local branch > remote branch > "snapshot".
                // - `branch` is set when HEAD points to a local branch
                // - `remote_branch` is set when HEAD is detached at a remote
                //   tracking branch (e.g., after `git checkout origin/main`)
                // - Neither is set for arbitrary detached commits
                let effective_branch = branch.or(remote_branch);

                let (filename, description) = if let Some(sha) = base_ref {
                    // Recent mode: base_ref is the SHA from rev-list
                    let short_sha = &sha[..sha.len().min(7)];
                    let branch_label = effective_branch.unwrap_or("snapshot");
                    let prefix = format!(
                        "{}_recent-3d_{}..HEAD",
                        sanitize_branch_for_filename(&fs, branch_label),
                        short_sha
                    );
                    let filename = generate_gist_filename_with_prefix(&fs, &prefix, now);
                    let description = match effective_branch {
                        Some(b) if !b.trim().is_empty() && b.trim() != "HEAD" => {
                            format!(
                                "Recent changes (3d) {}..HEAD on {} created by gunbc-gist",
                                short_sha, b
                            )
                        }
                        _ => format!(
                            "Recent changes (3d) {}..HEAD created by gunbc-gist",
                            short_sha
                        ),
                    };
                    (filename, description)
                } else {
                    // Snapshot/diff mode, or recent mode with young repo
                    // (young repo: parse_rev_list produces no output → absent from inputs)
                    let filename =
                        generate_gist_filename(&fs, effective_branch.unwrap_or("snapshot"), now);
                    let description = match effective_branch {
                        Some(b) if !b.trim().is_empty() && b.trim() != "HEAD" => {
                            format!("Code snapshot of {} created by gunbc-gist", b)
                        }
                        _ => "Code snapshot created by gunbc-gist".to_string(),
                    };
                    (filename, description)
                };

                let request = prepare_gist_request(markdown, *public, &description, &filename);

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GistOps::ParseGistResponse => {
                let response = require_response(&inputs, "response")?;

                let url = extract_gist_url(response);

                OutputMap::new().str("url", url).ok()
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Prepare a gist creation request.
///
/// Returns a `TransportRequest` that can be executed to create a gist.
/// This is PURE - it doesn't perform any I/O, just builds the request.
///
/// # Example
///
/// ```ignore
/// let request = prepare_gist_request("# Hello", true, "My public gist", "main_2024-01-15_14-30-00.md");
/// // Execute via TransportOps::Execute node in the DAG
/// ```
pub fn prepare_gist_request(
    content: &str,
    public: bool,
    description: &str,
    filename: &str,
) -> TransportRequest {
    GistRequest::new()
        .file(filename, content)
        .public(public)
        .description(description)
        .to_rest_request()
}

/// Sanitize a branch name for use as a filename component.
///
/// Falls back to `"snapshot-<hash>"` if the branch is empty or entirely degenerate.
///
/// Replaces spaces with the replacement char (convention, not a FS rule),
/// then routes through the filesystem gateway. Falls back to `"snapshot-<hash>"`
/// if the input is empty or sanitizes to the filesystem's default fallback.
///
/// # Examples
///
/// ```ignore
/// let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
/// assert_eq!(sanitize_branch_for_filename(&fs, "main"), "main");
/// assert_eq!(sanitize_branch_for_filename(&fs, "claude/branch-name"), "claude-branch-name");
/// ```
pub fn sanitize_branch_for_filename(fs: &filename::FilesystemHandle, branch: &str) -> String {
    let fallback = fallback_branch_name(branch);

    // Replace spaces before filesystem gateway (convention, not a FS rule)
    let no_spaces: String = branch
        .chars()
        .map(|c| if c == ' ' { fs.replacement() } else { c })
        .collect();

    let outcome = fs.prepare_filename(&no_spaces, filename::WritePolicy::Sanitize);

    // Detect the sanitizer's degenerate-input fallback ("untitled") vs a real branch
    // literally named "untitled". A real "untitled" branch passes validation unchanged
    // (outcome is Valid), while a degenerate input produces Sanitized { sanitized: "untitled" }.
    if outcome.was_sanitized() && outcome.filename() == Some("untitled") {
        return fallback;
    }

    match outcome.filename() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => fallback,
    }
}

fn fallback_branch_name(branch: &str) -> String {
    // 64-bit FNV-1a for a short, deterministic suffix without extra dependencies.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in branch.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let suffix = format!("{:012x}", hash & 0xFFFFFFFFFFFF);
    format!("snapshot-{}", suffix)
}

/// Generate a gist filename from a branch name, filesystem handle, and timestamp.
///
/// The branch prefix is sanitized and truncated to fit within the filesystem's
/// max component bytes after accounting for the suffix (`_YYYY-MM-DD_HH-MM-SS.md`).
///
/// # Examples
///
/// ```ignore
/// let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
/// let now = Timestamp::now();
/// let filename = generate_gist_filename(&fs, "claude/my-feature", now);
/// assert!(filename.starts_with("claude-my-feature_"));
/// assert!(filename.ends_with(".md"));
/// ```
pub fn generate_gist_filename(
    fs: &filename::FilesystemHandle,
    branch: &str,
    now: Timestamp,
) -> String {
    let sanitized = sanitize_branch_for_filename(fs, branch);
    let timestamp = format_utc_timestamp(now.to_system_time());
    let suffix = format!("_{}.md", timestamp); // e.g., "_2024-01-15_14-30-00.md" = 23 bytes

    // Ensure the full filename fits within the filesystem's component limit.
    let max_bytes = fs.max_component_bytes();
    let branch_budget = max_bytes.saturating_sub(suffix.len());

    let truncated = if sanitized.len() > branch_budget {
        // Truncate at UTF-8 boundary
        let mut end = branch_budget;
        while end > 0 && !sanitized.is_char_boundary(end) {
            end -= 1;
        }
        // Trim trailing replacement char from truncation point
        sanitized[..end].trim_end_matches(fs.replacement())
    } else {
        &sanitized
    };

    if truncated.is_empty() {
        format!("snapshot{}", suffix)
    } else {
        format!("{}{}", truncated, suffix)
    }
}

/// Generate a gist filename from a pre-built prefix and timestamp.
///
/// Unlike `generate_gist_filename`, the caller supplies the full prefix
/// (already sanitized). This is used when the prefix contains extra
/// metadata like a commit range (e.g., `main_recent-3d_abc123d..HEAD`).
pub fn generate_gist_filename_with_prefix(
    fs: &filename::FilesystemHandle,
    prefix: &str,
    now: Timestamp,
) -> String {
    let timestamp = format_utc_timestamp(now.to_system_time());
    let suffix = format!("_{}.md", timestamp);

    let max_bytes = fs.max_component_bytes();
    let prefix_budget = max_bytes.saturating_sub(suffix.len());

    let truncated = if prefix.len() > prefix_budget {
        let mut end = prefix_budget;
        while end > 0 && !prefix.is_char_boundary(end) {
            end -= 1;
        }
        prefix[..end].trim_end_matches(fs.replacement())
    } else {
        prefix
    };

    if truncated.is_empty() {
        format!("snapshot{}", suffix)
    } else {
        format!("{}{}", truncated, suffix)
    }
}

/// Format a SystemTime as a human-readable UTC timestamp for filenames.
///
/// Produces format: `YYYY-MM-DD_HH-MM-SS` (all filename-safe characters).
fn format_utc_timestamp(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Manual UTC breakdown (no chrono dependency)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Convert days since epoch to Y-M-D using a civil calendar algorithm
    // Based on Howard Hinnant's algorithm (public domain)
    let (year, month, day) = days_to_civil(days as i64);

    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn require_filesystem_handle(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<filename::FilesystemHandle, ExecError> {
    let value = inputs
        .get(key)
        .ok_or_else(|| ExecError::new(format!("missing '{}' input", key)))?;
    filename::FilesystemHandle::try_from(value)
        .with_exec_context(|| format!("invalid '{}' input", key))
}

fn require_timestamp(inputs: &HashMap<String, Value>, key: &str) -> Result<Timestamp, ExecError> {
    let value = inputs
        .get(key)
        .ok_or_else(|| ExecError::new(format!("missing '{}' input", key)))?;
    Timestamp::try_from(value).with_exec_context(|| format!("invalid '{}' input", key))
}

/// Convert days since Unix epoch to (year, month, day).
///
/// Uses Howard Hinnant's civil_from_days algorithm.
fn days_to_civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Extract gist URL from a transport response.
pub fn extract_gist_url(response: &TransportResponse) -> String {
    match response {
        TransportResponse::Shell(ShellResponse { stdout, .. }) => {
            gunbc_ir::transport::gist::parse_gist_url_from_shell(stdout)
                .unwrap_or_else(|| stdout.trim().to_string())
        }
        TransportResponse::Rest(r) => gunbc_ir::transport::gist::parse_gist_url_from_rest(&r.body)
            .unwrap_or_else(|| "unknown".to_string()),
        _ => "unknown".to_string(),
    }
}

// ============================================================================
// GistUploadOp — composite op type for the gist upload SubDag
// ============================================================================

/// Operation type for the gist upload SubDag.
///
/// Wraps all operations needed for the gist upload pipeline:
/// - Cloud credential lifecycle (IAM, secret resolution)
/// - Gist request preparation and response parsing
/// - Transport boundaries (actual I/O)
/// - Resource acquisition (filesystem, clock)
/// - Auth resolution (hardcoded GitHub gist credentials)
///
/// Consumers `map_ops` this into their own graph-op enum, exactly
/// like `CloudSecretManagerGraphOp` is lifted into consumer enums.
#[derive(Debug, Clone)]
pub enum GistUploadOp {
    /// Gist operations (PURE — request preparation, response parsing).
    Gist(GistOps),
    /// Cloud credential lifecycle operations.
    Cloud(CloudSecretManagerGraphOp),
    /// Transport boundary (actual I/O).
    Transport(TransportOps),
    /// Filesystem environment (resource acquisition).
    FsEnv(FsEnv),
    /// Clock environment (timestamp snapshot).
    ClockEnv(ClockEnv),
    /// Hardcoded GitHub gist auth resolution (PURE).
    ResolveAuth,
}

impl Executable for GistUploadOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistUploadOp::Gist(op) => op.execute(inputs),
            GistUploadOp::Cloud(op) => op.execute(inputs),
            GistUploadOp::Transport(op) => op.execute(inputs),
            GistUploadOp::FsEnv(op) => op.execute(inputs),
            GistUploadOp::ClockEnv(op) => op.execute(inputs),
            GistUploadOp::ResolveAuth => execute_gist_resolve_auth(inputs),
        }
    }
}

/// Hardcoded auth resolution for GitHub Gist operations.
///
/// Uses `GistRequest::credential_intent()` to derive the auth contract.
fn execute_gist_resolve_auth(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let fallback_intent = GistRequest::new().credential_intent();
    fallback_intent
        .validate()
        .map_err(|e| ExecError::new(format!("invalid gist credential contract: {e}")))?;
    let bound = bind_credential_intent_policy("github.gist.create", &fallback_intent)
        .map_err(|e| ExecError::new(format!("credential policy binding failed: {e}")))?;
    let allow_impersonation = policy_allows_impersonation(bound.impersonation.as_ref());
    let intent = bound.intent;

    let mut out = OutputMap::new()
        .str("service", intent.service)
        .str("scheme", intent.scheme)
        .str("header_name", intent.header_name)
        .str_list("required_scopes", intent.required_scopes)
        .bool("interactive_allowed", intent.interactive_allowed)
        .bool("allow_impersonation", allow_impersonation)
        .int("lifetime_seconds", 3600);

    if let Some(secret_name) = intent.secret_name {
        out = out.str("secret_name", secret_name);
    }

    out.ok()
}

// ============================================================================
// SubDag builder — gist upload pipeline
// ============================================================================

/// Build a self-contained gist upload SubDag.
///
/// The SubDag encapsulates the complete gist upload pipeline:
///
/// ```text
///   fs_env ──────────────────────────────────────────────────┐
///   clock_env ───────────────────────────────────────────┐   │
///   cloud_env ──────────────────────────────┐            │   │
///   resolve_auth ──────────────┐            │            │   │
///                  bind_secret ──> cloud_credential      │   │
///                       │               │                │   │
///                  scope_preflight       │                │   │
///                       │               │                │   │
///   prepare_gist_request ────────> execute_gist ──> parse_gist_response
/// ```
///
/// **Interface (auto-inferred from entrypoints/boundaries):**
/// - Inputs: `markdown: String`, `branch: OptionalString`,
///   `remote_branch: OptionalString`, `base_ref: OptionalString`
/// - Outputs: `url: String` (plus `ok: Bool` from cloud credential on some configs)
///
/// The credential chain is fully self-contained — consumers don't need to
/// understand cloud credentials at all. They just wire `markdown` in and get
/// `url` out.
pub fn build_gist_upload_subdag(
    config: CloudSecretConfig,
    public: bool,
) -> Result<Dag<GistUploadOp>, BuilderError> {
    validate_authenticate_bindings(&gist_authenticate_bindings())
        .expect("gist credential flow must follow canonical authenticate pattern");

    let mut dag = Dag::new();

    // ========================================================================
    // Environment nodes (roots — no inputs)
    // ========================================================================

    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        GistUploadOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ));

    dag.add_node(Node::opaque(
        "clock_env",
        vec![],
        vec![port("clock", "Timestamp")],
        GistUploadOp::ClockEnv(ClockEnv),
    ));

    dag.add_node(Node::opaque(
        "cloud_env",
        vec![],
        vec![
            port("config", "CloudSecretConfig"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        GistUploadOp::Cloud(CloudSecretManagerGraphOp::Cloud(
            CloudOps::ConstCloudConfig {
                config: config.clone(),
            },
        )),
    ));

    dag.add_node(Node::opaque(
        "resolve_auth",
        vec![],
        vec![
            port("service", "String"),
            optional("secret_name", "OptionalString"),
            optional("allow_impersonation", "OptionalBool"),
            port("scheme", "String"),
            port("header_name", "String"),
            list("required_scopes", "String"),
            port("interactive_allowed", "Bool"),
            optional("lifetime_seconds", "OptionalInt"),
        ],
        GistUploadOp::ResolveAuth,
    ));

    // ========================================================================
    // Credential chain
    // ========================================================================

    dag.add_node(Node::opaque(
        "bind_secret",
        vec![
            port("config", "CloudSecretConfig"),
            port("service", "String"),
            optional("secret_name", "OptionalString"),
        ],
        vec![port("config", "CloudSecretConfig")],
        GistUploadOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::BindSecretName)),
    ));

    let cloud_subdag = build_cloud_secret_manager_credential_graph_from_config(&config)?
        .map_ops(&mut GistUploadOp::Cloud);
    dag.add_node(Node::subdag("cloud_credential", cloud_subdag));

    dag.add_node(Node::opaque(
        "scope_preflight",
        vec![list("required_scopes", "String")],
        vec![scalar("scope_verified", "Bool")],
        GistUploadOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::ScopePreflight)),
    ));

    // ========================================================================
    // Gist request pipeline
    // ========================================================================

    dag.add_node(Node::opaque(
        "prepare_gist_request",
        vec![
            scalar("markdown", "String"),
            optional("branch", "OptionalString"),
            optional("remote_branch", "OptionalString"),
            resource("file", "FilesystemHandle", AccessMode::Read),
            resource("clock", "Timestamp", AccessMode::Read),
            optional("credential_expires_in", "OptionalInt"),
            list("required_scopes", "String"),
            optional("base_ref", "OptionalString"),
        ],
        vec![
            scalar("request", "TransportRequest"),
            scalar("skip", "Bool"),
        ],
        GistUploadOp::Gist(GistOps::PrepareRequest { public }),
    ));

    dag.add_node(Node::opaque(
        "execute_gist",
        vec![
            scalar("request", "TransportRequest"),
            scalar("skip", "Bool"),
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![scalar("response", "TransportResponse")],
        GistUploadOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_gist_response",
        vec![scalar("response", "TransportResponse")],
        vec![scalar("url", "String")],
        GistUploadOp::Gist(GistOps::ParseGistResponse),
    ));

    // ========================================================================
    // Edges: credential chain
    // ========================================================================

    dag.add_edge(Edge::new("cloud_env", "config", "bind_secret", "config"));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "service",
        "bind_secret",
        "service",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "secret_name",
        "bind_secret",
        "secret_name",
    ));
    dag.add_edge(Edge::new(
        "bind_secret",
        "config",
        "cloud_credential",
        "config",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "service",
        "cloud_credential",
        "source_id",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "allow_impersonation",
        "cloud_credential",
        "allow_impersonation",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "scheme",
        "cloud_credential",
        "scheme",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "header_name",
        "cloud_credential",
        "header_name",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "lifetime_seconds",
        "cloud_credential",
        "lifetime_seconds",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "interactive_allowed",
        "cloud_credential",
        "interactive_allowed",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "required_scopes",
        "cloud_credential",
        "required_scopes",
    ));
    dag.add_edge(Edge::new(
        "cloud_env",
        "request_url",
        "cloud_credential",
        "request_url",
    ));
    dag.add_edge(Edge::new(
        "cloud_env",
        "request_token",
        "cloud_credential",
        "request_token",
    ));

    // ========================================================================
    // Edges: scope preflight
    // ========================================================================

    dag.add_edge(Edge::new(
        "resolve_auth",
        "required_scopes",
        "scope_preflight",
        "required_scopes",
    ));

    // ========================================================================
    // Edges: gist request pipeline
    // ========================================================================

    dag.add_edge(Edge::new(
        "fs_env",
        FsEnv::WRITE_PORT,
        "prepare_gist_request",
        "res:file",
    ));
    dag.add_edge(Edge::new(
        "clock_env",
        "clock",
        "prepare_gist_request",
        "res:clock",
    ));
    dag.add_edge(Edge::new(
        "cloud_credential",
        "expires_in",
        "prepare_gist_request",
        "credential_expires_in",
    ));
    dag.add_edge(Edge::new(
        "resolve_auth",
        "required_scopes",
        "prepare_gist_request",
        "required_scopes",
    ));
    dag.add_edge(Edge::new(
        "prepare_gist_request",
        "request",
        "execute_gist",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_gist_request",
        "skip",
        "execute_gist",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "scope_preflight",
        "scope_verified",
        "execute_gist",
        "scope_verified",
    ));
    dag.add_edge(Edge::new(
        "cloud_credential",
        "credential",
        "execute_gist",
        "res:credential",
    ));
    dag.add_edge(Edge::new(
        "execute_gist",
        "response",
        "parse_gist_response",
        "response",
    ));

    Ok(dag)
}

fn gist_authenticate_bindings() -> Vec<AuthenticatePhaseBinding> {
    vec![
        AuthenticatePhaseBinding::new(AuthenticatePhase::ResolveContext, "cloud_env"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::SelectFlow, "resolve_auth"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::AcquireBaseIdentity, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::ExchangeOrDerive, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::MaybeImpersonate, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::FinalizeCredential, "scope_preflight"),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_lib_cloud_ops::{ENV_CREDENTIAL_POLICY_JSON, ENV_CREDENTIAL_POLICY_PROFILE};
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn gist_authenticate_bindings_follow_canonical_chain() {
        assert!(validate_authenticate_bindings(&gist_authenticate_bindings()).is_ok());
    }

    #[test]
    fn gist_resolve_auth_applies_policy_secret_binding() {
        with_env_lock(|| {
            std::env::set_var(
                ENV_CREDENTIAL_POLICY_JSON,
                r#"{
                    "version": 0,
                    "profiles": [{
                        "name": "prod",
                        "defaults": {
                            "provider": "Gcp",
                            "runtime": "GitHubActions"
                        },
                        "intents": [{
                            "intent": "github.gist.create",
                            "secret": { "name": "prod-github-token" },
                            "required_scopes": ["gist:write"]
                        }]
                    }]
                }"#,
            );
            std::env::set_var(ENV_CREDENTIAL_POLICY_PROFILE, "prod");

            let outputs = execute_gist_resolve_auth(HashMap::new()).expect("resolve auth");
            assert_eq!(
                outputs.get("secret_name"),
                Some(&Value::Str("prod-github-token".to_string()))
            );
            assert_eq!(
                outputs.get("required_scopes"),
                Some(&Value::str_list(vec!["gist:write".to_string()]))
            );
            assert_eq!(outputs.get("allow_impersonation"), Some(&Value::Bool(true)));
        });
    }

    fn request_filename(req: &gunbc_ir::transport::rest::RestRequest) -> String {
        let body = req.body.as_ref().expect("request body should exist");
        let files = body
            .get("files")
            .and_then(|v| v.as_object())
            .expect("request body should include files object");
        files
            .keys()
            .next()
            .cloned()
            .expect("request body should include one file entry")
    }

    fn request_description(req: &gunbc_ir::transport::rest::RestRequest) -> String {
        req.body
            .as_ref()
            .and_then(|b| b.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        clear_policy_env();
        let result = std::panic::catch_unwind(f);
        clear_policy_env();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    fn clear_policy_env() {
        std::env::remove_var(ENV_CREDENTIAL_POLICY_JSON);
        std::env::remove_var(ENV_CREDENTIAL_POLICY_PROFILE);
    }

    #[test]
    fn test_prepare_gist_request() {
        let request = prepare_gist_request("# Test", false, "Test gist", "test.md");

        match request {
            TransportRequest::Rest(req) => {
                assert_eq!(req.url, "https://api.github.com/gists");
                assert_eq!(request_filename(&req), "test.md");
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_prepare_gist_request_with_branch_filename() {
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "claude/my-feature", fixed_time);
        let request = prepare_gist_request("# Test", false, "Test gist", &filename);

        match request {
            TransportRequest::Rest(req) => {
                // Filename should start with sanitized branch name
                let file = request_filename(&req);
                let f_arg = Some(&file).filter(|a| a.starts_with("claude-my-feature_"));
                assert!(
                    f_arg.is_some(),
                    "expected filename with sanitized branch name, got filename: {}",
                    file
                );
                assert!(f_arg.unwrap().ends_with(".md"));
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_without_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                // Without branch, filename should start with "snapshot_"
                let filename = request_filename(req);
                let f_arg = Some(&filename).filter(|a| a.starts_with("snapshot_"));
                assert!(
                    f_arg.is_some(),
                    "expected snapshot filename, got filename: {}",
                    filename
                );
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_with_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        inputs.insert(
            "branch".to_string(),
            Value::Str("feature/cool-thing".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                let filename = request_filename(req);
                let f_arg = Some(&filename).filter(|a| a.starts_with("feature-cool-thing_"));
                assert!(
                    f_arg.is_some(),
                    "expected branch-based filename, got filename: {}",
                    filename
                );
                // Description should include the branch name
                let desc = request_description(req);
                assert!(
                    desc.contains("feature/cool-thing"),
                    "description should include original branch name"
                );
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_with_remote_branch_when_detached() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        // No local "branch" — simulates detached HEAD
        inputs.insert("remote_branch".to_string(), Value::Str("main".to_string()));
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                // Should use remote_branch ("main") for filename, not "snapshot"
                let filename = request_filename(req);
                let f_arg = Some(&filename).filter(|a| a.starts_with("main_"));
                assert!(
                    f_arg.is_some(),
                    "expected remote-branch-based filename, got filename: {}",
                    filename
                );
                // Description should mention the branch
                let desc = request_description(req);
                assert!(
                    desc.contains("main"),
                    "description should include remote branch name"
                );
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_gist_ops_local_branch_preferred_over_remote() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        // Both local and remote — local should win
        inputs.insert("branch".to_string(), Value::Str("my-feature".to_string()));
        inputs.insert("remote_branch".to_string(), Value::Str("main".to_string()));
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                // Should use local branch, not remote
                let filename = request_filename(req);
                let f_arg = Some(&filename).filter(|a| a.starts_with("my-feature_"));
                assert!(
                    f_arg.is_some(),
                    "local branch should take priority, got filename: {}",
                    filename
                );
            }
            _ => panic!("expected rest request"),
        }
    }

    // ========================================================================
    // Filename sanitization tests
    // ========================================================================

    fn test_fs() -> filename::FilesystemHandle {
        filename::FilesystemHandle::cross_platform(filename::Scope::Write)
    }

    #[test]
    fn test_sanitize_simple_branch() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "main"), "main");
        assert_eq!(sanitize_branch_for_filename(&fs, "develop"), "develop");
        assert_eq!(sanitize_branch_for_filename(&fs, "my-branch"), "my-branch");
    }

    #[test]
    fn test_sanitize_branch_with_slashes() {
        let fs = test_fs();
        assert_eq!(
            sanitize_branch_for_filename(&fs, "claude/branch-name"),
            "claude-branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "feature/foo/bar"),
            "feature-foo-bar"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "refs/heads/main"),
            "refs-heads-main"
        );
    }

    #[test]
    fn test_sanitize_branch_with_spaces() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "my branch"), "my-branch");
        assert_eq!(
            sanitize_branch_for_filename(&fs, "feature/foo bar"),
            "feature-foo-bar"
        );
    }

    #[test]
    fn test_sanitize_branch_windows_unsafe_chars() {
        let fs = test_fs();
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch:name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch*name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch?name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch<name>"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch|name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch\"name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch\\name"),
            "branch-name"
        );
    }

    #[test]
    fn test_sanitize_collapses_consecutive_hyphens() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "a//b"), "a-b");
        assert_eq!(sanitize_branch_for_filename(&fs, "a///b"), "a-b");
        assert_eq!(sanitize_branch_for_filename(&fs, "a/ /b"), "a-b");
    }

    #[test]
    fn test_sanitize_trims_leading_trailing_hyphens() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "/branch"), "branch");
        assert_eq!(sanitize_branch_for_filename(&fs, "branch/"), "branch");
        assert_eq!(sanitize_branch_for_filename(&fs, "/branch/"), "branch");
    }

    #[test]
    fn test_sanitize_empty_and_degenerate() {
        let fs = test_fs();
        assert_eq!(
            sanitize_branch_for_filename(&fs, ""),
            fallback_branch_name("")
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "/"),
            fallback_branch_name("/")
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "///"),
            fallback_branch_name("///")
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "   "),
            fallback_branch_name("   ")
        );
    }

    #[test]
    fn test_sanitize_preserves_dots_and_underscores() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "v1.0.0"), "v1.0.0");
        assert_eq!(sanitize_branch_for_filename(&fs, "my_branch"), "my_branch");
        assert_eq!(
            sanitize_branch_for_filename(&fs, "release/v2.0_rc1"),
            "release-v2.0_rc1"
        );
    }

    // ========================================================================
    // Timestamp and filename generation tests
    // ========================================================================

    #[test]
    fn test_format_utc_timestamp_known_value() {
        // 2024-01-15 13:30:00 UTC = 1705325400 seconds since epoch
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400);
        let ts = format_utc_timestamp(time);
        assert_eq!(ts, "2024-01-15_13-30-00");
    }

    #[test]
    fn test_format_utc_timestamp_epoch() {
        let ts = format_utc_timestamp(SystemTime::UNIX_EPOCH);
        assert_eq!(ts, "1970-01-01_00-00-00");
    }

    #[test]
    fn test_generate_gist_filename_format() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "main", fixed_time);
        assert_eq!(filename, "main_2024-01-15_13-30-00.md");
    }

    #[test]
    fn test_generate_gist_filename_sanitizes_branch() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "claude/improve-gist-filename", fixed_time);
        assert_eq!(
            filename,
            "claude-improve-gist-filename_2024-01-15_13-30-00.md"
        );
    }

    // ========================================================================
    // "untitled" sentinel collision — real branch vs degenerate input
    // ========================================================================

    #[test]
    fn test_sanitize_real_untitled_branch_preserved() {
        let fs = test_fs();
        // A real branch named "untitled" must NOT be turned into "snapshot"
        assert_eq!(sanitize_branch_for_filename(&fs, "untitled"), "untitled");
    }

    // ========================================================================
    // Deterministic timestamp tests
    // ========================================================================

    #[test]
    fn test_generate_gist_filename_deterministic_timestamp() {
        let fs = test_fs();
        let t1 = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let t2 = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );

        // Same inputs → same output (deterministic)
        let f1 = generate_gist_filename(&fs, "test", t1);
        let f2 = generate_gist_filename(&fs, "test", t2);
        assert_eq!(f1, f2);
    }

    // ========================================================================
    // Filename length capping
    // ========================================================================

    #[test]
    fn test_generate_gist_filename_caps_total_length() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );

        // Branch name that's 250 chars — after sanitization still 250 chars
        let long_branch = "a".repeat(250);
        let filename = generate_gist_filename(&fs, &long_branch, fixed_time);

        assert!(
            filename.len() <= 255,
            "filename {} bytes exceeds 255: {}",
            filename.len(),
            filename
        );
        assert!(filename.ends_with("_2024-01-15_13-30-00.md"));
    }

    // ========================================================================
    // Recent mode (base_ref) tests
    // ========================================================================

    #[test]
    fn test_gist_ops_prepare_with_base_ref() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Diff".to_string()));
        inputs.insert("branch".to_string(), Value::Str("main".to_string()));
        inputs.insert(
            "base_ref".to_string(),
            Value::Str("abc123def456".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                // Filename should contain recent-3d and short SHA
                let filename = request_filename(req);
                let f_arg = Some(&filename)
                    .filter(|a| a.contains("recent-3d") && a.contains("abc123d..HEAD"));
                assert!(
                    f_arg.is_some(),
                    "expected recent-mode filename with commit range, got filename: {}",
                    filename
                );
                assert!(f_arg.unwrap().starts_with("main_recent-3d_abc123d..HEAD_"));
                assert!(f_arg.unwrap().ends_with(".md"));

                // Description should mention the commit range
                let desc = request_description(req);
                assert!(
                    desc.contains("Recent changes (3d) abc123d..HEAD on main"),
                    "description should contain commit range and branch, got: {}",
                    desc
                );
            }
            _ => panic!("expected rest request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_without_base_ref_unchanged() {
        // Verify that snapshot/diff mode behavior is unchanged
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        inputs.insert("branch".to_string(), Value::Str("main".to_string()));
        // No base_ref — snapshot/diff mode
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:file".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                // Should NOT contain recent-3d
                let filename = request_filename(req);
                let has_recent = filename.contains("recent-3d");
                assert!(
                    !has_recent,
                    "snapshot mode should not have recent-3d, got filename: {}",
                    filename
                );
                // Should start with branch name
                let f_arg = Some(&filename).filter(|a| a.starts_with("main_"));
                assert!(f_arg.is_some());
            }
            _ => panic!("expected rest request"),
        }
    }
}
