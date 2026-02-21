//! Workspace binary invocation mapping.
//!
//! This is the single source of truth for how repo-local binaries are
//! invoked via cargo (package + bin name). Centralizing this mapping
//! prevents drift when binaries move between packages.

use gunbc_ir::CargoInvocation;
use gunbc_tool_registry::iter_tool_targets;

/// Repo-local workspace binaries (all live in gunbc-dag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceBinary {
    Build,
    Bootstrap,
    Ci,
    Codegen,
    CodegenDag,
    DepsConfig,
    Docgen,
    Infra,
    Makegen,
    Pragma,
    Sdlc,
    Testgen,
}

impl WorkspaceBinary {
    /// Canonical ordered registry of all workspace binaries.
    pub const ALL: [Self; 12] = [
        Self::Build,
        Self::Bootstrap,
        Self::Ci,
        Self::Codegen,
        Self::CodegenDag,
        Self::DepsConfig,
        Self::Docgen,
        Self::Infra,
        Self::Makegen,
        Self::Pragma,
        Self::Sdlc,
        Self::Testgen,
    ];

    /// Iterate all known workspace binaries.
    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    /// Tool registry name for this binary when present.
    pub fn tool_name(self) -> &'static str {
        match self {
            WorkspaceBinary::Build => "build",
            WorkspaceBinary::Bootstrap => "bootstrap",
            WorkspaceBinary::Ci => "ci",
            WorkspaceBinary::Codegen => "codegen",
            WorkspaceBinary::CodegenDag => "codegen-dag",
            WorkspaceBinary::DepsConfig => "deps-config",
            WorkspaceBinary::Docgen => "docgen",
            WorkspaceBinary::Infra => "infra",
            WorkspaceBinary::Makegen => "makegen",
            WorkspaceBinary::Pragma => "pragma",
            WorkspaceBinary::Sdlc => "sdlc",
            WorkspaceBinary::Testgen => "testgen",
        }
    }

    /// Resolve enum variant from tool registry name.
    pub fn from_tool_name(name: &str) -> Option<Self> {
        match name {
            "build" => Some(Self::Build),
            "bootstrap" => Some(Self::Bootstrap),
            "ci" => Some(Self::Ci),
            "codegen" => Some(Self::Codegen),
            "codegen-dag" => Some(Self::CodegenDag),
            "deps-config" => Some(Self::DepsConfig),
            "docgen" => Some(Self::Docgen),
            "infra" => Some(Self::Infra),
            "makegen" => Some(Self::Makegen),
            "pragma" => Some(Self::Pragma),
            "sdlc" => Some(Self::Sdlc),
            "testgen" => Some(Self::Testgen),
            _ => None,
        }
    }

    /// Component name used to compose the binary name.
    pub fn component(self) -> &'static str {
        match self {
            WorkspaceBinary::Build => "build",
            WorkspaceBinary::Bootstrap => "bootstrap",
            WorkspaceBinary::Ci => "ci",
            WorkspaceBinary::Codegen => "codegen",
            WorkspaceBinary::CodegenDag => "codegen-dag",
            WorkspaceBinary::DepsConfig => "deps-config",
            WorkspaceBinary::Docgen => "docgen",
            WorkspaceBinary::Infra => "infra",
            WorkspaceBinary::Makegen => "makegen",
            WorkspaceBinary::Pragma => "pragma",
            WorkspaceBinary::Sdlc => "sdlc",
            WorkspaceBinary::Testgen => "testgen",
        }
    }

    /// Whether this binary corresponds to a DSL pipeline module.
    pub fn is_dsl_pipeline_module(self) -> bool {
        matches!(self, Self::Ci | Self::Sdlc)
    }

    /// Whether this binary corresponds to a DSL tool module.
    pub fn is_dsl_tool_module(self) -> bool {
        !self.is_dsl_pipeline_module() && !matches!(self, Self::CodegenDag | Self::DepsConfig)
    }

    /// Cargo invocation for this workspace binary.
    pub fn invocation(self) -> CargoInvocation {
        self.registry_invocation()
            .unwrap_or_else(|| CargoInvocation::composed(self.component(), "dag"))
    }

    /// Full `cargo run ...` command string.
    pub fn command(self) -> String {
        self.invocation().command()
    }

    fn registry_invocation(self) -> Option<CargoInvocation> {
        let tool = iter_tool_targets().find(|tool| tool.tool_name == self.tool_name())?;
        if !tool.has_invocation {
            return None;
        }
        let package = tool.package?;
        let binary = tool.binary.unwrap_or(tool.tool_name);
        Some(CargoInvocation::composed(binary, package))
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceBinary;

    #[test]
    fn tool_name_round_trip_supports_workspace_binary_variants() {
        for binary in WorkspaceBinary::all() {
            let name = binary.tool_name();
            assert_eq!(
                WorkspaceBinary::from_tool_name(name),
                Some(*binary),
                "tool name should round-trip for {name}"
            );
        }
    }

    #[test]
    fn registry_invocation_is_used_when_tool_metadata_exists() {
        assert!(
            WorkspaceBinary::Bootstrap.registry_invocation().is_some(),
            "bootstrap should derive invocation from tool registry metadata"
        );
        assert!(
            WorkspaceBinary::Makegen.registry_invocation().is_some(),
            "makegen should derive invocation from tool registry metadata"
        );
    }

    #[test]
    fn invocation_falls_back_for_internal_binaries_without_tool_registration() {
        assert!(
            WorkspaceBinary::CodegenDag.registry_invocation().is_none(),
            "codegen-dag currently has no tool-target metadata"
        );
        assert_eq!(
            WorkspaceBinary::CodegenDag.invocation().binary,
            "gunbc-codegen-dag".to_string()
        );
        assert_eq!(
            WorkspaceBinary::DepsConfig.invocation().binary,
            "gunbc-deps-config".to_string()
        );
        assert_eq!(
            WorkspaceBinary::Infra.invocation().binary,
            "gunbc-infra".to_string()
        );
    }
}
