//! Pure expression evaluator for lowered DAG IR.
//!
//! Evaluates `LoweredFnBody` and collection operations using only `Value`
//! types from `gunbc-ir`. No side effects, no transport, no I/O.

pub mod eval;
pub mod expr;

pub use eval::*;
pub use expr::*;

#[cfg(test)]
mod v2_tests;
