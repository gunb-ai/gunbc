//! Managed resource trait for unified resource acquisition.
//!
//! This module defines the `ManagedResource` trait that unifies tools, build
//! artifacts, and other acquirable resources under a single pattern:
//! Check → Create → Resolve.

use super::def::ResourceDef;
use super::handle::ResourceHandle;
use super::{ContentHash, ManifestEntry, ResourceManifest};
use super::state::{ExecMode, ResourceState};
use super::super::ResourceId;
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

/// A resource that can be acquired with freshness checking.
///
/// This trait unifies tools, build artifacts, and other acquirable things.
/// All follow the same pattern: Check → Create → Resolve.
///
/// # Implementation Notes
///
/// - `definition()` returns the resource's inputs/outputs declaration
/// - `compute_key()` derives the freshness key from declared inputs
/// - `check_state()` compares computed key to manifest entry
/// - `create()` runs the provider to create/regenerate the resource
///
/// The trait provides default implementations for most methods; implementations
/// typically only need to provide `definition()` and `create()`.
pub trait ManagedResource: Clone + Sized {
    /// Get the resource definition (inputs, outputs, provider).
    fn definition(&self) -> &ResourceDef;

    /// Compute the current freshness key from declared inputs.
    ///
    /// The default implementation requires an external hash function.
    /// Implementations may override this if they have custom key computation.
    fn compute_key(&self) -> Result<ContentHash, ResourceError>;

    /// Create or regenerate this resource.
    ///
    /// Called when the resource is missing or stale and mode is `Ensure`.
    /// Returns the manifest entry to store.
    fn create(&self) -> Result<ManifestEntry, ResourceError>;

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

        let current_key = match self.compute_key() {
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
                let key = self.compute_key()?;
                Ok(ResourceHandle::acquire(self.resource_id().clone(), key))
            }

            // Missing/Stale in Ensure mode: create, update manifest, return handle
            (ResourceState::Missing | ResourceState::Stale { .. }, ExecMode::Ensure) => {
                let entry = self.create()?;
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

impl ManagedResource for SimpleResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn compute_key(&self) -> Result<ContentHash, ResourceError> {
        // For simple resources, this needs to be implemented based on the definition
        // In a real implementation, this would use HashBuilder to process InputPatterns
        // For now, return an empty hash as a placeholder
        Ok(ContentHash::empty())
    }

    fn create(&self) -> Result<ManifestEntry, ResourceError> {
        // Simple resources can't be created without a provider
        if self.def.provider.is_none() {
            return Err(ResourceError::NoProvider(self.def.id.clone()));
        }

        // In a real implementation, this would invoke the provider DAG
        // For now, return a placeholder entry
        Ok(ManifestEntry::new(ContentHash::empty(), 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = resource.create();
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

        // Add entry with matching key (empty, since compute_key returns empty)
        manifest.insert(
            ResourceId::new("test:fresh"),
            ManifestEntry::new(ContentHash::empty(), 0),
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
        manifest_with_entry.insert(
            ResourceId::new("test:is_fresh"),
            ManifestEntry::new(ContentHash::empty(), 0),
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
        manifest.insert(
            ResourceId::new("test:acquire_fresh"),
            ManifestEntry::new(ContentHash::empty(), 0),
        );

        let result = resource.acquire(ExecMode::Verify, &mut manifest);
        assert!(result.is_ok());
    }
}
