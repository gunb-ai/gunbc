//! Topo-ordered emission plan with data flow bindings.
//!
//! Built from a lowered DAG + [`Computation`](super::computation::Computation)
//! classifications. Every codegen path consumes the same [`EmitPlan`].
//!
//! # Pipeline position
//!
//! ```text
//! Dag<LoweredOp>
//!   → classify_computation() per node → Computation
//!   → build_emit_plan()               → EmitPlan       (this module)
//!   → lower_plan_to_abstract_ir()     → SourceFile      (lower_to_ir)
//!   → LowerIR<SourceFile, T>          → target IR       (lower_rust/go/c/mips)
//!   → CodeRenderer                    → text            (render_*)
//! ```
//!
//! **Owned by**: Task 4 (dsl-codegen-tasks.md)

use std::collections::{HashMap, HashSet, VecDeque};

use crate::computation::{classify_computation, Computation, PureBody};
use daglang_derive::DerivedArtifacts;
use daglang_lower::LoweredOp;
use gunbc_ir::{Dag, PortName};

// ===========================================================================
// EmitPlan — the backbone data structure for codegen
// ===========================================================================

/// A topo-ordered sequence of computation steps with data flow bindings.
///
/// Built once from the DAG and consumed by every codegen backend. The plan
/// encodes:
/// - What each step computes ([`Computation`]).
/// - Where each step's inputs come from ([`InputBinding`]).
/// - Where each step's outputs go ([`OutputBinding`]).
/// - Which steps are entrypoints (DAG inputs from the outside world).
/// - Which steps perform transport (I/O boundary crossings).
#[derive(Debug, Clone)]
pub struct EmitPlan {
    /// Steps in topological order — each step can only reference
    /// earlier steps via `InputBinding::FromStep`.
    pub steps: Vec<EmitStep>,
    /// Ports that receive values from outside the DAG (CLI args, config, etc.).
    pub entrypoints: Vec<EntrypointPort>,
    /// Node IDs of steps that perform transport (for mock generation / dry-run).
    pub transport_nodes: Vec<String>,
}

/// A single step in the emission plan.
///
/// Corresponds to one DAG node after classification. The `computation` field
/// describes *what* the step does; the bindings describe *where* data flows.
#[derive(Debug, Clone)]
pub struct EmitStep {
    /// Original DAG node ID (for diagnostics, test generation, manifests).
    pub node_id: String,
    /// Target-independent description of what this step computes.
    pub computation: Computation,
    /// Where each input port gets its value.
    pub input_sources: Vec<InputBinding>,
    /// Where each output port's value is consumed.
    pub output_bindings: Vec<OutputBinding>,
}

// ===========================================================================
// Bindings — data flow wiring
// ===========================================================================

/// Where a step input gets its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputBinding {
    /// Value produced by a previous step's output port.
    FromStep {
        /// Index into `EmitPlan::steps` (guaranteed < current step index).
        step_index: usize,
        /// Output port name on the source step.
        port: String,
    },
    /// Value injected from outside the DAG (CLI arg, env var, config).
    FromEntrypoint {
        /// Entrypoint port name.
        port: String,
    },
    /// Compile-time constant value baked into the generated code.
    Constant(serde_json::Value),
}

/// Where a step output's value goes.
///
/// A single output port can feed multiple downstream steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputBinding {
    /// Name of the output port on this step.
    pub port: String,
    /// Downstream consumers: `(step_index, input_port_name)` pairs.
    pub consumers: Vec<(usize, String)>,
}

// ===========================================================================
// Entrypoints
// ===========================================================================

/// A port that receives values from outside the DAG.
///
/// These become function parameters in the generated code's `main()` or
/// top-level entry function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrypointPort {
    /// Port name (becomes a parameter name in generated code).
    pub name: String,
    /// Abstract type of the value (e.g., "String", "Path", "ToolRegistry").
    pub abstract_type: String,
    /// Which step(s) consume this entrypoint: `(step_index, input_port_name)`.
    pub consumers: Vec<(usize, String)>,
}

// ===========================================================================
// Builder errors
// ===========================================================================

/// Errors that can occur when building an [`EmitPlan`].
#[derive(Debug, Clone)]
pub enum PlanError {
    /// A node could not be classified as a Computation.
    ClassifyFailed { node_id: String, detail: String },
    /// An edge references a node that doesn't exist in the plan.
    MissingNode { node_id: String },
    /// An edge references a port that doesn't exist on the node.
    MissingPort {
        node_id: String,
        port: String,
        direction: &'static str,
    },
    /// A structurally required internal input had no producer edge.
    MissingInternalInputBinding { node_id: String, port: String },
    /// The DAG contains a cycle (should be impossible after topo sort).
    CycleDetected,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClassifyFailed { node_id, detail } => {
                write!(f, "cannot classify node `{node_id}`: {detail}")
            }
            Self::MissingNode { node_id } => {
                write!(f, "edge references unknown node `{node_id}`")
            }
            Self::MissingPort {
                node_id,
                port,
                direction,
            } => {
                write!(f, "node `{node_id}` has no {direction} port `{port}`")
            }
            Self::MissingInternalInputBinding { node_id, port } => {
                write!(
                    f,
                    "node `{node_id}` is missing a data edge for required internal input `{port}`"
                )
            }
            Self::CycleDetected => write!(f, "DAG contains a cycle"),
        }
    }
}

// ===========================================================================
// Builder (A2.2)
// ===========================================================================

/// Build an [`EmitPlan`] from a lowered DAG and its derived artifacts.
///
/// Steps are emitted in topological order (Kahn's algorithm). Each node is
/// classified via [`classify_computation`], and edges are resolved to
/// [`InputBinding`] / [`OutputBinding`] pairs.
///
/// SubDag nodes are skipped (they are structural wrappers — their inner
/// nodes would need to be lowered separately).
pub fn build_emit_plan(
    dag: &Dag<LoweredOp>,
    _artifacts: &DerivedArtifacts,
) -> Result<EmitPlan, PlanError> {
    // 1. Topo sort node IDs via Kahn's algorithm.
    let topo_ids = topo_sort_dag(dag)?;

    // 2. Build node_id → step_index mapping.
    let mut id_to_step: HashMap<String, usize> = HashMap::new();
    let mut steps: Vec<EmitStep> = Vec::with_capacity(topo_ids.len());
    let mut transport_nodes: Vec<String> = Vec::new();

    // Track which (node_id, port) pairs are entrypoints (unconnected inputs).
    let connected_inputs: HashSet<(String, String)> = dag
        .edges
        .iter()
        .filter(|e| e.kind.carries_data())
        .map(|e| (e.to_node.0.clone(), e.to_port.0.clone()))
        .collect();

    // Build an edge lookup: (to_node, to_port) → (from_node, from_port).
    let mut edge_source: HashMap<(String, String), (String, String)> = HashMap::new();
    for edge in dag.edges.iter().filter(|e| e.kind.carries_data()) {
        edge_source.insert(
            (edge.to_node.0.clone(), edge.to_port.0.clone()),
            (edge.from_node.0.clone(), edge.from_port.0.clone()),
        );
    }

    // 3. Classify each node and build steps.
    for node_id_str in &topo_ids {
        let node =
            dag.get_node(&node_id_str.clone().into())
                .ok_or_else(|| PlanError::MissingNode {
                    node_id: node_id_str.clone(),
                })?;

        let computation = classify_computation(node).map_err(|e| PlanError::ClassifyFailed {
            node_id: node_id_str.clone(),
            detail: e.to_string(),
        })?;

        let step_index = steps.len();
        id_to_step.insert(node_id_str.clone(), step_index);

        // Detect transport steps.
        if matches!(computation, Computation::Transport { .. }) {
            transport_nodes.push(node_id_str.clone());
        }

        // Build input bindings.
        let input_sources: Vec<InputBinding> = node
            .inputs
            .iter()
            .filter(|p| should_bind_input_port(&p.name.0, &computation))
            .map(|port| -> Result<InputBinding, PlanError> {
                let key = (node_id_str.clone(), port.name.0.clone());
                if let Some((from_node, from_port)) = edge_source.get(&key) {
                    if let Some(&src_step) = id_to_step.get(from_node) {
                        Ok(InputBinding::FromStep {
                            step_index: src_step,
                            port: from_port.clone(),
                        })
                    } else {
                        // Source node not yet in plan (shouldn't happen after topo sort).
                        Ok(InputBinding::FromEntrypoint {
                            port: port.name.0.clone(),
                        })
                    }
                } else if is_user_input_port(&port.name.0) {
                    // No incoming edge → entrypoint.
                    Ok(InputBinding::FromEntrypoint {
                        port: port.name.0.clone(),
                    })
                } else {
                    Err(PlanError::MissingInternalInputBinding {
                        node_id: node_id_str.clone(),
                        port: port.name.0.clone(),
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Placeholder output bindings (consumers filled in pass 2).
        let output_bindings: Vec<OutputBinding> = node
            .outputs
            .iter()
            .map(|port| OutputBinding {
                port: port.name.0.clone(),
                consumers: Vec::new(),
            })
            .collect();

        steps.push(EmitStep {
            node_id: node_id_str.clone(),
            computation,
            input_sources,
            output_bindings,
        });
    }

    // 4. Pass 2: fill in output binding consumers.
    for edge in dag.edges.iter().filter(|e| e.kind.carries_data()) {
        let from_id = &edge.from_node.0;
        let to_id = &edge.to_node.0;

        if let (Some(&from_step), Some(&to_step)) = (
            id_to_step.get(from_id.as_str()),
            id_to_step.get(to_id.as_str()),
        ) {
            let from_port = &edge.from_port.0;
            if let Some(binding) = steps[from_step]
                .output_bindings
                .iter_mut()
                .find(|b| b.port == *from_port)
            {
                binding.consumers.push((to_step, edge.to_port.0.clone()));
            }
        }
    }

    // 5. Collect entrypoints.
    let mut entrypoints: Vec<EntrypointPort> = Vec::new();
    let mut seen_entrypoints: HashSet<String> = HashSet::new();

    for (step_idx, step) in steps.iter().enumerate() {
        let node = dag.get_node(&step.node_id.clone().into()).unwrap();
        for port in &node.inputs {
            if !is_user_input_port(&port.name.0) {
                continue;
            }
            let key = (step.node_id.clone(), port.name.0.clone());
            if !connected_inputs.contains(&key) && seen_entrypoints.insert(port.name.0.clone()) {
                entrypoints.push(EntrypointPort {
                    name: port.name.0.clone(),
                    abstract_type: port.type_id.0.clone(),
                    consumers: vec![(step_idx, port.name.0.clone())],
                });
            } else if !connected_inputs.contains(&key) {
                // Same-named entrypoint consumed by multiple steps.
                if let Some(ep) = entrypoints.iter_mut().find(|ep| ep.name == port.name.0) {
                    ep.consumers.push((step_idx, port.name.0.clone()));
                }
            }
        }
    }

    Ok(EmitPlan {
        steps,
        entrypoints,
        transport_nodes,
    })
}

fn is_user_input_port(name: &str) -> bool {
    let pn = PortName::from(name);
    pn.is_user()
}

fn should_bind_input_port(name: &str, computation: &Computation) -> bool {
    if is_user_input_port(name) {
        return true;
    }
    matches!(
        computation,
        Computation::Pure {
            body: PureBody::Passthrough,
            ..
        }
    ) && name.starts_with(PortName::OUTPUT_PASSTHROUGH_PREFIX)
}

/// Kahn's algorithm topo sort over a `Dag<LoweredOp>`.
fn topo_sort_dag(dag: &Dag<LoweredOp>) -> Result<Vec<String>, PlanError> {
    let node_ids: Vec<String> = dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    let id_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|id| (id.as_str(), 0)).collect();
    let mut adjacency: HashMap<&str, Vec<&str>> = node_ids
        .iter()
        .map(|id| (id.as_str(), Vec::new()))
        .collect();

    for edge in &dag.edges {
        let from = edge.from_node.0.as_str();
        let to = edge.to_node.0.as_str();
        if id_set.contains(from) && id_set.contains(to) {
            adjacency.get_mut(from).unwrap().push(to);
            *in_degree.get_mut(to).unwrap() += 1;
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    // Sort initial queue for deterministic ordering.
    let mut sorted_queue: Vec<&str> = queue.drain(..).collect();
    sorted_queue.sort();
    queue.extend(sorted_queue);

    let mut result: Vec<String> = Vec::with_capacity(node_ids.len());
    while let Some(node_id) = queue.pop_front() {
        result.push(node_id.to_string());
        let mut neighbors: Vec<&str> = adjacency[node_id].clone();
        neighbors.sort(); // deterministic
        for neighbor in neighbors {
            let deg = in_degree.get_mut(neighbor).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    if result.len() != node_ids.len() {
        return Err(PlanError::CycleDetected);
    }

    Ok(result)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computation::EmitCollectionFamily;
    use crate::computation::{Cardinality, PureBody, TransportKind, TypedPort};

    #[test]
    fn emit_plan_round_trip() {
        // Verify that EmitPlan can be constructed, cloned, and debug-printed.
        let plan = EmitPlan {
            steps: vec![
                EmitStep {
                    node_id: "load_registry".into(),
                    computation: Computation::Pure {
                        inputs: vec![],
                        outputs: vec![TypedPort {
                            name: "registry".into(),
                            abstract_type: "ToolRegistry".into(),
                            cardinality: Cardinality::Scalar,
                        }],
                        body: PureBody::Literal(serde_json::Value::Null),
                    },
                    input_sources: vec![],
                    output_bindings: vec![OutputBinding {
                        port: "registry".into(),
                        consumers: vec![(1, "registry".into())],
                    }],
                },
                EmitStep {
                    node_id: "render_makefile".into(),
                    computation: Computation::Pure {
                        inputs: vec![TypedPort {
                            name: "registry".into(),
                            abstract_type: "ToolRegistry".into(),
                            cardinality: Cardinality::Scalar,
                        }],
                        outputs: vec![TypedPort {
                            name: "return".into(),
                            abstract_type: "String".into(),
                            cardinality: Cardinality::Scalar,
                        }],
                        body: PureBody::Template {
                            pattern: "render_makefile".into(),
                            vars: vec!["registry".into()],
                        },
                    },
                    input_sources: vec![InputBinding::FromStep {
                        step_index: 0,
                        port: "registry".into(),
                    }],
                    output_bindings: vec![],
                },
            ],
            entrypoints: vec![],
            transport_nodes: vec![],
        };

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].node_id, "load_registry");
        assert_eq!(plan.steps[1].node_id, "render_makefile");

        // FromStep binding points back to step 0.
        assert_eq!(
            plan.steps[1].input_sources[0],
            InputBinding::FromStep {
                step_index: 0,
                port: "registry".into()
            }
        );
    }

    #[test]
    fn emit_plan_with_transport_and_entrypoint() {
        let plan = EmitPlan {
            steps: vec![EmitStep {
                node_id: "execute_read".into(),
                computation: Computation::Transport {
                    prepare: crate::computation::RequestSpec {
                        input_ports: vec!["request".into()],
                        kind: crate::computation::RequestKind::FilePath {
                            path_port: "request".into(),
                        },
                    },
                    execute: TransportKind::FileRead,
                    parse: crate::computation::ResponseSpec {
                        output_ports: vec!["response".into()],
                        kind: crate::computation::ResponseKind::RawContent,
                    },
                },
                input_sources: vec![InputBinding::FromEntrypoint {
                    port: "path".into(),
                }],
                output_bindings: vec![],
            }],
            entrypoints: vec![EntrypointPort {
                name: "path".into(),
                abstract_type: "String".into(),
                consumers: vec![(0, "request".into())],
            }],
            transport_nodes: vec!["execute_read".into()],
        };

        assert_eq!(plan.transport_nodes.len(), 1);
        assert_eq!(plan.entrypoints.len(), 1);
        assert_eq!(plan.entrypoints[0].name, "path");
    }

    #[test]
    fn input_binding_constant() {
        let binding = InputBinding::Constant(serde_json::json!({"key": "value"}));
        assert!(matches!(binding, InputBinding::Constant(_)));
    }

    #[test]
    fn plan_error_display() {
        let err = PlanError::MissingPort {
            node_id: "render_makefile".into(),
            port: "missing".into(),
            direction: "input",
        };
        let msg = err.to_string();
        assert!(msg.contains("render_makefile"));
        assert!(msg.contains("missing"));
    }

    #[test]
    fn emit_plan_collection_step() {
        let plan = EmitPlan {
            steps: vec![EmitStep {
                node_id: "map_items".into(),
                computation: Computation::Collection {
                    family: EmitCollectionFamily::Map,
                    element_type: "String".into(),
                },
                input_sources: vec![InputBinding::FromEntrypoint {
                    port: "items".into(),
                }],
                output_bindings: vec![OutputBinding {
                    port: "mapped".into(),
                    consumers: vec![],
                }],
            }],
            entrypoints: vec![EntrypointPort {
                name: "items".into(),
                abstract_type: "String".into(),
                consumers: vec![(0, "items".into())],
            }],
            transport_nodes: vec![],
        };

        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(
            &plan.steps[0].computation,
            Computation::Collection {
                family: EmitCollectionFamily::Map,
                ..
            }
        ));
    }

    // -- A2.3: makegen DAG → EmitPlan in topo order --

    use daglang_derive::derive_artifacts;
    use daglang_lower::{CallableKind, CallableObligation, LoweredOp, PrimitiveOpKind};
    use gunbc_ir::{Dag, Edge, Node, Port};

    /// Build a realistic makegen DAG matching the content_upsert pattern.
    ///
    /// Nodes (9 total — similar to production makegen):
    ///   0. load_registry       (Pure: Literal)
    ///   1. render_makefile     (Pure: Template)     ← load_registry
    ///   2. makegen             (Pure: entrypoint)    ← (external path input)
    ///   3. prepare_read_makegen (Pure: Literal)      ← makegen.path
    ///   4. execute_read_makegen (Transport: FileRead) ← prepare_read
    ///   5. compare_makegen_content (Pure: Compare)   ← render_makefile, execute_read
    ///   6. prepare_write_makegen (Pure: Literal)     ← render_makefile, makegen.path
    ///   7. execute_makegen_transport (Transport: FileWrite) ← prepare_write, compare
    fn build_makegen_dag() -> Dag<LoweredOp> {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "load_registry".into(),
                obligation: CallableObligation::PureDataLoad,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "render_makefile",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("content", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "render_makefile".into(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "makegen",
            vec![
                Port::scalar("path", "String"),
                Port::scalar("__out:path_out", "String"),
                Port::scalar("__out:written", "Bool"),
            ],
            vec![
                Port::scalar("path_out", "String"),
                Port::scalar("written", "Bool"),
            ],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Func,
                name: "makegen".into(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_read_makegen",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_read_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoPrepareFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_read_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "compare_makegen_content",
            vec![
                Port::scalar("expected_content", "String"),
                Port::scalar("response", "TransportResponse"),
            ],
            vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::compare_makegen_content".into(),
                kind: daglang_lower::PrimitiveOpKind::CompareEquality,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_write_makegen",
            vec![
                Port::scalar("content", "String"),
                Port::scalar("path", "String"),
            ],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_write_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoPrepareFileWrite,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_makegen_transport",
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_makegen_transport".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileWrite,
            },
        ));

        // Edges
        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "render_makefile",
            "registry",
        ));
        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "makegen",
            "__out:path_out",
        ));
        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "makegen",
            "__out:written",
        ));
        dag.add_edge(Edge::new(
            "makegen",
            "path_out",
            "prepare_read_makegen",
            "path",
        ));
        dag.add_edge(Edge::new(
            "prepare_read_makegen",
            "request",
            "execute_read_makegen",
            "request",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "content",
            "compare_makegen_content",
            "expected_content",
        ));
        dag.add_edge(Edge::new(
            "execute_read_makegen",
            "response",
            "compare_makegen_content",
            "response",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "content",
            "prepare_write_makegen",
            "content",
        ));
        dag.add_edge(Edge::new(
            "makegen",
            "path_out",
            "prepare_write_makegen",
            "path",
        ));
        dag.add_edge(Edge::new(
            "prepare_write_makegen",
            "request",
            "execute_makegen_transport",
            "request",
        ));
        dag.add_edge(Edge::new(
            "compare_makegen_content",
            "skip",
            "execute_makegen_transport",
            "skip",
        ));

        dag
    }

    #[test]
    fn build_makegen_emit_plan_topo_order() {
        let dag = build_makegen_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let plan = build_emit_plan(&dag, &artifacts).expect("build_emit_plan should succeed");

        // 8 nodes total.
        assert_eq!(
            plan.steps.len(),
            8,
            "expected 8 steps, got {}",
            plan.steps.len()
        );

        // Verify topo ordering: every FromStep reference points backward.
        for (i, step) in plan.steps.iter().enumerate() {
            for binding in &step.input_sources {
                if let InputBinding::FromStep { step_index, .. } = binding {
                    assert!(
                        *step_index < i,
                        "step {} ({}) references future step {} — not topo-ordered",
                        i,
                        step.node_id,
                        step_index
                    );
                }
            }
        }

        // Transport nodes identified.
        assert_eq!(
            plan.transport_nodes.len(),
            2,
            "expected 2 transport nodes (execute_read + execute_write), got {:?}",
            plan.transport_nodes
        );
        assert!(plan
            .transport_nodes
            .contains(&"execute_read_makegen".to_string()));
        assert!(plan
            .transport_nodes
            .contains(&"execute_makegen_transport".to_string()));

        // Entrypoint: "path" from the `makegen` node.
        assert!(
            !plan.entrypoints.is_empty(),
            "expected at least one entrypoint"
        );
    }

    #[test]
    fn build_emit_plan_fails_fast_on_interpreter_only_primitive() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "extract_field",
            vec![Port::scalar("record", "Json")],
            vec![Port::scalar("value", "String")],
            LoweredOp::Primitive {
                module: "test".into(),
                name: "extract_field".into(),
                kind: PrimitiveOpKind::GetField {
                    field: "name".into(),
                    input_port: "record".into(),
                },
            },
        ));
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");

        let error = build_emit_plan(&dag, &artifacts).expect_err("interpreter-only ops must fail");

        assert!(matches!(
            error,
            PlanError::ClassifyFailed {
                ref node_id,
                ref detail,
            } if node_id == "extract_field"
                && detail.contains("GetField(name) is interpreter-only and cannot be emitted")
        ));
    }

    #[test]
    fn build_emit_plan_rejects_unwired_internal_passthrough_inputs() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "surface",
            vec![
                Port::scalar("path", "String"),
                Port::scalar("__out:path_out", "String"),
            ],
            vec![Port::scalar("path_out", "String")],
            LoweredOp::Callable {
                module: "test".into(),
                kind: CallableKind::Func,
                name: "surface".into(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");

        let error =
            build_emit_plan(&dag, &artifacts).expect_err("missing __out: binding must fail");

        assert!(matches!(
            error,
            PlanError::MissingInternalInputBinding { ref node_id, ref port }
                if node_id == "surface" && port == "__out:path_out"
        ));
    }

    #[test]
    fn build_makegen_plan_data_flow_wiring() {
        let dag = build_makegen_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let plan = build_emit_plan(&dag, &artifacts).expect("build_emit_plan should succeed");

        // Find specific steps by node_id.
        let step_idx = |name: &str| -> usize {
            plan.steps
                .iter()
                .position(|s| s.node_id == name)
                .unwrap_or_else(|| panic!("step `{name}` not found"))
        };

        let load_idx = step_idx("load_registry");
        let render_idx = step_idx("render_makefile");
        let prep_read_idx = step_idx("prepare_read_makegen");
        let exec_read_idx = step_idx("execute_read_makegen");
        let compare_idx = step_idx("compare_makegen_content");

        // render_makefile gets registry from load_registry.
        assert!(
            plan.steps[render_idx]
                .input_sources
                .contains(&InputBinding::FromStep {
                    step_index: load_idx,
                    port: "registry".into(),
                }),
            "render_makefile should get registry from load_registry"
        );

        // execute_read gets request from prepare_read.
        assert!(
            plan.steps[exec_read_idx]
                .input_sources
                .contains(&InputBinding::FromStep {
                    step_index: prep_read_idx,
                    port: "request".into(),
                }),
            "execute_read should get request from prepare_read"
        );

        // compare gets response from execute_read.
        assert!(
            plan.steps[compare_idx]
                .input_sources
                .iter()
                .any(|b| matches!(b, InputBinding::FromStep { step_index, port }
                    if *step_index == exec_read_idx && port == "response")),
            "compare should get response from execute_read"
        );
    }

    // -- A2.4: pragma DAG with 3 parallel chains → EmitPlan --

    /// Build a pragma-like DAG with 3 parallel content_upsert chains.
    ///
    /// Chains: clippy, allowlist, lint_policy — each has:
    ///   render_X → prepare_read_X → execute_read_X → compare_X → prepare_write_X → execute_X_transport
    fn build_pragma_dag() -> Dag<LoweredOp> {
        let mut dag = Dag::new();

        // Shared load_registry.
        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "pragma".into(),
                kind: CallableKind::Fn,
                name: "load_registry".into(),
                obligation: CallableObligation::PureDataLoad,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));

        for chain in &["clippy", "allowlist", "lint_policy"] {
            let render_id = format!("render_{chain}");
            let prep_read_id = format!("prepare_read_{chain}");
            let exec_read_id = format!("execute_read_{chain}");
            let compare_id = format!("compare_{chain}_content");
            let prep_write_id = format!("prepare_write_{chain}");
            let exec_transport_id = format!("execute_{chain}_transport");

            dag.add_node(Node::opaque(
                render_id.as_str(),
                vec![Port::scalar("registry", "ToolRegistry")],
                vec![Port::scalar("content", "String")],
                LoweredOp::Callable {
                    module: "pragma".into(),
                    kind: CallableKind::Fn,
                    name: render_id.clone(),
                    obligation: CallableObligation::PureRender,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            ));
            dag.add_node(Node::opaque(
                prep_read_id.as_str(),
                vec![Port::scalar("path", "String")],
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Primitive {
                    module: "pragma".into(),
                    name: format!("content_upsert::{prep_read_id}"),
                    kind: PrimitiveOpKind::IoPrepareFileRead,
                },
            ));
            dag.add_node(Node::opaque(
                exec_read_id.as_str(),
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Primitive {
                    module: "pragma".into(),
                    name: format!("content_upsert::{exec_read_id}"),
                    kind: PrimitiveOpKind::IoExecuteFileRead,
                },
            ));
            dag.add_node(Node::opaque(
                compare_id.as_str(),
                vec![
                    Port::scalar("expected_content", "String"),
                    Port::scalar("response", "TransportResponse"),
                ],
                vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
                LoweredOp::Primitive {
                    module: "pragma".into(),
                    name: format!("content_upsert::{compare_id}"),
                    kind: PrimitiveOpKind::CompareEquality,
                },
            ));
            dag.add_node(Node::opaque(
                prep_write_id.as_str(),
                vec![
                    Port::scalar("content", "String"),
                    Port::scalar("path", "String"),
                ],
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Primitive {
                    module: "pragma".into(),
                    name: format!("content_upsert::{prep_write_id}"),
                    kind: PrimitiveOpKind::IoPrepareFileWrite,
                },
            ));
            dag.add_node(Node::opaque(
                exec_transport_id.as_str(),
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                ],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Primitive {
                    module: "pragma".into(),
                    name: format!("content_upsert::{exec_transport_id}"),
                    kind: PrimitiveOpKind::IoExecuteFileWrite,
                },
            ));

            // Edges within chain.
            dag.add_edge(Edge::new(
                "load_registry",
                "registry",
                render_id.as_str(),
                "registry",
            ));
            dag.add_edge(Edge::new(
                prep_read_id.as_str(),
                "request",
                exec_read_id.as_str(),
                "request",
            ));
            dag.add_edge(Edge::new(
                render_id.as_str(),
                "content",
                compare_id.as_str(),
                "expected_content",
            ));
            dag.add_edge(Edge::new(
                exec_read_id.as_str(),
                "response",
                compare_id.as_str(),
                "response",
            ));
            dag.add_edge(Edge::new(
                render_id.as_str(),
                "content",
                prep_write_id.as_str(),
                "content",
            ));
            dag.add_edge(Edge::new(
                prep_write_id.as_str(),
                "request",
                exec_transport_id.as_str(),
                "request",
            ));
            dag.add_edge(Edge::new(
                compare_id.as_str(),
                "skip",
                exec_transport_id.as_str(),
                "skip",
            ));
        }

        dag
    }

    #[test]
    fn build_pragma_emit_plan_three_chains() {
        let dag = build_pragma_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let plan = build_emit_plan(&dag, &artifacts).expect("build_emit_plan should succeed");

        // 1 shared load_registry + 3 chains × 6 nodes = 19 total.
        assert_eq!(
            plan.steps.len(),
            19,
            "expected 19 steps, got {}",
            plan.steps.len()
        );

        // Verify topo ordering.
        for (i, step) in plan.steps.iter().enumerate() {
            for binding in &step.input_sources {
                if let InputBinding::FromStep { step_index, .. } = binding {
                    assert!(
                        *step_index < i,
                        "step {} ({}) references future step {} — not topo-ordered",
                        i,
                        step.node_id,
                        step_index
                    );
                }
            }
        }

        // 6 transport nodes (2 per chain: execute_read + execute_transport).
        assert_eq!(
            plan.transport_nodes.len(),
            6,
            "expected 6 transport nodes, got {:?}",
            plan.transport_nodes
        );

        // load_registry must come before all render_* nodes.
        let load_idx = plan
            .steps
            .iter()
            .position(|s| s.node_id == "load_registry")
            .unwrap();
        for chain in &["clippy", "allowlist", "lint_policy"] {
            let render_idx = plan
                .steps
                .iter()
                .position(|s| s.node_id == format!("render_{chain}"))
                .unwrap();
            assert!(
                load_idx < render_idx,
                "load_registry (idx={load_idx}) should come before render_{chain} (idx={render_idx})"
            );
        }

        // Within each chain, ordering must be:
        //   render → compare → execute_transport (via prepare_write)
        for chain in &["clippy", "allowlist", "lint_policy"] {
            let render_idx = plan
                .steps
                .iter()
                .position(|s| s.node_id == format!("render_{chain}"))
                .unwrap();
            let compare_idx = plan
                .steps
                .iter()
                .position(|s| s.node_id == format!("compare_{chain}_content"))
                .unwrap();
            let exec_idx = plan
                .steps
                .iter()
                .position(|s| s.node_id == format!("execute_{chain}_transport"))
                .unwrap();
            assert!(
                render_idx < compare_idx,
                "render_{chain} should precede compare_{chain}_content"
            );
            assert!(
                compare_idx < exec_idx,
                "compare_{chain}_content should precede execute_{chain}_transport"
            );
        }
    }
}
