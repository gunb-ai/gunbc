//! Code review types and operations for gunbc DAGs.
//!
//! Pure operations for building review prompts and parsing review responses.
//! All operations are PURE (no I/O). I/O happens through `TransportOps::Execute` nodes.
//!
//! # Architecture
//!
//! ```text
//! Review Layer (domain-specific: criteria → question, response → findings)
//!    ↓
//! LlmOps Layer (generic: content + question → answer)
//!    ↓
//! TransportOps (I/O boundary)
//! ```
//!
//! # Example DAG Flow
//!
//! ```text
//! BlobOps::Acquire → ReviewOps::PrepareReviewPrompt → LlmOps::PrepareSimpleRequest
//!     → TransportOps::Execute → LlmOps::ParseSimpleResponse → ReviewOps::ParseReviewResponse
//! ```

pub mod dimension;
pub mod profile;

use gunbc_exec::{
    optional_str_strict, propagate_skipped, require_json, require_map_str_str, require_str,
    ExecError, Executable, IntoExecResult, OutputMap,
};
use gunbc_ir::transport::review::ReviewScopeContract;
use gunbc_ir::transport::ScopeContract;
use gunbc_ir::Value;
use gunbc_primitives::{FormatMapOp, StableHashOp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Types
// ============================================================================

/// Review criteria definition.
///
/// Criteria define what aspects of code to review and what questions to ask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criteria {
    /// Name of this criteria set (e.g., "security", "performance").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Individual checks within this criteria.
    pub checks: Vec<Check>,
}

/// A single check within a criteria set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    /// Unique identifier for this check (e.g., "sql-injection").
    pub id: String,
    /// The question to ask the LLM about this aspect.
    pub question: String,
    /// Example findings to guide the LLM.
    #[serde(default)]
    pub examples: Vec<String>,
}

/// A finding from a review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Stable hash ID from issue_key + check_id.
    pub id: String,
    /// Which check this finding is for.
    pub check_id: String,
    /// Reviewer-provided stable key (no line numbers, content-based).
    pub issue_key: String,
    /// Where the issue was found.
    pub location: Location,
    /// Description of the issue.
    pub observation: String,
    /// Suggested fix if available.
    #[serde(default)]
    pub candidate_fix: Option<String>,
}

/// Location of a finding in the reviewed content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Location {
    /// File and line number.
    FileLine { file: String, line: u32 },
    /// Span of lines.
    Span { file: String, start: u32, end: u32 },
    /// Line in a diff (left/right side).
    DiffLine { side: DiffSide, line: u32 },
    /// No specific location.
    Unlocated,
}

/// Side of a diff for DiffLine locations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffSide {
    Left,
    Right,
}

/// Output from a single review source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOutput {
    /// Schema version for forward compatibility.
    pub schema_version: String,
    /// Name of the criteria used.
    pub criteria_name: String,
    /// Source of this review ("llm", "clippy", "cargo-check").
    pub source: String,
    /// List of findings.
    pub findings: Vec<Finding>,
    /// Optional candidate remediations.
    #[serde(default)]
    pub candidate_remediations: Option<CandidateRemediations>,
    /// Summary of the review.
    pub summary: String,
}

impl ReviewOutput {
    /// Current schema version.
    pub const SCHEMA_VERSION: &'static str = "0.1.0";

    /// Create a new ReviewOutput with the current schema version.
    pub fn new(criteria_name: String, source: String) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            criteria_name,
            source,
            findings: Vec::new(),
            candidate_remediations: None,
            summary: String::new(),
        }
    }
}

/// Bundle of multiple review outputs (from different sources).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewBundle {
    /// Individual outputs from each source.
    pub outputs: Vec<ReviewOutput>,
    /// Merged and deduplicated findings.
    pub merged_findings: Vec<Finding>,
}

/// Candidate remediations suggested by the reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRemediations {
    /// Goals for the remediation.
    pub goals: Vec<String>,
    /// Constraints to respect.
    pub constraints: Vec<String>,
    /// Specific tasks to perform.
    pub tasks: Vec<CandidateTask>,
}

/// A single candidate remediation task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTask {
    /// Which finding this addresses.
    pub finding_id: String,
    /// File to modify.
    pub file: String,
    /// What to do.
    pub intent: String,
    /// Suggested patch content.
    #[serde(default)]
    pub candidate_patch: Option<String>,
    /// How to validate the fix.
    #[serde(default)]
    pub validation: Option<String>,
}

// ============================================================================
// Pipeline Configuration
// ============================================================================

/// Build-time configuration for a review pipeline.
///
/// These values are baked into the DAG at construction time. They are NOT CLI flags — the
/// pipeline owns these decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPipelineConfig {
    /// LLM provider (e.g., "openai", "anthropic").
    pub provider: String,
    /// LLM model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514").
    pub model: String,
    /// Review criteria to apply.
    pub criteria: Criteria,
    /// Default branch for diff operations (e.g., "main", "master", "develop").
    ///
    /// Sourced from `GitConfig::default_branch` — the single source of truth
    /// for the repo's branching model. The graph builder uses this instead of
    /// hardcoding a branch name.
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

fn default_branch() -> String {
    "main".to_string()
}

/// Default code review criteria.
///
/// Covers common code quality checks suitable for general-purpose review.
pub fn default_criteria() -> Criteria {
    Criteria {
        name: "code-review".to_string(),
        description: "General code review for correctness, security, and quality".to_string(),
        checks: vec![
            Check {
                id: "correctness".to_string(),
                question: "Are there any bugs, logic errors, or incorrect behavior?".to_string(),
                examples: vec![
                    "Off-by-one errors".to_string(),
                    "Null/None dereference".to_string(),
                    "Missing error handling".to_string(),
                ],
            },
            Check {
                id: "security".to_string(),
                question: "Are there any security vulnerabilities?".to_string(),
                examples: vec![
                    "SQL injection".to_string(),
                    "Command injection".to_string(),
                    "Hardcoded credentials".to_string(),
                ],
            },
            Check {
                id: "performance".to_string(),
                question: "Are there any performance issues or inefficiencies?".to_string(),
                examples: vec![
                    "Unnecessary allocations in hot paths".to_string(),
                    "O(n^2) where O(n) is possible".to_string(),
                ],
            },
            Check {
                id: "clarity".to_string(),
                question: "Is the code clear and maintainable?".to_string(),
                examples: vec![
                    "Confusing variable names".to_string(),
                    "Missing context for complex logic".to_string(),
                ],
            },
        ],
    }
}

impl ReviewPipelineConfig {
    /// Default pipeline config for gunbc's own repo.
    ///
    /// For repo-specific config, construct a `ReviewPipelineConfig` directly
    /// or deserialize from a config file (e.g., `gunbc.toml`). The
    /// `default_branch` field defaults to `"main"` via `GitConfig::default()`.
    pub fn gunbc_default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            criteria: crate::default_criteria(),
            default_branch: "main".to_string(),
        }
    }
}

// ============================================================================
// ReviewOps
// ============================================================================

/// Review operations for use in DAG nodes.
///
/// All operations are PURE - no I/O.
#[derive(Debug, Clone)]
pub enum ReviewOps {
    /// Emit pipeline configuration as port values (zero-input config node).
    ///
    /// Follows the same pattern as `EnvOp::Ci` in the CI pipeline — a node
    /// with no inputs that outputs build-time constants.
    ///
    /// Outputs:
    /// - `provider`: String — LLM provider ID
    /// - `model`: String — LLM model identifier
    /// - `criteria`: Json — Criteria definition
    LoadPipelineConfig(ReviewPipelineConfig),

    /// Build a review prompt from artifact and criteria.
    ///
    /// Inputs:
    /// - `artifact`: String - the code/content to review
    /// - `criteria`: Json - Criteria definition
    /// - `context` (optional): String - extra context
    ///
    /// Outputs:
    /// - `question`: String - formatted question for LLM
    /// - `system_prompt`: String - system prompt for LLM
    PrepareReviewPrompt,

    /// Resolve review auth requirements from the review scope contract.
    ///
    /// Inputs:
    /// - `provider`: String - LLM provider ID ("openai", "anthropic")
    ///
    /// Outputs:
    /// - `service`: String - credential service ID
    /// - `scheme`: String - auth scheme ("bearer" or "header")
    /// - `header_name`: String - header name for "header" scheme
    /// - `required_scopes`: List<String> - includes `review:code_review`
    /// - `interactive_allowed`: Bool - whether interactive remediation is allowed
    ResolveAuthContract,

    /// Parse an LLM response into structured findings.
    ///
    /// Inputs:
    /// - `answer`: String - LLM response text
    /// - `criteria`: Json - for check_id resolution
    ///
    /// Outputs:
    /// - `output`: Json - ReviewOutput as JSON
    /// - `errors`: Json - parse errors/warnings as array
    ParseReviewResponse,

    /// Merge multiple ReviewOutputs into a ReviewBundle.
    ///
    /// The `outputs` port is a list port (cardinality `[0, *)`) — the engine
    /// collects fan-in edges into a `Value::List` automatically. Each element
    /// is a `Value::Json` containing a serialized `ReviewOutput`.
    ///
    /// Inputs:
    /// - `outputs`: List<Json> - collected ReviewOutput values (engine-managed)
    ///
    /// Outputs:
    /// - `bundle`: Json - merged ReviewBundle
    /// - `conflicts`: Json - finding ID conflicts
    MergeOutputs,

    /// Generate a stable finding ID from issue_key and check_id.
    ///
    /// Inputs:
    /// - `issue_key`: String - reviewer-provided stable key
    /// - `check_id`: String - which check this finding is for
    ///
    /// Outputs:
    /// - `finding_id`: String - stable hash ID
    HashFinding,

    /// Format a diff_files map into a single artifact string for review.
    ///
    /// Inputs:
    /// - `diff_files`: Map - map of filename → unified diff chunk
    ///
    /// Outputs:
    /// - `artifact`: String - formatted diff text suitable for LLM review
    FormatDiffArtifact,
}

impl Executable for ReviewOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            ReviewOps::LoadPipelineConfig(config) => execute_load_pipeline_config(config),
            ReviewOps::PrepareReviewPrompt => execute_prepare_review_prompt(inputs),
            ReviewOps::ResolveAuthContract => execute_resolve_auth_contract(inputs),
            ReviewOps::ParseReviewResponse => execute_parse_review_response(inputs),
            ReviewOps::MergeOutputs => execute_merge_outputs(inputs),
            ReviewOps::HashFinding => execute_hash_finding(inputs),
            ReviewOps::FormatDiffArtifact => execute_format_diff_artifact(inputs),
        }
    }
}

// ============================================================================
// Operation Implementations
// ============================================================================

fn execute_load_pipeline_config(
    config: &ReviewPipelineConfig,
) -> Result<HashMap<String, Value>, ExecError> {
    let criteria_json =
        serde_json::to_value(&config.criteria).exec_context("failed to serialize criteria")?;

    OutputMap::new()
        .str("provider", config.provider.clone())
        .str("model", config.model.clone())
        .json("criteria", criteria_json)
        .ok()
}

fn execute_prepare_review_prompt(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // artifact is validated but passed separately to LLM via PrepareSimpleRequest.content
    let _artifact = require_str(&inputs, "artifact")?;

    let criteria: Criteria = serde_json::from_value(require_json(&inputs, "criteria")?.clone())
        .exec_context("invalid 'criteria' JSON")?;

    let context = optional_str_strict(&inputs, "context")?.unwrap_or("");

    // Build the question from criteria
    let mut question_parts = vec![format!(
        "Review the following code for: {}",
        criteria.description
    )];

    if !context.is_empty() {
        question_parts.push(format!("Context: {}", context));
    }

    question_parts.push(String::new()); // blank line
    question_parts.push("For each issue found, respond in JSON format:".to_string());
    question_parts.push(r#"{"findings": [{"check_id": "...", "issue_key": "...", "location": {...}, "observation": "...", "candidate_fix": "..."}], "summary": "..."}"#.to_string());
    question_parts.push(String::new());
    question_parts.push("Checks to perform:".to_string());

    for check in &criteria.checks {
        question_parts.push(format!("- [{}] {}", check.id, check.question));
        for example in &check.examples {
            question_parts.push(format!("  Example: {}", example));
        }
    }

    let question = question_parts.join("\n");

    let system_prompt = format!(
        "You are an expert code reviewer specializing in {}. \
        Analyze code thoroughly and report issues in valid JSON format. \
        Use stable issue_key values based on the nature of the issue (not line numbers). \
        Only report genuine issues - don't invent problems.",
        criteria.name
    );

    OutputMap::new()
        .str("question", question)
        .str("system_prompt", system_prompt)
        .ok()
}

fn execute_resolve_auth_contract(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let provider = require_str(&inputs, "provider")?;
    let intent = ReviewScopeContract::new(provider).credential_intent();
    intent
        .validate()
        .map_err(|e| ExecError::new(format!("invalid review credential contract: {e}")))?;

    let secret_name = intent.secret_name.ok_or_else(|| {
        ExecError::new(format!(
            "ResolveAuthContract: secret_name is required for service '{}'",
            intent.service
        ))
    })?;
    OutputMap::new()
        .str("service", intent.service)
        .str("secret_name", secret_name)
        .str("scheme", intent.scheme)
        .str("header_name", intent.header_name)
        .str_list("required_scopes", intent.required_scopes)
        .bool("interactive_allowed", intent.interactive_allowed)
        .ok()
}

pub(crate) fn execute_parse_review_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "answer", &["output", "errors"]) {
        return result;
    }

    let answer = require_str(&inputs, "answer")?;

    let criteria: Criteria = serde_json::from_value(require_json(&inputs, "criteria")?.clone())
        .exec_context("invalid 'criteria' JSON")?;

    let mut errors = Vec::new();

    // Try to parse the LLM response as JSON
    let parsed: serde_json::Value = match extract_json_from_response(answer) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("Failed to parse JSON: {}", e));
            serde_json::json!({"findings": [], "summary": "Parse error"})
        }
    };

    // Extract findings
    let mut findings = Vec::new();
    if let Some(arr) = parsed.get("findings").and_then(|v| v.as_array()) {
        for (i, item) in arr.iter().enumerate() {
            match parse_finding(item, &criteria) {
                Ok(finding) => findings.push(finding),
                Err(e) => errors.push(format!("Finding[{}]: {}", i, e)),
            }
        }
    }

    let summary = parsed
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let output = ReviewOutput {
        schema_version: ReviewOutput::SCHEMA_VERSION.to_string(),
        criteria_name: criteria.name,
        source: "llm".to_string(),
        findings,
        candidate_remediations: None,
        summary,
    };

    OutputMap::new()
        .json("output", serde_json::to_value(&output).unwrap())
        .json("errors", serde_json::to_value(&errors).unwrap())
        .ok()
}

fn execute_merge_outputs(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // The engine delivers Value::List for list ports (fan-in collected automatically).
    // Missing key means 0 upstream edges → empty list.
    let outputs: Vec<ReviewOutput> = match inputs.get("outputs") {
        None => vec![],
        Some(Value::List(items)) => {
            let mut outputs = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let j = item
                    .as_json()
                    .ok_or_else(|| ExecError::new(format!("outputs[{}] is not JSON", i)))?;
                let output: ReviewOutput = serde_json::from_value(j.clone())
                    .with_exec_context(|| format!("invalid outputs[{}]", i))?;
                outputs.push(output);
            }
            outputs
        }
        Some(other) => {
            return Err(ExecError::new(format!(
                "expected List for 'outputs', got {:?}",
                std::mem::discriminant(other)
            )))
        }
    };

    // Merge findings, deduplicating by ID
    let mut seen_ids: HashMap<String, usize> = HashMap::new();
    let mut merged_findings = Vec::new();
    let mut conflicts = Vec::new();

    for output in &outputs {
        for finding in &output.findings {
            if let Some(existing_idx) = seen_ids.get(&finding.id) {
                conflicts.push(serde_json::json!({
                    "finding_id": finding.id,
                    "existing_source": outputs.get(*existing_idx).map(|o| &o.source),
                    "duplicate_source": output.source,
                }));
            } else {
                seen_ids.insert(finding.id.clone(), merged_findings.len());
                merged_findings.push(finding.clone());
            }
        }
    }

    let bundle = ReviewBundle {
        outputs,
        merged_findings,
    };

    OutputMap::new()
        .json("bundle", serde_json::to_value(&bundle).unwrap())
        .json("conflicts", serde_json::to_value(&conflicts).unwrap())
        .ok()
}

fn execute_hash_finding(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let issue_key = require_str(&inputs, "issue_key")?;
    let check_id = require_str(&inputs, "check_id")?;

    let finding_id = hash_finding_id(issue_key, check_id);

    OutputMap::new().str("finding_id", finding_id).ok()
}

fn execute_format_diff_artifact(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let diff_files = match inputs.get("diff_files") {
        Some(Value::Skipped) => std::collections::BTreeMap::new(),
        _ => require_map_str_str(&inputs, "diff_files")?,
    };

    // Use FormatMapOp::DiffArtifact for consistent formatting
    let artifact = FormatMapOp::DiffArtifact.format_entries(&diff_files);

    OutputMap::new().str("artifact", artifact).ok()
}

// ============================================================================
// Helpers
// ============================================================================

/// Generate a stable finding ID from issue_key and check_id.
///
/// Uses `StableHashOp::hash_parts` internally for consistent hashing across
/// the codebase. The order is `[check_id, issue_key]` to maintain compatibility
/// with the original implementation.
pub fn hash_finding_id(issue_key: &str, check_id: &str) -> String {
    // Note: check_id comes first in the hash for historical reasons
    StableHashOp::hash_parts(&[check_id, issue_key])
}

/// Try to extract JSON from an LLM response.
///
/// Handles responses that might have markdown code blocks or prose around the JSON.
fn extract_json_from_response(response: &str) -> Result<serde_json::Value, String> {
    let trimmed = response.trim();

    // Try direct parse first
    if let Ok(v) = serde_json::from_str(trimmed) {
        return Ok(v);
    }

    // Try to find JSON in markdown code block
    if let Some(start) = trimmed.find("```json") {
        let after_marker = &trimmed[start + 7..];
        if let Some(end) = after_marker.find("```") {
            let json_str = after_marker[..end].trim();
            if let Ok(v) = serde_json::from_str(json_str) {
                return Ok(v);
            }
        }
    }

    // Try to find JSON in plain code block
    if let Some(start) = trimmed.find("```\n") {
        let after_marker = &trimmed[start + 4..];
        if let Some(end) = after_marker.find("```") {
            let json_str = after_marker[..end].trim();
            if let Ok(v) = serde_json::from_str(json_str) {
                return Ok(v);
            }
        }
    }

    // Try to find JSON object by looking for { ... }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if start < end {
                let json_str = &trimmed[start..=end];
                if let Ok(v) = serde_json::from_str(json_str) {
                    return Ok(v);
                }
            }
        }
    }

    Err("Could not extract valid JSON from response".to_string())
}

/// Parse a single finding from JSON.
fn parse_finding(item: &serde_json::Value, criteria: &Criteria) -> Result<Finding, String> {
    let check_id = item
        .get("check_id")
        .and_then(|v| v.as_str())
        .ok_or("missing check_id")?
        .to_string();

    // Validate check_id against criteria
    if !criteria.checks.iter().any(|c| c.id == check_id) {
        return Err(format!("unknown check_id: {}", check_id));
    }

    let issue_key = item
        .get("issue_key")
        .and_then(|v| v.as_str())
        .ok_or("missing issue_key")?
        .to_string();

    let location = parse_location(item.get("location"))?;

    let observation = item
        .get("observation")
        .and_then(|v| v.as_str())
        .ok_or("missing observation")?
        .to_string();

    let candidate_fix = item
        .get("candidate_fix")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let id = hash_finding_id(&issue_key, &check_id);

    Ok(Finding {
        id,
        check_id,
        issue_key,
        location,
        observation,
        candidate_fix,
    })
}

/// Parse a location from JSON.
fn parse_location(loc: Option<&serde_json::Value>) -> Result<Location, String> {
    let Some(loc) = loc else {
        return Ok(Location::Unlocated);
    };

    // Handle serde-style tagged enum format
    if let Some(type_tag) = loc.get("type").and_then(|v| v.as_str()) {
        match type_tag {
            "file_line" => {
                let file = loc
                    .get("file")
                    .and_then(|v| v.as_str())
                    .ok_or("file_line missing file")?
                    .to_string();
                let line = loc
                    .get("line")
                    .and_then(|v| v.as_u64())
                    .ok_or("file_line missing line")? as u32;
                return Ok(Location::FileLine { file, line });
            }
            "span" => {
                let file = loc
                    .get("file")
                    .and_then(|v| v.as_str())
                    .ok_or("span missing file")?
                    .to_string();
                let start = loc
                    .get("start")
                    .and_then(|v| v.as_u64())
                    .ok_or("span missing start")? as u32;
                let end = loc
                    .get("end")
                    .and_then(|v| v.as_u64())
                    .ok_or("span missing end")? as u32;
                return Ok(Location::Span { file, start, end });
            }
            "diff_line" => {
                let side = match loc.get("side").and_then(|v| v.as_str()) {
                    Some("left") => DiffSide::Left,
                    Some("right") => DiffSide::Right,
                    _ => return Err("diff_line missing or invalid side".to_string()),
                };
                let line = loc
                    .get("line")
                    .and_then(|v| v.as_u64())
                    .ok_or("diff_line missing line")? as u32;
                return Ok(Location::DiffLine { side, line });
            }
            "unlocated" => return Ok(Location::Unlocated),
            _ => return Err(format!("unknown location type: {}", type_tag)),
        }
    }

    // Handle simplified format (just file + line)
    if let (Some(file), Some(line)) = (
        loc.get("file").and_then(|v| v.as_str()),
        loc.get("line").and_then(|v| v.as_u64()),
    ) {
        if let Some(end) = loc.get("end").and_then(|v| v.as_u64()) {
            return Ok(Location::Span {
                file: file.to_string(),
                start: line as u32,
                end: end as u32,
            });
        }
        return Ok(Location::FileLine {
            file: file.to_string(),
            line: line as u32,
        });
    }

    Ok(Location::Unlocated)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_criteria;

    fn test_criteria() -> Criteria {
        Criteria {
            name: "security".to_string(),
            description: "Security code review".to_string(),
            checks: vec![
                Check {
                    id: "sql-injection".to_string(),
                    question: "Check for SQL injection vulnerabilities".to_string(),
                    examples: vec!["Using string concatenation in SQL queries".to_string()],
                },
                Check {
                    id: "xss".to_string(),
                    question: "Check for XSS vulnerabilities".to_string(),
                    examples: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_default_criteria() {
        let criteria = default_criteria();
        assert_eq!(criteria.name, "code-review");
        assert_eq!(criteria.checks.len(), 4);
        assert!(criteria.checks.iter().any(|c| c.id == "correctness"));
        assert!(criteria.checks.iter().any(|c| c.id == "security"));
    }

    #[test]
    fn test_prepare_review_prompt() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "artifact".to_string(),
            Value::Str(
                "fn foo() { query(format!(\"SELECT * FROM users WHERE id = {}\", id)); }"
                    .to_string(),
            ),
        );
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );

        let result = ReviewOps::PrepareReviewPrompt.execute(inputs).unwrap();

        let question = result.get("question").unwrap().as_str().unwrap();
        assert!(question.contains("Security code review"));
        assert!(question.contains("sql-injection"));
        assert!(question.contains("xss"));

        let system = result.get("system_prompt").unwrap().as_str().unwrap();
        assert!(system.contains("security"));
    }

    #[test]
    fn test_prepare_review_prompt_with_context() {
        let mut inputs = HashMap::new();
        inputs.insert("artifact".to_string(), Value::Str("code".to_string()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );
        inputs.insert(
            "context".to_string(),
            Value::Str("This is a web application backend".to_string()),
        );

        let result = ReviewOps::PrepareReviewPrompt.execute(inputs).unwrap();

        let question = result.get("question").unwrap().as_str().unwrap();
        assert!(question.contains("web application backend"));
    }

    #[test]
    fn test_resolve_auth_contract_openai_includes_review_scope() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));

        let result = ReviewOps::ResolveAuthContract
            .execute(inputs)
            .expect("resolve_auth_contract should succeed for openai");

        assert_eq!(
            result.get("service").and_then(Value::as_str),
            Some("openai")
        );
        assert_eq!(result.get("scheme").and_then(Value::as_str), Some("bearer"));
        let scopes = result
            .get("required_scopes")
            .and_then(Value::as_str_list)
            .expect("required_scopes should be present");
        assert!(scopes.iter().any(|s| s == "llm:chat_completion"));
        assert!(scopes.iter().any(|s| s == "review:code_review"));
    }

    #[test]
    fn test_resolve_auth_contract_anthropic_header() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));

        let result = ReviewOps::ResolveAuthContract
            .execute(inputs)
            .expect("resolve_auth_contract should succeed for anthropic");

        assert_eq!(result.get("scheme").and_then(Value::as_str), Some("header"));
        assert_eq!(
            result.get("header_name").and_then(Value::as_str),
            Some("x-api-key")
        );
    }

    #[test]
    fn test_parse_review_response_valid() {
        let response = r#"{
            "findings": [
                {
                    "check_id": "sql-injection",
                    "issue_key": "user_query_concat",
                    "location": {"type": "file_line", "file": "src/db.rs", "line": 42},
                    "observation": "SQL query built with string concatenation",
                    "candidate_fix": "Use parameterized queries"
                }
            ],
            "summary": "Found 1 SQL injection vulnerability"
        }"#;

        let mut inputs = HashMap::new();
        inputs.insert("answer".to_string(), Value::Str(response.to_string()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );

        let result = ReviewOps::ParseReviewResponse.execute(inputs).unwrap();

        let output: ReviewOutput =
            serde_json::from_value(result.get("output").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(output.findings.len(), 1);
        assert_eq!(output.findings[0].check_id, "sql-injection");
        assert_eq!(output.findings[0].issue_key, "user_query_concat");
        assert!(!output.findings[0].id.is_empty());
        assert_eq!(output.summary, "Found 1 SQL injection vulnerability");
    }

    #[test]
    fn test_parse_review_response_markdown() {
        let response = r#"Here are the findings:

```json
{
    "findings": [
        {
            "check_id": "xss",
            "issue_key": "unescaped_output",
            "observation": "Unescaped user input in HTML"
        }
    ],
    "summary": "Found XSS issue"
}
```

Please fix these issues."#;

        let mut inputs = HashMap::new();
        inputs.insert("answer".to_string(), Value::Str(response.to_string()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );

        let result = ReviewOps::ParseReviewResponse.execute(inputs).unwrap();

        let output: ReviewOutput =
            serde_json::from_value(result.get("output").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(output.findings.len(), 1);
        assert_eq!(output.findings[0].check_id, "xss");
    }

    #[test]
    fn test_parse_review_response_invalid_check_id() {
        let response = r#"{
            "findings": [
                {
                    "check_id": "unknown-check",
                    "issue_key": "test",
                    "observation": "test"
                }
            ],
            "summary": "test"
        }"#;

        let mut inputs = HashMap::new();
        inputs.insert("answer".to_string(), Value::Str(response.to_string()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );

        let result = ReviewOps::ParseReviewResponse.execute(inputs).unwrap();

        let errors: Vec<String> =
            serde_json::from_value(result.get("errors").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert!(errors.iter().any(|e| e.contains("unknown check_id")));
    }

    /// Helper: wrap ReviewOutputs as a Value::List of Value::Json,
    /// matching what the engine delivers to list ports.
    fn outputs_list(outputs: &[ReviewOutput]) -> Value {
        Value::List(
            outputs
                .iter()
                .map(|o| Value::Json(serde_json::to_value(o).unwrap()))
                .collect(),
        )
    }

    #[test]
    fn test_merge_outputs() {
        let output1 = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "security".to_string(),
            source: "llm".to_string(),
            findings: vec![Finding {
                id: "abc123".to_string(),
                check_id: "sql-injection".to_string(),
                issue_key: "test1".to_string(),
                location: Location::Unlocated,
                observation: "Issue 1".to_string(),
                candidate_fix: None,
            }],
            candidate_remediations: None,
            summary: "LLM review".to_string(),
        };

        let output2 = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "security".to_string(),
            source: "clippy".to_string(),
            findings: vec![Finding {
                id: "def456".to_string(),
                check_id: "xss".to_string(),
                issue_key: "test2".to_string(),
                location: Location::Unlocated,
                observation: "Issue 2".to_string(),
                candidate_fix: None,
            }],
            candidate_remediations: None,
            summary: "Clippy review".to_string(),
        };

        let mut inputs = HashMap::new();
        inputs.insert("outputs".to_string(), outputs_list(&[output1, output2]));

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();

        let bundle: ReviewBundle =
            serde_json::from_value(result.get("bundle").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(bundle.outputs.len(), 2);
        assert_eq!(bundle.merged_findings.len(), 2);
    }

    #[test]
    fn test_merge_outputs_single() {
        // MergeOutputs with a single-element list (one upstream edge)
        let output = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "security".to_string(),
            source: "llm".to_string(),
            findings: vec![Finding {
                id: "abc123".to_string(),
                check_id: "sql-injection".to_string(),
                issue_key: "test1".to_string(),
                location: Location::Unlocated,
                observation: "Issue 1".to_string(),
                candidate_fix: None,
            }],
            candidate_remediations: None,
            summary: "LLM review".to_string(),
        };

        let mut inputs = HashMap::new();
        inputs.insert("outputs".to_string(), outputs_list(&[output]));

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();

        let bundle: ReviewBundle =
            serde_json::from_value(result.get("bundle").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(bundle.outputs.len(), 1);
        assert_eq!(bundle.merged_findings.len(), 1);
        assert_eq!(bundle.outputs[0].source, "llm");
    }

    #[test]
    fn test_merge_outputs_empty_list() {
        // Engine delivers an empty list when list port has 0 fan-in edges
        let mut inputs = HashMap::new();
        inputs.insert("outputs".to_string(), Value::List(vec![]));

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();
        let bundle: ReviewBundle =
            serde_json::from_value(result.get("bundle").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(bundle.outputs.len(), 0);
        assert_eq!(bundle.merged_findings.len(), 0);
    }

    #[test]
    fn test_merge_outputs_missing() {
        // No upstream edges at all → key absent from inputs
        let inputs = HashMap::new();

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();
        let bundle: ReviewBundle =
            serde_json::from_value(result.get("bundle").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(bundle.outputs.len(), 0);
        assert_eq!(bundle.merged_findings.len(), 0);
    }

    #[test]
    fn test_merge_outputs_with_conflict() {
        let finding = Finding {
            id: "same-id".to_string(),
            check_id: "sql-injection".to_string(),
            issue_key: "test".to_string(),
            location: Location::Unlocated,
            observation: "Issue".to_string(),
            candidate_fix: None,
        };

        let output1 = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "security".to_string(),
            source: "llm".to_string(),
            findings: vec![finding.clone()],
            candidate_remediations: None,
            summary: "LLM".to_string(),
        };

        let output2 = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "security".to_string(),
            source: "clippy".to_string(),
            findings: vec![finding],
            candidate_remediations: None,
            summary: "Clippy".to_string(),
        };

        let mut inputs = HashMap::new();
        inputs.insert("outputs".to_string(), outputs_list(&[output1, output2]));

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();

        let conflicts: Vec<serde_json::Value> =
            serde_json::from_value(result.get("conflicts").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0]["finding_id"], "same-id");
    }

    #[test]
    fn test_hash_finding() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "issue_key".to_string(),
            Value::Str("user_query_concat".to_string()),
        );
        inputs.insert(
            "check_id".to_string(),
            Value::Str("sql-injection".to_string()),
        );

        let result = ReviewOps::HashFinding.execute(inputs).unwrap();

        let finding_id = result.get("finding_id").unwrap().as_str().unwrap();
        assert_eq!(finding_id.len(), 32); // 16 bytes as hex

        // Same inputs should produce same hash
        let mut inputs2 = HashMap::new();
        inputs2.insert(
            "issue_key".to_string(),
            Value::Str("user_query_concat".to_string()),
        );
        inputs2.insert(
            "check_id".to_string(),
            Value::Str("sql-injection".to_string()),
        );

        let result2 = ReviewOps::HashFinding.execute(inputs2).unwrap();
        assert_eq!(result.get("finding_id"), result2.get("finding_id"));
    }

    #[test]
    fn test_hash_finding_deterministic() {
        // Verify the hash is deterministic
        let id1 = hash_finding_id("key1", "check1");
        let id2 = hash_finding_id("key1", "check1");
        assert_eq!(id1, id2);

        // Different inputs produce different hashes
        let id3 = hash_finding_id("key2", "check1");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_location_parsing() {
        // FileLine
        let loc = parse_location(Some(&serde_json::json!({
            "type": "file_line",
            "file": "src/main.rs",
            "line": 42
        })))
        .unwrap();
        assert!(
            matches!(loc, Location::FileLine { file, line } if file == "src/main.rs" && line == 42)
        );

        // Span
        let loc = parse_location(Some(&serde_json::json!({
            "type": "span",
            "file": "src/lib.rs",
            "start": 10,
            "end": 20
        })))
        .unwrap();
        assert!(
            matches!(loc, Location::Span { file, start, end } if file == "src/lib.rs" && start == 10 && end == 20)
        );

        // Simplified format
        let loc = parse_location(Some(&serde_json::json!({
            "file": "src/foo.rs",
            "line": 5
        })))
        .unwrap();
        assert!(matches!(loc, Location::FileLine { line, .. } if line == 5));

        // Unlocated
        let loc = parse_location(None).unwrap();
        assert!(matches!(loc, Location::Unlocated));
    }

    #[test]
    fn test_extract_json_from_response() {
        // Direct JSON
        let json = extract_json_from_response(r#"{"key": "value"}"#).unwrap();
        assert_eq!(json["key"], "value");

        // Markdown code block
        let json = extract_json_from_response(
            r#"Here's the result:
```json
{"key": "value"}
```
"#,
        )
        .unwrap();
        assert_eq!(json["key"], "value");

        // JSON buried in text
        let json =
            extract_json_from_response(r#"The analysis shows {"key": "value"} as the result."#)
                .unwrap();
        assert_eq!(json["key"], "value");
    }

    #[test]
    fn test_review_output_schema_version() {
        let output = ReviewOutput::new("test".to_string(), "llm".to_string());
        assert_eq!(output.schema_version, ReviewOutput::SCHEMA_VERSION);
    }

    #[test]
    fn test_format_diff_artifact() {
        use std::collections::BTreeMap;

        let mut diff_files = BTreeMap::new();
        diff_files.insert(
            "src/main.rs".to_string(),
            "@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }".to_string(),
        );
        diff_files.insert(
            "src/lib.rs".to_string(),
            "@@ -1 +1,2 @@\n pub fn foo() {}\n+pub fn bar() {}".to_string(),
        );

        let mut inputs = HashMap::new();
        inputs.insert("diff_files".to_string(), Value::str_map(diff_files));

        let result = ReviewOps::FormatDiffArtifact.execute(inputs).unwrap();
        let artifact = result.get("artifact").unwrap().as_str().unwrap();

        assert!(artifact.contains("src/main.rs"));
        assert!(artifact.contains("src/lib.rs"));
        assert!(artifact.contains("println!"));
    }

    #[test]
    fn test_format_diff_artifact_empty() {
        use std::collections::BTreeMap;

        let mut inputs = HashMap::new();
        inputs.insert("diff_files".to_string(), Value::Map(BTreeMap::new()));

        let result = ReviewOps::FormatDiffArtifact.execute(inputs).unwrap();
        let artifact = result.get("artifact").unwrap().as_str().unwrap();
        assert_eq!(artifact, "(no changes)");
    }
}

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "review",
    crate_name = "gunbc-lib-review",
    description = "Review code changes using LLM analysis",
    builder = "build_review_graph_dsl",
    import = "use gunbc_dag::build_review_graph_dsl;",
    package = "dag",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","help":"Repository path to diff","make_var":"REPO"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff (default: main)"}]"#,
    dsl_module = "review",
    returns_result
)]
pub fn review_tool() {}

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}
