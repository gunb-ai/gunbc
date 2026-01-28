pub mod algebra;
pub mod build;
pub mod dag;
pub mod node;
pub mod transport;
pub mod types;
pub mod viz;

// Re-export core types at crate root for convenience.
pub use algebra::{Amount, ExclusionMode, Predicate, ResourceClaim, ResourceId, SetSpec, Value};
pub use build::{edge, eq_guarded_port, guarded_port, neq_guarded_port, port};
pub use dag::{BoundaryDeclaration, Dag, DagMetadata, Edge, PatternDecisionEntry, Port};
pub use node::{Node, NodeBody};
pub use types::*;
