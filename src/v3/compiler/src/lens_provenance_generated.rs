// AUTO-GENERATED from `src/v3/lenses/provenance.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum IntegerAlgebra {
    OrderedRingAlgebra,
    SemiringAlgebra,
}
#[derive(Clone, Debug)]
pub enum NonIntegerAlgebra {
    BooleanAlgebraAlgebra,
    TerminalAlgebra,
}
#[derive(Clone, Debug)]
pub enum TargetCarrier {
    BitCarrier,
    ByteCarrier,
    Word16Carrier,
    Word32Carrier,
    Word64Carrier,
    TerminalCarrier,
}
#[derive(Clone, Debug)]
pub enum IntegerOverflow {
    TwoComplementWrap,
    Saturating,
    Trap,
}
#[derive(Clone, Debug)]
pub enum RustPrimitive {
    IntegerPrimitive {
        target_name: String,
        algebra: IntegerAlgebra,
        carrier: TargetCarrier,
        is_copy: bool,
        overflow: IntegerOverflow,
    },
    NonIntegerPrimitive {
        target_name: String,
        algebra: NonIntegerAlgebra,
        carrier: TargetCarrier,
        is_copy: bool,
    },
}
#[derive(Clone, Debug)]
pub enum Origin {
    NoProducer,
    MissingPort,
    MissingBehavior,
    Source { _0: NodeId },
    Computed { _0: NodeId },
    Selected { _0: NodeId },
    Accumulated { _0: NodeId },
}
pub fn origin_of(p0: &Dag, p1: &PortId) -> Origin {
    match &((p0).port_opt(p1).cloned()) {
        None => Origin::MissingPort,
        Some(p) => match &((p).produced_by) {
            None => Origin::NoProducer,
            Some(producer_id) => match &((p0).node_opt(producer_id).cloned()) {
                None => Origin::MissingBehavior,
                Some(behavior) => origin_for_behavior(behavior),
            },
        },
    }
}
pub fn origin_for_behavior(p0: &Behavior) -> Origin {
    match p0 {
        Behavior::Value(v) => Origin::Source { _0: (v).id },
        Behavior::Transform(t) => Origin::Computed { _0: (t).id },
        Behavior::Branch(b) => Origin::Selected { _0: (b).id },
        Behavior::Loop(l) => Origin::Accumulated { _0: (l).id },
        Behavior::Bind(bind) => Origin::Source { _0: (bind).id },
    }
}
