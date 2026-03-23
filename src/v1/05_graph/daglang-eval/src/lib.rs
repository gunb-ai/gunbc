//! LEGACY (bootstrap-only): Pure expression evaluator for lowered DAG IR.
//!
//! The v2 self-hosted compiler pipeline is now authoritative. This crate is
//! retained only for bootstrapping the v2 compiler from `.dag` sources.
//!
//! Evaluates `LoweredFnBody` and collection operations using only `Value`
//! types from `gunbc-ir`. No side effects, no transport, no I/O.

pub mod eval;
pub mod eval_core;
pub mod eval_stack;
pub mod expr;

pub use eval::*;
pub use eval_core::*;
pub use eval_stack::{evaluate_stack_with_diagnostics, EvalOutcome};
pub use expr::*;

#[cfg(test)]
mod v2_tests;
