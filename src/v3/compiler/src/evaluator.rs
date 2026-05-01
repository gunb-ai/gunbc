//! E2 evaluator frame helpers.
//!
//! This module is the narrow Rust realization of the existing
//! `EvalFrame { bindings: Map<PortId, Value> }` and
//! `EvalStateStack { frames: List<EvalFrame> }` substrate carriers. The
//! value payload is generic on purpose: E2 owns binding-scope behavior, not a
//! new observable `Value` carrier. Later body-evaluator slices plug in the
//! runtime value representation without changing the frame rules here.
//!
//! Dissolution target: when the `.dag` evaluator body implementation owns
//! frame mutation directly, this host helper should shrink to generated or
//! substrate-backed calls instead of becoming a parallel evaluator runtime.

use std::collections::HashMap;

use crate::dag::PortId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalFrameError {
    EmptyStateStack,
    DuplicateBinding { port: PortId },
    UnboundPort { port: PortId },
}

/// One evaluator binding scope.
///
/// `HashMap<PortId, V>` is the Rust realization of the substrate
/// `Map<PortId, Value>` finite partial-function discipline. Do not replace
/// this with a duplicate-admitting `List<EvalBinding>` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalFrame<V> {
    bindings: HashMap<PortId, V>,
}

impl<V> EvalFrame<V> {
    pub fn empty() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn from_bindings(
        bindings: impl IntoIterator<Item = (PortId, V)>,
    ) -> Result<Self, EvalFrameError> {
        let mut frame = Self::empty();
        for (port, value) in bindings {
            frame.bind(port, value)?;
        }
        Ok(frame)
    }

    pub fn bind(&mut self, port: PortId, value: V) -> Result<(), EvalFrameError> {
        if self.bindings.contains_key(&port) {
            return Err(EvalFrameError::DuplicateBinding { port });
        }
        self.bindings.insert(port, value);
        Ok(())
    }

    pub fn lookup_local(&self, port: PortId) -> Option<&V> {
        self.bindings.get(&port)
    }
}

/// Evaluator frame stack. The final element is the innermost / top frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalStateStack<V> {
    frames: Vec<EvalFrame<V>>,
}

impl<V> EvalStateStack<V> {
    pub fn with_root_frame(frame: EvalFrame<V>) -> Self {
        Self {
            frames: vec![frame],
        }
    }

    pub fn from_outer_to_inner(frames: Vec<EvalFrame<V>>) -> Self {
        Self { frames }
    }

    pub fn push_frame(&mut self, frame: EvalFrame<V>) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> Result<EvalFrame<V>, EvalFrameError> {
        self.frames.pop().ok_or(EvalFrameError::EmptyStateStack)
    }

    pub fn lookup(&self, port: PortId) -> Result<&V, EvalFrameError> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.lookup_local(port))
            .ok_or(EvalFrameError::UnboundPort { port })
    }

    pub fn bind_top(&mut self, port: PortId, value: V) -> Result<(), EvalFrameError> {
        self.frames
            .last_mut()
            .ok_or(EvalFrameError::EmptyStateStack)?
            .bind(port, value)
    }

    pub fn frames_outer_to_inner(&self) -> &[EvalFrame<V>] {
        &self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::{EvalFrame, EvalFrameError, EvalStateStack};
    use crate::dag::{Dag, LiteralBits, PortId};
    use crate::diagnostics::SourceSpan;

    fn span() -> SourceSpan {
        SourceSpan::new("evaluator_frame.unit", 0, 1)
    }

    fn ports(count: usize) -> Vec<PortId> {
        let mut dag = Dag::new();
        (0..count)
            .map(|i| dag.push_value(LiteralBits::Int(i as i64), span()))
            .collect()
    }

    #[test]
    fn lookup_walks_innermost_frame_first() {
        let ids = ports(2);
        let outer = EvalFrame::from_bindings([(ids[0], "outer"), (ids[1], "outer-only")])
            .expect("outer frame");
        let inner = EvalFrame::from_bindings([(ids[0], "inner")]).expect("inner frame");
        let stack = EvalStateStack::from_outer_to_inner(vec![outer, inner]);

        assert_eq!(stack.lookup(ids[0]), Ok(&"inner"));
        assert_eq!(stack.lookup(ids[1]), Ok(&"outer-only"));
    }

    #[test]
    fn bind_top_writes_only_the_innermost_frame() {
        let ids = ports(1);
        let outer = EvalFrame::from_bindings([(ids[0], "outer")]).expect("outer frame");
        let inner = EvalFrame::empty();
        let mut stack = EvalStateStack::from_outer_to_inner(vec![outer, inner]);

        stack.bind_top(ids[0], "inner").expect("bind top");

        assert_eq!(stack.lookup(ids[0]), Ok(&"inner"));
        assert_eq!(
            stack.frames_outer_to_inner()[0].lookup_local(ids[0]),
            Some(&"outer")
        );
        assert_eq!(
            stack.frames_outer_to_inner()[1].lookup_local(ids[0]),
            Some(&"inner")
        );
    }

    #[test]
    fn bind_top_rejects_duplicate_binding_in_current_frame() {
        let ids = ports(1);
        let mut stack =
            EvalStateStack::with_root_frame(EvalFrame::from_bindings([(ids[0], 1)]).unwrap());

        let err = stack.bind_top(ids[0], 2).expect_err("duplicate rejected");

        assert_eq!(err, EvalFrameError::DuplicateBinding { port: ids[0] });
        assert_eq!(stack.lookup(ids[0]), Ok(&1));
    }

    #[test]
    fn lookup_reports_unbound_port_after_full_stack_walk() {
        let ids = ports(2);
        let stack =
            EvalStateStack::with_root_frame(EvalFrame::from_bindings([(ids[0], 1)]).unwrap());

        let err = stack.lookup(ids[1]).expect_err("unbound rejected");

        assert_eq!(err, EvalFrameError::UnboundPort { port: ids[1] });
    }

    #[test]
    fn bind_top_reports_empty_state_stack() {
        let ids = ports(1);
        let mut stack: EvalStateStack<i64> = EvalStateStack::from_outer_to_inner(Vec::new());

        let err = stack.bind_top(ids[0], 1).expect_err("empty stack rejected");

        assert_eq!(err, EvalFrameError::EmptyStateStack);
    }
}
