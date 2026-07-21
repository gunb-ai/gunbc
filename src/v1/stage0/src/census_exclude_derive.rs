//! Census exclude-closure derivation (option b scaffolding — fierce-heron-512).
//!
//! Authority: baked SEED rows (`whole_tree_strict_resolve_exclusion_substrings`) plus
//! fixed-point transitive-importer closure of strict-resolve failures. NOT wired into
//! census bin defaults until safe window (#6968 merge or stern-newt ping).
//!
//! Cascade semantics (sunny-wolf-225 resolution):
//! - (i) Silent live-importer loss → typed refusal.
//! - (ii) Derived closure exclusion of live importers → legitimate, reported in receipt.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const PINNED_ORACLE_EXCLUDES_REL: &str = "docs/probes/census_extra_excludes.txt";
pub const PINNED_COORDINATION_REL: &str = "docs/probes/still-hawk-row-coordination.txt";

/// Live pipeline module excluded via derived closure (reported, not refused).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveImporterExclusion {
    pub module_path: String,
    pub seed_chain: String,
    pub round: u32,
}

/// Derived exclude closure + provenance receipt. Wired after safe window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedExcludeClosure {
    pub module_paths: BTreeSet<String>,
    pub live_importers_excluded: Vec<LiveImporterExclusion>,
    pub convergence_rounds: u32,
    pub memo_content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSymmetryDiff {
    pub only_left: Vec<String>,
    pub only_right: Vec<String>,
}

impl SetSymmetryDiff {
    pub fn is_empty(&self) -> bool {
        self.only_left.is_empty() && self.only_right.is_empty()
    }
}

/// Load the git-readable 83-row oracle pin (stern-newt @ eaf13cd3c0).
pub fn load_pinned_oracle_module_paths(workspace_root: &Path) -> Result<BTreeSet<String>, String> {
    let path = workspace_root.join(PINNED_ORACLE_EXCLUDES_REL);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("pinned oracle: failed to read {}: {e}", path.display()))?;
    let mut paths = BTreeSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        paths.insert(line.to_string());
    }
    if paths.is_empty() {
        return Err(format!(
            "pinned oracle: {PINNED_ORACLE_EXCLUDES_REL} contains no module paths (fail-closed)"
        ));
    }
    Ok(paths)
}

/// Set equality witness helper for acceptance oracle (`derived == recovered-83`).
pub fn symmetric_module_path_diff(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> SetSymmetryDiff {
    let only_left: Vec<String> = left.difference(right).cloned().collect();
    let only_right: Vec<String> = right.difference(left).cloned().collect();
    SetSymmetryDiff {
        only_left,
        only_right,
    }
}

/// Fixed-point derivation: SEED ∪ transitive-importer-closure of strict-resolve failures.
/// Scaffold only — lands after safe window; memo keyed on content-hash of seeds + tree.
pub fn derive_census_exclude_closure(
    _workspace_root: &Path,
    _source_roots: &[String],
) -> Result<DerivedExcludeClosure, String> {
    Err(
        "derive_census_exclude_closure: scaffold only — blocked until safe window \
         (#6968 merge or stern-newt ping)"
            .to_string(),
    )
}

/// (i) Silent-loss detector: live importer dropped without a matching receipt row.
pub fn refuse_silent_live_importer_loss(
    _before: &BTreeSet<String>,
    _after: &BTreeSet<String>,
    _receipt: &DerivedExcludeClosure,
    _live_pipeline_modules: &[String],
) -> Result<(), String> {
    Err(
        "refuse_silent_live_importer_loss: scaffold only — blocked until safe window"
            .to_string(),
    )
}

pub fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("../../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../.."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_oracle_loads_eighty_three_paths() {
        let ws = workspace_root_from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
        let paths = load_pinned_oracle_module_paths(&ws).expect("pinned oracle paths");
        assert_eq!(
            paths.len(),
            83,
            "docs/probes/census_extra_excludes.txt must enumerate 83 module paths"
        );
        assert!(paths.contains("src/v2/compiler/00_compile.dag"));
        assert!(paths.contains("src/v2/compiler/03_ingest.dag"));
    }
}
