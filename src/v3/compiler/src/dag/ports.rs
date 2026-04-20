//! Port + port-reference carriers extracted from `dag.rs` (L4b).
//!
//! Holds [`Port`] itself plus the small handle/witness structs
//! ([`ParamRef`], [`TransformRef`], [`ElementRef`], [`BoolPortRef`]) and
//! the non-trivial-arity list helpers ([`NonEmptyList`],
//! [`NonSingletonList`]). No behavior change — the re-exports in the
//! `dag` module root preserve the external API.

use std::marker::PhantomData;

use crate::types::TypeShape;

use super::{NodeId, PortId, PortState};

/// Three-state port type. Illegal combinations of "has a type" and "has a
/// diagnostic" are unrepresentable by type:
///
///   - `Uninferred`: port exists but inference has not run on it yet.
///     Transitional state during DAG construction and fixpoint iteration. The
///     post-infer sweep drives every port to Resolved or Unresolved before
///     compile_to_dag returns.
///   - `Resolved(TypeShape)`: inference (or lowering from a declaration) has
///     committed to a type.
///   - `Unresolved`: inference or lowering detected a failure and called
///     `Dag::mark_unresolved`. A diagnostic exists in the DiagnosticTable
///     keyed by this port's id.
///
/// Biconditional (checked by the invariant audit test):
///   state == Unresolved  iff  diagnostics.contains(port.id())
///
/// **Dissolution receipt: TERMINAL.** PortState is substrate, not an annotation.
/// A Port carries a typed value forward in time.
///
/// `state` has a single authoritative location: the Port struct stored in
/// Dag.ports. There are no stale copies — behaviors hold PortId references, not
/// embedded Ports.
#[derive(Debug, Clone)]
pub struct Port {
    pub(super) id: PortId,
    pub(super) state: PortState,
    pub produced_by: Option<NodeId>,
}

impl Port {
    pub fn id(&self) -> PortId {
        self.id
    }

    pub fn state(&self) -> &PortState {
        &self.state
    }

    pub fn state_value(&self) -> PortState {
        self.state.clone()
    }

    /// Backward-compat accessor: returns `Some(&TypeShape)` for Resolved ports,
    /// `None` for Uninferred or Unresolved. Prefer `state()` when you need to
    /// distinguish the three cases.
    pub fn value_type(&self) -> Option<&TypeShape> {
        match &self.state {
            PortState::Resolved(ty) => Some(ty),
            PortState::Uninferred | PortState::Unresolved => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamRef {
    pub(super) member: NodeId,
    pub(super) slot: usize,
}

impl ParamRef {
    pub fn member_of(self) -> NodeId {
        self.member
    }

    pub fn slot_of(self) -> usize {
        self.slot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransformRef(pub(super) NodeId);

impl TransformRef {
    pub fn node_id(self) -> NodeId {
        self.0
    }
}

/// 🟢 **TERMINAL at current Track 9 scope.** Generic index witness parallel to
/// [`ParamRef`] / [`TransformRef`]. The only Rust constructor is
/// [`ElementRef::from_slice`], which validates the index against the slice in
/// scope. The handle does not retain owner identity after construction, so
/// read sites must still resolve it against the same authority list they
/// validated it against. The substrate field shape matches
/// `src/v3/std/substrate.dag`; direct `.dag` construction gains the same
/// authority in the Lane 3c cycle (ROADMAP Track 9 debt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementRef<T> {
    index: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ElementRef<T> {
    pub fn index_of(self) -> usize {
        self.index
    }

    pub fn from_slice(values: &[T], index: usize) -> Option<Self> {
        values.get(index)?;
        Some(Self {
            index,
            _marker: PhantomData,
        })
    }

    pub fn get(self, values: &[T]) -> Option<&T> {
        values.get(self.index)
    }
}

/// 🟢 **TERMINAL.** Bool-typed branch predicate port — Track 9 parallel to
/// [`ParamRef`] / [`TransformRef`]. The only Rust constructor is
/// [`super::Dag::bool_port_of`], which checks the port resolves to `Bool`. The
/// substrate field shape matches `src/v3/std/effects.dag`; direct `.dag`
/// construction gains the same authority in the Lane 3c cycle (ROADMAP Track 9 debt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoolPortRef {
    pub(super) port: PortId,
}

impl BoolPortRef {
    pub fn port_id(self) -> PortId {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyList<T> {
    pub first: T,
    pub rest: Vec<T>,
}

impl<T> NonEmptyList<T> {
    pub fn from_vec(values: Vec<T>) -> Option<Self> {
        let mut iter = values.into_iter();
        let first = iter.next()?;
        Some(Self {
            first,
            rest: iter.collect(),
        })
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        std::iter::once(self.first.clone())
            .chain(self.rest.iter().cloned())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonSingletonList<T> {
    pub first: T,
    pub second: T,
    pub rest: Vec<T>,
}

impl<T> NonSingletonList<T> {
    pub fn from_vec(values: Vec<T>) -> Option<Self> {
        let mut iter = values.into_iter();
        let first = iter.next()?;
        let second = iter.next()?;
        Some(Self {
            first,
            second,
            rest: iter.collect(),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.first)
            .chain(std::iter::once(&self.second))
            .chain(self.rest.iter())
    }

    pub fn len(&self) -> usize {
        2 + self.rest.len()
    }

    /// `NonSingletonList` always has at least two elements by construction;
    /// this exists to satisfy clippy's `len_without_is_empty` lint and always
    /// returns `false`.
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        std::iter::once(self.first.clone())
            .chain(std::iter::once(self.second.clone()))
            .chain(self.rest.iter().cloned())
            .collect()
    }
}
