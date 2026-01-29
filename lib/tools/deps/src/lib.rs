//! gunbc-deps: Tool dependency management with upsert pattern.
//!
//! This crate provides:
//! - Declarative tool dependency specification via `deps.toml`
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

pub mod graph;
pub mod installer;
pub mod manifest;
pub mod ops;
pub mod platform;
pub mod upsert;

pub use graph::build_deps_graph;
pub use installer::{InstallMethod, Installer};
pub use manifest::{Dependency, DepsManifest, PlatformInstall};
pub use ops::DepsOp;
pub use platform::Platform;
pub use upsert::{UpsertPhase, UpsertResult};
