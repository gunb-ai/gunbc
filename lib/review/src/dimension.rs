//! Four-dimension abstract review model (W4).
//!
//! The review pipeline is **domain-agnostic**. Four abstract dimensions,
//! each with its own criteria input port and depth port (Fermi size).
//! Criteria are provided by the caller — omitting a dimension's criteria
//! opts it out.
//!
//! # Dimensions
//!
//! 1. **Coherence**: Internal consistency — bugs, contradictions, state mismatches
//! 2. **Quality**: Against injected standards (clippy policy, style guides)
//! 3. **Requirements**: Does it accomplish the stated goal?
//! 4. **Aspirational**: What could be better? Must-fix / defer / accept
//!
//! # DAG Shape
//!
//! ```text
//!                        ┌─→ coherence(criteria, depth) ──────┐
//! artifact ──────────────┤                                     │
//!                        ├─→ quality(criteria, depth) ──────────┼─→ merge ─→ aspirational ─→ output
//!                        └─→ requirements(criteria, depth) ────┘
//! ```
//!
//! Coherence, quality, and requirements run in **parallel**.
//! Aspirational runs **last** — sees all prior findings.

use gunbc_exec::{
    optional_str_strict, require_json, require_str, ExecError, Executable, IntoExecResult,
    OutputMap,
};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Criteria, Finding, ReviewOutput};

// ============================================================================
// Fermi Depth
// ============================================================================

/// Cost/quality tradeoff signal for review dimensions.
///
/// Maps to concrete LLM parameters: prompt detail, number of passes,
/// context window usage, and follow-up question generation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FermiDepth {
    /// Single-pass sanity check, minimal context.
    Xs,
    /// Focused review, key issues only.
    S,
    /// Standard review, good coverage (default).
    #[default]
    M,
    /// Thorough review, edge cases explored.
    L,
    /// Exhaustive deep-dive, multi-pass.
    Xl,
}

impl FermiDepth {
    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "XS" => Some(Self::Xs),
            "S" => Some(Self::S),
            "M" => Some(Self::M),
            "L" => Some(Self::L),
            "XL" => Some(Self::Xl),
            _ => None,
        }
    }

    /// Prompt instruction suffix for this depth level.
    pub fn prompt_instruction(&self) -> &'static str {
        match self {
            Self::Xs => "Focus only on critical issues that would cause immediate failures. Skip minor concerns.",
            Self::S => "Focus on key issues. Skip minor style and documentation concerns.",
            Self::M => "Provide a balanced review covering correctness, security, and quality.",
            Self::L => "Be thorough. Explore edge cases, concurrency issues, and subtle bugs.",
            Self::Xl => "Perform an exhaustive deep-dive. Consider all possible failure modes, race conditions, security implications, and performance impacts. Use multiple review passes.",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "XS",
            Self::S => "S",
            Self::M => "M",
            Self::L => "L",
            Self::Xl => "XL",
        }
    }
}

impl std::fmt::Display for FermiDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// Review Dimensions
// ============================================================================

/// The four abstract review dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDimension {
    /// Internal consistency — bugs, contradictions, state mismatches, logic errors.
    Coherence,
    /// Against injected standards (clippy policy, style guides, AGENT.md).
    Quality,
    /// Does it accomplish the stated goal? Missing edge cases?
    Requirements,
    /// What could be better? Must-fix / defer / accept classification.
    Aspirational,
}

impl ReviewDimension {
    /// All dimensions in execution order.
    pub fn all() -> &'static [Self] {
        &[
            Self::Coherence,
            Self::Quality,
            Self::Requirements,
            Self::Aspirational,
        ]
    }

    /// Dimensions that run in parallel (first wave).
    pub fn parallel_dimensions() -> &'static [Self] {
        &[Self::Coherence, Self::Quality, Self::Requirements]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Coherence => "coherence",
            Self::Quality => "quality",
            Self::Requirements => "requirements",
            Self::Aspirational => "aspirational",
        }
    }

    /// System prompt preamble for this dimension.
    pub fn system_prompt_preamble(&self) -> &'static str {
        match self {
            Self::Coherence => "You are reviewing for internal consistency. Look for bugs, contradictions, state mismatches, logic errors, and anything that doesn't make sense on its own.",
            Self::Quality => "You are reviewing against coding standards and quality guidelines. Check for style violations, anti-patterns, and deviations from established conventions.",
            Self::Requirements => "You are reviewing whether the code accomplishes its stated goal. Check for missing edge cases, incomplete implementations, and requirement gaps.",
            Self::Aspirational => "You are performing a final synthesis review. Classify each finding from prior reviews as must-fix (blocks merge), defer (track for later), or accept (no action needed). Also identify any additional improvements.",
        }
    }
}

impl std::fmt::Display for ReviewDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// Severity Classification
// ============================================================================

/// Finding severity classification for aspirational dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Blocks merge — must be fixed before submission.
    MustFix,
    /// Track for later — not blocking but should be addressed.
    Defer,
    /// No action needed — acceptable as-is.
    Accept,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MustFix => "must-fix",
            Self::Defer => "defer",
            Self::Accept => "accept",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// Dimension Finding
// ============================================================================

/// A finding with dimension label and severity classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionFinding {
    /// The underlying finding.
    #[serde(flatten)]
    pub finding: Finding,
    /// Which dimension produced this finding.
    pub dimension: ReviewDimension,
    /// Severity classification (set by aspirational dimension).
    #[serde(default)]
    pub severity: Option<Severity>,
}

// ============================================================================
// Dimension Review Output
// ============================================================================

/// Output from the 4-dimension review pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionReviewOutput {
    /// Schema version.
    pub schema_version: String,
    /// All findings with dimension labels and severity.
    pub findings: Vec<DimensionFinding>,
    /// Per-dimension summaries.
    pub dimension_summaries: HashMap<String, String>,
    /// Overall summary from aspirational dimension.
    pub summary: String,
    /// Counts by severity.
    pub severity_counts: SeverityCounts,
    /// Which dimensions were active (had criteria).
    pub active_dimensions: Vec<ReviewDimension>,
}

/// Counts of findings by severity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub must_fix: usize,
    pub defer: usize,
    pub accept: usize,
    pub unclassified: usize,
}

impl DimensionReviewOutput {
    pub const SCHEMA_VERSION: &'static str = "0.2.0";

    /// Create from dimension outputs after aspirational classification.
    pub fn from_dimension_outputs(
        dimension_outputs: &[(ReviewDimension, ReviewOutput)],
        aspirational_output: Option<&ReviewOutput>,
    ) -> Self {
        let mut findings = Vec::new();
        let mut dimension_summaries = HashMap::new();
        let mut active_dimensions = Vec::new();

        // Collect findings from parallel dimensions
        for (dim, output) in dimension_outputs {
            active_dimensions.push(*dim);
            dimension_summaries.insert(dim.as_str().to_string(), output.summary.clone());

            for f in &output.findings {
                findings.push(DimensionFinding {
                    finding: f.clone(),
                    dimension: *dim,
                    severity: None,
                });
            }
        }

        // Apply aspirational classifications
        let mut summary = String::new();
        if let Some(asp_output) = aspirational_output {
            active_dimensions.push(ReviewDimension::Aspirational);
            summary = asp_output.summary.clone();
            dimension_summaries.insert("aspirational".to_string(), asp_output.summary.clone());

            // Aspirational findings contain severity classifications
            for asp_finding in &asp_output.findings {
                // Try to match by issue_key to existing findings
                if let Some(existing) = findings
                    .iter_mut()
                    .find(|f| f.finding.issue_key == asp_finding.issue_key)
                {
                    existing.severity = parse_severity_from_observation(&asp_finding.observation);
                } else {
                    // New finding from aspirational dimension
                    findings.push(DimensionFinding {
                        finding: asp_finding.clone(),
                        dimension: ReviewDimension::Aspirational,
                        severity: parse_severity_from_observation(&asp_finding.observation),
                    });
                }
            }
        }

        let severity_counts = SeverityCounts {
            must_fix: findings
                .iter()
                .filter(|f| f.severity == Some(Severity::MustFix))
                .count(),
            defer: findings
                .iter()
                .filter(|f| f.severity == Some(Severity::Defer))
                .count(),
            accept: findings
                .iter()
                .filter(|f| f.severity == Some(Severity::Accept))
                .count(),
            unclassified: findings.iter().filter(|f| f.severity.is_none()).count(),
        };

        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            findings,
            dimension_summaries,
            summary,
            severity_counts,
            active_dimensions,
        }
    }
}

/// Try to extract severity from an aspirational finding's observation text.
fn parse_severity_from_observation(observation: &str) -> Option<Severity> {
    let lower = observation.to_lowercase();
    if lower.contains("must-fix") || lower.contains("must fix") || lower.contains("blocking") {
        Some(Severity::MustFix)
    } else if lower.contains("defer") || lower.contains("track for later") {
        Some(Severity::Defer)
    } else if lower.contains("accept") || lower.contains("no action") {
        Some(Severity::Accept)
    } else {
        None
    }
}

// ============================================================================
// Dimension Ops (DAG operations for 4-dimension model)
// ============================================================================

/// Operations for the 4-dimension review model.
///
/// These extend the base `ReviewOps` with dimension-aware behavior.
#[derive(Debug, Clone)]
pub enum DimensionOps {
    /// Prepare a dimension-specific review prompt.
    ///
    /// Inputs:
    /// - `artifact`: String - content to review
    /// - `criteria`: Json - dimension-specific criteria
    /// - `dimension`: String - dimension name
    /// - `depth`: String - FermiDepth (XS/S/M/L/XL)
    /// - `context` (optional): String - additional context
    /// - `prior_findings` (optional): String - findings from other dimensions (aspirational only)
    ///
    /// Outputs:
    /// - `question`: String - formatted question
    /// - `system_prompt`: String - dimension-specific system prompt
    PrepareDimensionPrompt,

    /// Parse a dimension review response with dimension label.
    ///
    /// Inputs:
    /// - `answer`: String - LLM response
    /// - `criteria`: Json - for check_id resolution
    /// - `dimension`: String - dimension name
    ///
    /// Outputs:
    /// - `output`: Json - ReviewOutput with dimension label
    /// - `errors`: Json - parse errors
    ParseDimensionResponse,

    /// Merge findings from all dimensions into DimensionReviewOutput.
    ///
    /// Inputs:
    /// - `coherence_output` (optional): Json - coherence findings
    /// - `quality_output` (optional): Json - quality findings
    /// - `requirements_output` (optional): Json - requirements findings
    /// - `aspirational_output` (optional): Json - aspirational findings with severity
    ///
    /// Outputs:
    /// - `output`: Json - DimensionReviewOutput
    /// - `summary`: String - overall summary
    MergeDimensionOutputs,

    /// Format prior findings as context for the aspirational dimension.
    ///
    /// Inputs:
    /// - `coherence_output` (optional): Json - coherence findings
    /// - `quality_output` (optional): Json - quality findings
    /// - `requirements_output` (optional): Json - requirements findings
    ///
    /// Outputs:
    /// - `prior_findings`: String - formatted prior findings summary
    FormatPriorFindings,
}

impl Executable for DimensionOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DimensionOps::PrepareDimensionPrompt => execute_prepare_dimension_prompt(inputs),
            DimensionOps::ParseDimensionResponse => execute_parse_dimension_response(inputs),
            DimensionOps::MergeDimensionOutputs => execute_merge_dimension_outputs(inputs),
            DimensionOps::FormatPriorFindings => execute_format_prior_findings(inputs),
        }
    }
}

fn execute_prepare_dimension_prompt(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let _artifact = require_str(&inputs, "artifact")?;
    let criteria: Criteria = serde_json::from_value(require_json(&inputs, "criteria")?.clone())
        .exec_context("invalid 'criteria' JSON")?;
    let dimension_str = require_str(&inputs, "dimension")?;
    let depth_str = require_str(&inputs, "depth")?;
    let context = optional_str_strict(&inputs, "context")?.unwrap_or("");
    let prior_findings = optional_str_strict(&inputs, "prior_findings")?.unwrap_or("");

    let dimension = match dimension_str {
        "coherence" => ReviewDimension::Coherence,
        "quality" => ReviewDimension::Quality,
        "requirements" => ReviewDimension::Requirements,
        "aspirational" => ReviewDimension::Aspirational,
        _ => {
            return Err(ExecError::new(format!(
                "unknown dimension: {}",
                dimension_str
            )))
        }
    };

    let depth = FermiDepth::parse(depth_str).unwrap_or_default();

    // Build question
    let mut parts = vec![format!(
        "Review the following code for: {}",
        criteria.description
    )];

    parts.push(format!("\nReview depth: {}", depth));
    parts.push(depth.prompt_instruction().to_string());

    if !context.is_empty() {
        parts.push(format!("\nContext: {}", context));
    }

    if dimension == ReviewDimension::Aspirational && !prior_findings.is_empty() {
        parts.push(format!(
            "\n--- Prior findings from other dimensions ---\n{}",
            prior_findings
        ));
        parts.push(
            "\nFor each finding above, classify as must-fix, defer, or accept. \
             Also identify any additional improvements."
                .to_string(),
        );
    }

    parts.push(String::new());
    parts.push("For each issue found, respond in JSON format:".to_string());
    parts.push(
        r#"{"findings": [{"check_id": "...", "issue_key": "...", "location": {...}, "observation": "...", "candidate_fix": "..."}], "summary": "..."}"#
            .to_string(),
    );
    parts.push(String::new());
    parts.push("Checks to perform:".to_string());

    for check in &criteria.checks {
        parts.push(format!("- [{}] {}", check.id, check.question));
        for example in &check.examples {
            parts.push(format!("  Example: {}", example));
        }
    }

    let question = parts.join("\n");

    let system_prompt = format!(
        "{}\n\n\
         Analyze code thoroughly and report issues in valid JSON format. \
         Use stable issue_key values based on the nature of the issue (not line numbers). \
         Only report genuine issues - don't invent problems.",
        dimension.system_prompt_preamble()
    );

    OutputMap::new()
        .str("question", question)
        .str("system_prompt", system_prompt)
        .ok()
}

fn execute_parse_dimension_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let answer = require_str(&inputs, "answer")?;
    let criteria: Criteria = serde_json::from_value(require_json(&inputs, "criteria")?.clone())
        .exec_context("invalid 'criteria' JSON")?;
    let dimension_str = require_str(&inputs, "dimension")?;

    // Reuse the base parse logic
    let mut base_inputs = HashMap::new();
    base_inputs.insert("answer".to_string(), Value::Str(answer.to_string()));
    base_inputs.insert(
        "criteria".to_string(),
        Value::Json(serde_json::to_value(&criteria).unwrap()),
    );

    let base_result = crate::execute_parse_review_response(base_inputs)?;

    // Tag output with dimension
    if let Some(Value::Json(mut output_json)) = base_result.get("output").cloned() {
        if let Some(obj) = output_json.as_object_mut() {
            obj.insert(
                "dimension".into(),
                serde_json::Value::String(dimension_str.to_string()),
            );
        }
        OutputMap::new()
            .json("output", output_json)
            .json(
                "errors",
                base_result
                    .get("errors")
                    .and_then(|v| v.as_json())
                    .cloned()
                    .unwrap_or(serde_json::json!([])),
            )
            .ok()
    } else {
        Ok(base_result)
    }
}

fn execute_merge_dimension_outputs(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut dimension_outputs = Vec::new();
    let mut aspirational_output = None;

    for (key, dim) in [
        ("coherence_output", ReviewDimension::Coherence),
        ("quality_output", ReviewDimension::Quality),
        ("requirements_output", ReviewDimension::Requirements),
    ] {
        if let Some(val) = inputs.get(key) {
            if let Some(json) = val.as_json() {
                if let Ok(output) = serde_json::from_value::<ReviewOutput>(json.clone()) {
                    dimension_outputs.push((dim, output));
                }
            }
        }
    }

    if let Some(val) = inputs.get("aspirational_output") {
        if let Some(json) = val.as_json() {
            if let Ok(output) = serde_json::from_value::<ReviewOutput>(json.clone()) {
                aspirational_output = Some(output);
            }
        }
    }

    let result =
        DimensionReviewOutput::from_dimension_outputs(&dimension_outputs, aspirational_output.as_ref());

    let summary = result.summary.clone();

    OutputMap::new()
        .json("output", serde_json::to_value(&result).unwrap())
        .str("summary", summary)
        .ok()
}

fn execute_format_prior_findings(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut parts = Vec::new();

    for (key, label) in [
        ("coherence_output", "Coherence"),
        ("quality_output", "Quality"),
        ("requirements_output", "Requirements"),
    ] {
        if let Some(val) = inputs.get(key) {
            if let Some(json) = val.as_json() {
                if let Ok(output) = serde_json::from_value::<ReviewOutput>(json.clone()) {
                    if !output.findings.is_empty() {
                        parts.push(format!("## {} Dimension ({} findings)", label, output.findings.len()));
                        for f in &output.findings {
                            parts.push(format!(
                                "- [{}] {}: {}",
                                f.check_id, f.issue_key, f.observation
                            ));
                        }
                        if !output.summary.is_empty() {
                            parts.push(format!("Summary: {}", output.summary));
                        }
                        parts.push(String::new());
                    }
                }
            }
        }
    }

    let prior = if parts.is_empty() {
        "No findings from prior dimensions.".to_string()
    } else {
        parts.join("\n")
    };

    OutputMap::new().str("prior_findings", prior).ok()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Check, Criteria, Finding, Location, ReviewOutput};

    fn test_criteria() -> Criteria {
        Criteria {
            name: "coherence".to_string(),
            description: "Internal consistency review".to_string(),
            checks: vec![Check {
                id: "logic-error".to_string(),
                question: "Are there any logic errors?".to_string(),
                examples: vec![],
            }],
        }
    }

    #[test]
    fn test_fermi_depth_parse() {
        assert_eq!(FermiDepth::parse("XS"), Some(FermiDepth::Xs));
        assert_eq!(FermiDepth::parse("xs"), Some(FermiDepth::Xs));
        assert_eq!(FermiDepth::parse("M"), Some(FermiDepth::M));
        assert_eq!(FermiDepth::parse("XL"), Some(FermiDepth::Xl));
        assert_eq!(FermiDepth::parse("invalid"), None);
    }

    #[test]
    fn test_fermi_depth_default() {
        assert_eq!(FermiDepth::default(), FermiDepth::M);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::MustFix.as_str(), "must-fix");
        assert_eq!(Severity::Defer.as_str(), "defer");
        assert_eq!(Severity::Accept.as_str(), "accept");
    }

    #[test]
    fn test_parse_severity_from_observation() {
        assert_eq!(
            parse_severity_from_observation("This is a must-fix issue"),
            Some(Severity::MustFix)
        );
        assert_eq!(
            parse_severity_from_observation("Defer this for later"),
            Some(Severity::Defer)
        );
        assert_eq!(
            parse_severity_from_observation("Accept as-is"),
            Some(Severity::Accept)
        );
        assert_eq!(
            parse_severity_from_observation("Some random observation"),
            None
        );
    }

    #[test]
    fn test_dimension_review_output_from_outputs() {
        let coherence = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "coherence".to_string(),
            source: "llm".to_string(),
            findings: vec![Finding {
                id: "abc".to_string(),
                check_id: "logic-error".to_string(),
                issue_key: "null_deref".to_string(),
                location: Location::Unlocated,
                observation: "Possible null dereference".to_string(),
                candidate_fix: None,
            }],
            candidate_remediations: None,
            summary: "Found 1 coherence issue".to_string(),
        };

        let aspirational = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "aspirational".to_string(),
            source: "llm".to_string(),
            findings: vec![Finding {
                id: "asp1".to_string(),
                check_id: "logic-error".to_string(),
                issue_key: "null_deref".to_string(),
                location: Location::Unlocated,
                observation: "Must-fix: this null dereference will cause crashes".to_string(),
                candidate_fix: Some("Add null check".to_string()),
            }],
            candidate_remediations: None,
            summary: "1 must-fix, 0 defer, 0 accept".to_string(),
        };

        let result = DimensionReviewOutput::from_dimension_outputs(
            &[(ReviewDimension::Coherence, coherence)],
            Some(&aspirational),
        );

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.severity_counts.must_fix, 1);
        assert_eq!(result.active_dimensions.len(), 2);
        assert!(result.summary.contains("must-fix"));
    }

    #[test]
    fn test_prepare_dimension_prompt() {
        let mut inputs = HashMap::new();
        inputs.insert("artifact".to_string(), Value::Str("fn foo() {}".into()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );
        inputs.insert("dimension".to_string(), Value::Str("coherence".into()));
        inputs.insert("depth".to_string(), Value::Str("M".into()));

        let result = DimensionOps::PrepareDimensionPrompt
            .execute(inputs)
            .unwrap();

        let question = result.get("question").unwrap().as_str().unwrap();
        assert!(question.contains("Internal consistency"));
        assert!(question.contains("logic-error"));

        let system = result.get("system_prompt").unwrap().as_str().unwrap();
        assert!(system.contains("internal consistency"));
    }

    #[test]
    fn test_aspirational_prompt_includes_prior_findings() {
        let mut inputs = HashMap::new();
        inputs.insert("artifact".to_string(), Value::Str("fn foo() {}".into()));
        inputs.insert(
            "criteria".to_string(),
            Value::Json(serde_json::to_value(test_criteria()).unwrap()),
        );
        inputs.insert("dimension".to_string(), Value::Str("aspirational".into()));
        inputs.insert("depth".to_string(), Value::Str("M".into()));
        inputs.insert(
            "prior_findings".to_string(),
            Value::Str("## Coherence: Found 1 bug".into()),
        );

        let result = DimensionOps::PrepareDimensionPrompt
            .execute(inputs)
            .unwrap();

        let question = result.get("question").unwrap().as_str().unwrap();
        assert!(question.contains("Prior findings"));
        assert!(question.contains("must-fix, defer, or accept"));
    }

    #[test]
    fn test_merge_dimension_outputs_empty() {
        let inputs = HashMap::new();
        let result = DimensionOps::MergeDimensionOutputs
            .execute(inputs)
            .unwrap();

        let output: DimensionReviewOutput =
            serde_json::from_value(result.get("output").unwrap().as_json().unwrap().clone())
                .unwrap();
        assert!(output.findings.is_empty());
        assert_eq!(output.severity_counts.must_fix, 0);
    }

    #[test]
    fn test_format_prior_findings_empty() {
        let inputs = HashMap::new();
        let result = DimensionOps::FormatPriorFindings.execute(inputs).unwrap();

        let prior = result.get("prior_findings").unwrap().as_str().unwrap();
        assert!(prior.contains("No findings"));
    }

    #[test]
    fn test_format_prior_findings_with_data() {
        let coherence = ReviewOutput {
            schema_version: "0.1.0".to_string(),
            criteria_name: "coherence".to_string(),
            source: "llm".to_string(),
            findings: vec![Finding {
                id: "abc".to_string(),
                check_id: "logic-error".to_string(),
                issue_key: "null_deref".to_string(),
                location: Location::Unlocated,
                observation: "Null deref".to_string(),
                candidate_fix: None,
            }],
            candidate_remediations: None,
            summary: "Found 1 issue".to_string(),
        };

        let mut inputs = HashMap::new();
        inputs.insert(
            "coherence_output".to_string(),
            Value::Json(serde_json::to_value(&coherence).unwrap()),
        );

        let result = DimensionOps::FormatPriorFindings.execute(inputs).unwrap();
        let prior = result.get("prior_findings").unwrap().as_str().unwrap();
        assert!(prior.contains("Coherence Dimension"));
        assert!(prior.contains("null_deref"));
    }
}
