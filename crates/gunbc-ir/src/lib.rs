pub mod types;
pub mod metadata;
pub mod node;
pub mod dag;
pub mod viz;
pub mod build;

// Re-export core types at crate root for convenience.
pub use dag::{Dag, DagMetadata, Edge, PatternDecisionEntry, Port};
pub use metadata::NodeMetadata;
pub use node::{Node, NodeBody};
pub use types::*;
pub use build::{port, guarded_port, edge, node_meta};
