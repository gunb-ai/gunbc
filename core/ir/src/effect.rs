//! Effect classification for DAG nodes.
//!
//! The Effect model uses a 2-bit classification aligned with `the-gunbai`'s
//! `gunbai-types::Effect`:
//!
//! | writes_world | deterministic | Category | Example |
//! |:---:|:---:|---|---|
//! | false | true | **Pure** | string formatting, JSON parsing |
//! | false | false | **Read** | LLM calls, external reads |
//! | true | true | **WriteDeterministic** | idempotent file writes |
//! | true | false | **Write** | POST requests, side-effecting mutations |
//!
//! This classification is orthogonal to the `ObligationCategory` used by
//! `daglang-lower`. ObligationCategory describes *what* obligation a node
//! carries for testing; Effect describes *whether* the node mutates or
//! caches.

use serde::{Deserialize, Serialize};

/// 2-bit effect classification for a DAG node.
///
/// Mirrors `gunbai-types::Effect` from the-gunbai for cross-repo compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    /// Mutates external state (GitHub, cloud, filesystem, etc.)
    pub writes_world: bool,
    /// Safe to cache/memoize (pure computation).
    pub deterministic: bool,
}

impl Default for Effect {
    /// Defaults to `PURE` (deterministic, no world writes).
    fn default() -> Self {
        Self::PURE
    }
}

impl Effect {
    /// Pure computation — safe to cache, no external effects.
    pub const PURE: Effect = Effect {
        writes_world: false,
        deterministic: true,
    };

    /// Non-deterministic read — LLM calls, polls, external reads.
    pub const READ: Effect = Effect {
        writes_world: false,
        deterministic: false,
    };

    /// Deterministic write — idempotent mutations (rare).
    pub const WRITE_DETERMINISTIC: Effect = Effect {
        writes_world: true,
        deterministic: true,
    };

    /// Non-deterministic write — typical external mutations.
    pub const WRITE: Effect = Effect {
        writes_world: true,
        deterministic: false,
    };

    /// Create a new Effect with explicit classification.
    pub fn new(writes_world: bool, deterministic: bool) -> Self {
        Self {
            writes_world,
            deterministic,
        }
    }

    /// Whether this effect is safe to cache/memoize.
    pub fn cacheable(&self) -> bool {
        self.deterministic && !self.writes_world
    }

    /// Whether this effect requires policy approval before execution.
    pub fn requires_policy(&self) -> bool {
        self.writes_world
    }
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.writes_world, self.deterministic) {
            (false, true) => write!(f, "Pure"),
            (false, false) => write!(f, "Read"),
            (true, true) => write!(f, "WriteDeterministic"),
            (true, false) => write!(f, "Write"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_is_cacheable() {
        assert!(Effect::PURE.cacheable());
        assert!(!Effect::PURE.requires_policy());
    }

    #[test]
    fn read_is_not_cacheable() {
        assert!(!Effect::READ.cacheable());
        assert!(!Effect::READ.requires_policy());
    }

    #[test]
    fn write_requires_policy() {
        assert!(Effect::WRITE.requires_policy());
        assert!(!Effect::WRITE.cacheable());
    }

    #[test]
    fn write_deterministic_requires_policy_but_not_cacheable() {
        assert!(Effect::WRITE_DETERMINISTIC.requires_policy());
        assert!(!Effect::WRITE_DETERMINISTIC.cacheable());
    }

    #[test]
    fn default_is_pure() {
        assert_eq!(Effect::default(), Effect::PURE);
    }

    #[test]
    fn display() {
        assert_eq!(Effect::PURE.to_string(), "Pure");
        assert_eq!(Effect::READ.to_string(), "Read");
        assert_eq!(Effect::WRITE.to_string(), "Write");
        assert_eq!(Effect::WRITE_DETERMINISTIC.to_string(), "WriteDeterministic");
    }

    #[test]
    fn serde_round_trip() {
        let effect = Effect::WRITE;
        let json = serde_json::to_string(&effect).unwrap();
        let deserialized: Effect = serde_json::from_str(&json).unwrap();
        assert_eq!(effect, deserialized);
    }
}
