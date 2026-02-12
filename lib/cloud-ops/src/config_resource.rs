//! Cloud config as a ManagedResource.
//!
//! Declares `CloudConfigResource` which implements the `ManagedResource` trait.
//! Its inputs are:
//! - `InputPattern::File` for the generated TOML config
//! - `InputPattern::Env("GITHUB_ACTIONS")` for runtime detection
//! - `InputPattern::Env("GCE_METADATA_HOST")` for metadata server detection
//!
//! When the config is missing or stale, the provider DAG (infra discovery)
//! regenerates it by querying GCP APIs.

use gunbc_ir::resource::def::{DagRef, InputPattern, ResourceDef, ResourceScope};
use gunbc_ir::resource::managed::{ManagedResource, ResourceError, ResourceIo};
use gunbc_ir::resource::{ManifestEntry, ResourceManifest};
use gunbc_ir::ResourceId;
use std::path::PathBuf;

/// Cloud config resource for a specific deployment.
///
/// Tracks a generated TOML config file with freshness checking.
/// When stale, the infra discovery DAG regenerates it.
#[derive(Debug, Clone)]
pub struct CloudConfigResource {
    def: ResourceDef,
    /// Path to the generated TOML config file.
    config_path: PathBuf,
    /// Deployment name (e.g., "dev", "prod", "test", "local").
    deployment: String,
}

impl CloudConfigResource {
    /// Create a new cloud config resource for the given deployment.
    ///
    /// The config file is stored at `.gunbc/config-{deployment}.toml`.
    pub fn new(deployment: &str) -> Self {
        let config_path = PathBuf::from(format!(".gunbc/config-{}.toml", deployment));
        let def = ResourceDef::new(ResourceId::new(format!("config:cloud:{}", deployment)))
            .with_input(InputPattern::file(&config_path))
            // Runtime detection signals (cheap env reads for freshness key)
            .with_input(InputPattern::env("GITHUB_ACTIONS"))
            .with_input(InputPattern::env("GCE_METADATA_HOST"))
            .with_output(ResourceScope::named("CloudSecretConfig"))
            .with_provider(DagRef::new("infra-discover"));

        Self {
            def,
            config_path,
            deployment: deployment.to_string(),
        }
    }

    /// Get the path to the config file.
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get the deployment name.
    pub fn deployment(&self) -> &str {
        &self.deployment
    }
}

impl ManagedResource for CloudConfigResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn create(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError> {
        // In a full implementation, this would invoke the infra discovery DAG.
        // For now, compute the key from current inputs and record it.
        // The actual DAG invocation happens at the orchestrator level.
        let (key, file_count) = self.compute_key_with_stats(manifest, io)?;
        Ok(ManifestEntry::new(key, file_count).with_outputs(vec![self.config_path.clone()]))
    }
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Create a CloudConfigResource for the "dev" deployment.
pub fn dev_config_resource() -> CloudConfigResource {
    CloudConfigResource::new("dev")
}

/// Create a CloudConfigResource for the "prod" deployment.
pub fn prod_config_resource() -> CloudConfigResource {
    CloudConfigResource::new("prod")
}

/// Create a CloudConfigResource for the "test" deployment.
pub fn test_config_resource() -> CloudConfigResource {
    CloudConfigResource::new("test")
}

/// Create a CloudConfigResource for the "local" deployment.
pub fn local_config_resource() -> CloudConfigResource {
    CloudConfigResource::new("local")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::time::SystemTime;

    #[derive(Default)]
    struct TestIo {
        files: RefCell<HashMap<PathBuf, Vec<u8>>>,
    }

    impl ResourceIo for TestIo {
        fn read_file(&self, path: &std::path::Path) -> Result<Vec<u8>, ResourceError> {
            self.files.borrow().get(path).cloned().ok_or_else(|| {
                ResourceError::Io(std::io::Error::other(format!(
                    "file not found: {}",
                    path.display()
                )))
            })
        }

        fn write_file(&self, path: &std::path::Path, contents: &[u8]) -> Result<(), ResourceError> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_vec());
            Ok(())
        }

        fn file_exists(&self, path: &std::path::Path) -> Result<bool, ResourceError> {
            Ok(self.files.borrow().contains_key(path))
        }

        fn glob_paths(&self, _pattern: &str) -> Result<Vec<PathBuf>, ResourceError> {
            Ok(vec![])
        }

        fn command_output(
            &self,
            _command: &str,
            _args: &[String],
        ) -> Result<Vec<u8>, ResourceError> {
            Err(ResourceError::Io(std::io::Error::other("not supported")))
        }

        fn file_mtime(&self, _path: &std::path::Path) -> Result<SystemTime, ResourceError> {
            Ok(SystemTime::now())
        }
    }

    #[test]
    fn test_cloud_config_resource_definition() {
        let resource = CloudConfigResource::new("dev");
        let def = resource.definition();
        assert_eq!(def.id.0, "config:cloud:dev");
        assert!(def.has_provider());
        // Should have 3 inputs: file, GITHUB_ACTIONS env, GCE_METADATA_HOST env
        assert_eq!(def.inputs.len(), 3);
    }

    #[test]
    fn test_cloud_config_resource_config_path() {
        let resource = CloudConfigResource::new("prod");
        assert_eq!(
            resource.config_path(),
            &PathBuf::from(".gunbc/config-prod.toml")
        );
    }

    #[test]
    fn test_cloud_config_resource_missing_state() {
        let resource = CloudConfigResource::new("test");
        let manifest = ResourceManifest::new();
        let io = TestIo::default();
        let state = resource.check_state(&manifest, &io);
        assert!(matches!(
            state,
            gunbc_ir::resource::state::ResourceState::Missing
                | gunbc_ir::resource::state::ResourceState::Error(_)
        ));
    }
}
