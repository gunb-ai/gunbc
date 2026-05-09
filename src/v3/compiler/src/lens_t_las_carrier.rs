//! Structural carriers for Rust emission of T-LAS lens surfaces that reference
//! `v3.std.lens::Lens`, `std.algebra::Monoid`, and `v3.std.lens_application`
//! (`gate #92`). Shapes mirror the `.dag` authorities; they are not used by the
//! compiler runtime outside `emit_rust_module` snapshots linking against
//! `v3_compiler`.

use std::rc::Rc;

use crate::dag::{Behavior, Dag, LoopBound};
use crate::diagnostics::Diagnostic;
use crate::dimension::Witness;

/// Mirrors `OptionalDiagnostic` in `v3.std.dimensions` for emitted lens code.
#[derive(Clone, Debug)]
pub enum OptionalDiagnostic {
    NoDiagnostic,
    SomeDiagnostic { value: Diagnostic },
}

pub type LensReadFn<T> = dyn Fn(&Dag, &Behavior) -> Witness<T>;
pub type LensValidateFn<T> = dyn Fn(&Dag, T) -> OptionalDiagnostic;

/// Mirrors `Monoid<T>` in `dsl/std/algebra.dag`.
#[derive(Clone)]
pub struct Monoid<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
    pub identity: T,
}

/// Mirrors `Lens<T>` in `v3.std.lens`.
#[derive(Clone)]
pub struct Lens<T> {
    pub name: String,
    pub read: Rc<LensReadFn<T>>,
    pub sequential: Monoid<T>,
    pub branch: Rc<dyn Fn(T, T) -> T>,
    pub iterate: Rc<dyn Fn(T, LoopBound) -> T>,
    pub validate: Rc<LensValidateFn<T>>,
}

/// Mirrors `ProjectionFailure` sum-type in `v3.std.lens_application`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionFailure {
    MissingObservation,
    AmbiguousObservation,
    StaleObservation,
}

/// Mirrors `ProjectionResult<Budget>` bespoke sum-type in `v3.std.lens_application`.
#[derive(Clone, Debug)]
pub enum ProjectionResult<Budget> {
    ProjectedBudget { value: Budget },
    ProjectionFailed { failure: ProjectionFailure },
}

/// Mirrors `LensEnforcement<Output, Budget>` in `v3.std.lens_application`.
#[derive(Clone)]
pub struct LensEnforcement<Output, Budget> {
    pub project: Rc<dyn Fn(Output) -> ProjectionResult<Budget>>,
    pub violates: Rc<dyn Fn(Budget, Budget) -> bool>,
}

/// Mirrors `EnforceableLens<Output, Budget>` in `v3.std.lens_application`.
#[derive(Clone)]
pub struct EnforceableLens<Output, Budget> {
    pub lens: Lens<Output>,
    pub enforcement: LensEnforcement<Output, Budget>,
}
