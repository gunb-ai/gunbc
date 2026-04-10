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
mod source_audit;
#[cfg(test)]
mod parse;
#[cfg(test)]
mod pipeline;
#[cfg(test)]
mod infer_semantics;
#[cfg(test)]
mod diagnostics;
#[cfg(test)]
mod bootstrap;
#[cfg(test)]
mod impossible_bugs;
