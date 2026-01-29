//! gunbc-makegen: Makefile generation from DAG entrypoints.
//!
//! This crate generates Makefile targets from gunbc tool entrypoints.
//! Entrypoints are inputs with no upstream edge — they come from the world
//! and become Make variables.
//!
//! # Example Generated Makefile
//!
//! ```makefile
//! # gunbc-gist entrypoints: repo_path (String)
//! gist:
//!     @cargo run -p gunbc-gist -- $(if $(REPO),--repo $(REPO))
//! ```

pub mod ops;
pub mod graph;
pub mod registry;
pub mod render;

pub use graph::build_makegen_graph;
pub use ops::MakegenOp;
pub use registry::{ToolInfo, ToolRegistry, EntrypointParam};
pub use render::render_makefile;
