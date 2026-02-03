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

pub mod graph;
pub mod graph_mock;

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
// ReviewOps
// ============================================================================

/// Review operations for use in DAG nodes.
///
/// All operations are PURE - no I/O.
#[derive(Debug, Clone)]
pub enum ReviewOps {
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
    /// Inputs:
    /// - `outputs`: Json - array of ReviewOutput objects
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
    /// - `diff_files`: MapStrStr - map of filename → unified diff chunk
    ///
    /// Outputs:
    /// - `artifact`: String - formatted diff text suitable for LLM review
    FormatDiffArtifact,
}

impl Executable for ReviewOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            ReviewOps::PrepareReviewPrompt => execute_prepare_review_prompt(inputs),
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

fn execute_prepare_review_prompt(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // artifact is validated but passed separately to LLM via PrepareSimpleRequest.content
    let _artifact = inputs
        .get("artifact")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'artifact' input"))?;

    let criteria: Criteria = inputs
        .get("criteria")
        .and_then(|v| v.as_json())
        .map(|j| serde_json::from_value(j.clone()))
        .transpose()
        .map_err(|e| ExecError::new(format!("invalid 'criteria' JSON: {}", e)))?
        .ok_or_else(|| ExecError::new("missing 'criteria' input"))?;

    let context = inputs
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

    let mut out = HashMap::new();
    out.insert("question".to_string(), Value::Str(question));
    out.insert("system_prompt".to_string(), Value::Str(system_prompt));
    Ok(out)
}

fn execute_parse_review_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let answer = inputs
        .get("answer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'answer' input"))?;

    let criteria: Criteria = inputs
        .get("criteria")
        .and_then(|v| v.as_json())
        .map(|j| serde_json::from_value(j.clone()))
        .transpose()
        .map_err(|e| ExecError::new(format!("invalid 'criteria' JSON: {}", e)))?
        .ok_or_else(|| ExecError::new("missing 'criteria' input"))?;

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

    let mut out = HashMap::new();
    out.insert(
        "output".to_string(),
        Value::Json(serde_json::to_value(&output).unwrap()),
    );
    out.insert(
        "errors".to_string(),
        Value::Json(serde_json::to_value(&errors).unwrap()),
    );
    Ok(out)
}

fn execute_merge_outputs(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let outputs_json = inputs
        .get("outputs")
        .and_then(|v| v.as_json())
        .ok_or_else(|| ExecError::new("missing or invalid 'outputs' input"))?;

    let outputs: Vec<ReviewOutput> = serde_json::from_value(outputs_json.clone())
        .map_err(|e| ExecError::new(format!("invalid 'outputs' JSON: {}", e)))?;

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

    let mut out = HashMap::new();
    out.insert(
        "bundle".to_string(),
        Value::Json(serde_json::to_value(&bundle).unwrap()),
    );
    out.insert(
        "conflicts".to_string(),
        Value::Json(serde_json::to_value(&conflicts).unwrap()),
    );
    Ok(out)
}

fn execute_hash_finding(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let issue_key = inputs
        .get("issue_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'issue_key' input"))?;

    let check_id = inputs
        .get("check_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'check_id' input"))?;

    let finding_id = hash_finding_id(issue_key, check_id);

    let mut out = HashMap::new();
    out.insert("finding_id".to_string(), Value::Str(finding_id));
    Ok(out)
}

fn execute_format_diff_artifact(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let diff_files = inputs
        .get("diff_files")
        .and_then(|v| v.as_map_str_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'diff_files' input"))?;

    // Format each file's diff into a readable artifact
    let mut parts = Vec::new();
    for (file, diff_chunk) in &diff_files {
        parts.push(format!("--- {}\n{}", file, diff_chunk));
    }

    let artifact = if parts.is_empty() {
        "(no changes)".to_string()
    } else {
        parts.join("\n\n")
    };

    let mut out = HashMap::new();
    out.insert("artifact".to_string(), Value::Str(artifact));
    Ok(out)
}

// ============================================================================
// Helpers
// ============================================================================

/// Generate a stable finding ID from issue_key and check_id.
pub fn hash_finding_id(issue_key: &str, check_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(check_id.as_bytes());
    hasher.update(b":");
    hasher.update(issue_key.as_bytes());
    let result = hasher.finalize();
    // Use first 16 bytes as hex (32 chars)
    hex::encode(&result[..16])
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

// Hex encoding helper
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_prepare_review_prompt() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "artifact".to_string(),
            Value::Str("fn foo() { query(format!(\"SELECT * FROM users WHERE id = {}\", id)); }".to_string()),
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
        inputs.insert(
            "outputs".to_string(),
            Value::Json(serde_json::to_value(vec![output1, output2]).unwrap()),
        );

        let result = ReviewOps::MergeOutputs.execute(inputs).unwrap();

        let bundle: ReviewBundle =
            serde_json::from_value(result.get("bundle").unwrap().as_json().unwrap().clone())
                .unwrap();

        assert_eq!(bundle.outputs.len(), 2);
        assert_eq!(bundle.merged_findings.len(), 2);
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
        inputs.insert(
            "outputs".to_string(),
            Value::Json(serde_json::to_value(vec![output1, output2]).unwrap()),
        );

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
        assert_eq!(
            result.get("finding_id"),
            result2.get("finding_id")
        );
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
        assert!(matches!(loc, Location::FileLine { file, line } if file == "src/main.rs" && line == 42));

        // Span
        let loc = parse_location(Some(&serde_json::json!({
            "type": "span",
            "file": "src/lib.rs",
            "start": 10,
            "end": 20
        })))
        .unwrap();
        assert!(matches!(loc, Location::Span { file, start, end } if file == "src/lib.rs" && start == 10 && end == 20));

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
        let json = extract_json_from_response(
            r#"The analysis shows {"key": "value"} as the result."#,
        )
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
        inputs.insert(
            "diff_files".to_string(),
            Value::MapStrStr(diff_files),
        );

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
        inputs.insert(
            "diff_files".to_string(),
            Value::MapStrStr(BTreeMap::new()),
        );

        let result = ReviewOps::FormatDiffArtifact.execute(inputs).unwrap();
        let artifact = result.get("artifact").unwrap().as_str().unwrap();
        assert_eq!(artifact, "(no changes)");
    }
}
