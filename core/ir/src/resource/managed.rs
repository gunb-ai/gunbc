//! Managed resource trait for unified resource acquisition.
//!
//! This module defines the `ManagedResource` trait that unifies tools, build
//! artifacts, and other acquirable resources under a single pattern:
//! Check → Create → Resolve.

use super::def::{InputPattern, ResourceDef};
use super::handle::ResourceHandle;
use super::state::{ExecMode, ResourceState};
use super::{ContentHash, HashBuilder, ManifestEntry, ResourceManifest};
use super::super::ResourceId;
use gunbc_infra::freshness::{check_freshness_mtime, MtimeResult};
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
    MissingDependency { resource: ResourceId, dependency: ResourceId },

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
    fn compute_key(&self, manifest: &ResourceManifest) -> Result<ContentHash, ResourceError> {
        Ok(self.compute_key_with_stats(manifest)?.0)
    }

    /// Compute the freshness key and input file count from declared inputs.
    ///
    /// The default implementation derives both from declared inputs.
    fn compute_key_with_stats(
        &self,
        manifest: &ResourceManifest,
    ) -> Result<(ContentHash, usize), ResourceError> {
        compute_key_from_def(self.definition(), manifest)
    }

    /// Compute the freshness key, file count, and input file paths.
    ///
    /// The default implementation derives all three from declared inputs.
    fn compute_key_with_file_list(
        &self,
        manifest: &ResourceManifest,
    ) -> Result<(ContentHash, usize, Vec<String>), ResourceError> {
        compute_key_with_files(self.definition(), manifest)
    }

    /// Create or regenerate this resource.
    ///
    /// Called when the resource is missing or stale and mode is `Ensure`.
    /// Returns the manifest entry to store.
    fn create(&self, manifest: &ResourceManifest) -> Result<ManifestEntry, ResourceError>;

    /// Get the resource ID.
    fn resource_id(&self) -> &ResourceId {
        &self.definition().id
    }

    /// Check current state against manifest.
    fn check_state(&self, manifest: &ResourceManifest) -> ResourceState {
        let entry = match manifest.get(self.resource_id()) {
            None => return ResourceState::Missing,
            Some(entry) => entry,
        };

        let current_key = match self.compute_key(manifest) {
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
    ) -> Result<ResourceHandle<Self>, ResourceError> {
        let state = self.check_state(manifest);

        match (state, mode) {
            // Fresh in any mode: return handle with current key
            (ResourceState::Fresh, _) => {
                let key = self.compute_key(manifest)?;
                Ok(ResourceHandle::acquire(self.resource_id().clone(), key))
            }

            // Missing/Stale in Ensure mode: create, update manifest, return handle
            (ResourceState::Missing | ResourceState::Stale { .. }, ExecMode::Ensure) => {
                let entry = self.create(manifest)?;
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
    fn is_fresh(&self, manifest: &ResourceManifest) -> bool {
        self.check_state(manifest).is_fresh()
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
) -> Result<(ContentHash, usize), ResourceError> {
    let (hash, count, _files) = compute_key_with_files(def, manifest)?;
    Ok((hash, count))
}

/// Compute a resource key from its declared inputs, including the list of
/// input file paths.
///
/// Returns `(key, input_file_count, input_files)`.
pub fn compute_key_with_files(
    def: &ResourceDef,
    manifest: &ResourceManifest,
) -> Result<(ContentHash, usize, Vec<String>), ResourceError> {
    let mut builder = HashBuilder::new();
    let mut file_count: usize = 0;
    let mut file_paths: Vec<String> = Vec::new();

    for input in &def.inputs {
        match input {
            InputPattern::Glob(pattern) => {
                // Tag the input kind, then delegate to the glob helper (sorted).
                builder = builder.update(b"glob\0");
                let (next, count, paths) = builder.update_glob_with_paths(pattern)?;
                builder = next;
                file_count += count;
                file_paths.extend(paths);
            }
            InputPattern::File(path) => {
                builder = builder.update(b"file\0");
                builder = builder.update_file(path)?;
                file_count += 1;
                file_paths.push(path.to_string_lossy().to_string());
            }
            InputPattern::Env(var) => {
                let value = std::env::var(var).unwrap_or_default();
                builder = update_tagged_str(builder, "env", var);
                builder = update_len_prefixed(builder, &value);
            }
            InputPattern::CommandOutput { command, args } => {
                builder = builder.update(b"cmd\0");
                builder = builder.update_command_output(command, args)?;
            }
            InputPattern::Resource(dep_id) => {
                let entry = manifest.get(dep_id).ok_or_else(|| {
                    ResourceError::MissingDependency {
                        resource: def.id.clone(),
                        dependency: dep_id.clone(),
                    }
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
    files: Vec<String>,
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
            InputPattern::File(path) => files.push(path.to_string_lossy().to_string()),
            InputPattern::Env(_) | InputPattern::Resource(_) | InputPattern::CommandOutput { .. } => {
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
            let glob_refs: Vec<&str> = mtime_inputs
                .glob_patterns
                .iter()
                .map(|s| s.as_str())
                .collect();
            let file_refs: Vec<&str> = mtime_inputs.files.iter().map(|s| s.as_str()).collect();
            match check_freshness_mtime(entry, &glob_refs, &file_refs) {
                MtimeResult::Fresh => return ManifestFreshness::Fresh,
                MtimeResult::MaybeStale(_) => {
                    // Fall through to full hash comparison.
                }
            }
        }
    }

    let current_key = match resource.compute_key(manifest) {
        Ok(k) => k,
        Err(e) => return ManifestFreshness::Error(e.to_string()),
    };

    if entry.key == current_key {
        ManifestFreshness::Fresh
    } else {
        ManifestFreshness::Stale("inputs changed since last update".into())
    }
}

/// Update the default resource manifest for a managed resource.
///
/// Loads the default manifest, ensures the resource is fresh (creating it if
/// needed), and saves the manifest back to disk.
pub fn update_resource_manifest<R: ManagedResource>(
    resource: &R,
) -> Result<(), ManifestUpdateError> {
    let mut manifest = ResourceManifest::load_default().map_err(ManifestUpdateError::Load)?;
    resource
        .acquire(ExecMode::Ensure, &mut manifest)
        .map_err(ManifestUpdateError::Acquire)?;
    manifest
        .save_default()
        .map_err(ManifestUpdateError::Save)?;
    Ok(())
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

    fn create(&self, manifest: &ResourceManifest) -> Result<ManifestEntry, ResourceError> {
        // Simple resources can't be created without a provider
        if self.def.provider.is_none() {
            return Err(ResourceError::NoProvider(self.def.id.clone()));
        }

        // In a real implementation, this would invoke the provider DAG.
        // For now, return a computed entry based on declared inputs.
        let (key, file_count) = compute_key_from_def(&self.def, manifest)?;
        Ok(ManifestEntry::new(key, file_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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

        let result = resource.create(&ResourceManifest::new());
        assert!(matches!(result, Err(ResourceError::NoProvider(_))));
    }

    #[test]
    fn test_simple_resource_check_state_missing() {
        let def = ResourceDef::new(ResourceId::new("test:missing"));
        let resource = SimpleResource::new(def);
        let manifest = ResourceManifest::new();

        let state = resource.check_state(&manifest);
        assert!(matches!(state, ResourceState::Missing));
    }

    #[test]
    fn test_simple_resource_check_state_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:fresh"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();

        // Add entry with matching key
        let (key, file_count) = compute_key_from_def(resource.definition(), &manifest).unwrap();
        manifest.insert(
            ResourceId::new("test:fresh"),
            ManifestEntry::new(key, file_count),
        );

        let state = resource.check_state(&manifest);
        assert!(matches!(state, ResourceState::Fresh));
    }

    #[test]
    fn test_simple_resource_is_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:is_fresh"));
        let resource = SimpleResource::new(def);

        let manifest = ResourceManifest::new();
        assert!(!resource.is_fresh(&manifest)); // Not in manifest

        let mut manifest_with_entry = ResourceManifest::new();
        let (key, file_count) =
            compute_key_from_def(resource.definition(), &manifest_with_entry).unwrap();
        manifest_with_entry.insert(
            ResourceId::new("test:is_fresh"),
            ManifestEntry::new(key, file_count),
        );
        assert!(resource.is_fresh(&manifest_with_entry));
    }

    #[test]
    fn test_acquire_verify_mode_missing() {
        let def = ResourceDef::new(ResourceId::new("test:acquire"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();

        let result = resource.acquire(ExecMode::Verify, &mut manifest);
        assert!(matches!(result, Err(ResourceError::Missing(_))));
    }

    #[test]
    fn test_acquire_verify_mode_fresh() {
        let def = ResourceDef::new(ResourceId::new("test:acquire_fresh"));
        let resource = SimpleResource::new(def);
        let mut manifest = ResourceManifest::new();

        // Pre-populate manifest
        let (key, file_count) = compute_key_from_def(resource.definition(), &manifest).unwrap();
        manifest.insert(
            ResourceId::new("test:acquire_fresh"),
            ManifestEntry::new(key, file_count),
        );

        let result = resource.acquire(ExecMode::Verify, &mut manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_key_missing_dependency() {
        let def = ResourceDef::new(ResourceId::new("test:missing_dep"))
            .with_input(InputPattern::resource(ResourceId::new("dep:missing")));
        let manifest = ResourceManifest::new();

        let err = compute_key_from_def(&def, &manifest).unwrap_err();
        assert!(matches!(err, ResourceError::MissingDependency { .. }));
    }

    #[test]
    fn test_compute_key_changes_with_dependency() {
        let dep_id = ResourceId::new("dep:one");
        let def = ResourceDef::new(ResourceId::new("test:dep_key"))
            .with_input(InputPattern::resource(dep_id.clone()));
        let mut manifest = ResourceManifest::new();

        manifest.insert(dep_id.clone(), ManifestEntry::new(ContentHash::from_bytes(b"one"), 0));
        let (key1, _) = compute_key_from_def(&def, &manifest).unwrap();

        manifest.insert(dep_id.clone(), ManifestEntry::new(ContentHash::from_bytes(b"two"), 0));
        let (key2, _) = compute_key_from_def(&def, &manifest).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_compute_key_env_and_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let env_key = unique_env_key("GUNBC_TEST_ENV_HASH");

        let dir = temp_dir("env_file");
        let input = dir.join("input.txt");
        write_file(&input, "alpha");

        let def = ResourceDef::new(ResourceId::new("test:env_file"))
            .with_input(InputPattern::file(&input))
            .with_input(InputPattern::env(&env_key));

        let manifest = ResourceManifest::new();
        let old = std::env::var(&env_key).ok();

        std::env::set_var(&env_key, "A");
        let (key1, file_count1) = compute_key_from_def(&def, &manifest).unwrap();
        assert_eq!(file_count1, 1);

        std::env::set_var(&env_key, "B");
        let (key2, file_count2) = compute_key_from_def(&def, &manifest).unwrap();
        assert_eq!(file_count2, 1);
        assert_ne!(key1, key2);

        write_file(&input, "beta");
        let (key3, _) = compute_key_from_def(&def, &manifest).unwrap();
        assert_ne!(key2, key3);

        restore_env(&env_key, old);
        cleanup_dir(&dir);
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

            fn create(&self, manifest: &ResourceManifest) -> Result<ManifestEntry, ResourceError> {
                gunbc_infra::test_utils::write_file(&self.output, &self.contents);
                let (key, file_count) = compute_key_from_def(&self.def, manifest)?;
                Ok(ManifestEntry::new(key, file_count).with_outputs(vec![self.output.clone()]))
            }
        }

        let _guard = ENV_LOCK.lock().unwrap();
        let env_key = unique_env_key("GUNBC_TEST_PROVIDER_INPUT");

        let dir = temp_dir("provider");
        let output = dir.join("out.txt");
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
        let handle = resource.acquire(ExecMode::Ensure, &mut manifest).unwrap();

        assert!(output.exists());
        let entry = manifest.get(&ResourceId::new("test:provider")).unwrap();
        assert_eq!(entry.outputs, vec![output.clone()]);
        assert_eq!(handle.key(), &entry.key);

        restore_env(&env_key, old);
        cleanup_dir(&dir);
    }

    fn unique_env_key(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{}_{}", prefix, nanos)
    }

    fn temp_dir(label: &str) -> PathBuf {
        gunbc_infra::test_utils::temp_dir(label)
    }

    fn write_file(path: &Path, contents: &str) {
        gunbc_infra::test_utils::write_file(path, contents);
    }

    fn cleanup_dir(path: &Path) {
        gunbc_infra::test_utils::cleanup_dir(path);
    }

    fn restore_env(key: &str, old: Option<String>) {
        match old {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
