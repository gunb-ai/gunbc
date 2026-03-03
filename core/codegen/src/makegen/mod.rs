//! Makegen infrastructure: tool registry, target model, and renderers.
//!
//! Relocated from `gunbc-dag/src/makegen/` — these modules are generic
//! build-system infrastructure that does not depend on repo-specific code.

pub mod gitignore;
pub mod justfile;
pub mod model;
pub mod registry;
pub mod shared;
