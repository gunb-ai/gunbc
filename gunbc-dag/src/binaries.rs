//! Workspace binary invocation mapping.
//!
//! This is the single source of truth for how repo-local binaries are
//! invoked via cargo (package + bin name). Centralizing this mapping
//! prevents drift when binaries move between packages.
//!
//! **Adding a new binary**: add one line to the `workspace_binaries!` table
//! below. The enum variant, `ALL` array, `tool_name()`, `from_tool_name()`,
//! and `component()` are all derived automatically.

use gunbc_ir::CargoInvocation;
use gunbc_tool_registry::iter_tool_targets;

/// Generates the `WorkspaceBinary` enum and its core accessors from a single
/// definition table. Each entry is `VariantName => "tool-name"`.
macro_rules! workspace_binaries {
    ( $( $variant:ident => $tool_name:expr ),* $(,)? ) => {
        /// Repo-local workspace binaries (all live in gunbc-dag).
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum WorkspaceBinary {
            $( $variant, )*
        }

        impl WorkspaceBinary {
            /// Canonical ordered registry of all workspace binaries.
            pub const ALL: &[Self] = &[ $( Self::$variant, )* ];

            /// Iterate all known workspace binaries.
            pub fn all() -> &'static [Self] {
                Self::ALL
            }

            /// Tool registry name for this binary.
            pub fn tool_name(self) -> &'static str {
                match self {
                    $( Self::$variant => $tool_name, )*
                }
            }

            /// Resolve enum variant from tool registry name.
            pub fn from_tool_name(name: &str) -> Option<Self> {
                match name {
                    $( $tool_name => Some(Self::$variant), )*
                    _ => None,
                }
            }

            /// Component name used to compose the binary name.
            pub fn component(self) -> &'static str {
                self.tool_name()
            }
        }
    };
}

workspace_binaries! {
    Build       => "build",
    Bootstrap   => "bootstrap",
    Ci          => "ci",
    Codegen     => "codegen",
    CodegenDag  => "codegen-dag",
    DepsConfig  => "deps-config",
    Docgen      => "docgen",
    Infra       => "infra",
    Makegen     => "makegen",
    Pragma      => "pragma",
    Sdlc        => "sdlc",
    Testgen     => "testgen",
}

impl WorkspaceBinary {
    /// Whether this binary corresponds to a DSL pipeline module.
    pub fn is_dsl_pipeline_module(self) -> bool {
        matches!(self, Self::Ci)
    }

    /// Whether this binary corresponds to a DSL tool module.
    pub fn is_dsl_tool_module(self) -> bool {
        !self.is_dsl_pipeline_module()
            && !matches!(self, Self::CodegenDag | Self::DepsConfig | Self::Sdlc)
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
    use std::collections::BTreeSet;
    use toml_edit::DocumentMut;

    fn cargo_manifest_bin_tool_names() -> BTreeSet<String> {
        let manifest: DocumentMut = include_str!("../Cargo.toml")
            .parse()
            .expect("gunbc-dag/Cargo.toml should parse as TOML");
        let bins = manifest
            .get("bin")
            .and_then(|item| item.as_array_of_tables())
            .expect("gunbc-dag/Cargo.toml should have [[bin]] entries");
        bins.iter()
            .filter_map(|entry| entry.get("name").and_then(|item| item.as_str()))
            .filter_map(|name| name.strip_prefix("gunbc-"))
            .map(str::to_string)
            .collect()
    }

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

    #[test]
    fn workspace_binary_table_matches_cargo_manifest_bins() {
        let non_workspace_dispatch_bins: BTreeSet<&str> = BTreeSet::from([
            // DSL/runtime dispatches these through shared tool-family modules.
            "deps",
            "gist",
            "gist-diff",
            "gist-recent",
            // DSL/runtime dispatches these through shared tool-family modules.
            "dag-viz",
            "dag-viz-diff",
            "dag-viz-recent",
            "dag-snapshot",
            // Dedicated executors that are not `WorkspaceBinary` tool modules.
            "review",
            "pipeline",
            "workflow",
        ]);

        let manifest_bins = cargo_manifest_bin_tool_names();
        let expected_workspace_bins: BTreeSet<String> = manifest_bins
            .into_iter()
            .filter(|tool| !non_workspace_dispatch_bins.contains(tool.as_str()))
            .collect();
        let actual_workspace_bins: BTreeSet<String> = WorkspaceBinary::all()
            .iter()
            .map(|binary| binary.tool_name().to_string())
            .collect();

        assert_eq!(
            actual_workspace_bins, expected_workspace_bins,
            "WorkspaceBinary entries should stay aligned with gunbc-dag/Cargo.toml [[bin]] declarations"
        );
    }
}
