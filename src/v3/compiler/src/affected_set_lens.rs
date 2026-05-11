//! Affected-set lens prototype (gunbc#2699, companion design docs PR #2700).
//!
//! Pure-host analysis over two compiled [`Dag`] snapshots. Each **dimension slice**
//! starts from nodes with a **non-empty delta** for that dimension (fail-closed when
//! comparisons are unavailable), then closes **forward over downstream consumers**
//! using the substrate port graph (reverse of `resolve_producer_lookup`).
//!
//! ## Value dimension caveat
//!
//! Structural `Behavior` inequality does **not** prove return-value equivalence or
//! non-equivalence. Following design §2 `PROVEN` discipline, this slice uses the
//! structural-edit seed set (paired `Debug` inequality + orphan after-nodes) and the
//! same downstream closure as the naive baseline for that seed set. **Per-node proof
//! receipts that exclude downstream consumers on value grounds are not emitted yet**
//! because the proof substrate does not expose an I/O-equivalence oracle in-tree.
//!
//! Narrowing demos for gunbc#2699 lean on **`cost` / `effect` / `refinement`**
//! deltas plus commentary in `.dag` worked examples.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;

use crate::dag::{Behavior, Dag, Lookup as CostLookup, NodeId, PortId, ProducerLookup};
use crate::lens_cost::complexity_of;
use crate::lens_effect_enumeration::StructuralEffectShape;
use crate::serialize::first_difference;

/// One receipt row for exclusions / inclusion explanations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedSetReceipt {
    pub dimension: &'static str,
    pub dimension_node: Option<NodeId>,
    pub summary: String,
}

/// Full prototype report emitted to integration tests / tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedSetLensReport {
    pub structural_seed_count: usize,
    pub first_structural_difference: Option<String>,
    pub transitive_downstream: Vec<NodeId>,
    pub value: DimensionSlice,
    pub cost: DimensionSlice,
    pub effect: DimensionSlice,
    pub refinement: DimensionSlice,
    pub aggregate_union: Vec<NodeId>,
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
    pub fn build(dag: &Dag) -> Self {
        let mut outgoing: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        for consumer in dag.nodes() {
            for producer in direct_producer_nodes(dag, consumer) {
                outgoing
                    .entry(producer)
                    .or_default()
                    .insert(consumer.id());
            }
        }
        let mut outgoing_vec: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for (producer, consumers) in outgoing {
            let mut list: Vec<NodeId> = consumers.into_iter().collect();
            list.sort_by_key(|id| id.raw());
            outgoing_vec.insert(producer, list);
        }
        Self {
            outgoing: outgoing_vec,
        }
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
        let mut map = HashMap::new();
        for behavior in dag.nodes() {
            let rp = behavior_result_port(behavior);
            if let Some(fact) = report.facts.iter().find(|f| f.port == rp) {
                map.insert(behavior.id(), fact.shape.clone());
            }
        }
        Self { map }
    }

    fn shape_debug_of(&self, id: NodeId) -> Option<String> {
        self.map.get(&id).map(|shape| format!("{shape:?}"))
    }
}

pub fn compute_affected_set_lens_report(dag_before: &Dag, dag_after: &Dag) -> AffectedSetLensReport {
    let downstream = DownstreamAdjacency::build(dag_after);
    let pairing = BehaviorPairing::pair(dag_before, dag_after);
    let structural_seeds_after = pairing.structural_seed_ids_after(dag_before, dag_after);
    let first_structural_difference =
        first_difference(dag_before, dag_after).map(|differ| differ.detail);

    let value_slice = propagate_slice_value(
        &downstream,
        structural_seeds_after.clone(),
    );

    let cost_seeds =
        seeds_for_cost_dimension(dag_before, dag_after, &pairing);
    let cost_slice = propagate_slice_with_seeds_only("cost", &downstream, cost_seeds);

    let effect_seeds =
        seeds_for_effect_dimension(dag_before, dag_after, &pairing);
    let effect_slice =
        propagate_slice_with_seeds_only("effect", &downstream, effect_seeds);

    let refinement_seeds =
        seeds_for_refinement_dimension(dag_before, dag_after, &pairing);
    let refinement_slice = propagate_slice_with_seeds_only(
        "refinement",
        &downstream,
        refinement_seeds,
    );

    let structural_seed_count = structural_seeds_after.len();
    let transitive_downstream = downstream.transitive_forward(&structural_seeds_after);
    let aggregate_union = aggregate_union_sorted(&[
        &value_slice,
        &cost_slice,
        &effect_slice,
        &refinement_slice,
    ]);

    AffectedSetLensReport {
        structural_seed_count,
        first_structural_difference,
        transitive_downstream,
        value: value_slice,
        cost: cost_slice,
        effect: effect_slice,
        refinement: refinement_slice,
        aggregate_union,
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
            "value slice uses structural-edit seeds and full downstream closure; \
             I/O-equivalence exclusions require future proof substrate (design §2 PROVEN)",
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
    let mut receipts = Vec::new();
    if !seed_ids_sorted.is_empty() {
        let mut summary = String::new();
        let _ = write!(
            &mut summary,
            "dimension={}; seed_count={}; affected_count={}",
            dimension,
            seed_ids_sorted.len(),
            affected.len()
        );
        receipts.push(AffectedSetReceipt {
            dimension,
            dimension_node: seed_ids_sorted.first().copied(),
            summary,
        });
    }
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
            (CostLookup::Hit(cb), CostLookup::Hit(ca)) if cb != ca => {
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
    let port_state = dag
        .port_opt(&port)
        .ok_or(())?;
    Ok(format!("{:?}", port_state.state()))
}

/// Ports whose producers `consumer` awaits before executing.
pub fn referenced_ports_direct(behavior: &Behavior, _dag: &Dag) -> Vec<PortId> {
    let mut ports = Vec::new();
    match behavior {
        Behavior::Value(_) => {}
        Behavior::Transform(t) => {
            ports.extend(t.inputs.iter().copied());
        }
        Behavior::Branch(b) => {
            ports.push(b.input);
            for path in &b.paths {
                if let Some(binding) = &path.binding {
                    ports.push(binding.payload_port);
                }
            }
        }
        Behavior::Loop(l) => {
            ports.extend([l.source, l.init]);
            if let crate::dag::LoopBound::Cardinality { count } = &l.bound {
                ports.push(*count);
            }
        }
        Behavior::Bind(bind) => {
            ports.push(bind.value);
            ports.extend(bind.params.iter().copied());
        }
    }
    ports
}

pub fn direct_producer_nodes(dag: &Dag, behavior: &Behavior) -> Vec<NodeId> {
    let mut out = Vec::new();
    for pref in referenced_ports_direct(behavior, dag) {
        match dag.resolve_producer_lookup(&pref) {
            ProducerLookup::Found(producer) => {
                out.push(producer.id());
            }
            ProducerLookup::NoProducer
            | ProducerLookup::MissingPort { .. }
            | ProducerLookup::MissingNode { .. }
            | ProducerLookup::BindCycle { .. } => {}
        }
    }
    out.sort_by_key(|id| id.raw());
    out.dedup();
    out
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
