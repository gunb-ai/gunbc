//! Lint identifiers for pragma policy.

/// Source of a lint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LintSource {
    Rustc,
    Clippy,
}

/// Lint identifier with source and name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LintId {
    pub source: LintSource,
    pub name: &'static str,
}

impl LintId {
    /// Rustc lint (e.g., "unused").
    pub const fn rustc(name: &'static str) -> Self {
        Self {
            source: LintSource::Rustc,
            name,
        }
    }

    /// Clippy lint (e.g., "result_large_err").
    pub const fn clippy(name: &'static str) -> Self {
        Self {
            source: LintSource::Clippy,
            name,
        }
    }

    /// Render as the string used in #[allow(...)], e.g. "unused" or "clippy::result_large_err".
    pub fn allow_name(&self) -> String {
        match self.source {
            LintSource::Rustc => self.name.to_string(),
            LintSource::Clippy => format!("clippy::{}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_id_allow_name() {
        let rustc = LintId::rustc("unused");
        let clippy = LintId::clippy("result_large_err");

        assert_eq!(rustc.allow_name(), "unused");
        assert_eq!(clippy.allow_name(), "clippy::result_large_err");
    }
}
