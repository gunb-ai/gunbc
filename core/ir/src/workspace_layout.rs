//! Workspace layout modeling derived from Cargo metadata.
//!
//! Centralizes path discovery so callsites can stop hardcoding `../..`,
//! parent chains, and fixed-depth assumptions.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value as JsonValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceLayoutError {
    #[error("failed to read current directory: {0}")]
    CurrentDir(#[source] std::io::Error),
    #[error("failed to canonicalize path '{path}': {source}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to run `cargo metadata` in {cwd}: {source}")]
    MetadataCommand {
        cwd: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("`cargo metadata` failed in {cwd}: {stderr}")]
    MetadataFailed { cwd: PathBuf, stderr: String },
    #[error("failed to parse `cargo metadata` json: {0}")]
    MetadataJson(#[from] serde_json::Error),
    #[error("`cargo metadata` missing field `{0}`")]
    MissingField(&'static str),
    #[error("package '{name}' has invalid manifest_path '{manifest_path}'")]
    InvalidManifestPath { name: String, manifest_path: String },
    #[error("unable to locate Cargo workspace root from {start}")]
    WorkspaceRootNotFound { start: PathBuf },
}

/// Canonical path map for this Cargo workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceLayout {
    pub workspace_root: PathBuf,
    pub crates: BTreeMap<String, PathBuf>,
}

impl WorkspaceLayout {
    const CODEGEN_OUT_REL: &'static str = "target/codegen";
    const CODEGEN_BIN_REL: &'static str = "target/codegen/bin";
    const CODEGEN_LIB_REL: &'static str = "target/codegen/lib";
    const CODEGEN_STAMP_REL: &'static str = "target/codegen/.codegen-stamp";

    /// Resolve layout from `cargo metadata` starting in the current directory.
    pub fn from_cargo_metadata() -> Result<Self, WorkspaceLayoutError> {
        let cwd = std::env::current_dir().map_err(WorkspaceLayoutError::CurrentDir)?;
        Self::from_cargo_metadata_in(&cwd)
    }

    /// Resolve layout from `cargo metadata` starting in `cwd`.
    pub fn from_cargo_metadata_in(cwd: &Path) -> Result<Self, WorkspaceLayoutError> {
        let cwd = canonicalize_path(cwd)?;
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version=1")
            .arg("--no-deps")
            .current_dir(&cwd)
            .output()
            .map_err(|source| WorkspaceLayoutError::MetadataCommand {
                cwd: cwd.clone(),
                source,
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(WorkspaceLayoutError::MetadataFailed { cwd, stderr });
        }

        let json: JsonValue = serde_json::from_slice(&output.stdout)?;
        let workspace_root = json
            .get("workspace_root")
            .and_then(JsonValue::as_str)
            .ok_or(WorkspaceLayoutError::MissingField("workspace_root"))?;
        let workspace_root = canonicalize_path(Path::new(workspace_root))?;

        let packages = json
            .get("packages")
            .and_then(JsonValue::as_array)
            .ok_or(WorkspaceLayoutError::MissingField("packages"))?;

        let mut crates = BTreeMap::new();
        for pkg in packages {
            let Some(name) = pkg.get("name").and_then(JsonValue::as_str) else {
                continue;
            };
            let Some(manifest_path) = pkg.get("manifest_path").and_then(JsonValue::as_str) else {
                continue;
            };
            let manifest_path = PathBuf::from(manifest_path);
            let Some(crate_dir) = manifest_path.parent() else {
                return Err(WorkspaceLayoutError::InvalidManifestPath {
                    name: name.to_string(),
                    manifest_path: manifest_path.display().to_string(),
                });
            };
            crates.insert(name.to_string(), canonicalize_path(crate_dir)?);
        }

        Ok(Self {
            workspace_root,
            crates,
        })
    }

    /// Resolve layout from the crate compile-time manifest directory.
    pub fn from_env_manifest_dir() -> Result<Self, WorkspaceLayoutError> {
        Self::from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
    }

    /// Resolve layout from an arbitrary crate/package manifest directory.
    ///
    /// Walks ancestors until a workspace `Cargo.toml` is found, then loads
    /// full package layout via `cargo metadata`.
    pub fn from_manifest_dir(manifest_dir: &Path) -> Result<Self, WorkspaceLayoutError> {
        let manifest_dir = canonicalize_path(manifest_dir)?;
        for ancestor in manifest_dir.ancestors() {
            let cargo_toml = ancestor.join("Cargo.toml");
            if !cargo_toml.is_file() {
                continue;
            }
            if std::fs::read_to_string(&cargo_toml)
                .ok()
                .is_some_and(|contents| contents.contains("[workspace]"))
            {
                return Self::from_cargo_metadata_in(ancestor);
            }
        }
        Err(WorkspaceLayoutError::WorkspaceRootNotFound {
            start: manifest_dir,
        })
    }

    /// Return the absolute crate directory for a crate name.
    pub fn crate_dir(&self, crate_name: &str) -> Option<&Path> {
        self.crates.get(crate_name).map(PathBuf::as_path)
    }

    /// Compute a relative path from `from` to `to`.
    pub fn relative_path(&self, from: &Path, to: &Path) -> PathBuf {
        relative_path_between(
            &self.absolutize(from),
            &self.absolutize(to),
            self.workspace_root.as_path(),
        )
    }

    /// Derive source globs for the provided crate names.
    ///
    /// Each crate contributes:
    /// - `<crate>/src/**/*.rs`
    /// - `<crate>/Cargo.toml`
    pub fn source_globs(&self, crates: &[&str]) -> Vec<String> {
        let mut globs = Vec::new();
        for crate_name in crates {
            let Some(crate_dir) = self.crate_dir(crate_name) else {
                continue;
            };
            let rel = self.relative_path(&self.workspace_root, crate_dir);
            let rel = normalize_glob_path(rel);
            let prefix = if rel == "." {
                String::new()
            } else {
                format!("{rel}/")
            };
            globs.push(format!("{prefix}src/**/*.rs"));
            globs.push(format!("{prefix}Cargo.toml"));
        }
        globs.sort();
        globs.dedup();
        globs
    }

    /// Absolute `target/codegen` directory for this workspace.
    pub fn codegen_out_dir(&self) -> PathBuf {
        self.workspace_root.join(Self::CODEGEN_OUT_REL)
    }

    /// Absolute `target/codegen/bin` directory for this workspace.
    pub fn codegen_bin_dir(&self) -> PathBuf {
        self.workspace_root.join(Self::CODEGEN_BIN_REL)
    }

    /// Absolute `target/codegen/lib` directory for this workspace.
    pub fn codegen_lib_dir(&self) -> PathBuf {
        self.workspace_root.join(Self::CODEGEN_LIB_REL)
    }

    /// Absolute `target/codegen/.codegen-stamp` path for this workspace.
    pub fn codegen_stamp_path(&self) -> PathBuf {
        self.workspace_root.join(Self::CODEGEN_STAMP_REL)
    }

    /// Absolute DSL tool module root.
    pub fn dsl_tools_root(&self) -> PathBuf {
        self.workspace_root.join("dsl/tools")
    }

    /// Absolute DSL pipeline module root.
    pub fn dsl_pipelines_root(&self) -> PathBuf {
        self.workspace_root.join("dsl/pipelines")
    }

    fn absolutize(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        }
    }
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, WorkspaceLayoutError> {
    path.canonicalize()
        .map_err(|source| WorkspaceLayoutError::Canonicalize {
            path: path.to_path_buf(),
            source,
        })
}

fn normalize_glob_path(path: PathBuf) -> String {
    let display = path.to_string_lossy().replace('\\', "/");
    if display.is_empty() {
        ".".to_string()
    } else {
        display
    }
}

fn relative_path_between(from: &Path, to: &Path, fallback_root: &Path) -> PathBuf {
    let from = from
        .canonicalize()
        .ok()
        .or_else(|| absolutize_with_root(from, fallback_root))
        .unwrap_or_else(|| from.to_path_buf());
    let to = to
        .canonicalize()
        .ok()
        .or_else(|| absolutize_with_root(to, fallback_root))
        .unwrap_or_else(|| to.to_path_buf());

    let from_components: Vec<Component<'_>> = from.components().collect();
    let to_components: Vec<Component<'_>> = to.components().collect();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut out = PathBuf::new();
    for _ in common_len..from_components.len() {
        out.push("..");
    }
    for component in &to_components[common_len..] {
        out.push(component.as_os_str());
    }
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

fn absolutize_with_root(path: &Path, root: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else if root.is_absolute() {
        Some(root.join(path))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_layout_from_env_manifest_contains_current_crate() {
        let layout = WorkspaceLayout::from_env_manifest_dir().expect("resolve workspace layout");
        let ir_dir = layout
            .crate_dir("gunbc-ir")
            .expect("workspace should include gunbc-ir crate");
        assert!(
            ir_dir.ends_with("core/ir"),
            "expected gunbc-ir dir to end with core/ir, got {}",
            ir_dir.display()
        );
    }

    #[test]
    fn relative_path_computes_nested_workspace_paths() {
        let layout = WorkspaceLayout::from_env_manifest_dir().expect("resolve workspace layout");
        let from = layout.workspace_root.join("core/ir");
        let to = layout.workspace_root.join("dsl/std/types.dag");
        assert_eq!(
            layout.relative_path(&from, &to),
            PathBuf::from("../../dsl/std/types.dag")
        );
    }

    #[test]
    fn source_globs_derives_crate_sources_and_manifest() {
        let layout = WorkspaceLayout::from_env_manifest_dir().expect("resolve workspace layout");
        let globs = layout.source_globs(&["gunbc-ir"]);
        assert!(
            globs.iter().any(|g| g == "core/ir/src/**/*.rs"),
            "expected core/ir source glob, got: {globs:?}"
        );
        assert!(
            globs.iter().any(|g| g == "core/ir/Cargo.toml"),
            "expected core/ir manifest glob, got: {globs:?}"
        );
    }

    #[test]
    fn codegen_paths_are_workspace_relative() {
        let layout = WorkspaceLayout::from_env_manifest_dir().expect("resolve workspace layout");
        assert_eq!(
            layout.relative_path(&layout.workspace_root, &layout.codegen_out_dir()),
            PathBuf::from("target/codegen")
        );
        assert_eq!(
            layout.relative_path(&layout.workspace_root, &layout.codegen_bin_dir()),
            PathBuf::from("target/codegen/bin")
        );
        assert_eq!(
            layout.relative_path(&layout.workspace_root, &layout.codegen_lib_dir()),
            PathBuf::from("target/codegen/lib")
        );
        assert_eq!(
            layout.relative_path(&layout.workspace_root, &layout.codegen_stamp_path()),
            PathBuf::from("target/codegen/.codegen-stamp")
        );
    }
}
