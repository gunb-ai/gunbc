//! Managed resource trait for unified resource acquisition.
//!
//! This module defines the `ManagedResource` trait that unifies tools, build
//! artifacts, and other acquirable resources under a single pattern:
//! Check → Create → Resolve.

use super::super::ResourceId;
use super::def::{InputPattern, ResourceDef};
use super::handle::ResourceHandle;
use super::state::{ExecMode, ResourceState};
use super::{ContentHash, HashBuilder, ManifestEntry, ResourceManifest, DEFAULT_MANIFEST_PATH};
use gunbc_infra::freshness::{check_freshness_mtime, FileMtime, MtimeResult};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

/// Error type for resource operations.
#[derive(Debug, Error)]
pub enum ResourceError {
    /// Resource is missing and mode doesn't allow creation.
    #[error("Resource '{0}' is missing (run with --mode=ensure)")]
    Missing(ResourceId),

    /// Resource is stale and mode doesn't allow regeneration.
    #[error("Resource '{id}' is stale: {reason} (run with --mode=ensure)")]
    Stale { id: ResourceId, reason: String },

    /// Resource has no provider configured.
    #[error("Resource '{0}' has no provider configured")]
    NoProvider(ResourceId),

    /// A dependency resource required for hashing is missing.
    #[error("Resource '{resource}' depends on missing resource '{dependency}'")]
    MissingDependency {
        resource: ResourceId,
        dependency: ResourceId,
    },

    /// Error while checking resource state.
    #[error("Failed to check resource '{0}': {1}")]
    CheckFailed(ResourceId, String),

    /// Error while creating resource.
    #[error("Failed to create resource '{0}': {1}")]
    CreateFailed(ResourceId, String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Error type for updating resource manifests.
#[derive(Debug, Error)]
pub enum ManifestUpdateError {
    #[error("Failed to load manifest: {0}")]
    Load(std::io::Error),

    #[error("Failed to update manifest entry: {0}")]
    Acquire(ResourceError),

    #[error("Failed to save manifest: {0}")]
    Save(std::io::Error),
}

/// I/O interface for resolving resource inputs and manifest storage.
///
/// Implementations should route all filesystem and shell access through
/// the transport layer.
pub trait ResourceIo {
    /// Read a file's contents.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, ResourceError>;
    /// Write a file's contents.
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ResourceError>;
    /// Check whether a file exists.
    fn file_exists(&self, path: &Path) -> Result<bool, ResourceError>;
    /// Expand a glob pattern into file paths.
    fn glob_paths(&self, pattern: &str) -> Result<Vec<PathBuf>, ResourceError>;
    /// Execute a command and return stdout.
    fn command_output(&self, command: &str, args: &[String]) -> Result<Vec<u8>, ResourceError>;
    /// Get a file's modification time.
    fn file_mtime(&self, path: &Path) -> Result<SystemTime, ResourceError>;
    /// Resolve a file path to its canonical filesystem path.
    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, ResourceError>;
}

/// A resource that can be acquired with freshness checking.
///
/// This trait unifies tools, build artifacts, and other acquirable things.
/// All follow the same pattern: Check → Create → Resolve.
///
/// # Implementation Notes
///
/// - `definition()` returns the resource's inputs/outputs declaration
/// - `compute_key()` derives the freshness key from declared inputs + manifest
/// - `check_state()` compares computed key to manifest entry
/// - `create()` runs the provider to create/regenerate the resource (may read manifest)
///
/// The trait provides default implementations for most methods; implementations
/// typically only need to provide `definition()` and `create()`.
pub trait ManagedResource: Clone + Sized {
    /// Get the resource definition (inputs, outputs, provider).
    fn definition(&self) -> &ResourceDef;

    /// Compute the current freshness key from declared inputs.
    ///
    /// The default implementation derives the key from declared inputs.
    /// Implementations may override this if they have custom key computation.
    fn compute_key(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ContentHash, ResourceError> {
        Ok(self.compute_key_with_stats(manifest, io)?.0)
    }

    /// Compute the freshness key and input file count from declared inputs.
    ///
    /// The default implementation derives both from declared inputs.
    fn compute_key_with_stats(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<(ContentHash, usize), ResourceError> {
        compute_key_from_def(self.definition(), manifest, io)
    }

    /// Compute the freshness key, file count, and input file paths.
    ///
    /// The default implementation derives all three from declared inputs.
    fn compute_key_with_file_list(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<(ContentHash, usize, Vec<String>), ResourceError> {
        compute_key_with_files(self.definition(), manifest, io)
    }

    /// Create or regenerate this resource.
    ///
    /// Called when the resource is missing or stale and mode is `Ensure`.
    /// Returns the manifest entry to store.
    fn create(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError>;

    /// Get the resource ID.
    fn resource_id(&self) -> &ResourceId {
        &self.definition().id
    }

    /// Check current state against manifest.
    fn check_state(&self, manifest: &ResourceManifest, io: &dyn ResourceIo) -> ResourceState {
        let entry = match manifest.get(self.resource_id()) {
            None => return ResourceState::Missing,
            Some(entry) => entry,
        };

        let current_key = match self.compute_key(manifest, io) {
            Ok(k) => k,
            Err(e) => return ResourceState::Error(e.to_string()),
        };

        if entry.key != current_key {
            ResourceState::Stale {
                reason: "inputs changed".into(),
                stored_key: entry.key.clone(),
                current_key,
            }
        } else {
            ResourceState::Fresh
        }
    }

    /// Acquire a handle to this resource.
    ///
    /// Checks freshness, creates if needed (based on mode), returns handle.
    ///
    /// # Arguments
    ///
    /// * `mode` - Execution mode (Verify or Ensure)
    /// * `manifest` - The resource manifest for freshness checking
    ///
    /// # Returns
    ///
    /// A handle to the resource (proof of acquisition), or an error.
    fn acquire(
        &self,
        mode: ExecMode,
        manifest: &mut ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ResourceHandle<Self>, ResourceError> {
        let state = self.check_state(manifest, io);

        match (state, mode) {
            // Fresh in any mode: return handle with current key
            (ResourceState::Fresh, _) => {
                let key = self.compute_key(manifest, io)?;
                Ok(ResourceHandle::acquire(self.resource_id().clone(), key))
            }

            // Missing/Stale in Ensure mode: create, update manifest, return handle
            (ResourceState::Missing | ResourceState::Stale { .. }, ExecMode::Ensure) => {
                let entry = self.create(manifest, io)?;
                let key = entry.key.clone();
                manifest.insert(self.resource_id().clone(), entry);
                Ok(ResourceHandle::acquire(self.resource_id().clone(), key))
            }

            // Missing in Verify mode: error
            (ResourceState::Missing, ExecMode::Verify) => {
                Err(ResourceError::Missing(self.resource_id().clone()))
            }

            // Stale in Verify mode: error
            (ResourceState::Stale { reason, .. }, ExecMode::Verify) => Err(ResourceError::Stale {
                id: self.resource_id().clone(),
                reason,
            }),

            // Error state: propagate
            (ResourceState::Error(e), _) => {
                Err(ResourceError::CheckFailed(self.resource_id().clone(), e))
            }
        }
    }

    /// Check if this resource is fresh without acquiring it.
    fn is_fresh(&self, manifest: &ResourceManifest, io: &dyn ResourceIo) -> bool {
        self.check_state(manifest, io).is_fresh()
    }
}

/// Convenience struct for simple resource definitions.
///
/// This can be used directly as a `ManagedResource` when you don't need
/// custom behavior. Just provide the definition and a creation function.
#[derive(Debug, Clone)]
pub struct SimpleResource {
    def: ResourceDef,
}

impl SimpleResource {
    /// Create a new simple resource from a definition.
    pub fn new(def: ResourceDef) -> Self {
        Self { def }
    }
}

/// Compute a resource key from its declared inputs.
///
/// Returns `(key, input_file_count)` where `input_file_count` is the number of
/// files hashed (used by mtime fast paths).
fn compute_key_from_def(
    def: &ResourceDef,
    manifest: &ResourceManifest,
    io: &dyn ResourceIo,
) -> Result<(ContentHash, usize), ResourceError> {
    let (hash, count, _files) = compute_key_with_files(def, manifest, io)?;
    Ok((hash, count))
}

/// Compute a resource key from its declared inputs, including the list of
/// input file paths.
///
/// Returns `(key, input_file_count, input_files)`.
pub fn compute_key_with_files(
    def: &ResourceDef,
    manifest: &ResourceManifest,
    io: &dyn ResourceIo,
) -> Result<(ContentHash, usize, Vec<String>), ResourceError> {
    let mut builder = HashBuilder::new();
    let mut file_count: usize = 0;
    let mut file_paths: Vec<String> = Vec::new();

    for input in &def.inputs {
        match input {
            InputPattern::Glob(pattern) => {
                builder = builder.update(b"glob\0");
                builder = update_len_prefixed(builder, pattern);

                let mut canonical_paths = Vec::new();
                for path in io.glob_paths(pattern)? {
                    canonical_paths.push(io.canonicalize_path(&path)?);
                }
                canonical_paths.sort();
                for path in canonical_paths {
                    let contents = io.read_file(&path)?;
                    builder = builder.update_file_content(&path, &contents);
                    file_paths.push(path.to_string_lossy().into_owned());
                    file_count += 1;
                }
            }
            InputPattern::File(path) => {
                builder = builder.update(b"file\0");
                let canonical_path = io.canonicalize_path(path)?;
                let contents = io.read_file(&canonical_path)?;
                builder = builder.update_file_content(&canonical_path, &contents);
                file_count += 1;
                file_paths.push(canonical_path.to_string_lossy().into_owned());
            }
            InputPattern::Env(var) => {
                let value = std::env::var(var).unwrap_or_default();
                builder = update_tagged_str(builder, "env", var);
                builder = update_len_prefixed(builder, &value);
            }
            InputPattern::CommandOutput { command, args } => {
                builder = builder.update(b"cmd\0");
                let stdout = io.command_output(command, args)?;
                builder = builder.update_command_output_bytes(command, args, &stdout);
            }
            InputPattern::Resource(dep_id) => {
                let entry =
                    manifest
                        .get(dep_id)
                        .ok_or_else(|| ResourceError::MissingDependency {
                            resource: def.id.clone(),
                            dependency: dep_id.clone(),
                        })?;
                builder = update_tagged_str(builder, "resource", dep_id.0.as_str());
                builder = update_len_prefixed(builder, entry.key.as_str());
            }
        }
    }

    Ok((builder.finalize(), file_count, file_paths))
}

/// Inputs that can be checked via the mtime fast path.
#[derive(Debug, Clone)]
struct MtimeInputs {
    glob_patterns: Vec<String>,
    files: Vec<PathBuf>,
    has_non_file_inputs: bool,
}

/// Extract mtime-checkable inputs from a resource definition.
///
/// Returns glob patterns and file paths, plus a flag indicating whether the
/// definition includes non-file inputs (env/resource) that require full hashing.
fn mtime_inputs_from_def(def: &ResourceDef) -> MtimeInputs {
    let mut glob_patterns = Vec::new();
    let mut files = Vec::new();
    let mut has_non_file_inputs = false;

    for input in &def.inputs {
        match input {
            InputPattern::Glob(pattern) => glob_patterns.push(pattern.clone()),
            InputPattern::File(path) => files.push(path.clone()),
            InputPattern::Env(_)
            | InputPattern::Resource(_)
            | InputPattern::CommandOutput { .. } => {
                has_non_file_inputs = true;
            }
        }
    }

    MtimeInputs {
        glob_patterns,
        files,
        has_non_file_inputs,
    }
}

/// Result of checking manifest freshness for a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestFreshness {
    Fresh,
    FreshWithDiagnostic(String),
    Stale(String),
    Missing,
    Error(String),
}

/// Options for manifest freshness checks.
#[derive(Debug, Clone, Copy)]
pub struct FreshnessOptions {
    pub output_exists: Option<bool>,
    pub use_mtime: bool,
}

impl Default for FreshnessOptions {
    fn default() -> Self {
        Self {
            output_exists: None,
            use_mtime: true,
        }
    }
}

/// Check manifest freshness for a resource.
///
/// This is a shared helper for tooling that needs:
/// - manifest presence checks
/// - optional output existence checks
/// - optional mtime fast path
/// - full hash comparison on fallback
pub fn check_manifest_freshness<R: ManagedResource>(
    resource: &R,
    manifest: &ResourceManifest,
    options: FreshnessOptions,
    io: &dyn ResourceIo,
) -> ManifestFreshness {
    let def = resource.definition();

    let entry = match manifest.get(&def.id) {
        Some(e) => e,
        None => return ManifestFreshness::Missing,
    };

    if matches!(options.output_exists, Some(false)) {
        return ManifestFreshness::Stale("manifest present but output files missing".into());
    }

    if options.use_mtime {
        let mtime_inputs = mtime_inputs_from_def(def);
        if !mtime_inputs.has_non_file_inputs {
            let mut files: Vec<FileMtime> = Vec::new();
            let mut mtime_fallback_reason: Option<String> = None;
            let mtime_note: Option<String>;

            for pattern in &mtime_inputs.glob_patterns {
                let mut canonical_paths = Vec::new();
                let paths = match io.glob_paths(pattern) {
                    Ok(p) => p,
                    Err(e) => {
                        mtime_fallback_reason =
                            Some(format!("glob failed for '{}': {}", pattern, e));
                        break;
                    }
                };
                for path in paths {
                    let canonical_path = match io.canonicalize_path(&path) {
                        Ok(path) => path,
                        Err(e) => {
                            mtime_fallback_reason = Some(format!(
                                "canonicalize failed for '{}': {}",
                                path.display(),
                                e
                            ));
                            break;
                        }
                    };
                    canonical_paths.push(canonical_path);
                }
                if mtime_fallback_reason.is_some() {
                    break;
                }
                canonical_paths.sort();
                for path in canonical_paths {
                    match io.file_mtime(&path) {
                        Ok(modified) => files.push(FileMtime { path, modified }),
                        Err(e) => {
                            mtime_fallback_reason =
                                Some(format!("mtime failed for '{}': {}", path.display(), e));
                            break;
                        }
                    }
                }
                if mtime_fallback_reason.is_some() {
                    break;
                }
            }

            if mtime_fallback_reason.is_none() {
                for path in &mtime_inputs.files {
                    let canonical_path = match io.canonicalize_path(path) {
                        Ok(path) => path,
                        Err(e) => {
                            mtime_fallback_reason = Some(format!(
                                "canonicalize failed for '{}': {}",
                                path.display(),
                                e
                            ));
                            break;
                        }
                    };
                    match io.file_mtime(&canonical_path) {
                        Ok(modified) => files.push(FileMtime {
                            path: canonical_path,
                            modified,
                        }),
                        Err(e) => {
                            mtime_fallback_reason =
                                Some(format!("mtime failed for '{}': {}", path.display(), e));
                            break;
                        }
                    }
                }
            }

            match mtime_fallback_reason {
                None => match check_freshness_mtime(entry, &files) {
                    MtimeResult::Fresh => return ManifestFreshness::Fresh,
                    MtimeResult::MaybeStale(reason) => {
                        mtime_note = Some(format!(
                            "mtime indicates stale ({reason:?}); verified with full hash"
                        ));
                    }
                },
                Some(reason) => {
                    mtime_note = Some(format!(
                        "mtime fast path unavailable ({reason}); verified with full hash"
                    ));
                }
            }

            let current_key = match resource.compute_key(manifest, io) {
                Ok(k) => k,
                Err(e) => return ManifestFreshness::Error(e.to_string()),
            };

            if entry.key == current_key {
                return match mtime_note {
                    Some(note) => ManifestFreshness::FreshWithDiagnostic(note),
                    None => ManifestFreshness::Fresh,
                };
            }
            let stale_reason = match mtime_note {
                Some(note) => format!("inputs changed since last update ({note})"),
                None => "inputs changed since last update".to_string(),
            };
            return ManifestFreshness::Stale(stale_reason);
        }
    }

    let current_key = match resource.compute_key(manifest, io) {
        Ok(k) => k,
        Err(e) => return ManifestFreshness::Error(e.to_string()),
    };

    if entry.key == current_key {
        ManifestFreshness::Fresh
    } else {
        ManifestFreshness::Stale("inputs changed since last update".into())
    }
}

/// Load the default resource manifest using the provided I/O resolver.
pub fn load_manifest_default(io: &dyn ResourceIo) -> Result<ResourceManifest, io::Error> {
    load_manifest(io, Path::new(DEFAULT_MANIFEST_PATH))
}

/// Save the default resource manifest using the provided I/O resolver.
pub fn save_manifest_default(
    io: &dyn ResourceIo,
    manifest: &ResourceManifest,
) -> Result<(), io::Error> {
    save_manifest(io, Path::new(DEFAULT_MANIFEST_PATH), manifest)
}

/// Load a resource manifest from a path using the provided I/O resolver.
///
/// Returns an empty manifest if the path does not exist.
pub fn load_manifest(io: &dyn ResourceIo, path: &Path) -> Result<ResourceManifest, io::Error> {
    let exists = io.file_exists(path).map_err(resource_io_to_io)?;
    if !exists {
        return Ok(ResourceManifest::new());
    }

    let bytes = io.read_file(path).map_err(resource_io_to_io)?;
    let content = String::from_utf8(bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("manifest is not valid UTF-8: {}", e),
        )
    })?;
    ResourceManifest::from_json_str(&content)
}

/// Save a resource manifest to a path using the provided I/O resolver.
pub fn save_manifest(
    io: &dyn ResourceIo,
    path: &Path,
    manifest: &ResourceManifest,
) -> Result<(), io::Error> {
    let content = manifest.to_json_pretty()?;
    io.write_file(path, content.as_bytes())
        .map_err(resource_io_to_io)
}

/// Update the default resource manifest for a managed resource.
///
/// Loads the default manifest, ensures the resource is fresh (creating it if
/// needed), and saves the manifest back to disk.
pub fn update_resource_manifest<R: ManagedResource>(
    resource: &R,
    io: &dyn ResourceIo,
) -> Result<(), ManifestUpdateError> {
    let mut manifest = load_manifest_default(io).map_err(ManifestUpdateError::Load)?;
    resource
        .acquire(ExecMode::Ensure, &mut manifest, io)
        .map_err(ManifestUpdateError::Acquire)?;
    save_manifest_default(io, &manifest).map_err(ManifestUpdateError::Save)?;
    Ok(())
}

fn resource_io_to_io(err: ResourceError) -> io::Error {
    match err {
        ResourceError::Io(inner) => inner,
        other => io::Error::other(other.to_string()),
    }
}

fn update_tagged_str(mut builder: HashBuilder, tag: &str, value: &str) -> HashBuilder {
    builder = builder.update(tag.as_bytes()).update(&[0u8]);
    update_len_prefixed(builder, value)
}

fn update_len_prefixed(mut builder: HashBuilder, value: &str) -> HashBuilder {
    let len = (value.len() as u64).to_le_bytes();
    builder = builder.update(&len);
    builder.update(value.as_bytes())
}

impl ManagedResource for SimpleResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn create(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError> {
        // Simple resources can't be created without a provider
        if self.def.provider.is_none() {
            return Err(ResourceError::NoProvider(self.def.id.clone()));
        }

        // In a real implementation, this would invoke the provider DAG.
        // For now, return a computed entry based on declared inputs.
        let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
        Ok(ManifestEntry::new(key, file_count).with_input_files(input_files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Default)]
    struct TestIo {
        files: RefCell<HashMap<PathBuf, Vec<u8>>>,
        mtimes: RefCell<HashMap<PathBuf, SystemTime>>,
        canonical_paths: RefCell<HashMap<PathBuf, PathBuf>>,
    }

    impl TestIo {
        fn write_text(&self, path: &Path, contents: &str) {
            let _ = self.write_file(path, contents.as_bytes());
        }

        fn add_canonical_alias(&self, alias: &Path, canonical: &Path) {
            self.canonical_paths
                .borrow_mut()
                .insert(alias.to_path_buf(), canonical.to_path_buf());
        }
    }

    impl ResourceIo for TestIo {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, ResourceError> {
            self.files.borrow().get(path).cloned().ok_or_else(|| {
                ResourceError::Io(io::Error::other(format!(
                    "file not found: {}",
                    path.display()
                )))
            })
        }

        fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ResourceError> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_vec());
            self.mtimes
                .borrow_mut()
                .insert(path.to_path_buf(), SystemTime::now());
            Ok(())
        }

        fn file_exists(&self, path: &Path) -> Result<bool, ResourceError> {
            Ok(self.files.borrow().contains_key(path))
        }

        fn glob_paths(&self, pattern: &str) -> Result<Vec<PathBuf>, ResourceError> {
            let pattern = glob::Pattern::new(pattern)
                .map_err(|e| ResourceError::Io(io::Error::other(e.to_string())))?;
            let mut out: Vec<PathBuf> = self
                .files
                .borrow()
                .keys()
                .filter(|path| pattern.matches_path(path))
                .cloned()
                .collect();
            out.sort();
            Ok(out)
        }

        fn command_output(
            &self,
            _command: &str,
            _args: &[String],
        ) -> Result<Vec<u8>, ResourceError> {
            Err(ResourceError::Io(io::Error::other(
                "command output not supported in tests",
            )))
        }

        fn file_mtime(&self, path: &Path) -> Result<SystemTime, ResourceError> {
            self.mtimes.borrow().get(path).cloned().ok_or_else(|| {
                ResourceError::Io(io::Error::other(format!(
                    "mtime not found: {}",
                    path.display()
                )))
            })
        }

        fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, ResourceError> {
            if let Some(canonical) = self.canonical_paths.borrow().get(path) {
                return Ok(canonical.clone());
            }
            if self.files.borrow().contains_key(path) {
                return Ok(path.to_path_buf());
            }
            Err(ResourceError::Io(io::Error::other(format!(
                "canonical path not found: {}",
                path.display()
            ))))
        }
    }

    #[test]
    fn test_resource_error_display() {
        let err = ResourceError::Missing(ResourceId::new("test"));
        assert!(err.to_string().contains("missing"));

        let err = ResourceError::Stale {
            id: ResourceId::new("test"),
            reason: "inputs changed".into(),
        };
        assert!(err.to_string().contains("stale"));
    }

    #[test]
    fn test_simple_resource_no_provider() {
        let def = ResourceDef::new(ResourceId::new("test"));
        let resource = SimpleResource::new(def);
        let io = TestIo::default();

        let result = resource.create(&ResourceManifest::new(), &io);
        assert!(matches!(result, Err(ResourceError::NoProvider(_))));
    }

    #[test]
    fn test_simple_resource_check_state_missing() {
        let def = ResourceDef::new(ResourceId::new("test:missing"));
        let resource = SimpleResource::new(def);
        let manifest = ResourceManifest::new();
        let io = TestIo::default();

        let state = resource.check_state(&manifest, &io);
        assert!(matches!(state, ResourceState::Missing));
    }

    #[test]
    fn test_simple_resource_check_state_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:fresh"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();
        let io = TestIo::default();

        // Add entry with matching key
        let (key, file_count) =
            compute_key_from_def(resource.definition(), &manifest, &io).unwrap();
        manifest.insert(
            ResourceId::new("test:fresh"),
            ManifestEntry::new(key, file_count),
        );

        let state = resource.check_state(&manifest, &io);
        assert!(matches!(state, ResourceState::Fresh));
    }

    #[test]
    fn test_simple_resource_is_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:is_fresh"));
        let resource = SimpleResource::new(def);
        let io = TestIo::default();

        let manifest = ResourceManifest::new();
        assert!(!resource.is_fresh(&manifest, &io)); // Not in manifest

        let mut manifest_with_entry = ResourceManifest::new();
        let (key, file_count) =
            compute_key_from_def(resource.definition(), &manifest_with_entry, &io).unwrap();
        manifest_with_entry.insert(
            ResourceId::new("test:is_fresh"),
            ManifestEntry::new(key, file_count),
        );
        assert!(resource.is_fresh(&manifest_with_entry, &io));
    }

    #[test]
    fn test_acquire_verify_mode_missing() {
        let def = ResourceDef::new(ResourceId::new("test:acquire"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();
        let io = TestIo::default();

        let result = resource.acquire(ExecMode::Verify, &mut manifest, &io);
        assert!(matches!(result, Err(ResourceError::Missing(_))));
    }

    #[test]
    fn test_acquire_verify_mode_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:acquire_fresh"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();
        let io = TestIo::default();

        // Pre-populate manifest
        let (key, file_count) =
            compute_key_from_def(resource.definition(), &manifest, &io).unwrap();
        manifest.insert(
            ResourceId::new("test:acquire_fresh"),
            ManifestEntry::new(key, file_count),
        );

        let result = resource.acquire(ExecMode::Verify, &mut manifest, &io);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_key_missing_dependency() {
        let def = ResourceDef::new(ResourceId::new("test:missing_dep"))
            .with_input(InputPattern::resource(ResourceId::new("dep:missing")));
        let manifest = ResourceManifest::new();
        let io = TestIo::default();

        let err = compute_key_from_def(&def, &manifest, &io).unwrap_err();
        assert!(matches!(err, ResourceError::MissingDependency { .. }));
    }

    #[test]
    fn test_compute_key_changes_with_dependency() {
        let dep_id = ResourceId::new("dep:one");
        let def = ResourceDef::new(ResourceId::new("test:dep_key"))
            .with_input(InputPattern::resource(dep_id.clone()));
        let mut manifest = ResourceManifest::new();
        let io = TestIo::default();

        manifest.insert(
            dep_id.clone(),
            ManifestEntry::new(ContentHash::from_bytes(b"one"), 0),
        );
        let (key1, _) = compute_key_from_def(&def, &manifest, &io).unwrap();

        manifest.insert(
            dep_id.clone(),
            ManifestEntry::new(ContentHash::from_bytes(b"two"), 0),
        );
        let (key2, _) = compute_key_from_def(&def, &manifest, &io).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_compute_key_env_and_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env_key = unique_env_key("GUNBC_TEST_ENV_HASH");

        let io = TestIo::default();
        let input = PathBuf::from("input.txt");
        io.write_text(&input, "alpha");

        let def = ResourceDef::new(ResourceId::new("test:env_file"))
            .with_input(InputPattern::file(&input))
            .with_input(InputPattern::env(&env_key));

        let manifest = ResourceManifest::new();
        let old = std::env::var(&env_key).ok();

        std::env::set_var(&env_key, "A");
        let (key1, file_count1) = compute_key_from_def(&def, &manifest, &io).unwrap();
        assert_eq!(file_count1, 1);

        std::env::set_var(&env_key, "B");
        let (key2, file_count2) = compute_key_from_def(&def, &manifest, &io).unwrap();
        assert_eq!(file_count2, 1);
        assert_ne!(key1, key2);

        io.write_text(&input, "beta");
        let (key3, _) = compute_key_from_def(&def, &manifest, &io).unwrap();
        assert_ne!(key2, key3);

        restore_env(&env_key, old);
    }

    #[test]
    fn test_provider_example_creates_manifest_entry() {
        use super::super::def::{DagRef, ResourceScope};

        #[derive(Clone)]
        struct TestProviderResource {
            def: ResourceDef,
            output: PathBuf,
            contents: String,
        }

        impl ManagedResource for TestProviderResource {
            fn definition(&self) -> &ResourceDef {
                &self.def
            }

            fn create(
                &self,
                manifest: &ResourceManifest,
                io: &dyn ResourceIo,
            ) -> Result<ManifestEntry, ResourceError> {
                io.write_file(&self.output, self.contents.as_bytes())?;
                let (key, file_count) = compute_key_from_def(&self.def, manifest, io)?;
                Ok(ManifestEntry::new(key, file_count).with_outputs(vec![self.output.clone()]))
            }
        }

        let _guard = ENV_LOCK.lock().unwrap();
        let env_key = unique_env_key("GUNBC_TEST_PROVIDER_INPUT");

        let io = TestIo::default();
        let output = PathBuf::from("out.txt");
        let def = ResourceDef::new(ResourceId::new("test:provider"))
            .with_input(InputPattern::env(&env_key))
            .with_output(ResourceScope::file(&output))
            .with_provider(DagRef::new("test_provider"));

        let old = std::env::var(&env_key).ok();
        std::env::set_var(&env_key, "v1");

        let resource = TestProviderResource {
            def,
            output: output.clone(),
            contents: "hello".to_string(),
        };
        let mut manifest = ResourceManifest::new();
        let handle = resource
            .acquire(ExecMode::Ensure, &mut manifest, &io)
            .unwrap();

        assert!(io.file_exists(&output).unwrap());
        let entry = manifest.get(&ResourceId::new("test:provider")).unwrap();
        assert_eq!(entry.outputs, vec![output.clone()]);
        assert_eq!(handle.key(), &entry.key);

        restore_env(&env_key, old);
    }

    #[test]
    fn check_manifest_freshness_surfaces_mtime_fallback_on_fresh_hash() {
        let io = TestIo::default();
        let input = PathBuf::from("mtime_fallback_fresh.txt");
        io.write_text(&input, "alpha");

        let def = ResourceDef::new(ResourceId::new("test:mtime_fallback_fresh"))
            .with_input(InputPattern::file(&input));
        let resource = SimpleResource::new(def.clone());
        let mut manifest = ResourceManifest::new();
        let (key, file_count) = compute_key_from_def(&def, &manifest, &io).unwrap();
        manifest.insert(def.id.clone(), ManifestEntry::new(key, file_count));

        // Force mtime fast-path failure.
        io.mtimes.borrow_mut().remove(&input);

        let freshness = check_manifest_freshness(
            &resource,
            &manifest,
            FreshnessOptions {
                output_exists: Some(true),
                use_mtime: true,
            },
            &io,
        );
        match freshness {
            ManifestFreshness::FreshWithDiagnostic(note) => {
                assert!(
                    note.contains("mtime fast path unavailable"),
                    "diagnostic should include fallback reason: {note}"
                );
            }
            other => panic!("expected FreshWithDiagnostic, got {other:?}"),
        }
    }

    #[test]
    fn check_manifest_freshness_includes_fallback_reason_when_stale() {
        let io = TestIo::default();
        let input = PathBuf::from("mtime_fallback_stale.txt");
        io.write_text(&input, "alpha");

        let def = ResourceDef::new(ResourceId::new("test:mtime_fallback_stale"))
            .with_input(InputPattern::file(&input));
        let resource = SimpleResource::new(def.clone());
        let mut manifest = ResourceManifest::new();
        let (key, file_count) = compute_key_from_def(&def, &manifest, &io).unwrap();
        manifest.insert(def.id.clone(), ManifestEntry::new(key, file_count));

        // Change input and force mtime failure.
        io.write_text(&input, "beta");
        io.mtimes.borrow_mut().remove(&input);

        let freshness = check_manifest_freshness(
            &resource,
            &manifest,
            FreshnessOptions {
                output_exists: Some(true),
                use_mtime: true,
            },
            &io,
        );
        match freshness {
            ManifestFreshness::Stale(reason) => {
                assert!(
                    reason.contains("mtime fast path unavailable"),
                    "stale reason should carry fallback diagnostic: {reason}"
                );
            }
            other => panic!("expected Stale with diagnostic, got {other:?}"),
        }
    }

    #[test]
    fn simple_resource_records_canonical_input_paths_and_stales_same_count_substitution() {
        let io = TestIo::default();
        let canonical_one = PathBuf::from("real/one.rs");
        let canonical_two = PathBuf::from("real/two.rs");
        let alias_one = PathBuf::from("alias/one.rs");
        let alias_two = PathBuf::from("alias/two.rs");

        io.write_text(&canonical_one, "fn one() {}\n");
        io.write_text(&canonical_two, "fn two() {}\n");
        io.add_canonical_alias(&alias_one, &canonical_one);
        io.add_canonical_alias(&alias_two, &canonical_two);

        let def_v1 = ResourceDef::new(ResourceId::new("test:canonical_inputs"))
            .with_input(InputPattern::file(&alias_one))
            .with_provider(super::super::def::DagRef::new("test_provider"));
        let resource_v1 = SimpleResource::new(def_v1.clone());
        let mut manifest = ResourceManifest::new();
        let entry = resource_v1
            .create(&manifest, &io)
            .expect("initial manifest entry should be created");

        assert_eq!(entry.input_file_count, 1);
        assert_eq!(
            entry.input_files,
            Some(vec![canonical_one.to_string_lossy().into_owned()])
        );
        manifest.insert(def_v1.id.clone(), entry);

        let def_v2 = ResourceDef::new(ResourceId::new("test:canonical_inputs"))
            .with_input(InputPattern::file(&alias_two));
        let resource_v2 = SimpleResource::new(def_v2);

        let freshness = check_manifest_freshness(
            &resource_v2,
            &manifest,
            FreshnessOptions {
                output_exists: Some(true),
                use_mtime: true,
            },
            &io,
        );
        match freshness {
            ManifestFreshness::Stale(reason) => {
                assert!(
                    reason.contains("InputFilesChanged"),
                    "same-count path substitution should surface input path change: {reason}"
                );
            }
            other => panic!("expected Stale from same-count path substitution, got {other:?}"),
        }
    }

    fn unique_env_key(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{}_{}", prefix, nanos)
    }

    fn restore_env(key: &str, old: Option<String>) {
        match old {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
