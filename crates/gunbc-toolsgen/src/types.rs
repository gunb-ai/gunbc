/// Configuration for toolsgen.
#[derive(Debug, Clone)]
pub struct ToolsgenConfig {
    /// Path to workspace root
    pub workspace_path: String,
    /// Output file path (relative to workspace)
    pub output_path: String,
    /// Force regeneration even if up-to-date
    pub force: bool,
}

impl Default for ToolsgenConfig {
    fn default() -> Self {
        Self {
            workspace_path: ".".into(),
            output_path: "tools/cargo_wrapper.c".into(),
            force: false,
        }
    }
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
