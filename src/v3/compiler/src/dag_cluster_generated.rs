// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberDescent {
    pub param: ParamRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntraClusterCall {
    pub transform: TransformRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cluster {
    pub members: NonSingletonList<MemberDescent>,
    pub intra_cluster_calls: NonEmptyList<IntraClusterCall>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopBound {
    Cardinality {
        count: PortId,
    },
    Descent {
        cluster: ClusterId,
    },
}
