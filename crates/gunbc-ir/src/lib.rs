pub mod types;
pub mod metadata;
pub mod node;
pub mod dag;
pub mod viz;

// Re-export core types at crate root for convenience.
pub use dag::{Dag, DagMetadata, Edge, PatternDecisionEntry, Port};
pub use metadata::NodeMetadata;
pub use node::{Node, NodeBody};
pub use types::*;
