//! Resource state and execution mode types.
//!
//! This module defines the states a managed resource can be in and the
//! execution modes that determine how to handle stale resources.

use super::hash::ContentHash;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The state of a managed resource relative to its manifest entry.
///
/// This is determined by comparing the computed input hash to the stored hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource doesn't exist in the manifest (never been created).
    Missing,

    /// Resource exists but the input hash doesn't match (inputs changed).
    Stale {
        /// Human-readable reason for staleness.
        reason: String,
        /// The hash stored in the manifest.
        stored_key: ContentHash,
        /// The currently computed hash from inputs.
        current_key: ContentHash,
    },

    /// Resource exists and the input hash matches (up to date).
    Fresh,

    /// Error occurred while computing state (e.g., can't read input file).
    Error(String),
}

impl ResourceState {
    /// Create a stale state with a reason.
    pub fn stale(
        reason: impl Into<String>,
        stored_key: ContentHash,
        current_key: ContentHash,
    ) -> Self {
        Self::Stale {
            reason: reason.into(),
            stored_key,
            current_key,
        }
    }

    /// Create an error state.
    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }

    /// Returns true if the resource is fresh (up to date).
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh)
    }

    /// Returns true if the resource needs to be (re)created.
    pub fn needs_creation(&self) -> bool {
        matches!(self, Self::Missing | Self::Stale { .. })
    }

    /// Returns true if there was an error computing state.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl fmt::Display for ResourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "missing"),
            Self::Stale { reason, .. } => write!(f, "stale: {}", reason),
            Self::Fresh => write!(f, "fresh"),
            Self::Error(e) => write!(f, "error: {}", e),
        }
    }
}

/// Execution mode for resource acquisition.
///
/// This is executor context (like DryRun), not DAG structure.
/// The same DAG runs in both CI (Verify) and dev (Ensure) modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ExecMode {
    /// Check that resources are fresh, fail if stale (CI mode).
    ///
    /// In Verify mode:
    /// - Fresh resources: proceed
    /// - Missing/Stale resources: fail with error
    ///
    /// This is the default mode for CI pipelines.
    #[default]
    Verify,

    /// Ensure resources are fresh, regenerate if stale (dev mode).
    ///
    /// In Ensure mode:
    /// - Fresh resources: proceed
    /// - Missing/Stale resources: run the provider to create/update
    ///
    /// This is used during development to auto-fix staleness.
    Ensure,
}

impl ExecMode {
    /// Returns true if this mode allows creating/updating resources.
    pub fn allows_creation(&self) -> bool {
        matches!(self, Self::Ensure)
    }

    /// Returns true if this mode fails on stale resources.
    pub fn fails_on_stale(&self) -> bool {
        matches!(self, Self::Verify)
    }

    /// Parse mode from a string (for CLI flag parsing).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "verify" | "check" => Some(Self::Verify),
            "ensure" | "fix" => Some(Self::Ensure),
            _ => None,
        }
    }
}

impl fmt::Display for ExecMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verify => write!(f, "verify"),
            Self::Ensure => write!(f, "ensure"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_state_fresh() {
        let state = ResourceState::Fresh;
        assert!(state.is_fresh());
        assert!(!state.needs_creation());
        assert!(!state.is_error());
    }

    #[test]
    fn test_resource_state_missing() {
        let state = ResourceState::Missing;
        assert!(!state.is_fresh());
        assert!(state.needs_creation());
        assert!(!state.is_error());
    }

    #[test]
    fn test_resource_state_stale() {
        let state = ResourceState::stale(
            "inputs changed",
            ContentHash::new("old"),
            ContentHash::new("new"),
        );
        assert!(!state.is_fresh());
        assert!(state.needs_creation());
        assert!(!state.is_error());
    }

    #[test]
    fn test_resource_state_error() {
        let state = ResourceState::error("file not found");
        assert!(!state.is_fresh());
        assert!(!state.needs_creation());
        assert!(state.is_error());
    }

    #[test]
    fn test_exec_mode_default() {
        assert_eq!(ExecMode::default(), ExecMode::Verify);
    }

    #[test]
    fn test_exec_mode_allows_creation() {
        assert!(!ExecMode::Verify.allows_creation());
        assert!(ExecMode::Ensure.allows_creation());
    }

    #[test]
    fn test_exec_mode_fails_on_stale() {
        assert!(ExecMode::Verify.fails_on_stale());
        assert!(!ExecMode::Ensure.fails_on_stale());
    }

    #[test]
    fn test_exec_mode_parse() {
        assert_eq!(ExecMode::parse("verify"), Some(ExecMode::Verify));
        assert_eq!(ExecMode::parse("check"), Some(ExecMode::Verify));
        assert_eq!(ExecMode::parse("ensure"), Some(ExecMode::Ensure));
        assert_eq!(ExecMode::parse("fix"), Some(ExecMode::Ensure));
        assert_eq!(ExecMode::parse("VERIFY"), Some(ExecMode::Verify));
        assert_eq!(ExecMode::parse("unknown"), None);
    }

    #[test]
    fn test_resource_state_display() {
        assert_eq!(format!("{}", ResourceState::Fresh), "fresh");
        assert_eq!(format!("{}", ResourceState::Missing), "missing");
        assert!(format!(
            "{}",
            ResourceState::stale("test", ContentHash::empty(), ContentHash::empty())
        )
        .starts_with("stale:"));
    }
}
