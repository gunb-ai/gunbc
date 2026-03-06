//! Crate-level policy modeling for clippy enforcement.
//!
//! This models crate "roles" and exceptions in a way that can be reused by
//! repo-specific policy layers (e.g., gunbc-app).

use crate::config::CrateAllowance;

/// High-level role a crate plays in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateRole {
    /// Transport boundary: the designated I/O hub.
    TransportBoundary,
    /// Bootstrap/codegen crate (chicken/egg with transport).
    CodegenBootstrap,
    /// Infra hub (low-level helpers, test utilities).
    Infra,
    /// Repo-specific or misc role.
    Other,
}

/// Policy describing a crate's clippy exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CratePolicy {
    pub crate_name: &'static str,
    pub role: CrateRole,
    pub allow_disallowed_methods: bool,
    pub rationale: &'static str,
}

impl CratePolicy {
    /// Create a policy entry with explicit disallowed-methods allowance.
    pub const fn new(
        crate_name: &'static str,
        role: CrateRole,
        allow_disallowed_methods: bool,
        rationale: &'static str,
    ) -> Self {
        Self {
            crate_name,
            role,
            allow_disallowed_methods,
            rationale,
        }
    }

    /// Convenience: allow disallowed-methods for this crate.
    pub const fn allow_disallowed_methods(
        crate_name: &'static str,
        role: CrateRole,
        rationale: &'static str,
    ) -> Self {
        Self::new(crate_name, role, true, rationale)
    }

    /// Convert to a clippy crate allowance if disallowed methods are allowed.
    pub fn disallowed_methods_allowance(&self) -> Option<CrateAllowance> {
        if self.allow_disallowed_methods {
            Some(CrateAllowance::new(self.crate_name, self.rationale))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_to_allowance() {
        let policy = CratePolicy::allow_disallowed_methods(
            "gunbc-lib-transport",
            CrateRole::TransportBoundary,
            "I/O boundary",
        );

        let allowance = policy.disallowed_methods_allowance().unwrap();
        assert_eq!(allowance.crate_name, "gunbc-lib-transport");
        assert_eq!(allowance.reason, "I/O boundary");
    }

    #[test]
    fn test_policy_without_allowance() {
        let policy = CratePolicy::new("gunbc-foo", CrateRole::Other, false, "no exceptions");
        assert!(policy.disallowed_methods_allowance().is_none());
    }
}
