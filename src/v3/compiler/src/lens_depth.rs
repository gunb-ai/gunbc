// Depth lens — for each port, computes the longest path (in edges)
// from a leaf port (one with no producer) to this port. A leaf is a
// function parameter or a Value literal — anything whose
// `produced_by` is None.
//
// Second observational lens. Purpose: validate the v3 success bar
// empirically — a new observational lens over the substrate should
// land in tens of lines with zero substrate modifications. The
// provenance lens (M0.4) was the first data point; this is the
// second. If the substrate supports observational queries cheaply,
// this lens costs no more than reading `produced_by` edges and
// dispatching on the producer's behavior kind.
//
// Size budget: ~50 lines. If this grows, the substrate is gapped
// for this kind of analysis and M1 should fix it before building
// anything bigger.
//
// NOT a writer lens — the depth is computed on demand by query,
// not stored anywhere. M1's cost lens will be the first writer
// and will force the "how do lenses store results" decision.

use crate::dag::{Behavior, Dag, NodeId, PortId};

pub struct DepthLens<'a> {
    dag: &'a Dag,
}

impl<'a> DepthLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    /// Longest path (in edges) from any leaf port to this port.
    /// Leaves are: ports with `produced_by == None` (function
    /// parameters), and Value node output ports (literal sources).
    /// Values are sources by construction, so even though their
    /// output port has a producer, the port itself counts as a
    /// leaf for depth purposes.
    pub fn depth_of(&self, port: PortId) -> usize {
        match self.dag.port(port).produced_by {
            None => 0,
            Some(node_id) => match self.dag.node(node_id) {
                Behavior::Value(_) => 0,
                _ => 1 + self.node_input_depth(node_id),
            },
        }
    }

    fn node_input_depth(&self, node_id: NodeId) -> usize {
        match self.dag.node(node_id) {
            // Unreachable — depth_of short-circuits Value above. Kept
            // for match exhaustiveness and clarity.
            Behavior::Value(_) => 0,
            Behavior::Transform(t) => t
                .inputs
                .iter()
                .map(|&p| self.depth_of(p))
                .max()
                .unwrap_or(0),
            Behavior::Branch(b) => {
                let cond = self.depth_of(b.input);
                let paths_max = b
                    .paths
                    .iter()
                    .map(|path| self.depth_of(path.output))
                    .max()
                    .unwrap_or(0);
                cond.max(paths_max)
            }
            Behavior::Loop(l) => self.depth_of(l.source).max(self.depth_of(l.init)),
            Behavior::Bind(b) => self.depth_of(b.value),
        }
    }
}
