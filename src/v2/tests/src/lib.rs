//! Integration tests for the v2 self-hosted compiler.
//!
//! Tests call stage0 functions directly — no v1 interpreter, no Value wrapping.
//! Stage0 is a Rust crate generated from .dag source files by the v1 emitter.

#![allow(
    clippy::disallowed_macros,
    clippy::absurd_extreme_comparisons,
    dead_code
)]

pub mod helpers;

#[cfg(test)]
mod bootstrap;
#[cfg(test)]
mod bug_sentinel_ratchet;
#[cfg(test)]
mod derive_bound_fail_closed_test;
#[cfg(test)]
mod diagnostics;
#[cfg(test)]
mod effects;
#[cfg(test)]
mod infer_semantics;
#[cfg(test)]
mod int_pow_bounded_test;
#[cfg(test)]
mod parse;
#[cfg(test)]
mod peano_materialization_cap_test;
#[cfg(test)]
mod pipeline;
#[cfg(test)]
mod render_repeat_test;
#[cfg(test)]
mod source_audit;
#[cfg(test)]
mod sub_value_lattice_factor_test;
