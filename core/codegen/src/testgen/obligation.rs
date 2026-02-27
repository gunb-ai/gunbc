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

use gunbc_ir::coerce::validate_coercions_with_registry;
use gunbc_ir::resource::{detect_conflicts, ResourceAccess, ResourceConflict};
use gunbc_ir::types::{Cardinality, NodeId, PortName, TypeId};
use gunbc_ir::{contract, detect_boundaries, Dag, NodeKind, TypeRegistry};

use crate::testgen::cardinality::fermi_test_cases;

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

    /// Mark as statically proven INVALID (structural error).
    ///
    /// This is not "unknown" — the obligation is provably violated.
    /// Codegen should surface this as an error, not generate a runtime test.
    pub fn invalidate(mut self, reason: impl Into<String>) -> Self {
        self.status = DischargeStatus::Invalid {
            reason: reason.into(),
        };
        self
    }

    /// Whether this obligation needs a runtime test.
    ///
    /// Returns true for Unknown and RuntimeOnly.
    /// Returns false for Verified (proven correct) and Invalid (proven wrong).
    /// Invalid obligations are surfaced separately — they are errors, not tests.
    pub fn needs_test(&self) -> bool {
        matches!(
            self.status,
            DischargeStatus::Unknown | DischargeStatus::RuntimeOnly
        )
    }

    /// Whether this obligation is provably invalid.
    pub fn is_invalid(&self) -> bool {
        matches!(self.status, DischargeStatus::Invalid { .. })
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
    TransportInterceptable { node_id: NodeId },

    /// DryRun completes without crash for the full workflow.
    DryRunCompletion,

    /// Pure nodes are deterministic: same inputs + same mocks → same outputs.
    PureNodeDeterminism { node_id: NodeId },

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

    /// Cardinality boundary coverage: test that boundary ports handle their full
    /// cardinality range correctly.
    ///
    /// For non-scalar boundary ports, runtime behavior may differ across the
    /// cardinality cases (Empty, One, Many). This obligation is NOT discharged
    /// by construction because:
    /// - Guarded/skipped nodes may receive `Skipped` at runtime
    /// - Fan-in last-writer-wins can silently overwrite
    /// - Ops may defensively handle null/array without engine enforcement
    CardinalityCoverage {
        node_id: NodeId,
        port_name: PortName,
        cardinality: Cardinality,
        /// Boundary values (element counts) to test at this port.
        boundary_values: Vec<u32>,
    },

    /// Coercion coverage: verify that implicit cardinality coercions at edges
    /// produce correctly shaped values.
    ///
    /// When a scalar output connects to a list input, the engine wraps the
    /// value in `Value::List`. These tests verify the engine handles each
    /// coercion point correctly.
    CoercionCoverage {
        from_node: NodeId,
        from_port: PortName,
        to_node: NodeId,
        to_port: PortName,
        from_cardinality: Cardinality,
        to_cardinality: Cardinality,
        kind: gunbc_ir::coerce::CoercionKind,
    },

    /// Coercion compatibility error: edge contracts cannot be safely coerced.
    EdgeCoercionCompatibility {
        from_node: NodeId,
        from_port: PortName,
        to_node: NodeId,
        to_port: PortName,
        from_cardinality: Cardinality,
        to_cardinality: Cardinality,
        reason: String,
    },

    // NOTE: WitnessCompatibility (L4) removed — requires L4 witness-based
    // obligations (types-as-DAGs + witness tests). Re-add when obligation model
    // grows witness coverage beyond mock generation.
    /// Node contract compliance: given valid inputs, outputs satisfy contracts.
    ///
    /// Non-tautological because the node's implementation can be wrong
    /// even if the wiring is correct.
    NodeContractCompliance { node_id: NodeId },

    /// Optional input handling: for inputs that allow absence, nodes must
    /// accept missing inputs but reject wrong-typed inputs.
    OptionalInputHandling {
        node_id: NodeId,
        port_name: PortName,
    },

    /// Variant coverage: test that a boundary port handles ALL shapes a coproduct
    /// type can emit. One test per variant.
    ///
    /// For `EntryKind = RegularFile | Directory | Symlink | Missing | Other`,
    /// this generates 5 tests, each injecting one variant as a bare string.
    VariantCoverage {
        node_id: NodeId,
        port_name: PortName,
        type_id: TypeId,
        /// Variant names from the coproduct (e.g., ["RegularFile", "Directory", ...])
        variants: Vec<String>,
    },

    /// CLI contract round-trip: generated CLI harness must verify argument
    /// parsing and `--print-inputs json` behavior for this tool.
    CliContractRoundTrip { tool_name: String },

    // -----------------------------------------------------------------------
    // Bucket C: Scenario Coverage (graph + transport mocks)
    //
    // "N+1 instead of 2^N": one all-succeed + one per-transport failure.
    // -----------------------------------------------------------------------
    /// Happy path: all transports succeed, workflow reaches terminal outputs.
    AllTransportsSucceed,

    /// Single transport fails: verify failure propagation semantics.
    SingleTransportFailure { node_id: NodeId },

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
    /// Transport executors must declare at least one resource input port.
    ///
    /// A transport node without a resource input is an unmodeled I/O escape hatch.
    TransportResourceDeclared { node_id: NodeId },

    /// Every resource/tool input port has an incoming edge.
    ResourceInputConnected {
        node_id: NodeId,
        port_name: PortName,
    },

    /// Resource owners (providers) are valid env/acquisition nodes.
    ResourceOwnerValid { node_id: NodeId },

    /// No orphan resources: acquired resources are consumed by someone.
    ResourceOrphan {
        node_id: NodeId,
        port_name: PortName,
    },

    /// Resource conflicts are absent (DAG ordering prevents parallel conflicts).
    ResourceConflictAbsence { conflicts: Vec<ResourceConflict> },

    /// Resource contention handling: consumer handles failed acquisition.
    ResourceContentionHandling { resource_port: String },

    /// Credential chain integrity: transport execute nodes with an auth scheme
    /// must have a `res:credential` input port that is connected to a credential
    /// source. If `res:credential` is missing or disconnected, the request will
    /// be sent unauthenticated (silent failure).
    CredentialChainIntegrity {
        node_id: NodeId,
        /// Whether the port exists and is connected.
        connected: bool,
    },
    // NOTE: ResourceSimulation removed — resource simulation tests are
    // generated directly from MockSpec resource mocks in codegen, not from
    // obligations. Will be re-added if resource simulation needs obligation tracking.
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
    /// Statically proven INVALID — this is a structural error, not "unknown".
    ///
    /// Invalid obligations are not tests waiting to be run — they are
    /// errors that should be surfaced immediately. Codegen emits a
    /// `#[test] #[should_panic]` or a compile_error-style message.
    Invalid { reason: String },
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
    /// Get only obligations that need runtime tests (Unknown or RuntimeOnly).
    pub fn testable(&self) -> Vec<&ProofObligation> {
        self.all.iter().filter(|o| o.needs_test()).collect()
    }

    /// Get obligations that are provably invalid (structural errors).
    ///
    /// These should be surfaced as errors, not as tests. Codegen emits
    /// a failing test with a crisp message for each invalid obligation.
    pub fn invalids(&self) -> Vec<&ProofObligation> {
        self.all.iter().filter(|o| o.is_invalid()).collect()
    }

    /// Whether any obligations are provably invalid.
    pub fn has_invalids(&self) -> bool {
        self.all.iter().any(|o| o.is_invalid())
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

    /// Get only cardinality coverage obligations from Bucket B.
    pub fn cardinality_obligations(&self) -> Vec<&ProofObligation> {
        self.bucket_b()
            .into_iter()
            .filter(|o| matches!(&o.kind, Obligation::CardinalityCoverage { .. }))
            .collect()
    }

    /// Get only coercion coverage obligations from Bucket B.
    pub fn coercion_obligations(&self) -> Vec<&ProofObligation> {
        self.bucket_b()
            .into_iter()
            .filter(|o| matches!(&o.kind, Obligation::CoercionCoverage { .. }))
            .collect()
    }

    /// Get only variant coverage obligations from Bucket B.
    pub fn variant_coverage_obligations(&self) -> Vec<&ProofObligation> {
        self.bucket_b()
            .into_iter()
            .filter(|o| matches!(&o.kind, Obligation::VariantCoverage { .. }))
            .collect()
    }

    /// Get only optional input handling obligations from Bucket B.
    pub fn optional_input_obligations(&self) -> Vec<&ProofObligation> {
        self.bucket_b()
            .into_iter()
            .filter(|o| matches!(&o.kind, Obligation::OptionalInputHandling { .. }))
            .collect()
    }

    /// Get only CLI contract round-trip obligations from Bucket B.
    pub fn cli_contract_obligations(&self) -> Vec<&ProofObligation> {
        self.bucket_b()
            .into_iter()
            .filter(|o| matches!(&o.kind, Obligation::CliContractRoundTrip { .. }))
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
        let invalids = self.invalids();
        ObligationStats {
            total: self.all.len(),
            discharged: self
                .all
                .iter()
                .filter(|o| matches!(o.status, DischargeStatus::Verified { .. }))
                .count(),
            invalid: invalids.len(),
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
    pub invalid: usize,
    pub testable: usize,
    pub bucket_a: usize,
    pub bucket_b: usize,
    pub bucket_c: usize,
    pub bucket_d: usize,
}

impl std::fmt::Display for ObligationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.invalid > 0 {
            write!(
                f,
                "{} obligations ({} discharged, {} INVALID, {} testable: A={}, B={}, C={}, D={})",
                self.total,
                self.discharged,
                self.invalid,
                self.testable,
                self.bucket_a,
                self.bucket_b,
                self.bucket_c,
                self.bucket_d
            )
        } else {
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
    registry: &TypeRegistry,
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
        if node.kind == Some(NodeKind::TransportExecute) {
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
        let is_effectful = matches!(
            node.kind,
            Some(
                NodeKind::TransportExecute
                    | NodeKind::ToolEnvironment
                    | NodeKind::ToolConsumer
                    | NodeKind::ResourceEnvironment
                    | NodeKind::ResourceAcquire
            )
        );

        if !is_effectful {
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
    registry: &TypeRegistry,
    obligations: &mut Vec<ProofObligation>,
) {
    // B.1: Edge predicate entailment
    for edge in &dag.edges {
        let Some(ports) = dag.resolve_edge_ports(edge) else {
            continue;
        };

        // L1 + L2 are statically verified (type/cardinality compat)
        // We only care about L3: predicate entailment
        let entailment =
            check_predicate_entailment(ports.from.type_id(), ports.to.type_id(), registry);

        let edge_label = format!(
            "{}.{} → {}.{}",
            edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
        );

        // Determine reason and discharge/invalidate status from entailment
        let (reason, status) = match &entailment {
            EntailmentStatus::Verified => (
                format!("Edge {}: predicate entailment verified", edge_label),
                "verified",
            ),
            EntailmentStatus::Unknown { reason } => (
                format!(
                    "Edge {}: predicate entailment unknown ({})",
                    edge_label, reason
                ),
                "unknown",
            ),
            EntailmentStatus::Invalid { reason } => (
                format!(
                    "Edge {}: predicate entailment INVALID ({})",
                    edge_label, reason
                ),
                "invalid",
            ),
        };

        let obligation = ProofObligation::new(
            Obligation::EdgePredicateEntailment {
                from_node: edge.from_node.clone(),
                from_port: edge.from_port.clone(),
                to_node: edge.to_node.clone(),
                to_port: edge.to_port.clone(),
                from_type: ports.from.type_id().clone(),
                to_type: ports.to.type_id().clone(),
                entailment,
            },
            &reason,
            ObligationSource::Contract,
        );

        match status {
            "verified" => {
                obligations.push(obligation.discharge("Predicate entailment statically verified"));
            }
            "invalid" => {
                obligations.push(obligation.invalidate(reason));
            }
            _ => {
                // Unknown — needs runtime test
                obligations.push(obligation);
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

    // B.2b: Optional input handling — inputs that allow absence should accept
    // missing values but reject wrong-typed values.
    for node in &dag.nodes {
        for port in &node.inputs {
            if port.has_guard() {
                continue;
            }
            if port.cardinality.allows_empty() && port.cardinality.allows_one() {
                obligations.push(ProofObligation::runtime(
                    Obligation::OptionalInputHandling {
                        node_id: node.id.clone(),
                        port_name: port.name.clone(),
                    },
                    format!(
                        "Optional input {}.{} should accept missing and reject wrong types",
                        node.id.0, port.name.0
                    ),
                    ObligationSource::Contract,
                ));
            }
        }
    }

    // B.3: Cardinality boundary coverage — for each boundary output port with
    // non-trivial cardinality (more than one test case), add a coverage obligation.
    // This is NOT proven by construction because runtime behavior may differ
    // across cardinality cases (empty vs one vs many).
    let boundaries = detect_boundaries(dag);
    for (node_id, port_name) in &boundaries.boundary_ports {
        let Some(port) = dag.resolve_output_port(node_id, port_name) else {
            continue;
        };
        let bvs = fermi_test_cases(port.cardinality());
        if bvs.len() > 1 {
            obligations.push(ProofObligation::runtime(
                Obligation::CardinalityCoverage {
                    node_id: node_id.clone(),
                    port_name: port_name.clone(),
                    cardinality: port.cardinality(),
                    boundary_values: bvs.clone(),
                },
                format!(
                    "Boundary port {}.{} has cardinality {} — test {} boundary values: {:?}",
                    node_id.0,
                    port_name.0,
                    port.cardinality(),
                    bvs.len(),
                    bvs
                ),
                ObligationSource::Contract,
            ));
        }
    }

    // B.3b: Variant coverage — for each boundary output port whose type is a
    // registered coproduct with 2+ variants, emit a VariantCoverage obligation.
    // Per-port, not per-pair: avoids combinatorial explosion.
    for (node_id, port_name) in &boundaries.boundary_ports {
        let Some(port) = dag.resolve_output_port(node_id, port_name) else {
            continue;
        };
        let variants = contract::variant_witnesses(port.type_id().0.as_str(), registry);
        if variants.len() > 1 {
            let variant_names: Vec<String> =
                variants.iter().map(|(name, _)| name.clone()).collect();
            obligations.push(ProofObligation::runtime(
                Obligation::VariantCoverage {
                    node_id: node_id.clone(),
                    port_name: port_name.clone(),
                    type_id: port.type_id().clone(),
                    variants: variant_names.clone(),
                },
                format!(
                    "Boundary port {}.{} has coproduct type {} with {} variants: {:?}",
                    node_id.0,
                    port_name.0,
                    port.type_id().0,
                    variant_names.len(),
                    variant_names,
                ),
                ObligationSource::Contract,
            ));
        }
    }

    // B.4: Coercion compatibility + coverage — validate edge contracts with
    // contract-aware coercion. Emit invalid obligations for incompatibilities,
    // and coverage obligations for implicit cardinality coercions.
    let coercion_report = validate_coercions_with_registry(dag, Some(registry));
    for error in coercion_report.errors {
        let edge_label = format!(
            "{}.{} → {}.{}",
            error.from_node.0, error.from_port.0, error.to_node.0, error.to_port.0
        );
        let reason = format!("Edge {}: coercion invalid ({})", edge_label, error.reason);
        obligations.push(
            ProofObligation::new(
                Obligation::EdgeCoercionCompatibility {
                    from_node: error.from_node.clone(),
                    from_port: error.from_port.clone(),
                    to_node: error.to_node.clone(),
                    to_port: error.to_port.clone(),
                    from_cardinality: error.from_cardinality,
                    to_cardinality: error.to_cardinality,
                    reason: error.reason.clone(),
                },
                &reason,
                ObligationSource::Contract,
            )
            .invalidate(reason),
        );
    }

    for coercion in coercion_report.coercions {
        obligations.push(ProofObligation::runtime(
            Obligation::CoercionCoverage {
                from_node: coercion.from_node.clone(),
                from_port: coercion.from_port.clone(),
                to_node: coercion.to_node.clone(),
                to_port: coercion.to_port.clone(),
                from_cardinality: coercion.from_cardinality,
                to_cardinality: coercion.to_cardinality,
                kind: coercion.kind,
            },
            format!(
                "Coercion at {}.{} → {}.{}: {} ({} → {})",
                coercion.from_node.0,
                coercion.from_port.0,
                coercion.to_node.0,
                coercion.to_port.0,
                coercion.kind,
                coercion.from_cardinality,
                coercion.to_cardinality,
            ),
            ObligationSource::Contract,
        ));
    }
}

/// Bucket C: Scenario coverage obligations.
fn collect_scenario_obligations<T>(dag: &Dag<T>, obligations: &mut Vec<ProofObligation>) {
    let transport_executors: Vec<&NodeId> = dag
        .nodes
        .iter()
        .filter(|n| n.kind == Some(NodeKind::TransportExecute))
        .map(|n| &n.id)
        .collect();

    // C.1-C.3 only apply when there are transport executors
    if !transport_executors.is_empty() {
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
            let has_downstream = dag.edges.iter().any(|e| &e.from_node == *node_id);

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

/// Whether a type_id identifies a resource type.
///
/// Resource types are handles and locks that require connectivity and
/// contention handling obligations.
fn is_resource_type(type_id: &TypeId) -> bool {
    matches!(
        type_id.0.as_str(),
        "ToolHandle" | "Lock" | "Lease" | "SharedLock"
    )
}

/// Whether a port is a resource port (by name prefix or type).
fn is_resource_port(port: &gunbc_ir::dag::Port) -> bool {
    port.resource_access.is_some()
        || port.name.0.starts_with("res:")
        || port.name.0.starts_with("resource:")
        || port.name.0.starts_with("tool:")
        || is_resource_type(&port.type_id)
}

/// Bucket D: Resource hygiene obligations.
fn collect_resource_obligations<T>(
    dag: &Dag<T>,
    resource_accesses: Option<&[ResourceAccess]>,
    obligations: &mut Vec<ProofObligation>,
) {
    // D.0: Transport nodes must declare at least one resource input
    for node in &dag.nodes {
        if node.kind != Some(NodeKind::TransportExecute) {
            continue;
        }

        let has_resource_input = node.inputs.iter().any(is_resource_port);
        if has_resource_input {
            obligations.push(
                ProofObligation::new(
                    Obligation::TransportResourceDeclared {
                        node_id: node.id.clone(),
                    },
                    format!(
                        "Transport node '{}' declares at least one resource input",
                        node.id.0
                    ),
                    ObligationSource::ResourceModel,
                )
                .discharge("Resource input declared on transport node"),
            );
        } else {
            obligations.push(
                ProofObligation::new(
                    Obligation::TransportResourceDeclared {
                        node_id: node.id.clone(),
                    },
                    format!(
                        "Transport node '{}' has NO resource inputs (unmodeled I/O)",
                        node.id.0
                    ),
                    ObligationSource::ResourceModel,
                )
                .invalidate(format!(
                    "Transport node '{}' missing resource input",
                    node.id.0
                )),
            );
        }
    }

    // D.1: Resource input connectivity — every resource/tool input has an edge
    for node in &dag.nodes {
        for port in &node.inputs {
            if is_resource_port(port) {
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
                            format!("Resource input {}.{} is connected", node.id.0, port.name.0),
                            ObligationSource::ResourceModel,
                        )
                        .discharge("Edge exists to resource input"),
                    );
                } else {
                    // Disconnected resource input is a static structural error.
                    // No runtime test can help — the edge is simply missing.
                    obligations.push(
                        ProofObligation::new(
                            Obligation::ResourceInputConnected {
                                node_id: node.id.clone(),
                                port_name: port.name.clone(),
                            },
                            format!(
                                "Resource input {}.{} has NO incoming edge — resource not provided",
                                node.id.0, port.name.0
                            ),
                            ObligationSource::ResourceModel,
                        )
                        .invalidate(format!(
                            "Resource input {}.{} is disconnected",
                            node.id.0, port.name.0
                        )),
                    );
                }
            }
        }
    }

    // D.2: Resource owner validity — nodes that output resources should be env/owner nodes
    for node in &dag.nodes {
        let outputs_resource = node.outputs.iter().any(is_resource_port);

        if outputs_resource {
            obligations.push(ProofObligation::runtime(
                Obligation::ResourceOwnerValid {
                    node_id: node.id.clone(),
                },
                format!("Resource owner '{}' is a valid acquisition node", node.id.0),
                ObligationSource::ResourceModel,
            ));
        }
    }

    // D.3: No orphan resources — resources acquired should be consumed
    for node in &dag.nodes {
        for port in &node.outputs {
            if is_resource_port(port) {
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
                            format!("Resource output {}.{} has consumer", node.id.0, port.name.0),
                            ObligationSource::ResourceModel,
                        )
                        .discharge("Edge exists from resource output to consumer"),
                    );
                } else {
                    // Orphan resource is a static structural error.
                    // Resource acquired but never consumed — wasted or leaked.
                    obligations.push(
                        ProofObligation::new(
                            Obligation::ResourceOrphan {
                                node_id: node.id.clone(),
                                port_name: port.name.clone(),
                            },
                            format!(
                                "Resource output {}.{} is acquired but never consumed (orphan)",
                                node.id.0, port.name.0
                            ),
                            ObligationSource::ResourceModel,
                        )
                        .invalidate(format!(
                            "Resource {}.{} is orphaned (no consumer edge)",
                            node.id.0, port.name.0
                        )),
                    );
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
                    Obligation::ResourceConflictAbsence { conflicts: vec![] },
                    "No resource conflicts detected",
                    ObligationSource::ResourceModel,
                )
                .discharge("detect_conflicts returned empty"),
            );
        } else {
            // Detected conflicts are provable structural errors —
            // parallel nodes access the same resource without ordering.
            obligations.push(
                ProofObligation::new(
                    Obligation::ResourceConflictAbsence {
                        conflicts: conflicts.clone(),
                    },
                    format!(
                        "{} resource conflict(s) detected — DAG ordering may be insufficient",
                        conflicts.len()
                    ),
                    ObligationSource::ResourceModel,
                )
                .invalidate(format!(
                    "{} resource conflict(s): parallel nodes access same resource without ordering",
                    conflicts.len()
                )),
            );
        }
    }

    // D.5: Resource contention handling — for each resource type input,
    // verify the consumer can handle contention (acquisition failure).
    for node in &dag.nodes {
        for port in &node.inputs {
            if is_resource_type(&port.type_id) {
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

    // D.6: Credential chain integrity — transport execute nodes that
    // declare an auth scheme MUST have res:credential connected.
    // Without this, requests are sent unauthenticated (the silent 401 bug).
    for node in &dag.nodes {
        if node.kind != Some(NodeKind::TransportExecute) {
            continue;
        }

        let cred_port = node
            .inputs
            .iter()
            .find(|p| p.name.0 == "res:credential");

        let Some(port) = cred_port else {
            // No res:credential port — not an authenticated endpoint.
            continue;
        };

        let has_edge = dag
            .edges
            .iter()
            .any(|e| e.to_node == node.id && e.to_port == port.name);

        if has_edge {
            obligations.push(
                ProofObligation::new(
                    Obligation::CredentialChainIntegrity {
                        node_id: node.id.clone(),
                        connected: true,
                    },
                    format!(
                        "Transport '{}': res:credential is connected to credential source",
                        node.id.0
                    ),
                    ObligationSource::ResourceModel,
                )
                .discharge("Credential edge exists from source to transport execute node"),
            );
        } else {
            // res:credential port exists but no edge — unauthenticated request.
            // This is the exact pattern that caused the gist 401 bug.
            obligations.push(
                ProofObligation::new(
                    Obligation::CredentialChainIntegrity {
                        node_id: node.id.clone(),
                        connected: false,
                    },
                    format!(
                        "Transport '{}': res:credential port exists but has NO incoming edge — \
                         service declares auth but credential is never wired (request will be unauthenticated)",
                        node.id.0
                    ),
                    ObligationSource::ResourceModel,
                )
                .invalidate(format!(
                    "Transport '{}': credential not wired — service has config {{ auth }} \
                     but auth_input is missing or not connected",
                    node.id.0
                )),
            );
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
    registry: &TypeRegistry,
) -> EntailmentStatus {
    // Same type → trivially verified
    if from_type.0 == to_type.0 {
        return EntailmentStatus::Verified;
    }

    // Target is "Any" → verified (target accepts anything)
    if to_type.0 == "Any" {
        return EntailmentStatus::Verified;
    }

    // Source is "Any" but target is specific → Unknown.
    // A value of type Any does NOT entail it satisfies a more specific
    // target predicate/type. This must be tested empirically.
    if from_type.0 == "Any" {
        return EntailmentStatus::Unknown {
            reason: format!(
                "source is 'Any' but target '{}' has specific constraints",
                to_type.0
            ),
        };
    }

    let from_dag = registry.get_by_name(&from_type.0);
    let to_dag = registry.get_by_name(&to_type.0);

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

            // Both have predicates — check if source predicates subsume target.
            // This is conservative but allows provable entailments
            // (e.g., InRange subsumption, And/Or structure).
            let all_target_covered = to_preds
                .iter()
                .all(|tp| from_preds.iter().any(|fp| fp.entails(tp)));

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
    use gunbc_ir::{build::*, Dag, Node, NodeKind, Port};

    #[test]
    fn test_collect_basic_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque("source", vec![], vec![Port::scalar("out", "String")], ())
                .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "sink",
                vec![Port::scalar("in", "String")],
                vec![Port::scalar("result", "String")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("source", "out", "sink", "in"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);
        let stats = obligations.stats();

        // Should have obligations from all buckets
        assert!(stats.total > 0);
        assert!(stats.testable > 0);

        // Should have DryRun completion
        assert!(obligations
            .all
            .iter()
            .any(|o| matches!(o.kind, Obligation::DryRunCompletion)));

        // Should have determinism for pure nodes
        assert!(obligations
            .all
            .iter()
            .any(|o| matches!(o.kind, Obligation::PureNodeDeterminism { .. })));

        // Should have node contract compliance
        assert!(obligations
            .all
            .iter()
            .any(|o| matches!(o.kind, Obligation::NodeContractCompliance { .. })));
    }

    #[test]
    fn test_optional_input_obligations_collected() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "opt",
                vec![Port::optional("maybe", "String")],
                vec![Port::scalar("out", "String")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::OptionalInputHandling { node_id, port_name }
                if node_id.0 == "opt" && port_name.0 == "maybe"
        )));
    }

    #[test]
    fn test_transport_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "fs_env",
                vec![],
                vec![Port::resource("file", "FilesystemHandle", AccessMode::Read)],
                (),
            )
            .with_kind(NodeKind::ResourceEnvironment),
        );
        dag.add_node(
            Node::opaque(
                "prepare",
                vec![],
                vec![Port::scalar("request", "TransportRequest")],
                (),
            )
            .with_kind(NodeKind::TransportPrepare),
        );
        dag.add_node(
            Node::opaque(
                "execute",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::resource("file", "FilesystemHandle", AccessMode::Read),
                ],
                vec![Port::scalar("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "parse",
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar("result", "String")],
                (),
            )
            .with_kind(NodeKind::TransportParse),
        );
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("fs_env", "res:file", "execute", "res:file"));
        dag.add_edge(edge("execute", "response", "parse", "response"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        // Should have transport interception obligation
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::TransportInterceptable { node_id } if node_id.0 == "execute"
        )));

        // Should have scenario obligations
        assert!(obligations
            .all
            .iter()
            .any(|o| matches!(o.kind, Obligation::AllTransportsSucceed)));
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::SingleTransportFailure { node_id } if node_id.0 == "execute"
        )));

        // Transport node should declare a resource input (discharged)
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::TransportResourceDeclared { node_id } if node_id.0 == "execute"
        )));
    }

    #[test]
    fn test_resource_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "env",
                vec![],
                vec![Port::scalar("tool:clippy", "ToolHandle")],
                (),
            )
            .with_kind(NodeKind::ToolEnvironment),
        );
        dag.add_node(
            Node::opaque(
                "lint",
                vec![Port::scalar("tool:clippy", "ToolHandle")],
                vec![Port::scalar("result", "String")],
                (),
            )
            .with_kind(NodeKind::ToolConsumer),
        );
        dag.add_edge(edge("env", "tool:clippy", "lint", "tool:clippy"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        // Resource input connected should be discharged (edge exists)
        let connected = obligations.all.iter().find(|o| {
            matches!(
                &o.kind,
                Obligation::ResourceInputConnected { node_id, .. } if node_id.0 == "lint"
            )
        });
        assert!(connected.is_some());
        assert!(!connected.unwrap().needs_test()); // Discharged

        // Resource owner valid should need test
        assert!(obligations.all.iter().any(|o| matches!(
            &o.kind,
            Obligation::ResourceOwnerValid { node_id } if node_id.0 == "env"
        )));
    }

    #[test]
    fn test_disconnected_resource_is_invalid() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "lint",
                vec![Port::scalar("tool:clippy", "ToolHandle")],
                vec![Port::scalar("result", "String")],
                (),
            )
            .with_kind(NodeKind::ToolConsumer),
        );
        // No edge providing the tool!

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        // Disconnected resource input is now Invalid (structural error),
        // not Unknown (needs test). There's nothing to test — the edge is missing.
        let connected = obligations.all.iter().find(|o| {
            matches!(
                &o.kind,
                Obligation::ResourceInputConnected { node_id, .. } if node_id.0 == "lint"
            )
        });
        assert!(connected.is_some());
        let connected = connected.unwrap();
        assert!(
            connected.is_invalid(),
            "disconnected resource should be Invalid"
        );
        assert!(
            !connected.needs_test(),
            "Invalid obligations don't need runtime tests"
        );
        assert!(obligations.has_invalids());
    }

    #[test]
    fn test_resource_conflict_is_invalid() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![], ()).with_kind(NodeKind::Pure));
        dag.add_node(Node::opaque("b", vec![], vec![], ()).with_kind(NodeKind::Pure));
        // No edge between a and b — they're parallel

        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, Some(&accesses));

        // Conflict is a provable structural error, now Invalid
        let conflict = obligations.all.iter().find(|o| {
            matches!(
                &o.kind,
                Obligation::ResourceConflictAbsence { conflicts } if !conflicts.is_empty()
            )
        });
        assert!(conflict.is_some());
        let conflict = conflict.unwrap();
        assert!(conflict.is_invalid(), "resource conflict should be Invalid");
        assert!(
            !conflict.needs_test(),
            "Invalid obligations don't need runtime tests"
        );
    }

    #[test]
    fn test_credential_chain_integrity_connected() {
        // Transport execute node with res:credential that IS connected
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "cred_source",
                vec![],
                vec![Port::scalar("token", "Secret")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "execute_rest",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::with_cardinality("res:credential", "Credential", Cardinality::ZERO_OR_ONE),
                ],
                vec![Port::scalar("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_edge(edge("cred_source", "token", "execute_rest", "res:credential"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        let cred_obligation = obligations.all.iter().find(|o| {
            matches!(
                &o.kind,
                Obligation::CredentialChainIntegrity { node_id, connected }
                    if node_id.0 == "execute_rest" && *connected
            )
        });
        assert!(cred_obligation.is_some(), "connected credential should generate discharged obligation");
        assert!(!cred_obligation.unwrap().needs_test(), "connected credential should be discharged");
    }

    #[test]
    fn test_credential_chain_integrity_disconnected_is_invalid() {
        // Transport execute node with res:credential but NO edge — the 401 bug pattern
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute_rest",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::with_cardinality("res:credential", "Credential", Cardinality::ZERO_OR_ONE),
                ],
                vec![Port::scalar("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        // No edge to res:credential!

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);

        let cred_obligation = obligations.all.iter().find(|o| {
            matches!(
                &o.kind,
                Obligation::CredentialChainIntegrity { node_id, connected }
                    if node_id.0 == "execute_rest" && !*connected
            )
        });
        assert!(cred_obligation.is_some(), "disconnected credential should be invalid");
        assert!(cred_obligation.unwrap().is_invalid(), "disconnected credential is a structural error");
    }

    #[test]
    fn test_obligation_stats() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque("a", vec![], vec![Port::scalar("out", "String")], ())
                .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "b",
                vec![Port::scalar("in", "String")],
                vec![],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("a", "out", "b", "in"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);
        let stats = obligations.stats();

        assert!(stats.total > 0);
        assert_eq!(
            stats.discharged + stats.invalid + stats.testable,
            stats.total
        );
    }

    #[test]
    fn test_entailment_same_type() {
        let registry = TypeRegistry::with_core_types();
        let status = check_predicate_entailment(
            &TypeId("String".into()),
            &TypeId("String".into()),
            &registry,
        );
        assert!(matches!(status, EntailmentStatus::Verified));
    }

    #[test]
    fn test_entailment_target_any() {
        // Target is Any → verified (accepts anything)
        let registry = TypeRegistry::with_core_types();
        let status =
            check_predicate_entailment(&TypeId("Url".into()), &TypeId("Any".into()), &registry);
        assert!(matches!(status, EntailmentStatus::Verified));
    }

    #[test]
    fn test_entailment_source_any_target_specific() {
        // Source is Any, target is specific → Unknown (can't prove satisfaction)
        let registry = TypeRegistry::with_core_types();
        let status =
            check_predicate_entailment(&TypeId("Any".into()), &TypeId("Url".into()), &registry);
        assert!(matches!(status, EntailmentStatus::Unknown { .. }));
    }

    #[test]
    fn test_entailment_unregistered_types() {
        // Types not in registry → Unknown (can't check predicates)
        let registry = TypeRegistry::with_core_types();
        let status = check_predicate_entailment(
            &TypeId("UnknownTypeA".into()),
            &TypeId("UnknownTypeB".into()),
            &registry,
        );
        assert!(matches!(status, EntailmentStatus::Unknown { .. }));
    }

    #[test]
    fn test_coercion_obligations_collected() {
        // DAG with scalar output → list input should produce coercion obligation
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque("producer", vec![], vec![Port::scalar("out", "Json")], ())
                .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "consumer",
                vec![Port::list("inputs", "JsonList")],
                vec![Port::scalar("result", "Json")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("producer", "out", "consumer", "inputs"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);
        let coercion_obs = obligations.coercion_obligations();

        assert_eq!(coercion_obs.len(), 1, "should detect one coercion");
        if let Obligation::CoercionCoverage {
            from_node,
            to_node,
            kind,
            ..
        } = &coercion_obs[0].kind
        {
            assert_eq!(from_node.0, "producer");
            assert_eq!(to_node.0, "consumer");
            assert_eq!(*kind, gunbc_ir::coerce::CoercionKind::WrapScalar);
        } else {
            panic!("expected CoercionCoverage obligation");
        }
    }

    #[test]
    fn test_no_coercion_for_matching_cardinalities() {
        // DAG with matching cardinalities should produce no coercion obligations
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque("a", vec![], vec![Port::scalar("out", "String")], ())
                .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "b",
                vec![Port::scalar("in", "String")],
                vec![],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("a", "out", "b", "in"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);
        let coercion_obs = obligations.coercion_obligations();
        assert!(
            coercion_obs.is_empty(),
            "no coercion for matching cardinalities"
        );
    }

    #[test]
    fn test_coercion_errors_surface_invalid_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "list_producer",
                vec![],
                vec![Port::list("out", "JsonList")],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "scalar_consumer",
                vec![Port::scalar("in", "Json")],
                vec![],
                (),
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_edge(edge("list_producer", "out", "scalar_consumer", "in"));

        let registry = TypeRegistry::with_core_types();
        let obligations = collect_obligations(&dag, &registry, None);
        let invalids = obligations.invalids();
        assert!(
            invalids
                .iter()
                .any(|o| matches!(o.kind, Obligation::EdgeCoercionCompatibility { .. })),
            "expected an invalid coercion obligation"
        );
    }
}
