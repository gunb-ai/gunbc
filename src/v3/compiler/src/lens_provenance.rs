// Provenance lens — reads Port.produced_by and classifies by the
// producer's behavior kind. Zero reconstruction.
//
// v2 equivalent: a ~5000-line reconstruction engine that tried to
// compute "where did this value come from?" because TypeBinding had
// thrown away origin during inference. v3 never throws it away —
// every Port has produced_by, and the lens just follows the edge
// backward and reads the producer's behavior.
//
// Size budget: ~40 lines. If this file balloons past 60 lines, STOP
// — the physics has a hole. Fix the substrate, not the lens. That
// is the v3-vs-v2 proof point Test 4 is asserting.

use crate::dag::{Behavior, Dag, NodeId, PortId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Parameter of an enclosing function, or a literal Value. Both
    /// mean "this port carries a source value, not a computed one."
    Source { by: Option<NodeId> },
    /// Produced by a Transform (function application).
    Computed { by: NodeId },
    /// Produced by a Branch (one path fired).
    Selected { by: NodeId },
    /// Produced by a Loop (iteration result).
    Accumulated { by: NodeId },
}

pub struct ProvenanceLens<'a> {
    dag: &'a Dag,
}

impl<'a> ProvenanceLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn origin_of(&self, port: PortId) -> Origin {
        match self.dag.port(port).produced_by {
            None => Origin::Source { by: None },
            Some(producer) => match self.dag.node(producer) {
                Behavior::Value(v) => Origin::Source { by: Some(v.id) },
                Behavior::Transform(t) => Origin::Computed { by: t.id },
                Behavior::Branch(b) => Origin::Selected { by: b.id },
                Behavior::Loop(l) => Origin::Accumulated { by: l.id },
                Behavior::Bind(b) => Origin::Source { by: Some(b.id) },
            },
        }
    }
}
