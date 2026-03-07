#![recursion_limit = "1024"]
//! gunbc-deps: Tool dependency management with upsert pattern.
//!
//! This crate provides:
//! - Declarative tool dependency specification via `deps.toml`
//! - deps.toml generation from tool registry (owns the file)
//! - Platform-agnostic installation (apt, brew, cargo, script, etc.)
//! - Idempotent upsert pattern: Check → Create → Resolve
//!
//! # Example deps.toml
//!
//! ```toml
//! [[dependency]]
//! name = "gh"
//! verify = "gh --version"
//!
//! [dependency.install.linux]
//! method = "apt"
//! packages = ["gh"]
//!
//! [dependency.install.macos]
//! method = "brew"
//! packages = ["gh"]
//! ```
//!
//! # Generated File Ownership
//!
//! The `deps.toml` generation is handled via the DSL (dsl/tools/deps.dag).
//! The filename is centralized in `DEFAULT_MANIFEST_FILENAME`.
//!
#![deny(dead_code)]
pub mod env;
pub mod installer;
pub mod manifest;
pub mod ops;
pub mod package_manager;
pub mod platform;
pub mod tool_upsert;
pub mod upsert;

pub use env::{strict_dry_run_enabled, PlatformEnv, STRICT_DRY_RUN_ENV};
pub use installer::{InstallMethod, InstallPlan, Installer};
pub use manifest::{
    Dependency, DepsManifest, ManifestConfig, PlatformInstall, DEFAULT_MANIFEST_FILENAME,
    MANIFEST_CONFIG,
};
pub use ops::DepsOp;
pub use package_manager::PackageManagerId;
pub use platform::Platform;
pub use tool_upsert::{
    find_install_option, find_install_option_with_policy, generate_deps_toml,
    generate_deps_toml_from_registry, generate_tool_deps_entry, generate_tool_idempotent_script,
    generate_tool_install_cmd, install_inputs_to_platform_install, tool_to_platform_install,
    InstallSelectionPolicy,
};
pub use upsert::{UpsertPhase, UpsertResult};

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, OnceLock};

    pub(crate) fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = std::panic::catch_unwind(f);
        std::env::remove_var(crate::STRICT_DRY_RUN_ENV);
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }
}
