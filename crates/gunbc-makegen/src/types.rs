/// Configuration for makegen tool.
#[derive(Debug, Clone)]
pub struct MakegenConfig {
    /// Path to workspace root
    pub workspace_path: String,
    /// Generate per-crate build targets
    pub per_crate_targets: bool,
    /// Generate lint targets (clippy, fmt)
    pub lint_targets: bool,
    /// Output file path (relative to workspace)
    pub output_path: String,
    /// Force regeneration even if up-to-date
    pub force: bool,
}

impl Default for MakegenConfig {
    fn default() -> Self {
        Self {
            workspace_path: ".".into(),
            per_crate_targets: true,
            lint_targets: true,
            output_path: "Makefile".into(),
            force: false,
        }
    }
}

/// Information about a crate in the workspace.
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub path: String,
    pub is_binary: bool,
    pub is_library: bool,
}

/// A Make target definition.
#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub dependencies: Vec<String>,
    pub phony: bool,
}

/// A Make rule with commands.
#[derive(Debug, Clone)]
pub struct Rule {
    pub target: Target,
    pub commands: Vec<String>,
}

/// Final status after upsert operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertStatus {
    /// File was created (didn't exist before)
    Created,
    /// File was updated (hash changed)
    Updated,
    /// File was unchanged (hash matched)
    Unchanged,
    /// Dry run mode - would have written
    DryRun,
}

impl std::fmt::Display for UpsertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpsertStatus::Created => write!(f, "Created"),
            UpsertStatus::Updated => write!(f, "Updated"),
            UpsertStatus::Unchanged => write!(f, "Unchanged"),
            UpsertStatus::DryRun => write!(f, "DryRun"),
        }
    }
}
