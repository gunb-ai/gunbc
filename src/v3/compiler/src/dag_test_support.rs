//! Narrow test-support wrappers around the internal Dag builder.
//!
//! Integration tests in `tests/` compile as external crates, so they
//! cannot call `Dag`'s `pub(crate)` builder methods directly. Keep the
//! core builder surface crate-private and expose only the small wrapper
//! set the hand-authored direct-Dag tests need.

use crate::dag::{Dag, LiteralBits, LoopBound, Path, PortId, TransformTarget};
use crate::diagnostics::SourceSpan;
use crate::types::TypeShape;

#[doc(hidden)]
pub fn alloc_port_with_shape(dag: &mut Dag, shape: TypeShape) -> PortId {
    dag.alloc_port_with_shape(shape)
}

#[doc(hidden)]
pub fn push_value(dag: &mut Dag, bits: LiteralBits, span: SourceSpan) -> PortId {
    dag.push_value(bits, span)
}

#[doc(hidden)]
pub fn push_transform(
    dag: &mut Dag,
    target: TransformTarget,
    inputs: Vec<PortId>,
    span: SourceSpan,
) -> PortId {
    dag.push_transform(target, inputs, span)
}

#[doc(hidden)]
pub fn push_bind(
    dag: &mut Dag,
    name: impl Into<String>,
    value: PortId,
    params: Vec<PortId>,
    span: SourceSpan,
) -> crate::dag::NodeId {
    dag.push_bind(name, value, params, span)
}

#[doc(hidden)]
pub fn push_branch(dag: &mut Dag, input: PortId, paths: Vec<Path>, span: SourceSpan) -> PortId {
    dag.push_branch(input, paths, span)
}

#[doc(hidden)]
pub fn push_loop(
    dag: &mut Dag,
    source: PortId,
    init: PortId,
    body: crate::dag::NodeId,
    bound: LoopBound,
    span: SourceSpan,
) -> PortId {
    dag.push_loop(source, init, body, bound, span)
}
