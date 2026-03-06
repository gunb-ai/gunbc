//! Testgen DAG module: mock interpretation and profile scanning.
//!
//! Relocated from `gunbc-app/src/testgen_dag/` to decouple generic testgen
//! infrastructure from the repo-specific crate.
//!
//! DAG test discovery (`dag_test_discovery.rs`), graph builder (`graph.rs`),
//! and runtime ops (`ops.rs`) remain in `gunbc-app` because they depend on
//! `gunbc-testgen-registry` (which depends on this crate, creating a cycle
//! if included here).

pub mod mock_interpreter;
pub mod profile_discovery;

pub use profile_discovery::{discover_profiles, profiles_for_module, DiscoveredProfile};
