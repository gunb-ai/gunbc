//! gunbc-viz: Interactive DAG visualizer.
//!
//! This crate provides:
//! - JSON export of DAG structures for visualization
//! - Generic DAG discovery from all tool crates
//! - An interactive HTML/JS visualizer using Cytoscape.js
//! - Support for fractal/nested DAGs (compound nodes)
//!
//! Pipeline:
//! ```text
//! CollectDags -> ExportJson -> PrepareFileWrite -> ExecuteTransport
//!    (viz)        (viz)            (fs)             (transport)
//! ```

pub mod discover;
pub mod export;
pub mod graph;
pub mod ops;

pub use discover::discover_all_dags;
pub use export::{export_dag, VizCollection, VizGraph};
pub use graph::{build_viz_graph, VizGraphOp};
pub use ops::VizOp;
