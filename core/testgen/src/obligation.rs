//! Proof obligation IR for test generation.
//!
//! A proof obligation represents a property that **cannot be fully discharged**
//! by compile-time validation. Only obligations that are "Unknown" or
//! "runtime-only" produce tests — this is the anti-tautology rule:
//!
//! > Only generate tests for obligations that are not fully discharged by
//! > compile-time validation, plus runtime/executor invariants that cannot
//! > be statically guaranteed.
//!
//! # Obligation Buckets
//!
//! | Bucket | What It Proves | Source |
//! |--------|---------------|--------|
//! | **A: Execution Semantics** | Executor/boundary model correctness | Framework |
//! | **B: Contract Obligations** | Semantic compatibility when proof engine can't decide | Contract tower |
//! | **C: Scenario Coverage** | Graph + transport mock behavior | Graph structure |
//! | **D: Resource Hygiene** | Resource capability wiring correctness | Resource model |
//!
//! # Design Principle
//!
//! The generator:
//! 1. Runs the validator/analyzer
//! 2. Collects only obligations that are Unknown / runtime-only
//! 3. Produces tests
//!
//! This gives "ALL deducible non-tautological tests" by construction.

use gunbc_ir::resource::{detect_conflicts, ResourceAccess, ResourceConflict};
use gunbc_ir::types::{NodeId, PortName, TypeId};
use gunbc_ir::{contract, Dag, TypeRegistry};

// ---------------------------------------------------------------------------
// Obligation IR
// ---------------------------------------------------------------------------

/// A proof obligation: something that must be empirically verified because
/// compile-time validation cannot fully discharge it.
#[derive(Debug, Clone)]
pub struct ProofObligation {
    /// What needs to be proven.
    pub kind: Obligation,
    /// Why this obligation exists (human-readable).
    pub reason: String,
    /// Where this obligation came from.
    pub source: ObligationSource,
    /// Whether this obligation was discharged statically.
    /// Only `Unknown` and `RuntimeOnly` obligations produce tests.
    pub status: DischargeStatus,
}

impl ProofObligation {
    /// Create a new undischarged obligation.
    pub fn new(kind: Obligation, reason: impl Into<String>, source: ObligationSource) -> Self {
        Self {
            kind,
            reason: reason.into(),
            source,
            status: DischargeStatus::Unknown,
        }
    }

    /// Create a runtime-only obligation (cannot be statically discharged).
    pub fn runtime(kind: Obligation, reason: impl Into<String>, source: ObligationSource) -> Self {
        Self {
            kind,
            reason: reason.into(),
            source,
            status: DischargeStatus::RuntimeOnly,
        }
    }

    /// Mark as statically discharged (no test needed).
    pub fn discharge(mut self, proof: impl Into<String>) -> Self {
        self.status = DischargeStatus::Verified {
            proof: proof.into(),
        };
        self
    }

    /// Whether this obligation needs a test.
    pub fn needs_test(&self) -> bool {
        matches!(
            self.status,
            DischargeStatus::Unknown | DischargeStatus::RuntimeOnly
        )
    }
}

/// The specific obligation to prove.
#[derive(Debug, Clone)]
pub enum Obligation {
    // -----------------------------------------------------------------------
    // Bucket A: Execution Semantics (framework-level, not graph tautology)
    //
    // These validate the executor/boundary model against the graph.
    // Static typing cannot prove "DryRun truly intercepts all transports."
    // -----------------------------------------------------------------------
    /// Every transport executor is interceptable in DryRun.
    TransportInterceptable {
        node_id: NodeId,
    },

    /// DryRun completes without crash for the full workflow.
    DryRunCompletion,

    /// Pure nodes are deterministic: same inputs + same mocks → same outputs.
    PureNodeDeterminism {
        node_id: NodeId,
    },

    // -----------------------------------------------------------------------
    // Bucket B: Contract Obligations (graph-specific, high value)
    //
    // Generate tests only when the type/contract system can't fully prove
    // something. The big win: L3 "Unknown" entailments.
    // -----------------------------------------------------------------------
    /// Edge predicate entailment is Unknown — need empirical test.
    ///
    /// For a given edge, source contract values must satisfy target contract.
    /// When the proof engine returns Unknown, we discharge by testing.
    EdgePredicateEntailment {
        from_node: NodeId,
        from_port: PortName,
        to_node: NodeId,
        to_port: PortName,
        from_type: TypeId,
        to_type: TypeId,
        entailment: EntailmentStatus,
    },

    /// Witness values are accepted by downstream contracts (L4 compat).
    ///
    /// Validates that mock/witness values used in testing are actually
    /// accepted by the contracts they're meant to satisfy.
    WitnessCompatibility {
        node_id: NodeId,
        port_name: PortName,
        type_id: TypeId,
    },

    /// Node contract compliance: given valid inputs, outputs satisfy contracts.
    ///
    /// Non-tautological because the node's implementation can be wrong
    /// even if the wiring is correct.
    NodeContractCompliance {
        node_id: NodeId,
    },

    // -----------------------------------------------------------------------
    // Bucket C: Scenario Coverage (graph + transport mocks)
    //
    // "N+1 instead of 2^N": one all-succeed + one per-transport failure.
    // -----------------------------------------------------------------------
    /// Happy path: all transports succeed, workflow reaches terminal outputs.
    AllTransportsSucceed,

    /// Single transport fails: verify failure propagation semantics.
    SingleTransportFailure {
        node_id: NodeId,
    },

    /// Skip-path propagation: when upstream skips, downstream handles it.
    SkipPathPropagation {
        /// The node whose failure/skip triggers downstream effects.
        trigger_node: NodeId,
    },

    /// Guard/skip branch coverage: for each node with a guarded input,
    /// generate two scenarios (guard passes, guard fails → skip).
    ///
    /// The executor implements "skip ⇒ all outputs are Value::Skipped",
    /// so we verify both paths produce valid results.
    GuardBranchCoverage {
        /// The node with the guarded input.
        node_id: NodeId,
        /// The guarded input port.
        guard_port: PortName,
    },

    // -----------------------------------------------------------------------
    // Bucket D: Resource Hygiene (structural, capability-grant model)
    //
    // In the capability-grant model, resources are input ports and edges.
    // These tests verify the structural wiring is correct.
    // -----------------------------------------------------------------------
    /// Every resource/tool input port has an incoming edge.
    ResourceInputConnected {
        node_id: NodeId,
        port_name: PortName,
    },

    /// Resource owners (providers) are valid env/acquisition nodes.
    ResourceOwnerValid {
        node_id: NodeId,
    },

    /// No orphan resources: acquired resources are consumed by someone.
    ResourceOrphan {
        node_id: NodeId,
        port_name: PortName,
    },

    /// Resource conflicts are absent (DAG ordering prevents parallel conflicts).
    ResourceConflictAbsence {
        conflicts: Vec<ResourceConflict>,
    },

    /// Resource contention handling: consumer handles failed acquisition.
    ResourceContentionHandling {
        resource_port: String,
    },

    /// Resource simulation: acquisition/timeout behavior.
    ResourceSimulation {
        resource_id: String,
    },
}

/// Status of predicate entailment checking.
#[derive(Debug, Clone)]
pub enum EntailmentStatus {
    /// Statically verified: source predicates imply target predicates.
    Verified,
    /// Statically disproven: source predicates do NOT imply target.
    Invalid { reason: String },
    /// Cannot determine statically — need empirical test.
    Unknown { reason: String },
}

/// Where an obligation came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationSource {
    /// Derived from graph structure (edges, ports, connectivity).
    Structure,
    /// From contract tower analysis (L1-L4).
    Contract,
    /// From execution model (DryRun, transport interception).
    ExecutionModel,
    /// From resource system (capabilities, conflicts).
    ResourceModel,
}

/// Whether a proof obligation has been discharged.
#[derive(Debug, Clone)]
pub enum DischargeStatus {
    /// Statically verified — no test needed.
    Verified { proof: String },
    /// Cannot be determined statically — needs test.
    Unknown,
    /// Can only be verified at runtime by design — needs test.
    RuntimeOnly,
}

// ---------------------------------------------------------------------------
// Obligation Collector
// ---------------------------------------------------------------------------

/// Result of obligation collection.
#[derive(Debug)]
pub struct ObligationSet {
    /// All collected obligations (including discharged ones).
    pub all: Vec<ProofObligation>,
}

impl ObligationSet {
    /// Get only obligations that need tests (not statically discharged).
    pub fn testable(&self) -> Vec<&ProofObligation> {
        self.all.iter().filter(|o| o.needs_test()).collect()
    }

    /// Get obligations by bucket.
    pub fn bucket_a(&self) -> Vec<&ProofObligation> {
        self.testable()
            .into_iter()
            .filter(|o| o.source == ObligationSource::ExecutionModel)
            .collect()
    }

    pub fn bucket_b(&self) -> Vec<&ProofObligation> {
        self.testable()
            .into_iter()
            .filter(|o| o.source == ObligationSource::Contract)
            .collect()
    }

    pub fn bucket_c(&self) -> Vec<&ProofObligation> {
        self.testable()
            .into_iter()
            .filter(|o| {
                o.source == ObligationSource::Structure
                    && matches!(
                        o.kind,
                        Obligation::AllTransportsSucceed
                            | Obligation::SingleTransportFailure { .. }
                            | Obligation::SkipPathPropagation { .. }
                            | Obligation::GuardBranchCoverage { .. }
                    )
            })
            .collect()
    }

    pub fn bucket_d(&self) -> Vec<&ProofObligation> {
        self.testable()
            .into_iter()
            .filter(|o| o.source == ObligationSource::ResourceModel)
            .collect()
    }

    /// Summary statistics.
    pub fn stats(&self) -> ObligationStats {
        let testable = self.testable();
        ObligationStats {
            total: self.all.len(),
            discharged: self.all.len() - testable.len(),
            testable: testable.len(),
            bucket_a: self.bucket_a().len(),
            bucket_b: self.bucket_b().len(),
            bucket_c: self.bucket_c().len(),
            bucket_d: self.bucket_d().len(),
        }
    }
}

/// Summary statistics for obligations.
#[derive(Debug)]
pub struct ObligationStats {
    pub total: usize,
    pub discharged: usize,
    pub testable: usize,
    pub bucket_a: usize,
    pub bucket_b: usize,
    pub bucket_c: usize,
    pub bucket_d: usize,
}

impl std::fmt::Display for ObligationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} obligations ({} discharged, {} testable: A={}, B={}, C={}, D={})",
            self.total,
            self.discharged,
            self.testable,
            self.bucket_a,
            self.bucket_b,
            self.bucket_c,
            self.bucket_d
        )
    }
}

// ---------------------------------------------------------------------------
// Collector: analyze DAG → produce obligations
// ---------------------------------------------------------------------------

/// Collect all proof obligations from a DAG.
///
/// This is the core "proof obligation emitter": it analyzes the DAG and
/// produces obligations for every non-tautological property that needs testing.
///
/// Obligations that the compiler already proves (type compatibility, cardinality
/// satisfaction, acyclicity) are either omitted or marked as `Verified`.
pub fn collect_obligations<T>(
    dag: &Dag<T>,
    registry: Option<&TypeRegistry>,
    resource_accesses: Option<&[ResourceAccess]>,
) -> ObligationSet {
    let mut obligations = Vec::new();

    // Bucket A: Execution semantics
    collect_execution_obligations(dag, &mut obligations);

    // Bucket B: Contract obligations
    collect_contract_obligations(dag, registry, &mut obligations);

    // Bucket C: Scenario coverage
    collect_scenario_obligations(dag, &mut obligations);

    // Bucket D: Resource hygiene
    collect_resource_obligations(dag, resource_accesses, &mut obligations);

    ObligationSet { all: obligations }
}

/// Bucket A: Execution semantics obligations.
fn collect_execution_obligations<T>(dag: &Dag<T>, obligations: &mut Vec<ProofObligation>) {
    // A.1: DryRun completion — always needed (runtime-only)
    obligations.push(ProofObligation::runtime(
        Obligation::DryRunCompletion,
        "DryRun execution must complete without crash",
        ObligationSource::ExecutionModel,
    ));

    // A.2: Transport interception — one per transport executor
    for node in &dag.nodes {
        let is_transport = node
            .inputs
            .iter()
            .any(|p| p.type_id.0 == "TransportRequest");
        if is_transport {
            obligations.push(ProofObligation::runtime(
                Obligation::TransportInterceptable {
                    node_id: node.id.clone(),
                },
                format!(
                    "Transport executor '{}' must be interceptable in DryRun",
                    node.id.0
                ),
                ObligationSource::ExecutionModel,
            ));
        }
    }

    // A.3: Pure node determinism — one per pure node
    for node in &dag.nodes {
        let is_transport = node
            .inputs
            .iter()
            .any(|p| p.type_id.0 == "TransportRequest");
        let is_tool_env = node
            .outputs
            .iter()
            .any(|p| p.type_id.0 == "ToolHandle");
        let consumes_tool = node.inputs.iter().any(|p| p.type_id.0 == "ToolHandle");

        if !is_transport && !is_tool_env && !consumes_tool {
            obligations.push(ProofObligation::runtime(
                Obligation::PureNodeDeterminism {
                    node_id: node.id.clone(),
                },
                format!(
                    "Pure node '{}' must be deterministic (same inputs → same outputs)",
                    node.id.0
                ),
                ObligationSource::ExecutionModel,
            ));
        }
    }
}

/// Bucket B: Contract obligations.
fn collect_contract_obligations<T>(
    dag: &Dag<T>,
    registry: Option<&TypeRegistry>,
    obligations: &mut Vec<ProofObligation>,
) {
    // B.1: Edge predicate entailment
    for edge in &dag.edges {
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_port = from.outputs.iter().find(|p| p.name == edge.from_port);
            let to_port = to.inputs.iter().find(|p| p.name == edge.to_port);

            if let (Some(fp), Some(tp)) = (from_port, to_port) {
                // L1 + L2 are statically verified (type/cardinality compat)
                // We only care about L3: predicate entailment
                let entailment = check_predicate_entailment(
                    &fp.type_id,
                    &tp.type_id,
                    registry,
                );

                let edge_label = format!(
                    "{}.{} → {}.{}",
                    edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
                );

                let (reason, discharged) = match &entailment {
                    EntailmentStatus::Verified => {
                        (format!("Edge {}: predicate entailment verified", edge_label), true)
                    }
                    EntailmentStatus::Unknown { reason } => {
                        (format!("Edge {}: predicate entailment unknown ({})", edge_label, reason), false)
                    }
                    EntailmentStatus::Invalid { reason } => {
                        (format!("Edge {}: predicate entailment INVALID ({})", edge_label, reason), false)
                    }
                };

                let obligation = ProofObligation::new(
                    Obligation::EdgePredicateEntailment {
                        from_node: edge.from_node.clone(),
                        from_port: edge.from_port.clone(),
                        to_node: edge.to_node.clone(),
                        to_port: edge.to_port.clone(),
                        from_type: fp.type_id.clone(),
                        to_type: tp.type_id.clone(),
                        entailment,
                    },
                    reason,
                    ObligationSource::Contract,
                );

                if discharged {
                    obligations.push(obligation.discharge("Predicate entailment statically verified"));
                } else {
                    obligations.push(obligation);
                }
            }
        }
    }

    // B.2: Node contract compliance — one per node with inputs/outputs
    for node in &dag.nodes {
        if !node.inputs.is_empty() && !node.outputs.is_empty() {
            obligations.push(ProofObligation::runtime(
                Obligation::NodeContractCompliance {
                    node_id: node.id.clone(),
                },
                format!(
                    "Node '{}': given valid inputs, outputs must satisfy contracts",
                    node.id.0
                ),
                ObligationSource::Contract,
            ));
        }
    }
}

/// Bucket C: Scenario coverage obligations.
fn collect_scenario_obligations<T>(dag: &Dag<T>, obligations: &mut Vec<ProofObligation>) {
    let transport_executors: Vec<&NodeId> = dag
        .nodes
        .iter()
        .filter(|n| {
            n.inputs
                .iter()
                .any(|p| p.type_id.0 == "TransportRequest")
        })
        .map(|n| &n.id)
        .collect();

    if transport_executors.is_empty() {
        return;
    }

    // C.1: All transports succeed scenario
    obligations.push(ProofObligation::runtime(
        Obligation::AllTransportsSucceed,
        "Happy path: all transports succeed, workflow reaches terminal outputs",
        ObligationSource::Structure,
    ));

    // C.2: Single transport failure scenarios (N tests, not 2^N)
    for node_id in &transport_executors {
        obligations.push(ProofObligation::runtime(
            Obligation::SingleTransportFailure {
                node_id: (*node_id).clone(),
            },
            format!(
                "Failure at '{}': verify failure propagation semantics",
                node_id.0
            ),
            ObligationSource::Structure,
        ));
    }

    // C.3: Skip-path propagation — for each transport executor that has
    // downstream nodes, verify skip propagation when it fails.
    for node_id in &transport_executors {
        // Check if this node has downstream nodes
        let has_downstream = dag
            .edges
            .iter()
            .any(|e| &e.from_node == *node_id);

        if has_downstream {
            obligations.push(ProofObligation::runtime(
                Obligation::SkipPathPropagation {
                    trigger_node: (*node_id).clone(),
                },
                format!(
                    "Skip propagation: when '{}' fails, downstream nodes handle correctly",
                    node_id.0
                ),
                ObligationSource::Structure,
            ));
        }
    }

    // C.4: Guard/skip branch coverage — for each node with a guarded input,
    // generate two scenarios: guard passes (node executes) and guard fails
    // (node skips, all outputs become Value::Skipped).
    for node in &dag.nodes {
        for port in &node.inputs {
            if port.has_guard() {
                obligations.push(ProofObligation::runtime(
                    Obligation::GuardBranchCoverage {
                        node_id: node.id.clone(),
                        guard_port: port.name.clone(),
                    },
                    format!(
                        "Guard branch: node '{}' has guarded port '{}' — test both skip=true and skip=false paths",
                        node.id.0, port.name.0
                    ),
                    ObligationSource::Structure,
                ));
            }
        }
    }
}

/// Bucket D: Resource hygiene obligations.
fn collect_resource_obligations<T>(
    dag: &Dag<T>,
    resource_accesses: Option<&[ResourceAccess]>,
    obligations: &mut Vec<ProofObligation>,
) {
    // D.1: Resource input connectivity — every resource/tool input has an edge
    for node in &dag.nodes {
        for port in &node.inputs {
            let is_resource = port.name.0.starts_with("resource:")
                || port.name.0.starts_with("tool:")
                || port.type_id.0 == "ToolHandle"
                || port.type_id.0 == "Lock"
                || port.type_id.0 == "Lease"
                || port.type_id.0 == "SharedLock";

            if is_resource {
                let has_edge = dag
                    .edges
                    .iter()
                    .any(|e| e.to_node == node.id && e.to_port == port.name);

                if has_edge {
                    obligations.push(
                        ProofObligation::new(
                            Obligation::ResourceInputConnected {
                                node_id: node.id.clone(),
                                port_name: port.name.clone(),
                            },
                            format!(
                                "Resource input {}.{} is connected",
                                node.id.0, port.name.0
                            ),
                            ObligationSource::ResourceModel,
                        )
                        .discharge("Edge exists to resource input"),
                    );
                } else {
                    obligations.push(ProofObligation::new(
                        Obligation::ResourceInputConnected {
                            node_id: node.id.clone(),
                            port_name: port.name.clone(),
                        },
                        format!(
                            "Resource input {}.{} has NO incoming edge — resource not provided",
                            node.id.0, port.name.0
                        ),
                        ObligationSource::ResourceModel,
                    ));
                }
            }
        }
    }

    // D.2: Resource owner validity — nodes that output resources should be env/owner nodes
    for node in &dag.nodes {
        let outputs_resource = node.outputs.iter().any(|p| {
            p.name.0.starts_with("resource:")
                || p.name.0.starts_with("tool:")
                || p.type_id.0 == "ToolHandle"
                || p.type_id.0 == "Lock"
                || p.type_id.0 == "Lease"
                || p.type_id.0 == "SharedLock"
        });

        if outputs_resource {
            obligations.push(ProofObligation::runtime(
                Obligation::ResourceOwnerValid {
                    node_id: node.id.clone(),
                },
                format!(
                    "Resource owner '{}' is a valid acquisition node",
                    node.id.0
                ),
                ObligationSource::ResourceModel,
            ));
        }
    }

    // D.3: No orphan resources — resources acquired should be consumed
    for node in &dag.nodes {
        for port in &node.outputs {
            let is_resource = port.name.0.starts_with("resource:")
                || port.name.0.starts_with("tool:")
                || port.type_id.0 == "ToolHandle"
                || port.type_id.0 == "Lock"
                || port.type_id.0 == "Lease"
                || port.type_id.0 == "SharedLock";

            if is_resource {
                let has_consumer = dag
                    .edges
                    .iter()
                    .any(|e| e.from_node == node.id && e.from_port == port.name);

                if has_consumer {
                    obligations.push(
                        ProofObligation::new(
                            Obligation::ResourceOrphan {
                                node_id: node.id.clone(),
                                port_name: port.name.clone(),
                            },
                            format!(
                                "Resource output {}.{} has consumer",
                                node.id.0, port.name.0
                            ),
                            ObligationSource::ResourceModel,
                        )
                        .discharge("Edge exists from resource output to consumer"),
                    );
                } else {
                    obligations.push(ProofObligation::new(
                        Obligation::ResourceOrphan {
                            node_id: node.id.clone(),
                            port_name: port.name.clone(),
                        },
                        format!(
                            "Resource output {}.{} is acquired but never consumed (orphan)",
                            node.id.0, port.name.0
                        ),
                        ObligationSource::ResourceModel,
                    ));
                }
            }
        }
    }

    // D.4: Resource conflict absence
    if let Some(accesses) = resource_accesses {
        let conflicts = detect_conflicts(dag, accesses);
        if conflicts.is_empty() {
            obligations.push(
                ProofObligation::new(
                    Obligation::ResourceConflictAbsence {
                        conflicts: vec![],
                    },
                    "No resource conflicts detected",
                    ObligationSource::ResourceModel,
                )
                .discharge("detect_conflicts returned empty"),
            );
        } else {
            obligations.push(ProofObligation::new(
                Obligation::ResourceConflictAbsence {
                    conflicts: conflicts.clone(),
                },
                format!(
                    "{} resource conflict(s) detected — DAG ordering may be insufficient",
                    conflicts.len()
                ),
                ObligationSource::ResourceModel,
            ));
        }
    }

    // D.5: Resource contention handling — for each resource type input,
    // verify the consumer can handle contention (acquisition failure).
    for node in &dag.nodes {
        for port in &node.inputs {
            let is_resource = port.type_id.0 == "ToolHandle"
                || port.type_id.0 == "Lock"
                || port.type_id.0 == "Lease"
                || port.type_id.0 == "SharedLock";

            if is_resource {
                obligations.push(ProofObligation::runtime(
                    Obligation::ResourceContentionHandling {
                        resource_port: format!("{}.{}", node.id.0, port.name.0),
                    },
                    format!(
                        "Node '{}' handles contention on resource port '{}'",
                        node.id.0, port.name.0
                    ),
                    ObligationSource::ResourceModel,
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate entailment checker
// ---------------------------------------------------------------------------

/// Check whether source type predicates entail target type predicates.
///
/// This is the L3 check from the contract tower. When the proof engine
/// can't decide, it returns `Unknown` and testgen will produce a test.
///
/// Current implementation is conservative: if both types are registered and
/// have predicates, we attempt structural comparison. Otherwise → Unknown.
fn check_predicate_entailment(
    from_type: &TypeId,
    to_type: &TypeId,
    registry: Option<&TypeRegistry>,
) -> EntailmentStatus {
    // Same type → trivially verified
    if from_type.0 == to_type.0 {
        return EntailmentStatus::Verified;
    }

    // "Any" type → verified (accepts anything)
    if to_type.0 == "Any" || from_type.0 == "Any" {
        return EntailmentStatus::Verified;
    }

    // If we have a registry, check contracts
    let Some(reg) = registry else {
        return EntailmentStatus::Unknown {
            reason: "no type registry available".to_string(),
        };
    };

    let from_dag = reg.get_by_name(&from_type.0);
    let to_dag = reg.get_by_name(&to_type.0);

    match (from_dag, to_dag) {
        (Some(from_td), Some(to_td)) => {
            let from_preds = contract::predicates(from_td);
            let to_preds = contract::predicates(to_td);

            // No predicates on target → always satisfied
            if to_preds.is_empty() {
                return EntailmentStatus::Verified;
            }

            // No predicates on source but target has them → Unknown
            if from_preds.is_empty() && !to_preds.is_empty() {
                return EntailmentStatus::Unknown {
                    reason: format!(
                        "source '{}' has no predicates but target '{}' requires {:?}",
                        from_type.0, to_type.0, to_preds
                    ),
                };
            }

            // Both have predicates — check if source predicates subsume target
            // This is a conservative check: we only verify exact matches.
            // More sophisticated entailment (e.g., InRange subsumption) is future work.
            let all_target_covered = to_preds.iter().all(|tp| {
                from_preds.iter().any(|fp| format!("{:?}", fp) == format!("{:?}", tp))
            });

            if all_target_covered {
                EntailmentStatus::Verified
            } else {
                EntailmentStatus::Unknown {
                    reason: format!(
                        "source '{}' predicates {:?} may not entail target '{}' predicates {:?}",
                        from_type.0, from_preds, to_type.0, to_preds
                    ),
                }
            }
        }
        (None, _) | (_, None) => {
            // Type(s) not in registry — can't check, but L1/L2 compat was
            // already verified by the compiler if types match by name.
            // Since names differ and we can't look up the type, → Unknown.
            EntailmentStatus::Unknown {
                reason: format!(
                    "type(s) not in registry: from='{}', to='{}'",
                    from_type.0, to_type.0
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::*, Dag, Node, Port};

    #[test]
    fn test_collect_basic_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![Port::scalar("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("result", "String")],
            (),
        ));
        dag.add_edge(edge("source", "out", "sink", "in"));

        let obligations = collect_obligations(&dag, None, None);
        let stats = obligations.stats();

        // Should have obligations from all buckets
        assert!(stats.total > 0);
        assert!(stats.testable > 0);

        // Should have DryRun completion
        assert!(obligations.all.iter().any(|o| matches!(
            o.kind,
            Obligation::DryRunCompletion
        )));

        // Should have determinism for pure nodes
        assert!(obligations.all.iter().any(|o| matches!(
            o.kind,
            Obligation::PureNodeDeterminism { .. }
        )));

        // Should have node contract compliance
        assert!(obligations.all.iter().any(|o| matches!(
            o.kind,
            Obligation::NodeContractCompliance { .. }
        )));
    }

    #[test]
    fn test_transport_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![],
            vec![Port::scalar("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            (),
        ));
        dag.add_node(Node::opaque(
            "parse",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("result", "String")],
            (),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("execute", "response", "parse", "response"));

        let obligations = collect_obligations(&dag, None, None);

        // Should have transport interception obligation
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::TransportInterceptable { node_id } if node_id.0 == "execute"
        )));

        // Should have scenario obligations
        assert!(obligations.all.iter().any(|o| matches!(
            o.kind,
            Obligation::AllTransportsSucceed
        )));
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::SingleTransportFailure { node_id } if node_id.0 == "execute"
        )));
    }

    #[test]
    fn test_resource_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "env",
            vec![],
            vec![Port::scalar("tool:clippy", "ToolHandle")],
            (),
        ));
        dag.add_node(Node::opaque(
            "lint",
            vec![Port::scalar("tool:clippy", "ToolHandle")],
            vec![Port::scalar("result", "String")],
            (),
        ));
        dag.add_edge(edge("env", "tool:clippy", "lint", "tool:clippy"));

        let obligations = collect_obligations(&dag, None, None);

        // Resource input connected should be discharged (edge exists)
        let connected = obligations.all.iter().find(|o| matches!(
            &o.kind,
            Obligation::ResourceInputConnected { node_id, .. } if node_id.0 == "lint"
        ));
        assert!(connected.is_some());
        assert!(!connected.unwrap().needs_test()); // Discharged

        // Resource owner valid should need test
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::ResourceOwnerValid { node_id } if node_id.0 == "env"
        )));
    }

    #[test]
    fn test_disconnected_resource_obligation() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "lint",
            vec![Port::scalar("tool:clippy", "ToolHandle")],
            vec![Port::scalar("result", "String")],
            (),
        ));
        // No edge providing the tool!

        let obligations = collect_obligations(&dag, None, None);

        // Resource input connected should NOT be discharged
        let connected = obligations.all.iter().find(|o| matches!(
            &o.kind,
            Obligation::ResourceInputConnected { node_id, .. } if node_id.0 == "lint"
        ));
        assert!(connected.is_some());
        assert!(connected.unwrap().needs_test()); // Not discharged
    }

    #[test]
    fn test_resource_conflict_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![], ()));
        dag.add_node(Node::opaque("b", vec![], vec![], ()));
        // No edge between a and b — they're parallel

        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];

        let obligations = collect_obligations(&dag, None, Some(&accesses));

        // Should have a conflict obligation
        let conflict = obligations.all.iter().find(|o| matches!(
            &o.kind,
            Obligation::ResourceConflictAbsence { conflicts } if !conflicts.is_empty()
        ));
        assert!(conflict.is_some());
        assert!(conflict.unwrap().needs_test());
    }

    #[test]
    fn test_obligation_stats() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::scalar("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::scalar("in", "String")],
            vec![],
            (),
        ));
        dag.add_edge(edge("a", "out", "b", "in"));

        let obligations = collect_obligations(&dag, None, None);
        let stats = obligations.stats();

        assert!(stats.total > 0);
        assert!(stats.discharged + stats.testable == stats.total);
    }

    #[test]
    fn test_entailment_same_type() {
        let status = check_predicate_entailment(
            &TypeId("String".into()),
            &TypeId("String".into()),
            None,
        );
        assert!(matches!(status, EntailmentStatus::Verified));
    }

    #[test]
    fn test_entailment_any_type() {
        let status = check_predicate_entailment(
            &TypeId("Url".into()),
            &TypeId("Any".into()),
            None,
        );
        assert!(matches!(status, EntailmentStatus::Verified));
    }

    #[test]
    fn test_entailment_unknown_without_registry() {
        let status = check_predicate_entailment(
            &TypeId("Url".into()),
            &TypeId("String".into()),
            None,
        );
        assert!(matches!(status, EntailmentStatus::Unknown { .. }));
    }
}
