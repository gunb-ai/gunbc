//! Pure expression evaluator for lowered DAG IR.
//!
//! Evaluates `LoweredFnBody` and collection operations using only `Value`
//! types from `gunbc-ir`. No side effects, no transport, no I/O.

pub mod eval;
pub mod eval_core;
pub mod eval_stack;
pub mod expr;

pub use eval::*;
pub use eval_core::*;
pub use eval_stack::{take_type_warnings, EvalOutcome, evaluate_stack_with_diagnostics};
pub use expr::*;

#[cfg(test)]
mod v2_tests;
