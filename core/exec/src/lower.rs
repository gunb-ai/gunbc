//! Lowering: flatten sub-DAGs into a single flat DAG.
//!
//! Loop patterns (LoopBuilder) are detected during lowering. The body SubDag
//! is **not** flattened — instead it is preserved as a template in [`LoopInfo`].
//! The executor iterates the body once per element at runtime.

use gunbc_ir::{
    detect_boundaries, detect_entrypoints, Dag, Edge, Node, NodeBody, NodeId, PortName,
};
use std::collections::HashMap;
use thiserror::Error;

/// Error during lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("node '{0}' has SubDag with no export_node defined")]
    NoExportNode(String),
    #[error("SubDag '{node}' has no inner entrypoint for input port '{port}'")]
    NoInnerEntrypoint { node: String, port: String },
    #[error("SubDag '{node}' has no inner boundary for output port '{port}'")]
    NoInnerBoundary { node: String, port: String },
}

/// Result of lowering a DAG.
#[derive(Debug, Clone)]
pub struct LowerResult<T> {
    /// The flat DAG with all SubDags flattened (except loop bodies).
    pub dag: Dag<T>,
    /// Loop patterns detected during lowering.
    /// Each entry describes one loop: the unpack/pack node IDs and
    /// the body template DAG to be iterated at execution time.
    pub loops: Vec<LoopInfo<T>>,
    /// Input mock remappings: maps (original SubDag ID, port) to
    /// the lowered inner entrypoint (node_id, port) pairs.
    /// Used by the executor to remap BoundaryMocks keys.
    pub input_remaps: HashMap<(String, String), Vec<(String, String)>>,
}

/// Describes a loop pattern detected during lowering.
///
/// The body template DAG is executed once per element by the executor.
/// It receives a single element value on its element input port and
/// produces a single result value on its "result" output port.
#[derive(Debug, Clone)]
pub struct LoopInfo<T> {
    /// Prefixed ID of the unpack node in the flat DAG.
    pub unpack_id: NodeId,
    /// Prefixed ID of the pack node in the flat DAG.
    pub pack_id: NodeId,
    /// Port name for the element output from unpack / input to body.
    pub element_port: String,
    /// The body template DAG (not lowered — will be lowered per iteration).
    pub body_dag: Dag<T>,
    /// Additional input port names that flow from outside the loop into
    /// the body (e.g., `repo_path`). These are wired as extra entrypoints
    /// in the body DAG that share a port name with the unpack node's inputs.
    pub extra_input_ports: Vec<String>,
}

/// Mapping info for a SubDag's ports to its inner nodes.
struct SubDagMapping {
    /// Maps parent input port name -> list of (inner_node_id, inner_port_name)
    input_mappings: HashMap<PortName, Vec<(NodeId, PortName)>>,
    /// Maps parent output port name -> (inner_node_id, inner_port_name)
    output_mappings: HashMap<PortName, (NodeId, PortName)>,
}

/// Build the port mapping for a SubDag node.
fn build_subdag_mapping<T>(
    parent_node: &Node<T>,
    inner_dag: &Dag<T>,
    parent_prefix: &str,
) -> Result<SubDagMapping, LowerError> {
    let entrypoints = detect_entrypoints(inner_dag);
    let boundaries = detect_boundaries(inner_dag);

    let mut input_mappings: HashMap<PortName, Vec<(NodeId, PortName)>> = HashMap::new();
    let mut output_mappings: HashMap<PortName, (NodeId, PortName)> = HashMap::new();

    // Map parent input ports to inner entrypoints by matching port names
    for parent_port in &parent_node.inputs {
        let mut targets = Vec::new();
        for (inner_node_id, inner_port_name, _type_id) in &entrypoints.entrypoint_ports {
            if inner_port_name == &parent_port.name {
                // Prefix the inner node ID
                let prefixed_id = NodeId::new(format!("{}/{}", parent_prefix, inner_node_id.0));
                targets.push((prefixed_id, inner_port_name.clone()));
            }
        }
        if targets.is_empty() {
            return Err(LowerError::NoInnerEntrypoint {
                node: parent_prefix.to_string(),
                port: parent_port.name.0.clone(),
            });
        }
        input_mappings.insert(parent_port.name.clone(), targets);
    }

    // Map parent output ports to inner boundaries by matching port names
    for parent_port in &parent_node.outputs {
        let mut found = None;
        for (inner_node_id, inner_port_name) in &boundaries.boundary_ports {
            if inner_port_name == &parent_port.name {
                let prefixed_id = NodeId::new(format!("{}/{}", parent_prefix, inner_node_id.0));
                found = Some((prefixed_id, inner_port_name.clone()));
                break;
            }
        }
        match found {
            Some(mapping) => {
                output_mappings.insert(parent_port.name.clone(), mapping);
            }
            None => {
                return Err(LowerError::NoInnerBoundary {
                    node: parent_prefix.to_string(),
                    port: parent_port.name.0.clone(),
                });
            }
        }
    }

    Ok(SubDagMapping {
        input_mappings,
        output_mappings,
    })
}

/// Detect whether a SubDag's inner DAG follows the loop pattern:
/// nodes "unpack" (Opaque), "body" (SubDag), "pack" (Opaque).
fn detect_loop_pattern<T>(inner_dag: &Dag<T>) -> Option<LoopPatternInfo<T>>
where
    T: Clone,
{
    if inner_dag.nodes.len() != 3 {
        return None;
    }

    let unpack = inner_dag.nodes.iter().find(|n| n.id.0 == "unpack")?;
    let body = inner_dag.nodes.iter().find(|n| n.id.0 == "body")?;
    let pack = inner_dag.nodes.iter().find(|n| n.id.0 == "pack")?;

    // Unpack and pack must be opaque; body must be a SubDag
    if !matches!(&unpack.body, NodeBody::Opaque(_)) {
        return None;
    }
    if !body.is_subdag() {
        return None;
    }
    if !matches!(&pack.body, NodeBody::Opaque(_)) {
        return None;
    }

    // Find the element port: the edge from unpack → body (not "count")
    let element_edge = inner_dag.edges.iter().find(|e| {
        e.from_node.0 == "unpack" && e.to_node.0 == "body" && e.from_port.0 != "count"
    })?;
    let element_port = element_edge.from_port.0.clone();

    // Extract body SubDag
    let body_dag = match &body.body {
        NodeBody::SubDag(dag) => dag.clone(),
        _ => return None,
    };

    // Detect extra input ports: unpack's inputs that are also entrypoints
    // in the body DAG (e.g., repo_path flows through unpack into body).
    // We identify these as unpack input ports that are NOT the main element
    // input (i.e., not the port with the same name as the element port's
    // source on the unpack's input side).
    let extra_input_ports = vec![];

    Some(LoopPatternInfo {
        element_port,
        body_dag,
        extra_input_ports,
    })
}

/// Internal result from loop pattern detection.
struct LoopPatternInfo<T> {
    element_port: String,
    body_dag: Dag<T>,
    extra_input_ports: Vec<String>,
}

/// Lower a DAG by flattening all SubDag nodes into Opaque nodes.
///
/// After lowering, the DAG contains only Opaque nodes and can be executed.
/// Node IDs are prefixed with the parent's ID (e.g., "parent/child").
///
/// Loop patterns (unpack → body SubDag → pack) are detected and the body
/// is preserved as a template in the returned [`LowerResult::loops`].
///
/// ## SubDag Boundary Wiring
///
/// When flattening a SubDag:
/// - Edges INTO the SubDag parent are rewired to inner entrypoint nodes (by port name)
/// - Edges FROM the SubDag parent are rewired from inner boundary nodes (by port name)
/// - A single parent input may fan out to multiple inner entrypoints with the same name
pub fn lower<T: Clone>(dag: &Dag<T>) -> Result<LowerResult<T>, LowerError> {
    let mut result = Dag::new();
    let mut subdag_mappings: HashMap<NodeId, SubDagMapping> = HashMap::new();
    let mut loops = Vec::new();

    // First pass: collect nodes and build SubDag mappings
    for node in &dag.nodes {
        match &node.body {
            NodeBody::Opaque(_) => {
                // Opaque nodes pass through unchanged
                result.add_node(node.clone());
            }
            NodeBody::SubDag(subdag) => {
                // Check for loop pattern before recursing
                if let Some(loop_info) = detect_loop_pattern(subdag) {
                    // Loop pattern detected: flatten unpack+pack but keep body as template
                    let (loop_result, mapping) =
                        lower_loop_subdag(node, subdag, &loop_info, &node.id.0)?;
                    subdag_mappings.insert(node.id.clone(), mapping);

                    // Add unpack and pack nodes (prefixed)
                    for sub_node in &loop_result.dag.nodes {
                        result.add_node(sub_node.clone());
                    }
                    // Add the direct unpack→pack edge for count
                    for sub_edge in &loop_result.dag.edges {
                        result.add_edge(sub_edge.clone());
                    }
                    // Record the loop info with prefixed IDs
                    loops.extend(loop_result.loops);
                } else {
                    // Regular SubDag: recursively lower
                    let lowered_sub = lower(subdag)?;

                    // Build mapping before we modify the lowered_sub
                    let mapping =
                        build_subdag_mapping(node, &lowered_sub.dag, &node.id.0)?;
                    subdag_mappings.insert(node.id.clone(), mapping);

                    // Add all nodes from the sub-DAG with prefixed IDs
                    for sub_node in &lowered_sub.dag.nodes {
                        let prefixed_id = format!("{}/{}", node.id.0, sub_node.id.0);
                        let prefixed_node = Node {
                            id: NodeId::new(prefixed_id),
                            inputs: sub_node.inputs.clone(),
                            outputs: sub_node.outputs.clone(),
                            body: sub_node.body.clone(),
                            examples: sub_node.examples.clone(),
                        };
                        result.add_node(prefixed_node);
                    }

                    // Add internal edges from the sub-DAG with prefixed node IDs
                    for sub_edge in &lowered_sub.dag.edges {
                        let prefixed_edge = Edge::new(
                            format!("{}/{}", node.id.0, sub_edge.from_node.0),
                            sub_edge.from_port.0.clone(),
                            format!("{}/{}", node.id.0, sub_edge.to_node.0),
                            sub_edge.to_port.0.clone(),
                        );
                        result.add_edge(prefixed_edge);
                    }

                    // Propagate any loops from the recursive lower
                    loops.extend(lowered_sub.loops);
                }
            }
        }
    }

    // Second pass: rewire edges, handling SubDag boundaries
    for edge in &dag.edges {
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        let from_is_subdag = from_node.map(|n| n.is_subdag()).unwrap_or(false);
        let to_is_subdag = to_node.map(|n| n.is_subdag()).unwrap_or(false);

        match (from_is_subdag, to_is_subdag) {
            // Both opaque: edge passes through unchanged
            (false, false) => {
                result.add_edge(edge.clone());
            }

            // Source is SubDag: rewire from inner boundary node
            (true, false) => {
                if let Some(mapping) = subdag_mappings.get(&edge.from_node) {
                    if let Some((inner_node, inner_port)) =
                        mapping.output_mappings.get(&edge.from_port)
                    {
                        result.add_edge(Edge::new(
                            inner_node.0.clone(),
                            inner_port.0.clone(),
                            edge.to_node.0.clone(),
                            edge.to_port.0.clone(),
                        ));
                    }
                }
            }

            // Target is SubDag: rewire to inner entrypoint node(s)
            (false, true) => {
                if let Some(mapping) = subdag_mappings.get(&edge.to_node) {
                    if let Some(targets) = mapping.input_mappings.get(&edge.to_port) {
                        // Fan out to all inner entrypoints with matching name
                        for (inner_node, inner_port) in targets {
                            result.add_edge(Edge::new(
                                edge.from_node.0.clone(),
                                edge.from_port.0.clone(),
                                inner_node.0.clone(),
                                inner_port.0.clone(),
                            ));
                        }
                    }
                }
            }

            // Both SubDag: rewire from inner boundary to inner entrypoints
            (true, true) => {
                let from_mapping = subdag_mappings.get(&edge.from_node);
                let to_mapping = subdag_mappings.get(&edge.to_node);

                if let (Some(from_map), Some(to_map)) = (from_mapping, to_mapping) {
                    if let Some((from_inner, from_port)) =
                        from_map.output_mappings.get(&edge.from_port)
                    {
                        if let Some(targets) = to_map.input_mappings.get(&edge.to_port) {
                            for (to_inner, to_port) in targets {
                                result.add_edge(Edge::new(
                                    from_inner.0.clone(),
                                    from_port.0.clone(),
                                    to_inner.0.clone(),
                                    to_port.0.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Build input remaps from SubDag mappings so the executor can
    // remap BoundaryMocks keys from original SubDag IDs to lowered inner IDs.
    let mut input_remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    for (subdag_id, mapping) in &subdag_mappings {
        for (port_name, targets) in &mapping.input_mappings {
            let key = (subdag_id.0.clone(), port_name.0.clone());
            let remapped: Vec<(String, String)> = targets
                .iter()
                .map(|(inner_id, inner_port)| (inner_id.0.clone(), inner_port.0.clone()))
                .collect();
            input_remaps.insert(key, remapped);
        }
    }

    Ok(LowerResult {
        dag: result,
        loops,
        input_remaps,
    })
}

/// Lower a loop SubDag: flatten unpack+pack, preserve body as template.
///
/// Returns a LowerResult containing only the unpack and pack nodes (prefixed)
/// plus a LoopInfo recording the body template for runtime iteration.
fn lower_loop_subdag<T: Clone>(
    parent_node: &Node<T>,
    inner_dag: &Dag<T>,
    loop_pattern: &LoopPatternInfo<T>,
    parent_prefix: &str,
) -> Result<(LowerResult<T>, SubDagMapping), LowerError> {
    let unpack = inner_dag
        .nodes
        .iter()
        .find(|n| n.id.0 == "unpack")
        .unwrap();
    let pack = inner_dag
        .nodes
        .iter()
        .find(|n| n.id.0 == "pack")
        .unwrap();

    let unpack_id = NodeId::new(format!("{}/unpack", parent_prefix));
    let pack_id = NodeId::new(format!("{}/pack", parent_prefix));

    // Create prefixed unpack and pack nodes
    let prefixed_unpack = Node {
        id: unpack_id.clone(),
        inputs: unpack.inputs.clone(),
        outputs: unpack.outputs.clone(),
        body: unpack.body.clone(),
        examples: unpack.examples.clone(),
    };
    let prefixed_pack = Node {
        id: pack_id.clone(),
        inputs: pack.inputs.clone(),
        outputs: pack.outputs.clone(),
        body: pack.body.clone(),
        examples: pack.examples.clone(),
    };

    let mut flat_dag = Dag::new();
    flat_dag.add_node(prefixed_unpack);
    flat_dag.add_node(prefixed_pack);

    // Keep the unpack→pack "count" edge, and add a direct unpack→pack edge
    // for the element/result port. The executor will replace the element list
    // with body execution results before pack runs.
    for edge in &inner_dag.edges {
        if edge.from_node.0 == "unpack" && edge.to_node.0 == "pack" {
            flat_dag.add_edge(Edge::new(
                unpack_id.0.clone(),
                edge.from_port.0.clone(),
                pack_id.0.clone(),
                edge.to_port.0.clone(),
            ));
        }
    }
    // Direct element→result edge: unpack's element output feeds pack's result input.
    // At runtime, the executor replaces this with the iterated body results.
    flat_dag.add_edge(Edge::new(
        unpack_id.0.clone(),
        loop_pattern.element_port.clone(),
        pack_id.0.clone(),
        "result",
    ));

    // Auto-detect extra inputs BEFORE building the port mapping.
    // Body entrypoints beyond the element port (e.g., repo_path) flow
    // through unpack at runtime. We need to identify them first so the
    // mapping function knows to skip them (they aren't entrypoints of
    // the flat_dag which only contains unpack + pack).
    let mut extra_input_ports = loop_pattern.extra_input_ports.clone();
    let body_entrypoints = detect_entrypoints(&loop_pattern.body_dag);
    for (_, port_name, _) in &body_entrypoints.entrypoint_ports {
        if port_name.0 != loop_pattern.element_port
            && !port_name.0.starts_with("res:")
            && !extra_input_ports.contains(&port_name.0)
        {
            extra_input_ports.push(port_name.0.clone());
        }
    }

    // Build port mapping for parent: the parent's input ports map to unpack,
    // and the parent's output ports map to pack. Extra input ports are skipped
    // here — they'll be routed through unpack separately below.
    let mut mapping =
        build_subdag_mapping_for_loop(parent_node, &flat_dag, parent_prefix, &extra_input_ports)?;

    // Extra input ports flow through unpack to the body at runtime.
    // Map them to unpack so the executor can route values correctly.
    for port_name in &extra_input_ports {
        let pn = PortName::new(port_name.clone());
        if !mapping.input_mappings.contains_key(&pn) {
            mapping.input_mappings.insert(
                pn.clone(),
                vec![(unpack_id.clone(), pn)],
            );
        }
    }

    let loop_info = LoopInfo {
        unpack_id: unpack_id.clone(),
        pack_id: pack_id.clone(),
        element_port: loop_pattern.element_port.clone(),
        body_dag: loop_pattern.body_dag.clone(),
        extra_input_ports,
    };

    Ok((
        LowerResult {
            dag: flat_dag,
            loops: vec![loop_info],
            input_remaps: HashMap::new(),
        },
        mapping,
    ))
}

/// Build port mapping for a loop SubDag (only unpack + pack present).
///
/// `extra_ports` are input port names that will be mapped separately by the
/// caller (they flow through unpack to the body at runtime). These are skipped
/// during entrypoint matching since they don't exist as ports on the flat_dag.
fn build_subdag_mapping_for_loop<T>(
    parent_node: &Node<T>,
    flat_dag: &Dag<T>,
    parent_prefix: &str,
    extra_ports: &[String],
) -> Result<SubDagMapping, LowerError> {
    let entrypoints = detect_entrypoints(flat_dag);
    let boundaries = detect_boundaries(flat_dag);

    let mut input_mappings: HashMap<PortName, Vec<(NodeId, PortName)>> = HashMap::new();
    let mut output_mappings: HashMap<PortName, (NodeId, PortName)> = HashMap::new();

    for parent_port in &parent_node.inputs {
        // Extra ports are mapped by the caller — skip them here
        if extra_ports.contains(&parent_port.name.0) {
            continue;
        }
        let mut targets = Vec::new();
        for (inner_node_id, inner_port_name, _type_id) in &entrypoints.entrypoint_ports {
            if inner_port_name == &parent_port.name {
                targets.push((inner_node_id.clone(), inner_port_name.clone()));
            }
        }
        if targets.is_empty() {
            return Err(LowerError::NoInnerEntrypoint {
                node: parent_prefix.to_string(),
                port: parent_port.name.0.clone(),
            });
        }
        input_mappings.insert(parent_port.name.clone(), targets);
    }

    for parent_port in &parent_node.outputs {
        let mut found = None;
        for (inner_node_id, inner_port_name) in &boundaries.boundary_ports {
            if inner_port_name == &parent_port.name {
                found = Some((inner_node_id.clone(), inner_port_name.clone()));
                break;
            }
        }
        match found {
            Some(mapping) => {
                output_mappings.insert(parent_port.name.clone(), mapping);
            }
            None => {
                return Err(LowerError::NoInnerBoundary {
                    node: parent_prefix.to_string(),
                    port: parent_port.name.0.clone(),
                });
            }
        }
    }

    Ok(SubDagMapping {
        input_mappings,
        output_mappings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;

    #[test]
    fn test_lower_flat_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "S")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let result = lower(&dag).unwrap();

        assert_eq!(result.dag.nodes.len(), 2);
        assert_eq!(result.dag.edges.len(), 1);
        assert!(result.loops.is_empty());
    }

    #[test]
    fn test_lower_subdag() {
        // Create a sub-DAG with input and output ports that match parent
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("in", "S")],  // entrypoint
            vec![port("out", "S")], // boundary
            (),
        ));

        // Create the parent DAG with a SubDag node
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", subdag));

        let result = lower(&dag).unwrap();

        // The inner node should be prefixed with "wrapper/"
        assert_eq!(result.dag.nodes.len(), 1);
        assert_eq!(result.dag.nodes[0].id.0, "wrapper/inner");
        assert!(result.loops.is_empty());
    }

    #[test]
    fn test_lower_subdag_with_edge_into() {
        // Create a sub-DAG
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("data", "S")],
            vec![port("result", "S")],
            (),
        ));

        // Create parent DAG: A -> SubDag
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::subdag("wrapper", subdag));
        dag.add_edge(edge("A", "out", "wrapper", "data"));

        let result = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(result.dag.nodes.len(), 2);

        // Edge should be rewired: A -> wrapper/inner
        assert_eq!(result.dag.edges.len(), 1);
        let e = &result.dag.edges[0];
        assert_eq!(e.from_node.0, "A");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "wrapper/inner");
        assert_eq!(e.to_port.0, "data");
    }

    #[test]
    fn test_lower_subdag_with_edge_from() {
        // Create a sub-DAG
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        // Create parent DAG: SubDag -> B
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", subdag));
        dag.add_node(Node::opaque("B", vec![port("data", "S")], vec![], ()));
        dag.add_edge(edge("wrapper", "out", "B", "data"));

        let result = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(result.dag.nodes.len(), 2);

        // Edge should be rewired: wrapper/inner -> B
        assert_eq!(result.dag.edges.len(), 1);
        let e = &result.dag.edges[0];
        assert_eq!(e.from_node.0, "wrapper/inner");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "B");
        assert_eq!(e.to_port.0, "data");
    }

    #[test]
    fn test_lower_subdag_fanout() {
        // Create a sub-DAG with multiple nodes having the same input port name
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "node1",
            vec![port("data", "S")],
            vec![port("out1", "S")],
            (),
        ));
        subdag.add_node(Node::opaque(
            "node2",
            vec![port("data", "S")],
            vec![port("out2", "S")],
            (),
        ));

        // Create parent DAG: A -> SubDag (should fan out to both inner nodes)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::subdag("wrapper", subdag));
        dag.add_edge(edge("A", "out", "wrapper", "data"));

        let result = lower(&dag).unwrap();

        // Should have 3 nodes
        assert_eq!(result.dag.nodes.len(), 3);

        // Should have 2 edges (fanned out)
        assert_eq!(result.dag.edges.len(), 2);

        // Both edges should come from A.out
        for e in &result.dag.edges {
            assert_eq!(e.from_node.0, "A");
            assert_eq!(e.from_port.0, "out");
            assert!(e.to_node.0 == "wrapper/node1" || e.to_node.0 == "wrapper/node2");
            assert_eq!(e.to_port.0, "data");
        }
    }

    #[test]
    fn test_lower_subdag_to_subdag() {
        // Two SubDags connected to each other
        let mut subdag1: Dag<()> = Dag::new();
        subdag1.add_node(Node::opaque(
            "inner1",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        let mut subdag2: Dag<()> = Dag::new();
        subdag2.add_node(Node::opaque(
            "inner2",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("sub1", subdag1));
        dag.add_node(Node::subdag("sub2", subdag2));
        dag.add_edge(edge("sub1", "out", "sub2", "in"));

        let result = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(result.dag.nodes.len(), 2);

        // Edge should connect inner nodes
        assert_eq!(result.dag.edges.len(), 1);
        let e = &result.dag.edges[0];
        assert_eq!(e.from_node.0, "sub1/inner1");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "sub2/inner2");
        assert_eq!(e.to_port.0, "in");
    }
}
