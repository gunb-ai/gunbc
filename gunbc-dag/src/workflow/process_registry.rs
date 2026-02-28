//! gunbc-dag workflow registry adapters.
//!
//! Generic process/unit registry types live in `core/workflow`.
//! This module keeps repo-specific default derivation from DSL workflows.

pub use gunbc_workflow::{
    claim_handle_type_id, ClaimId, ProcessId, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec,
    UnitClaim,
};

/// Default registry for planner bootstrap, derived from `dsl/workflows/*.dag`.
pub fn default_process_unit_registry() -> ProcessUnitRegistry {
    super::catalog::build_process_unit_registry().unwrap_or_else(|error| {
        panic!("failed to derive process unit registry from DSL workflows: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::capabilities::{
        CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
    };

    #[test]
    fn default_registry_contains_core_and_tool_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&ProcessUnitRef::new("ci", "ci.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("test_all", "test_all.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.gist_create")));
    }

    #[test]
    fn default_registry_contains_canonical_capability_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&ProcessUnitRef::new(
            COMPILATION_PROCESS_ID,
            COMPILATION_ENSURE_UNIT
        )));
        assert!(registry.contains(&ProcessUnitRef::new(
            CODEGEN_PROCESS_ID,
            CODEGEN_ENSURE_UNIT
        )));
    }

    #[test]
    fn claim_handle_type_policy_maps_common_prefixes() {
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("file:workspace")),
            "FilesystemHandle"
        );
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("tool:cargo")),
            "ToolHandle"
        );
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("network:github_gist")),
            "NetworkHandle"
        );
    }
}
