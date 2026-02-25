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
    Bootstrap      => "bootstrap",
    Ci             => "ci",
    Codegen        => "codegen",
    CodegenDag     => "codegen-dag",
    DepsConfig     => "deps-config",
    GenerateDesign => "generate-design",
    Gist           => "gist",
    GistDiff       => "gist-diff",
    GistRecent     => "gist-recent",
    Infra          => "infra",
    Makegen        => "makegen",
    Pragma         => "pragma",
    ReviewDesign   => "review-design",
    Testgen        => "testgen",
}

impl WorkspaceBinary {
    /// Cargo invocation for this workspace binary.
    ///
    /// All binaries live in `gunbc-dag` and follow the `gunbc-{name}` naming
    /// convention, so the invocation is always `cargo run -p gunbc-dag --bin gunbc-{name}`.
    pub fn invocation(self) -> CargoInvocation {
        CargoInvocation::composed(self.component(), "dag")
    }

    /// Full `cargo run ...` command string.
    pub fn command(self) -> String {
        self.invocation().command()
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
    fn invocation_produces_correct_binary_names() {
        assert_eq!(
            WorkspaceBinary::Bootstrap.invocation().binary,
            "gunbc-bootstrap".to_string()
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
            // Dedicated executors that are not `WorkspaceBinary` tool modules.
            "deps",
            "docgen",
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
