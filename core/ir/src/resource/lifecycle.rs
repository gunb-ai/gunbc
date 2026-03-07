//! Managed lifecycle IR types for existence-level resource management.
//!
//! Distinct from run-scope acquire/release (which are per-invocation),
//! these types model the full lifecycle of a managed unit:
//! ensure_present, verify_present, disable, drain, destroy, verify_absent.
//!
//! Design reference: docs/design/horizon/h12-managed-lifecycle-control.md

use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle verbs for managed resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleVerb {
    EnsurePresent,
    VerifyPresent,
    Disable,
    Drain,
    Destroy,
    VerifyAbsent,
}

impl LifecycleVerb {
    /// All verbs in canonical order.
    pub const ALL: &'static [LifecycleVerb] = &[
        LifecycleVerb::EnsurePresent,
        LifecycleVerb::VerifyPresent,
        LifecycleVerb::Disable,
        LifecycleVerb::Drain,
        LifecycleVerb::Destroy,
        LifecycleVerb::VerifyAbsent,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleVerb::EnsurePresent => "ensure_present",
            LifecycleVerb::VerifyPresent => "verify_present",
            LifecycleVerb::Disable => "disable",
            LifecycleVerb::Drain => "drain",
            LifecycleVerb::Destroy => "destroy",
            LifecycleVerb::VerifyAbsent => "verify_absent",
        }
    }
}

impl fmt::Display for LifecycleVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How destruction is supported for a managed resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DestroySupport {
    /// Supports only graceful destroy (disable → drain → destroy).
    GracefulOnly,
    /// Supports both graceful and brutal (direct destroy).
    GracefulAndBrutal,
    /// Destruction not supported — resource is permanent.
    Unsupported,
}

impl DestroySupport {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "GracefulOnly" => Some(DestroySupport::GracefulOnly),
            "GracefulAndBrutal" => Some(DestroySupport::GracefulAndBrutal),
            "Unsupported" => Some(DestroySupport::Unsupported),
            _ => None,
        }
    }
}

impl fmt::Display for DestroySupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DestroySupport::GracefulOnly => write!(f, "GracefulOnly"),
            DestroySupport::GracefulAndBrutal => write!(f, "GracefulAndBrutal"),
            DestroySupport::Unsupported => write!(f, "Unsupported"),
        }
    }
}

/// IR-level managed lifecycle definition for a resource.
///
/// This is the lowered form of the AST `ManagedLifecycleDef`.
/// Each verb maps to a bool indicating whether the resource
/// declares a handler for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrManagedLifecycle {
    pub resource_name: String,
    pub destroy_support: DestroySupport,
    pub has_ensure_present: bool,
    pub has_verify_present: bool,
    pub has_disable: bool,
    pub has_drain: bool,
    pub has_destroy: bool,
    pub has_verify_absent: bool,
}

impl IrManagedLifecycle {
    /// Which lifecycle verbs this resource declares.
    pub fn declared_verbs(&self) -> Vec<LifecycleVerb> {
        let mut verbs = Vec::new();
        if self.has_ensure_present {
            verbs.push(LifecycleVerb::EnsurePresent);
        }
        if self.has_verify_present {
            verbs.push(LifecycleVerb::VerifyPresent);
        }
        if self.has_disable {
            verbs.push(LifecycleVerb::Disable);
        }
        if self.has_drain {
            verbs.push(LifecycleVerb::Drain);
        }
        if self.has_destroy {
            verbs.push(LifecycleVerb::Destroy);
        }
        if self.has_verify_absent {
            verbs.push(LifecycleVerb::VerifyAbsent);
        }
        verbs
    }

    /// Validate lifecycle invariants. Returns errors if any are violated.
    pub fn validate(&self) -> Vec<LifecycleValidationError> {
        let mut errors = Vec::new();

        // ensure_present requires verify_present
        if self.has_ensure_present && !self.has_verify_present {
            errors.push(LifecycleValidationError::EnsureWithoutVerify {
                resource: self.resource_name.clone(),
            });
        }

        // GracefulOnly requires disable + drain
        if self.destroy_support == DestroySupport::GracefulOnly {
            if !self.has_disable {
                errors.push(LifecycleValidationError::GracefulWithoutDisable {
                    resource: self.resource_name.clone(),
                });
            }
            if !self.has_drain {
                errors.push(LifecycleValidationError::GracefulWithoutDrain {
                    resource: self.resource_name.clone(),
                });
            }
        }

        // destroy requires verify_absent
        if self.has_destroy && !self.has_verify_absent {
            errors.push(LifecycleValidationError::DestroyWithoutVerifyAbsent {
                resource: self.resource_name.clone(),
            });
        }

        // Unsupported must not have destroy
        if self.destroy_support == DestroySupport::Unsupported && self.has_destroy {
            errors.push(LifecycleValidationError::UnsupportedWithDestroy {
                resource: self.resource_name.clone(),
            });
        }

        errors
    }

    /// Return the graceful destroy path: disable → drain → destroy → verify_absent.
    pub fn graceful_destroy_path(&self) -> Vec<LifecycleVerb> {
        let mut path = Vec::new();
        if self.has_disable {
            path.push(LifecycleVerb::Disable);
        }
        if self.has_drain {
            path.push(LifecycleVerb::Drain);
        }
        if self.has_destroy {
            path.push(LifecycleVerb::Destroy);
        }
        if self.has_verify_absent {
            path.push(LifecycleVerb::VerifyAbsent);
        }
        path
    }

    /// Return the brutal destroy path: destroy → verify_absent.
    pub fn brutal_destroy_path(&self) -> Vec<LifecycleVerb> {
        let mut path = Vec::new();
        if self.has_destroy {
            path.push(LifecycleVerb::Destroy);
        }
        if self.has_verify_absent {
            path.push(LifecycleVerb::VerifyAbsent);
        }
        path
    }
}

/// Validation errors for managed lifecycle definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleValidationError {
    /// ensure_present declared without verify_present.
    EnsureWithoutVerify { resource: String },
    /// GracefulOnly destroy_support without disable verb.
    GracefulWithoutDisable { resource: String },
    /// GracefulOnly destroy_support without drain verb.
    GracefulWithoutDrain { resource: String },
    /// destroy declared without verify_absent.
    DestroyWithoutVerifyAbsent { resource: String },
    /// Unsupported destroy_support with destroy verb declared.
    UnsupportedWithDestroy { resource: String },
}

impl fmt::Display for LifecycleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleValidationError::EnsureWithoutVerify { resource } => {
                write!(f, "resource '{resource}': ensure_present requires verify_present")
            }
            LifecycleValidationError::GracefulWithoutDisable { resource } => {
                write!(f, "resource '{resource}': GracefulOnly requires disable")
            }
            LifecycleValidationError::GracefulWithoutDrain { resource } => {
                write!(f, "resource '{resource}': GracefulOnly requires drain")
            }
            LifecycleValidationError::DestroyWithoutVerifyAbsent { resource } => {
                write!(f, "resource '{resource}': destroy requires verify_absent")
            }
            LifecycleValidationError::UnsupportedWithDestroy { resource } => {
                write!(f, "resource '{resource}': Unsupported destroy_support cannot have destroy verb")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lifecycle(name: &str) -> IrManagedLifecycle {
        IrManagedLifecycle {
            resource_name: name.to_string(),
            destroy_support: DestroySupport::GracefulAndBrutal,
            has_ensure_present: true,
            has_verify_present: true,
            has_disable: true,
            has_drain: true,
            has_destroy: true,
            has_verify_absent: true,
        }
    }

    #[test]
    fn full_lifecycle_validates() {
        let lc = make_lifecycle("fleet");
        assert!(lc.validate().is_empty());
    }

    #[test]
    fn ensure_without_verify_is_error() {
        let mut lc = make_lifecycle("fleet");
        lc.has_verify_present = false;
        let errors = lc.validate();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            LifecycleValidationError::EnsureWithoutVerify { .. }
        ));
    }

    #[test]
    fn graceful_only_without_disable_is_error() {
        let mut lc = make_lifecycle("fleet");
        lc.destroy_support = DestroySupport::GracefulOnly;
        lc.has_disable = false;
        let errors = lc.validate();
        assert!(errors.iter().any(|e| matches!(
            e,
            LifecycleValidationError::GracefulWithoutDisable { .. }
        )));
    }

    #[test]
    fn graceful_only_without_drain_is_error() {
        let mut lc = make_lifecycle("fleet");
        lc.destroy_support = DestroySupport::GracefulOnly;
        lc.has_drain = false;
        let errors = lc.validate();
        assert!(errors.iter().any(|e| matches!(
            e,
            LifecycleValidationError::GracefulWithoutDrain { .. }
        )));
    }

    #[test]
    fn destroy_without_verify_absent_is_error() {
        let mut lc = make_lifecycle("fleet");
        lc.has_verify_absent = false;
        let errors = lc.validate();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            LifecycleValidationError::DestroyWithoutVerifyAbsent { .. }
        ));
    }

    #[test]
    fn unsupported_with_destroy_is_error() {
        let mut lc = make_lifecycle("fleet");
        lc.destroy_support = DestroySupport::Unsupported;
        let errors = lc.validate();
        assert!(errors.iter().any(|e| matches!(
            e,
            LifecycleValidationError::UnsupportedWithDestroy { .. }
        )));
    }

    #[test]
    fn graceful_destroy_path() {
        let lc = make_lifecycle("fleet");
        assert_eq!(
            lc.graceful_destroy_path(),
            vec![
                LifecycleVerb::Disable,
                LifecycleVerb::Drain,
                LifecycleVerb::Destroy,
                LifecycleVerb::VerifyAbsent,
            ]
        );
    }

    #[test]
    fn brutal_destroy_path() {
        let lc = make_lifecycle("fleet");
        assert_eq!(
            lc.brutal_destroy_path(),
            vec![LifecycleVerb::Destroy, LifecycleVerb::VerifyAbsent]
        );
    }

    #[test]
    fn declared_verbs() {
        let mut lc = make_lifecycle("fleet");
        lc.has_disable = false;
        lc.has_drain = false;
        let verbs = lc.declared_verbs();
        assert_eq!(verbs.len(), 4);
        assert!(!verbs.contains(&LifecycleVerb::Disable));
        assert!(!verbs.contains(&LifecycleVerb::Drain));
    }

    #[test]
    fn destroy_support_from_str() {
        assert_eq!(
            DestroySupport::parse("GracefulOnly"),
            Some(DestroySupport::GracefulOnly)
        );
        assert_eq!(
            DestroySupport::parse("GracefulAndBrutal"),
            Some(DestroySupport::GracefulAndBrutal)
        );
        assert_eq!(
            DestroySupport::parse("Unsupported"),
            Some(DestroySupport::Unsupported)
        );
        assert_eq!(DestroySupport::parse("Invalid"), None);
    }
}
