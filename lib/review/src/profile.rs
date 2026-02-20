//! Review profiles — criteria bundles for domain-specific reviews (W5).
//!
//! A `ReviewProfile` maps dimensions to domain-specific criteria + depth.
//! Profiles are the composition unit for domain specialization — a coding
//! review profile, a security audit profile, a design review profile each
//! provide different criteria values but wire into the same dimension SubDags.
//!
//! # Coding Profile
//!
//! The coding profile loads criteria from:
//! - **Coherence**: invariant docs, type safety rules
//! - **Quality**: `AGENT.md` + `clippy.toml` content
//! - **Requirements**: GitHub issue body or PR description
//! - **Aspirational**: refactoring heuristics, perf patterns

use crate::dimension::{FermiDepth, ReviewDimension};
use crate::{Check, Criteria};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ReviewProfile
// ============================================================================

/// A review profile maps dimensions to domain-specific criteria + depth.
///
/// Dimensions without criteria are opted out (skipped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewProfile {
    /// Profile name (e.g., "coding", "security", "design").
    pub name: String,
    /// Default depth for all dimensions.
    pub default_depth: FermiDepth,
    /// Per-dimension criteria. Missing = dimension is skipped.
    pub dimension_criteria: HashMap<String, Criteria>,
    /// Per-dimension depth overrides. Missing = use default_depth.
    #[serde(default)]
    pub dimension_depths: HashMap<String, FermiDepth>,
}

impl ReviewProfile {
    /// Get criteria for a dimension, or None to skip it.
    pub fn criteria_for(&self, dimension: ReviewDimension) -> Option<&Criteria> {
        self.dimension_criteria.get(dimension.as_str())
    }

    /// Get depth for a dimension.
    pub fn depth_for(&self, dimension: ReviewDimension) -> FermiDepth {
        self.dimension_depths
            .get(dimension.as_str())
            .copied()
            .unwrap_or(self.default_depth)
    }

    /// Whether a dimension is active (has criteria).
    pub fn is_active(&self, dimension: ReviewDimension) -> bool {
        self.dimension_criteria.contains_key(dimension.as_str())
    }

    /// Active dimensions (those with criteria).
    pub fn active_dimensions(&self) -> Vec<ReviewDimension> {
        ReviewDimension::all()
            .iter()
            .filter(|d| self.is_active(**d))
            .copied()
            .collect()
    }
}

// ============================================================================
// Coding Review Profile (W5)
// ============================================================================

/// Build the default coding review profile.
///
/// Loads criteria from:
/// - Coherence: type safety rules, invariant docs
/// - Quality: AGENT.md + clippy.toml (file-based)
/// - Requirements: GitHub issue body (injected at runtime)
/// - Aspirational: refactoring heuristics
pub fn coding_review_profile(depth: FermiDepth) -> ReviewProfile {
    let mut criteria = HashMap::new();

    criteria.insert(
        "coherence".to_string(),
        coherence_criteria(),
    );

    criteria.insert(
        "quality".to_string(),
        quality_criteria_default(),
    );

    criteria.insert(
        "requirements".to_string(),
        requirements_criteria_default(),
    );

    criteria.insert(
        "aspirational".to_string(),
        aspirational_criteria(),
    );

    ReviewProfile {
        name: "coding".to_string(),
        default_depth: depth,
        dimension_criteria: criteria,
        dimension_depths: HashMap::new(),
    }
}

/// Pre-read project context for quality criteria enrichment.
///
/// The caller is responsible for reading file contents through the proper
/// transport boundary (I6: no escape hatches). This struct holds the
/// pre-read contents.
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    /// Contents of AGENT.md (if present).
    pub agent_md: Option<String>,
    /// Contents of clippy.toml (if present).
    pub clippy_toml: Option<String>,
}

/// Build a coding review profile with explicit project context.
///
/// Quality criteria are enriched with AGENT.md and clippy.toml content
/// when available. File reading is the caller's responsibility (I6:
/// no escape hatches — all I/O through transport boundaries).
///
/// # Example
///
/// ```ignore
/// let ctx = ProjectContext {
///     agent_md: Some(agent_md_content),
///     clippy_toml: Some(clippy_toml_content),
/// };
/// let profile = coding_review_profile_with_context(FermiDepth::M, &ctx);
/// ```
pub fn coding_review_profile_with_context(
    depth: FermiDepth,
    context: &ProjectContext,
) -> ReviewProfile {
    let mut profile = coding_review_profile(depth);

    let mut quality_context = Vec::new();

    if let Some(ref content) = context.agent_md {
        quality_context.push(format!(
            "Project conventions (from AGENT.md):\n{}",
            truncate_content(content, 2000)
        ));
    }

    if let Some(ref content) = context.clippy_toml {
        quality_context.push(format!(
            "Clippy configuration (from clippy.toml):\n{}",
            truncate_content(content, 1000)
        ));
    }

    if !quality_context.is_empty() {
        if let Some(quality) = profile.dimension_criteria.get_mut("quality") {
            quality.description = format!(
                "{}\n\n{}",
                quality.description,
                quality_context.join("\n\n")
            );
        }
    }

    profile
}

/// Build a coding review profile with requirements context.
///
/// Extends [`coding_review_profile_with_context`] by injecting requirements
/// text (from a GitHub issue or PR description) into the requirements
/// dimension criteria.
pub fn coding_review_profile_with_requirements(
    depth: FermiDepth,
    context: &ProjectContext,
    requirements_text: &str,
) -> ReviewProfile {
    let mut profile = coding_review_profile_with_context(depth, context);

    if let Some(req) = profile.dimension_criteria.get_mut("requirements") {
        req.description = format!(
            "{}\n\nRequirements context:\n{}",
            req.description, requirements_text
        );
    }

    profile
}

// ============================================================================
// Dimension-Specific Criteria
// ============================================================================

/// Coherence criteria: internal consistency checks.
fn coherence_criteria() -> Criteria {
    Criteria {
        name: "coherence".to_string(),
        description: "Internal consistency — bugs, contradictions, state mismatches, logic errors"
            .to_string(),
        checks: vec![
            Check {
                id: "logic-error".to_string(),
                question: "Are there any logic errors or incorrect behavior?".to_string(),
                examples: vec![
                    "Off-by-one errors".to_string(),
                    "Wrong comparison operator".to_string(),
                    "Missing break/return".to_string(),
                ],
            },
            Check {
                id: "state-mismatch".to_string(),
                question: "Are there state mismatches or inconsistent invariants?".to_string(),
                examples: vec![
                    "Lock acquired but not released".to_string(),
                    "Resource opened but not closed".to_string(),
                    "State set in one branch but not another".to_string(),
                ],
            },
            Check {
                id: "null-safety".to_string(),
                question: "Are there null/None dereference or unwrap risks?".to_string(),
                examples: vec![
                    "unwrap() on user input".to_string(),
                    "Missing Option/Result handling".to_string(),
                ],
            },
            Check {
                id: "error-handling".to_string(),
                question: "Is error handling complete and correct?".to_string(),
                examples: vec![
                    "Swallowed errors".to_string(),
                    "Wrong error type returned".to_string(),
                    "Missing error propagation".to_string(),
                ],
            },
        ],
    }
}

/// Default quality criteria (without file-based enrichment).
fn quality_criteria_default() -> Criteria {
    Criteria {
        name: "quality".to_string(),
        description: "Code quality against project standards and best practices".to_string(),
        checks: vec![
            Check {
                id: "style".to_string(),
                question: "Does the code follow project style conventions?".to_string(),
                examples: vec![
                    "Naming conventions".to_string(),
                    "Code organization".to_string(),
                    "Consistent patterns".to_string(),
                ],
            },
            Check {
                id: "security".to_string(),
                question: "Are there any security vulnerabilities?".to_string(),
                examples: vec![
                    "SQL injection".to_string(),
                    "Command injection".to_string(),
                    "Hardcoded credentials".to_string(),
                    "Path traversal".to_string(),
                ],
            },
            Check {
                id: "performance".to_string(),
                question: "Are there performance issues or inefficiencies?".to_string(),
                examples: vec![
                    "Unnecessary allocations".to_string(),
                    "O(n^2) algorithms".to_string(),
                    "Missing caching opportunities".to_string(),
                ],
            },
            Check {
                id: "maintainability".to_string(),
                question: "Is the code maintainable and well-structured?".to_string(),
                examples: vec![
                    "Overly complex functions".to_string(),
                    "Magic numbers".to_string(),
                    "Missing abstraction boundaries".to_string(),
                ],
            },
        ],
    }
}

/// Default requirements criteria.
fn requirements_criteria_default() -> Criteria {
    Criteria {
        name: "requirements".to_string(),
        description: "Does the implementation accomplish its stated goal?".to_string(),
        checks: vec![
            Check {
                id: "completeness".to_string(),
                question: "Is the implementation complete for the stated requirements?"
                    .to_string(),
                examples: vec![
                    "Missing edge cases".to_string(),
                    "Partial implementation".to_string(),
                    "Unhandled scenarios".to_string(),
                ],
            },
            Check {
                id: "correctness".to_string(),
                question: "Does the implementation correctly solve the stated problem?"
                    .to_string(),
                examples: vec![
                    "Wrong algorithm for the problem".to_string(),
                    "Misunderstood requirements".to_string(),
                ],
            },
            Check {
                id: "integration".to_string(),
                question: "Does the change integrate properly with the existing codebase?"
                    .to_string(),
                examples: vec![
                    "Breaking existing APIs".to_string(),
                    "Missing migration steps".to_string(),
                    "Incompatible type changes".to_string(),
                ],
            },
        ],
    }
}

/// Aspirational criteria: what could be better.
fn aspirational_criteria() -> Criteria {
    Criteria {
        name: "aspirational".to_string(),
        description: "What could be improved? Classify findings as must-fix, defer, or accept"
            .to_string(),
        checks: vec![
            Check {
                id: "classification".to_string(),
                question: "For each finding from prior dimensions, classify as must-fix (blocks merge), defer (track for later), or accept (no action needed)."
                    .to_string(),
                examples: vec![
                    "must-fix: Security vulnerability that could be exploited".to_string(),
                    "defer: Performance optimization that isn't critical".to_string(),
                    "accept: Minor style preference, already functional".to_string(),
                ],
            },
            Check {
                id: "improvements".to_string(),
                question: "Are there additional improvements not caught by other dimensions?"
                    .to_string(),
                examples: vec![
                    "Refactoring opportunities".to_string(),
                    "Better abstractions".to_string(),
                    "Documentation improvements".to_string(),
                ],
            },
        ],
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Truncate content to a maximum character length, appending "..." if truncated.
fn truncate_content(content: &str, max_chars: usize) -> &str {
    if content.len() <= max_chars {
        content
    } else {
        // Find a safe UTF-8 boundary
        let mut end = max_chars;
        while end > 0 && !content.is_char_boundary(end) {
            end -= 1;
        }
        &content[..end]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coding_profile_has_all_dimensions() {
        let profile = coding_review_profile(FermiDepth::M);
        assert_eq!(profile.name, "coding");
        assert!(profile.is_active(ReviewDimension::Coherence));
        assert!(profile.is_active(ReviewDimension::Quality));
        assert!(profile.is_active(ReviewDimension::Requirements));
        assert!(profile.is_active(ReviewDimension::Aspirational));
        assert_eq!(profile.active_dimensions().len(), 4);
    }

    #[test]
    fn test_coding_profile_depth_override() {
        let mut profile = coding_review_profile(FermiDepth::M);
        profile
            .dimension_depths
            .insert("coherence".to_string(), FermiDepth::L);

        assert_eq!(
            profile.depth_for(ReviewDimension::Coherence),
            FermiDepth::L
        );
        assert_eq!(
            profile.depth_for(ReviewDimension::Quality),
            FermiDepth::M
        );
    }

    #[test]
    fn test_profile_without_dimension() {
        let mut profile = coding_review_profile(FermiDepth::M);
        profile.dimension_criteria.remove("aspirational");

        assert!(!profile.is_active(ReviewDimension::Aspirational));
        assert_eq!(profile.active_dimensions().len(), 3);
    }

    #[test]
    fn test_coherence_criteria() {
        let c = coherence_criteria();
        assert_eq!(c.name, "coherence");
        assert!(c.checks.iter().any(|ch| ch.id == "logic-error"));
        assert!(c.checks.iter().any(|ch| ch.id == "null-safety"));
    }

    #[test]
    fn test_truncate_content() {
        assert_eq!(truncate_content("hello", 10), "hello");
        assert_eq!(truncate_content("hello world", 5), "hello");
    }

    #[test]
    fn test_profile_with_empty_context() {
        let profile =
            coding_review_profile_with_context(FermiDepth::M, &ProjectContext::default());
        assert!(profile.is_active(ReviewDimension::Quality));
    }

    #[test]
    fn test_profile_with_project_context() {
        let ctx = ProjectContext {
            agent_md: Some("## Rules\n- No unwrap()".to_string()),
            clippy_toml: Some("too-many-lines-threshold = 100".to_string()),
        };
        let profile = coding_review_profile_with_context(FermiDepth::M, &ctx);
        let quality = profile.criteria_for(ReviewDimension::Quality).unwrap();
        assert!(quality.description.contains("AGENT.md"));
        assert!(quality.description.contains("clippy.toml"));
    }

    #[test]
    fn test_profile_with_requirements() {
        let profile = coding_review_profile_with_requirements(
            FermiDepth::M,
            &ProjectContext::default(),
            "Implement user authentication with OAuth2",
        );
        let req = profile.criteria_for(ReviewDimension::Requirements).unwrap();
        assert!(req.description.contains("OAuth2"));
    }
}
