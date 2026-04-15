// Cost lens — counts structural operations in the sub-DAG rooted at
// a port. A pure-reader lens in the same template as `lens_depth`
// and `lens_provenance`: takes `&Dag`, returns a computed value, no
// storage, no side tables.
//
// Third observational lens. Purpose: validate that PR-B's substrate
// is still under the "new lens lands in tens of lines" success bar
// set by M0.3/M0.4. Depth was ~50 lines, provenance was ~40 lines;
// cost is another data point on the same curve.
//
// Cost semantics:
//   - Value literal       → 0 (leaf, no work)
//   - Function parameter  → 0 (leaf, no work)
//   - Transform           → 1 + sum(input costs) (one op + its args)
//   - Branch              → 1 + cond cost + max(path costs) (runtime
//                             fires exactly one path, so paths use max,
//                             not sum; the branch itself costs 1)
//   - Loop                → 1 + source + init (one op + its setup)
//   - Bind                → pass through (naming is free)
//
// Size budget: ~80 lines. If this balloons, the substrate is gapped
// for structural counting — fix the substrate before growing the
// lens.
//
// NOT a writer lens. The cost is computed on demand by query, not
// cached. When real performance benchmarks force caching, the
// "how do lenses store results" question from lens_depth's comment
// fires and gets answered uniformly for every lens at once.

use crate::dag::{Behavior, Dag, NodeId, PortId};

pub struct CostLens<'a> {
    dag: &'a Dag,
}

impl<'a> CostLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    /// Structural cost for the sub-DAG rooted at `port`. Counts
    /// Transform / Branch / Loop operations recursively; Value and
    /// function-parameter leaves contribute 0; Bind passes through
    /// to the bound value.
    pub fn cost_of(&self, port: PortId) -> usize {
        match self.dag.port(port).produced_by {
            None => 0,
            Some(node_id) => match self.dag.node(node_id) {
                Behavior::Value(_) => 0,
                _ => self.node_cost(node_id),
            },
        }
    }

    fn node_cost(&self, node_id: NodeId) -> usize {
        match self.dag.node(node_id) {
            // Unreachable — cost_of short-circuits Value above. Kept
            // for match exhaustiveness and parity with lens_depth.
            Behavior::Value(_) => 0,
            Behavior::Transform(t) => {
                let input_cost: usize = t.inputs.iter().map(|&p| self.cost_of(p)).sum();
                1 + input_cost
            }
            Behavior::Branch(b) => {
                let cond = self.cost_of(b.input);
                let paths_max = b
                    .paths
                    .iter()
                    .map(|path| self.cost_of(path.output))
                    .max()
                    .unwrap_or(0);
                1 + cond + paths_max
            }
            Behavior::Loop(l) => 1 + self.cost_of(l.source) + self.cost_of(l.init),
            // Bind passes through — a named binding does no work,
            // it just labels an existing port for downstream reads.
            Behavior::Bind(b) => self.cost_of(b.value),
        }
    }
}
