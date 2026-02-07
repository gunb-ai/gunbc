//! Composition test helpers.
//!
//! Composition tests verify that edges between nodes have compatible types.

use gunbc_ir::{Dag, TypeId, TypeRegistry};

/// Result of type compatibility check.
#[derive(Debug)]
pub struct TypeCompatibility {
    /// Whether the types are compatible
    pub compatible: bool,
    /// Source type
    pub from_type: TypeId,
    /// Target type
    pub to_type: TypeId,
    /// Edge description
    pub edge: String,
}

impl TypeCompatibility {
    /// Check if types are compatible.
    pub fn is_compatible(&self) -> bool {
        self.compatible
    }
}

/// Check if two types are compatible.
///
/// For now, types are compatible if they are equal.
/// This could be extended to support subtyping or coercion.
pub fn types_compatible(from: &TypeId, to: &TypeId) -> bool {
    TypeRegistry::with_core_types().is_compatible(from, to)
}

/// Assert that all edges in a DAG have compatible types.
///
/// Returns a list of compatibility results for each edge.
pub fn assert_types_compatible<T>(dag: &Dag<T>) -> Vec<TypeCompatibility> {
    let mut results = Vec::new();

    for edge in &dag.edges {
        // Find the source node and port
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        let (from_type, to_type) = match (from_node, to_node) {
            (Some(from), Some(to)) => {
                let from_port = from.outputs.iter().find(|p| p.name == edge.from_port);
                let to_port = to.inputs.iter().find(|p| p.name == edge.to_port);

                match (from_port, to_port) {
                    (Some(fp), Some(tp)) => (fp.type_id.clone(), tp.type_id.clone()),
                    _ => continue, // Skip if ports not found
                }
            }
            _ => continue, // Skip if nodes not found
        };

        let compatible = types_compatible(&from_type, &to_type);

        results.push(TypeCompatibility {
            compatible,
            from_type,
            to_type,
            edge: format!(
                "{}.{} -> {}.{}",
                edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
            ),
        });
    }

    results
}

/// Check that all edges are compatible, panicking if not.
pub fn verify_all_edges_compatible<T>(dag: &Dag<T>) {
    let results = assert_types_compatible(dag);
    let mut failed = Vec::new();

    for result in &results {
        if !result.is_compatible() {
            failed.push(format!(
                "{}: {} -> {} (incompatible)",
                result.edge, result.from_type, result.to_type
            ));
        }
    }

    if !failed.is_empty() {
        panic!(
            "Type compatibility check failed for {} edge(s):\n  {}",
            failed.len(),
            failed.join("\n  ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;
    use gunbc_ir::{Dag, Node};

    #[test]
    fn test_compatible_types() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "String")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let results = assert_types_compatible(&dag);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_compatible());
    }

    #[test]
    fn test_incompatible_types() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "Int")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let results = assert_types_compatible(&dag);
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_compatible());
    }

    #[test]
    fn test_any_type_compatible() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "Any")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let results = assert_types_compatible(&dag);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_compatible());
    }
}
