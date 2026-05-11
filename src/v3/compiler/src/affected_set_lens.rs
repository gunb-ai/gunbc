//! Affected-set lens prototype (gunbc#2699, companion design docs PR #2700).
//!
//! Pure-host analysis over two compiled [`Dag`] snapshots. Each **dimension slice**
//! starts from nodes with a **non-empty delta** for that dimension (fail-closed when
//! comparisons are unavailable), then closes **forward over downstream consumers**
//! using the substrate port graph (reverse of `resolve_producer_lookup`). Every exported
//! dimension carries at least one human-readable receipt in [`DimensionSlice::receipts`]
//! (explicit seed/affected/discipline bookkeeping — prototype host projection, see gunbc#2699).
//!
//! ## Value dimension caveat
//!
//! Structural `Behavior` inequality does **not** prove return-value equivalence or
//! non-equivalence. Following design §2 `PROVEN` discipline, this slice uses the
//! structural-edit seed set (paired span-key `Behavior` inequality via host `Debug`
//! snapshots—**not** under-approximated by attribution path; orphan after-nodes are
//! always seeds) and the same downstream closure as the naive baseline for that seed set.
//! **Per-node proof receipts that exclude downstream consumers on value grounds are not
//! emitted yet** because the proof substrate does not expose an I/O-equivalence oracle
//! in-tree.
//!
//! Narrowing demos for gunbc#2699 lean on **`cost` / `effect` / `refinement`**
//! deltas plus commentary in `.dag` worked examples.
//!
//! ## Prototype debt (Debug snapshots / module footprint)
//!
//! Cost, refinement, effect-shape, and paired-behavior deltas still lean on **`Debug` text**
//! comparisons in places — acceptable for exercised examples (gunbc#2699) but brittle if rustfmt
//! or `Debug` impl layout shifts without semantic substrate drift; dissolution is explicit in
//! ROADMAP **T-PB-A/T-PB-B** alongside other host-query mirrors.
//!
//! This file overshoots `CODING.md`'s heuristic (~500 LOC for **new** modules); split pairing vs
//! downstream closure vs dimension seed wiring on the next substantive change, not for line-count alone.
//!
//! ## Cross-`Dag` pairing (P1 / P2)
//!
//! `BehaviorPairing` keys behaviors only by compiler `SourceSpan` identity (`file` + `byte_start`)
//! carried on every `Behavior` — not by `NodeId` ordinal. Positional `zip` by enumeration order
//! is intentionally absent: equal node counts do not imply semantic alignment. Unmatched
//! after-nodes are fail-closed seeds; collision buckets use deterministic per-key `NodeId` order.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;

use crate::dag::{
    ArrowBody, Behavior, Dag, DeclarationId, Lookup as CostLookup, NodeId, PortId, ProducerLookup,
    TransformTarget, TypeConnective,
};
use crate::lens_cost::complexity_of;
use crate::lens_effect_enumeration::StructuralEffectShape;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedSetReceipt {
    pub dimension: &'static str,
    pub dimension_node: Option<NodeId>,
    pub summary: String,
}

/// Typed receipt for [`DownstreamAdjacency::build`] — surfaces P3 producer-walk
/// outcomes without collapsing malformed substrate into [`ProducerLookup::NoProducer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownstreamBuildReceipt {
    pub malformed_producer_walks: Vec<String>,
    /// Consumers conservatively treated as affected seeds when a dependency anchor
    /// could not be resolved (INVARIANTS P3 fail-closed).
    pub fail_closed_consumer_seeds: Vec<NodeId>,
}

/// Full prototype report emitted to integration tests / tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedSetLensReport {
    pub structural_seed_count: usize,
    /// Best-effort coarse diff string. Omitted in the gunbc#2699 prototype: running
    /// `serialize::first_difference` stringify-walks declarations, behaviors, clusters, and
    /// ports on the full bootstrapped substrate and does not meet integration-test wall time.
    pub first_structural_difference: Option<String>,
    pub downstream_build_receipt: DownstreamBuildReceipt,
    /// Seeds and transitive downstream closure for paired structural deltas (producer graph).
    pub structural: DimensionSlice,
    /// Convenience field: duplicates `structural.affected_ids` (structural downstream closure).
    pub transitive_downstream: Vec<NodeId>,
    pub value: DimensionSlice,
    pub cost: DimensionSlice,
    pub effect: DimensionSlice,
    pub refinement: DimensionSlice,
    pub aggregate_union: Vec<NodeId>,
    /// Count of paired `(dag_before,dag_after)` behaviors under `(file, byte_start)` keys.
    pub span_key_behavior_pair_count: usize,
    /// `dag_after` behaviors with no counterpart key in `dag_before` (fail-closed as seeds).
    pub span_key_behavior_orphan_after_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionSlice {
    pub dimension: &'static str,
    pub seed_ids: Vec<NodeId>,
    pub affected_ids: Vec<NodeId>,
    pub receipts: Vec<AffectedSetReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanPair {
    file: String,
    byte_start: u32,
}

/// Host-side reverse consumer map: `producer -> {consumers depending on its outputs}`.
pub struct DownstreamAdjacency {
    outgoing: HashMap<NodeId, Vec<NodeId>>,
}

impl DownstreamAdjacency {
    pub fn build(dag: &Dag) -> (Self, DownstreamBuildReceipt) {
        let mut outgoing: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        let mut malformed_producer_walks: Vec<String> = Vec::new();
        let mut fail_closed_consumer_seeds: HashSet<NodeId> = HashSet::new();
        for consumer in dag.nodes() {
            let walk = resolve_behavior_upstream_producers(dag, consumer);
            for receipt in walk.malformed_producer_walks {
                malformed_producer_walks.push(receipt);
            }
            for id in walk.fail_closed_consumer_seeds {
                fail_closed_consumer_seeds.insert(id);
            }
            for producer in walk.producer_ids {
                outgoing.entry(producer).or_default().insert(consumer.id());
            }
        }
        malformed_producer_walks.sort();
        malformed_producer_walks.dedup();
        let mut fc: Vec<NodeId> = fail_closed_consumer_seeds.into_iter().collect();
        fc.sort_by_key(|id| id.raw());
        let receipt = DownstreamBuildReceipt {
            malformed_producer_walks,
            fail_closed_consumer_seeds: fc,
        };
        let mut outgoing_vec: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (producer, consumers) in outgoing {
            let mut list: Vec<NodeId> = consumers.into_iter().collect();
            list.sort_by_key(|id| id.raw());
            outgoing_vec.insert(producer, list);
        }
        (
            Self {
                outgoing: outgoing_vec,
            },
            receipt,
        )
    }

    pub fn transitive_forward(&self, seeds: &[NodeId]) -> Vec<NodeId> {
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for s in seeds {
            if seen.insert(*s) {
                queue.push_back(*s);
            }
        }
        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.outgoing.get(&current) {
                for child in children {
                    if seen.insert(*child) {
                        queue.push_back(*child);
                    }
                }
            }
        }
        let mut ordered: Vec<NodeId> = seen.into_iter().collect();
        ordered.sort_by_key(|id| id.raw());
        ordered
    }
}

struct BehaviorPairing {
    pairs: Vec<(NodeId, NodeId)>,
    orphans_after: Vec<NodeId>,
}

impl BehaviorPairing {
    fn pair(dag_before: &Dag, dag_after: &Dag) -> Self {
        let mut bucket: HashMap<SpanPair, Vec<NodeId>> = HashMap::new();
        for behavior in dag_before.nodes() {
            let span = behavior.span();
            let key = SpanPair {
                file: span.file.clone(),
                byte_start: span.byte_start,
            };
            bucket.entry(key).or_default().push(behavior.id());
        }
        for lists in bucket.values_mut() {
            lists.sort_by_key(|id| id.raw());
        }
        let mut pairs = Vec::new();
        let mut orphans_after = Vec::new();
        for behavior in dag_after.nodes() {
            let span = behavior.span();
            let key = SpanPair {
                file: span.file.clone(),
                byte_start: span.byte_start,
            };
            match bucket.get_mut(&key) {
                Some(stack) => {
                    if let Some(peer) = stack.pop() {
                        pairs.push((peer, behavior.id()));
                    } else {
                        orphans_after.push(behavior.id());
                    }
                }
                None => {
                    orphans_after.push(behavior.id());
                }
            }
        }

        pairs.sort_by_key(|(_, after_id)| after_id.raw());
        orphans_after.sort_by_key(|id| id.raw());
        Self {
            pairs,
            orphans_after,
        }
    }

    fn structural_seed_ids_after(&self, dag_before: &Dag, dag_after: &Dag) -> Vec<NodeId> {
        let mut seeds: HashSet<NodeId> = HashSet::new();
        for (before_id, after_id) in &self.pairs {
            let b = dag_before.node(*before_id);
            let a = dag_after.node(*after_id);
            // Fail-closed (INVARIANTS P1/P3): never drop a paired structural delta based on
            // `SourceSpan::file` heuristics—integration fixtures and production DAGs use the
            // same seeding rule.
            if format!("{b:?}") != format!("{a:?}") {
                seeds.insert(*after_id);
            }
        }
        for orphan in &self.orphans_after {
            seeds.insert(*orphan);
        }
        let mut seeds: Vec<NodeId> = seeds.into_iter().collect();
        seeds.sort_by_key(|id| id.raw());
        seeds
    }
}

struct ShapeMapAfter {
    map: HashMap<NodeId, StructuralEffectShape>,
}

impl ShapeMapAfter {
    fn effects(dag: &Dag) -> Self {
        let report = crate::lens_effect_enumeration::enumerate_effects(dag);
        let mut facts_by_port: HashMap<PortId, StructuralEffectShape> =
            HashMap::with_capacity(report.facts.len());
        for fact in &report.facts {
            facts_by_port.insert(fact.port, fact.shape.clone());
        }
        let mut map = HashMap::new();
        for behavior in dag.nodes() {
            let rp = behavior_result_port(behavior);
            if let Some(shape) = facts_by_port.get(&rp) {
                map.insert(behavior.id(), shape.clone());
            }
        }
        Self { map }
    }

    fn shape_debug_of(&self, id: NodeId) -> Option<String> {
        self.map.get(&id).map(|shape| format!("{shape:?}"))
    }
}

pub fn compute_affected_set_lens_report(
    dag_before: &Dag,
    dag_after: &Dag,
) -> AffectedSetLensReport {
    let (downstream, downstream_integrity) = DownstreamAdjacency::build(dag_after);
    let pairing = BehaviorPairing::pair(dag_before, dag_after);
    let mut structural_seed_set: HashSet<NodeId> = pairing
        .structural_seed_ids_after(dag_before, dag_after)
        .into_iter()
        .collect();
    structural_seed_set.extend(
        downstream_integrity
            .fail_closed_consumer_seeds
            .iter()
            .copied(),
    );
    let mut structural_seeds_after: Vec<NodeId> = structural_seed_set.into_iter().collect();
    structural_seeds_after.sort_by_key(|id| id.raw());
    let first_structural_difference = None;
    let transitive_downstream = downstream.transitive_forward(&structural_seeds_after);
    let structural_slice = propagate_slice_structural(
        &downstream_integrity,
        structural_seeds_after.clone(),
        transitive_downstream.clone(),
    );

    let value_slice = propagate_slice_value(&downstream, structural_seeds_after.clone());

    let cost_seeds = seeds_for_cost_dimension(dag_before, dag_after, &pairing);
    let cost_slice = propagate_slice_with_seeds_only("cost", &downstream, cost_seeds);

    let effect_seeds = seeds_for_effect_dimension(dag_before, dag_after, &pairing);
    let effect_slice = propagate_slice_with_seeds_only("effect", &downstream, effect_seeds);

    let refinement_seeds = seeds_for_refinement_dimension(dag_before, dag_after, &pairing);
    let refinement_slice =
        propagate_slice_with_seeds_only("refinement", &downstream, refinement_seeds);

    let structural_seed_count = structural_seeds_after.len();
    let span_key_behavior_pair_count = pairing.pairs.len();
    let span_key_behavior_orphan_after_count = pairing.orphans_after.len();
    let aggregate_union = aggregate_union_sorted(&[
        &structural_slice,
        &value_slice,
        &cost_slice,
        &effect_slice,
        &refinement_slice,
    ]);

    AffectedSetLensReport {
        structural_seed_count,
        first_structural_difference,
        downstream_build_receipt: downstream_integrity,
        structural: structural_slice,
        transitive_downstream: transitive_downstream.clone(),
        value: value_slice,
        cost: cost_slice,
        effect: effect_slice,
        refinement: refinement_slice,
        aggregate_union,
        span_key_behavior_pair_count,
        span_key_behavior_orphan_after_count,
    }
}

fn propagate_slice_structural(
    integrity: &DownstreamBuildReceipt,
    structural_seeds_after: Vec<NodeId>,
    transitive_downstream: Vec<NodeId>,
) -> DimensionSlice {
    let mut seed_ids_sorted = structural_seeds_after;
    seed_ids_sorted.sort_by_key(|id| id.raw());
    let mut summary = String::new();
    let _ = write!(
        &mut summary,
        "dimension=structural; seed_count={}; affected_count={}; discipline=paired_span_key_behavior_snapshot_diff_orphans_downstream_anchor_fail_closed; downstream_walk_fail_closed_seeds={}; malformed_upstream_walk_strings={}",
        seed_ids_sorted.len(),
        transitive_downstream.len(),
        integrity.fail_closed_consumer_seeds.len(),
        integrity.malformed_producer_walks.len(),
    );
    let dimension_node = seed_ids_sorted.first().copied();
    DimensionSlice {
        dimension: "structural",
        seed_ids: seed_ids_sorted,
        affected_ids: transitive_downstream,
        receipts: vec![AffectedSetReceipt {
            dimension: "structural",
            dimension_node,
            summary,
        }],
    }
}

fn propagate_slice_value(
    downstream: &DownstreamAdjacency,
    structural_seeds_after: Vec<NodeId>,
) -> DimensionSlice {
    let mut seed_ids_sorted = structural_seeds_after;
    seed_ids_sorted.sort_by_key(|id| id.raw());
    let affected = downstream.transitive_forward(&seed_ids_sorted);
    let receipts = vec![AffectedSetReceipt {
        dimension: "value",
        dimension_node: seed_ids_sorted.first().copied(),
        summary: String::from(
            "dimension=value; mirrors structural-edit seeds via full downstream closure; \
             discipline=host_structural_projection_not_io_proof; oracle=unknown (design §2 PROVEN)",
        ),
    }];
    DimensionSlice {
        dimension: "value",
        seed_ids: seed_ids_sorted,
        affected_ids: affected,
        receipts,
    }
}

fn propagate_slice_with_seeds_only(
    dimension: &'static str,
    downstream: &DownstreamAdjacency,
    seeds: HashSet<NodeId>,
) -> DimensionSlice {
    let mut seed_ids_sorted: Vec<NodeId> = seeds.into_iter().collect();
    seed_ids_sorted.sort_by_key(|id| id.raw());
    let affected = downstream.transitive_forward(&seed_ids_sorted);
    let mut summary = String::new();
    let _ = write!(
        &mut summary,
        "dimension={}; seed_count={}; affected_count={}; discipline=paired_behavior_span_key_miss_or_carrier_diff_then_fail_closed",
        dimension,
        seed_ids_sorted.len(),
        affected.len()
    );
    let receipts = vec![AffectedSetReceipt {
        dimension,
        dimension_node: seed_ids_sorted.first().copied(),
        summary,
    }];
    DimensionSlice {
        dimension,
        seed_ids: seed_ids_sorted,
        affected_ids: affected,
        receipts,
    }
}

fn aggregate_union_sorted(slices: &[&DimensionSlice]) -> Vec<NodeId> {
    let mut set: HashSet<NodeId> = HashSet::new();
    for slice in slices {
        for id in &slice.affected_ids {
            set.insert(*id);
        }
    }
    let mut out: Vec<NodeId> = set.into_iter().collect();
    out.sort_by_key(|id| id.raw());
    out
}

fn seeds_for_cost_dimension(
    dag_before: &Dag,
    dag_after: &Dag,
    pairing: &BehaviorPairing,
) -> HashSet<NodeId> {
    let mut seeds = HashSet::new();
    for (before_id, after_id) in &pairing.pairs {
        let before_node = dag_before.node(*before_id);
        let after_node = dag_after.node(*after_id);
        let pb = behavior_result_port(before_node);
        let pa = behavior_result_port(after_node);
        match (
            complexity_of(dag_before, &pb),
            complexity_of(dag_after, &pa),
        ) {
            (CostLookup::Miss, _) | (_, CostLookup::Miss) => {
                seeds.insert(*after_id);
            }
            (CostLookup::Hit(cb), CostLookup::Hit(ca))
                if format!("{cb:?}") != format!("{ca:?}") =>
            {
                seeds.insert(*after_id);
            }
            _ => {}
        }
    }
    for orphan in &pairing.orphans_after {
        seeds.insert(*orphan);
    }
    seeds
}

fn seeds_for_effect_dimension(
    dag_before: &Dag,
    dag_after: &Dag,
    pairing: &BehaviorPairing,
) -> HashSet<NodeId> {
    let shapes_after = ShapeMapAfter::effects(dag_after);
    let shapes_before = ShapeMapAfter::effects(dag_before);
    let mut seeds = HashSet::new();
    for (before_id, after_id) in &pairing.pairs {
        match (
            shapes_before.shape_debug_of(*before_id),
            shapes_after.shape_debug_of(*after_id),
        ) {
            (Some(sb), Some(sa)) if sb != sa => {
                seeds.insert(*after_id);
            }
            (Some(_), Some(_)) => {}
            (None, _) | (_, None) => {
                seeds.insert(*after_id);
            }
        }
    }
    for orphan in &pairing.orphans_after {
        seeds.insert(*orphan);
    }
    seeds
}

fn seeds_for_refinement_dimension(
    dag_before: &Dag,
    dag_after: &Dag,
    pairing: &BehaviorPairing,
) -> HashSet<NodeId> {
    let mut seeds = HashSet::new();
    for (before_id, after_id) in &pairing.pairs {
        let before_node = dag_before.node(*before_id);
        let after_node = dag_after.node(*after_id);
        let pb = behavior_result_port(before_node);
        let pa = behavior_result_port(after_node);
        match (
            refinement_projection(dag_before, pb),
            refinement_projection(dag_after, pa),
        ) {
            (Ok(sb), Ok(sa)) if sb == sa => {}
            _ => {
                seeds.insert(*after_id);
            }
        }
    }
    for orphan in &pairing.orphans_after {
        seeds.insert(*orphan);
    }
    seeds
}

fn refinement_projection(dag: &Dag, port: PortId) -> Result<String, ()> {
    let port_state = dag.port_opt(&port).ok_or(())?;
    Ok(format!("{:?}", port_state.state()))
}

fn behavior_result_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.result_port(),
        Behavior::Transform(t) => t.result_port(),
        Behavior::Branch(b) => b.result_port(),
        Behavior::Loop(l) => l.result_port(),
        Behavior::Bind(bind) => bind.result_port(),
    }
}

/// Structural dependency anchor: port-level dataflow **or** an explicit subgraph
/// body root (`Branch` arm / `Loop` kernel / `Transform` callee `Arrow` bind) per
/// substrate `NodeId` (P2 facts-flow).
///
/// Classification: 🟡 scaffold coproduct (`DataPort` | `BodySubgraphRoot`) — mirrors
/// substrate carriers already present on `Behavior` / `BranchPath` / `LoopNode` without
/// inventing an independent taxonomy (`docs/modeling-discipline.md` Practice 4), but remains
/// a host-side mirror rather than authoritative substrate enumeration.
///
/// Dissolution ledger: retire when `v3.std.substrate` exposes an authoritative
/// “dependency operand” row set so lenses enumerate these edges from data alone
/// (`ROADMAP.md` T-PB-A / consolidation thesis), not a Rust-side mirror enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorStructuralOperand {
    DataPort(PortId),
    BodySubgraphRoot(NodeId),
}

struct UpstreamProducerWalk {
    producer_ids: Vec<NodeId>,
    malformed_producer_walks: Vec<String>,
    fail_closed_consumer_seeds: Vec<NodeId>,
}

fn user_defined_arrow_bind_root_for_declaration(
    dag: &Dag,
    decl_id: DeclarationId,
) -> Option<NodeId> {
    let decl = dag.declaration_opt(&decl_id)?;
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    else {
        return None;
    };
    Some(bind_id.node_id())
}

fn behavior_structural_operands(dag: &Dag, behavior: &Behavior) -> Vec<BehaviorStructuralOperand> {
    match behavior {
        Behavior::Value(_) => vec![],
        Behavior::Transform(t) => {
            let mut ops: Vec<BehaviorStructuralOperand> = t
                .inputs
                .iter()
                .copied()
                .map(BehaviorStructuralOperand::DataPort)
                .collect();
            if let TransformTarget::Callable(decl_id) = &t.target {
                if let Some(bind_root) = user_defined_arrow_bind_root_for_declaration(dag, *decl_id)
                {
                    ops.push(BehaviorStructuralOperand::BodySubgraphRoot(bind_root));
                }
            }
            ops
        }
        Behavior::Branch(b) => {
            let mut v = vec![BehaviorStructuralOperand::DataPort(b.input)];
            for path in &b.paths {
                v.push(BehaviorStructuralOperand::DataPort(path.output));
                if let Some(binding) = &path.binding {
                    v.push(BehaviorStructuralOperand::DataPort(binding.payload_port));
                }
                v.push(BehaviorStructuralOperand::BodySubgraphRoot(path.body));
            }
            v
        }
        Behavior::Loop(l) => {
            let mut v = vec![
                BehaviorStructuralOperand::DataPort(l.source),
                BehaviorStructuralOperand::DataPort(l.init),
            ];
            if let crate::dag::LoopBound::Cardinality { count } = &l.bound {
                v.push(BehaviorStructuralOperand::DataPort(*count));
            }
            v.push(BehaviorStructuralOperand::BodySubgraphRoot(l.body));
            v
        }
        Behavior::Bind(bind) => {
            let mut v = vec![BehaviorStructuralOperand::DataPort(bind.value)];
            v.extend(
                bind.params
                    .iter()
                    .copied()
                    .map(BehaviorStructuralOperand::DataPort),
            );
            v
        }
    }
}

fn ingest_port_producers(
    dag: &Dag,
    consumer_id: NodeId,
    port: PortId,
    producer_ids: &mut Vec<NodeId>,
    malformed_producer_walks: &mut Vec<String>,
    fail_closed_consumer_seeds: &mut HashSet<NodeId>,
) {
    match dag.resolve_producer_lookup(&port) {
        ProducerLookup::Found(producer) => {
            producer_ids.push(producer.id());
        }
        ProducerLookup::NoProducer => {}
        ProducerLookup::MissingPort { port: bad } => {
            malformed_producer_walks
                .push(format!("MissingPort consumer={consumer_id:?} port={bad:?}"));
            fail_closed_consumer_seeds.insert(consumer_id);
        }
        ProducerLookup::MissingNode { producer } => {
            malformed_producer_walks.push(format!(
                "MissingNode consumer={consumer_id:?} producer_link={producer:?}"
            ));
            fail_closed_consumer_seeds.insert(consumer_id);
        }
        ProducerLookup::BindCycle { detected_at } => {
            malformed_producer_walks.push(format!(
                "BindCycle consumer={consumer_id:?} detected_at={detected_at:?}"
            ));
            fail_closed_consumer_seeds.insert(consumer_id);
        }
    }
}

fn resolve_behavior_upstream_producers(dag: &Dag, consumer: &Behavior) -> UpstreamProducerWalk {
    let mut producer_ids: Vec<NodeId> = Vec::new();
    let mut malformed_producer_walks: Vec<String> = Vec::new();
    let mut fail_closed_consumer_seeds: HashSet<NodeId> = HashSet::new();
    let consumer_id = consumer.id();

    for operand in behavior_structural_operands(dag, consumer) {
        match operand {
            BehaviorStructuralOperand::DataPort(port) => ingest_port_producers(
                dag,
                consumer_id,
                port,
                &mut producer_ids,
                &mut malformed_producer_walks,
                &mut fail_closed_consumer_seeds,
            ),
            BehaviorStructuralOperand::BodySubgraphRoot(root) => {
                let Some(body_behavior) = dag.node_opt(&root) else {
                    malformed_producer_walks.push(format!(
                        "BodySubgraphRootMissingNode consumer={consumer_id:?} body_root={root:?}"
                    ));
                    fail_closed_consumer_seeds.insert(consumer_id);
                    continue;
                };
                // P2 facts-flow: the subgraph root is a structural operand on its own. If the
                // body's result port resolves to [`ProducerLookup::NoProducer`] (e.g.
                // unproduced-parameter style bodies), callers must still consume the callee
                // root — do not collapse the operand to port-only walkers.
                producer_ids.push(root);
                ingest_port_producers(
                    dag,
                    consumer_id,
                    behavior_result_port(body_behavior),
                    &mut producer_ids,
                    &mut malformed_producer_walks,
                    &mut fail_closed_consumer_seeds,
                );
            }
        }
    }

    producer_ids.sort_by_key(|id| id.raw());
    producer_ids.dedup();
    malformed_producer_walks.sort();

    let mut fc: Vec<NodeId> = fail_closed_consumer_seeds.into_iter().collect();
    fc.sort_by_key(|id| id.raw());

    UpstreamProducerWalk {
        producer_ids,
        malformed_producer_walks,
        fail_closed_consumer_seeds: fc,
    }
}
