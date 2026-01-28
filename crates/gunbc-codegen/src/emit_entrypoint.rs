//! Entrypoint detection and scaffolding utilities.
//!
//! This module provides utilities for identifying source/sink nodes based on TypeId conventions
//! and generating common entrypoint scaffolding.
//!
//! ## TypeId Conventions
//!
//! **Source types (entrypoints):**
//! - `CLI::Args` - command line arguments
//! - `CLI::Env` - environment variables
//! - `CLI::Stdin` - standard input
//! - `HTTP::Request` - HTTP request body/params
//! - `HTTP::Headers` - HTTP headers
//!
//! **Sink types (effects):**
//! - `CLI::Stdout` - standard output
//! - `CLI::Stderr` - standard error
//! - `CLI::ExitCode` - process exit code
//! - `HTTP::Response` - HTTP response body
//! - `File::Write` - file system write

use std::collections::HashSet;

use gunbc_ir::{BoundaryDeclaration, Dag, Node, NodeBody, Port, TypeId};

/// Entrypoint interface type determined by source/sink TypeIds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntrypointKind {
    /// Command-line interface (args, stdin, stdout, stderr, exit code)
    Cli,
    /// HTTP REST interface (request, response, headers)
    Rest,
    /// AWS Lambda interface
    Lambda,
    /// No recognized entrypoint pattern
    None,
}

/// Information about a detected entrypoint in a DAG.
#[derive(Debug, Clone)]
pub struct EntrypointInfo {
    /// The kind of entrypoint detected.
    pub kind: EntrypointKind,
    /// Source nodes (nodes that receive external input).
    pub source_nodes: Vec<String>,
    /// Sink nodes (nodes that produce external output).
    pub sink_nodes: Vec<String>,
    /// CLI argument specifications extracted from source nodes.
    pub cli_args: Vec<CliArgSpec>,
}

/// Specification for a CLI argument derived from port metadata.
#[derive(Debug, Clone)]
pub struct CliArgSpec {
    /// The argument name (used for --name flag).
    pub name: String,
    /// The port type (used to determine clap type).
    pub type_id: String,
    /// Whether this is a required argument.
    pub required: bool,
    /// Default value if any.
    pub default: Option<String>,
    /// Help text for the argument.
    pub help: Option<String>,
}

/// Check if a TypeId represents a CLI source type.
pub fn is_cli_source(type_id: &TypeId) -> bool {
    type_id.0.starts_with("CLI::") && matches!(
        type_id.0.as_str(),
        "CLI::Args" | "CLI::Env" | "CLI::Stdin"
    )
}

/// Check if a TypeId represents a CLI sink type.
pub fn is_cli_sink(type_id: &TypeId) -> bool {
    type_id.0.starts_with("CLI::") && matches!(
        type_id.0.as_str(),
        "CLI::Stdout" | "CLI::Stderr" | "CLI::ExitCode"
    )
}

/// Check if a TypeId represents an HTTP source type.
pub fn is_http_source(type_id: &TypeId) -> bool {
    type_id.0.starts_with("HTTP::") && matches!(
        type_id.0.as_str(),
        "HTTP::Request" | "HTTP::Headers"
    )
}

/// Check if a TypeId represents an HTTP sink type.
pub fn is_http_sink(type_id: &TypeId) -> bool {
    type_id.0.starts_with("HTTP::") && matches!(
        type_id.0.as_str(),
        "HTTP::Response"
    )
}

/// Check if a port has a source type (receives external input).
pub fn is_source_port(port: &Port) -> bool {
    is_cli_source(&port.type_id) || is_http_source(&port.type_id)
}

/// Check if a port has a sink type (produces external output).
pub fn is_sink_port(port: &Port) -> bool {
    is_cli_sink(&port.type_id) || is_http_sink(&port.type_id)
}

/// Detect the entrypoint kind from a DAG by examining source/sink types.
pub fn detect_entrypoint_kind<T>(dag: &Dag<T>) -> EntrypointKind {
    let has_cli = dag.nodes.iter().any(|n| {
        n.outputs.iter().any(|p| is_cli_source(&p.type_id)) ||
        n.inputs.iter().any(|p| is_cli_sink(&p.type_id))
    });

    let has_http = dag.nodes.iter().any(|n| {
        n.outputs.iter().any(|p| is_http_source(&p.type_id)) ||
        n.inputs.iter().any(|p| is_http_sink(&p.type_id))
    });

    match (has_cli, has_http) {
        (true, false) => EntrypointKind::Cli,
        (false, true) => EntrypointKind::Rest,
        (true, true) => EntrypointKind::Cli, // CLI takes precedence if both
        (false, false) => EntrypointKind::None,
    }
}

/// Analyze a DAG and extract entrypoint information.
pub fn analyze_entrypoint<T>(dag: &Dag<T>) -> EntrypointInfo {
    let kind = detect_entrypoint_kind(dag);

    let mut source_nodes = Vec::new();
    let mut sink_nodes = Vec::new();
    let mut cli_args = Vec::new();
    let mut seen_args = HashSet::new();
    let mut seen_sources = HashSet::new();
    let mut seen_sinks = HashSet::new();

    let mut wired_inputs: HashSet<(String, String)> = HashSet::new();
    for edge in &dag.edges {
        wired_inputs.insert((edge.to_node.0.clone(), edge.to_port.0.clone()));
    }

    for node in &dag.nodes {
        // Nodes with any unbound input are entrypoint sources.
        let has_unbound = node
            .inputs
            .iter()
            .any(|p| !wired_inputs.contains(&(node.id.0.clone(), p.name.0.clone())));
        if has_unbound && seen_sources.insert(node.id.0.clone()) {
            source_nodes.push(node.id.0.clone());
        }

        // Unbound input ports are external inputs (CLI args for CLI entrypoints).
        for port in &node.inputs {
            if !wired_inputs.contains(&(node.id.0.clone(), port.name.0.clone())) {
                if seen_args.insert(port.name.0.clone()) {
                    cli_args.push(CliArgSpec {
                        name: port.name.0.clone(),
                        type_id: port.type_id.0.clone(),
                        required: port.guard.is_none(),
                        default: None,
                        help: None,
                    });
                }
            }
        }

        // Check if this node produces source types (entrypoint input)
        let is_source = node.outputs.iter().any(is_source_port);
        if is_source {
            if seen_sources.insert(node.id.0.clone()) {
                source_nodes.push(node.id.0.clone());
            }

            // Extract CLI arg specs from output ports
            for port in &node.outputs {
                if is_cli_source(&port.type_id) || !port.type_id.0.starts_with("CLI::") {
                    // For non-CLI types on source nodes, treat as CLI args
                    if seen_args.insert(port.name.0.clone()) {
                        cli_args.push(CliArgSpec {
                            name: port.name.0.clone(),
                            type_id: port.type_id.0.clone(),
                            required: port.guard.is_none(),
                            default: None,
                            help: None,
                        });
                    }
                }
            }
        }

        // Check if this node consumes sink types (entrypoint output)
        let is_sink = node.inputs.iter().any(is_sink_port);
        if is_sink {
            if seen_sinks.insert(node.id.0.clone()) {
                sink_nodes.push(node.id.0.clone());
            }
        }
    }

    EntrypointInfo {
        kind,
        source_nodes,
        sink_nodes,
        cli_args,
    }
}

/// Check if a node is a source node (has no inputs from other nodes in the DAG).
pub fn is_source_node<T>(node: &Node<T>, dag: &Dag<T>) -> bool {
    // A source node either:
    // 1. Has no inputs at all
    // 2. Has inputs but none are connected via edges
    if node.inputs.is_empty() {
        return true;
    }

    // Check if any edge targets this node
    !dag.edges.iter().any(|e| e.to_node == node.id)
}

/// Check if a node is a sink node (has no outputs consumed by other nodes in the DAG).
pub fn is_sink_node<T>(node: &Node<T>, dag: &Dag<T>) -> bool {
    // A sink node either:
    // 1. Has no outputs at all
    // 2. Has outputs but none are connected via edges
    if node.outputs.is_empty() {
        return true;
    }

    // Check if any edge originates from this node
    !dag.edges.iter().any(|e| e.from_node == node.id)
}

/// Find all root nodes in a DAG (nodes with no incoming edges).
pub fn find_root_nodes<T>(dag: &Dag<T>) -> Vec<&Node<T>> {
    dag.nodes.iter().filter(|n| is_source_node(n, dag)).collect()
}

/// Find all leaf nodes in a DAG (nodes with no outgoing edges).
pub fn find_leaf_nodes<T>(dag: &Dag<T>) -> Vec<&Node<T>> {
    dag.nodes.iter().filter(|n| is_sink_node(n, dag)).collect()
}

/// Extract SubDAG if the node body is a SubDAG.
pub fn get_subdag<T>(node: &Node<T>) -> Option<&Dag<T>> {
    match &node.body {
        NodeBody::SubDag(dag) => Some(dag),
        NodeBody::Opaque(_) => None,
    }
}

/// Recursively find all boundary declarations in a DAG and its nested SubDAGs.
///
/// This function traverses the entire DAG structure to discover all external
/// boundaries, which enables automatic derivation of mock flags for each
/// transport layer.
pub fn find_all_boundaries_recursive<T>(dag: &Dag<T>) -> Vec<BoundaryDeclaration> {
    let mut boundaries = dag.metadata.boundary_declarations.clone();

    // Recurse into SubDAGs to find nested transport layers
    for node in &dag.nodes {
        if let NodeBody::SubDag(sub) = &node.body {
            boundaries.extend(find_all_boundaries_recursive(sub));
        }
    }

    // Dedupe by external_type (we want one flag per layer)
    boundaries.sort_by(|a, b| a.external_type.0.cmp(&b.external_type.0));
    boundaries.dedup_by(|a, b| a.external_type.0 == b.external_type.0);

    boundaries
}

/// Extract the layer name from an External type ID.
///
/// e.g., "External::GitHub::Gist" -> "gist"
/// e.g., "External::HTTP::Request" -> "http"
pub fn extract_layer_name(type_id: &TypeId) -> Option<String> {
    let s = &type_id.0;
    if !s.starts_with("External::") {
        return None;
    }
    // Get the second segment (the layer) and lowercase it
    // External::GitHub::Gist -> GitHub (middle segment)
    // External::HTTP::Request -> HTTP (middle segment)
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() >= 2 {
        Some(parts[1].to_lowercase())
    } else {
        None
    }
}

/// Check if a TypeId represents an external boundary type.
pub fn is_external_type(type_id: &TypeId) -> bool {
    type_id.0.starts_with("External::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{edge, port, Dag, DagMetadata, Node, NodeBody, NodeId};

    #[derive(Debug, Clone)]
    struct DummyOp;

    fn make_cli_dag() -> Dag<DummyOp> {
        Dag {
            nodes: vec![
                Node {
                    id: NodeId("args".into()),
                    inputs: vec![],
                    outputs: vec![
                        port("path", "String"),
                        port("glob", "String"),
                    ],
                    body: NodeBody::Opaque(DummyOp),
                },
                Node {
                    id: NodeId("process".into()),
                    inputs: vec![port("path", "String"), port("glob", "String")],
                    outputs: vec![port("result", "String")],
                    body: NodeBody::Opaque(DummyOp),
                },
            ],
            edges: vec![
                edge("args", "path", "process", "path"),
                edge("args", "glob", "process", "glob"),
            ],
            metadata: DagMetadata::default(),
        }
    }

    #[test]
    fn test_is_cli_source() {
        assert!(is_cli_source(&TypeId("CLI::Args".into())));
        assert!(is_cli_source(&TypeId("CLI::Env".into())));
        assert!(is_cli_source(&TypeId("CLI::Stdin".into())));
        assert!(!is_cli_source(&TypeId("CLI::Stdout".into())));
        assert!(!is_cli_source(&TypeId("String".into())));
    }

    #[test]
    fn test_is_cli_sink() {
        assert!(is_cli_sink(&TypeId("CLI::Stdout".into())));
        assert!(is_cli_sink(&TypeId("CLI::Stderr".into())));
        assert!(is_cli_sink(&TypeId("CLI::ExitCode".into())));
        assert!(!is_cli_sink(&TypeId("CLI::Args".into())));
        assert!(!is_cli_sink(&TypeId("String".into())));
    }

    #[test]
    fn test_find_root_nodes() {
        let dag = make_cli_dag();
        let roots = find_root_nodes(&dag);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id.0, "args");
    }

    #[test]
    fn test_find_leaf_nodes() {
        let dag = make_cli_dag();
        let leaves = find_leaf_nodes(&dag);
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].id.0, "process");
    }

    #[test]
    fn test_analyze_entrypoint_unbound_inputs() {
        let dag = Dag {
            nodes: vec![
                Node {
                    id: NodeId("worker".into()),
                    inputs: vec![port("repo", "String")],
                    outputs: vec![port("out", "String")],
                    body: NodeBody::Opaque(DummyOp),
                },
            ],
            edges: vec![],
            metadata: DagMetadata::default(),
        };

        let info = analyze_entrypoint(&dag);
        assert!(info.source_nodes.iter().any(|n| n == "worker"));
        let repo_arg = info.cli_args.iter().find(|a| a.name == "repo").unwrap();
        assert_eq!(repo_arg.type_id, "String");
        assert!(repo_arg.required);
    }

    #[test]
    fn test_detect_entrypoint_kind_none() {
        let dag = make_cli_dag();
        // No CLI:: prefixed types, so should be None
        assert_eq!(detect_entrypoint_kind(&dag), EntrypointKind::None);
    }

    #[test]
    fn test_extract_layer_name() {
        assert_eq!(
            extract_layer_name(&TypeId("External::GitHub::Gist".into())),
            Some("github".into())
        );
        assert_eq!(
            extract_layer_name(&TypeId("External::HTTP::Request".into())),
            Some("http".into())
        );
        assert_eq!(
            extract_layer_name(&TypeId("External::REST::Response".into())),
            Some("rest".into())
        );
        assert_eq!(
            extract_layer_name(&TypeId("String".into())),
            None
        );
    }

    #[test]
    fn test_is_external_type() {
        assert!(is_external_type(&TypeId("External::GitHub::Gist".into())));
        assert!(is_external_type(&TypeId("External::HTTP::Request".into())));
        assert!(!is_external_type(&TypeId("String".into())));
        assert!(!is_external_type(&TypeId("CLI::Args".into())));
    }

    #[test]
    fn test_find_all_boundaries_recursive_empty() {
        let dag = make_cli_dag();
        let boundaries = find_all_boundaries_recursive(&dag);
        assert!(boundaries.is_empty());
    }

    #[test]
    fn test_find_all_boundaries_recursive_with_boundaries() {
        use gunbc_ir::PortName;

        let inner_dag = Dag {
            nodes: vec![
                Node {
                    id: NodeId("inner".into()),
                    inputs: vec![],
                    outputs: vec![port("result", "String")],
                    body: NodeBody::Opaque(DummyOp),
                },
            ],
            edges: vec![],
            metadata: DagMetadata {
                boundary_declarations: vec![
                    BoundaryDeclaration {
                        node: NodeId("inner".into()),
                        port: PortName("result".into()),
                        external_type: TypeId("External::HTTP::Request".into()),
                    },
                ],
                ..Default::default()
            },
        };

        let dag = Dag {
            nodes: vec![
                Node {
                    id: NodeId("wrapper".into()),
                    inputs: vec![],
                    outputs: vec![port("out", "String")],
                    body: NodeBody::SubDag(inner_dag),
                },
            ],
            edges: vec![],
            metadata: DagMetadata {
                boundary_declarations: vec![
                    BoundaryDeclaration {
                        node: NodeId("wrapper".into()),
                        port: PortName("out".into()),
                        external_type: TypeId("External::GitHub::Gist".into()),
                    },
                ],
                ..Default::default()
            },
        };

        let boundaries = find_all_boundaries_recursive(&dag);
        assert_eq!(boundaries.len(), 2);
        // Should be sorted alphabetically by external_type
        assert_eq!(boundaries[0].external_type.0, "External::GitHub::Gist");
        assert_eq!(boundaries[1].external_type.0, "External::HTTP::Request");
    }
}
