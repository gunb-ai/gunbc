//! v2 DAG compiler — generated from .dag source files.

#![allow(unused_imports, unused_variables, unused_mut, dead_code, unreachable_patterns, suspicious_double_ref_op, non_shorthand_field_patterns, clippy::all)]

pub mod rust_emit;
pub mod python_emit;
pub mod go_emit;
pub mod v2_core;
pub mod tokenize;
pub mod parse;
pub mod resolve;
pub mod normalize;
pub mod infer_types;
pub mod infer_env;
pub mod infer_method;
pub mod infer_cycle;
pub mod infer_resolve;
pub mod infer_emit_info;
pub mod infer_sigs;
pub mod infer;
pub mod languages;
pub mod emit;
pub mod emit_rust;
pub mod emit_python;
pub mod emit_go;
pub mod compile;
pub mod complexity;
pub mod ownership;
pub mod artifact;
pub mod runtime_rust;
pub mod v2_rt;

#[cfg(test)]
mod generated_tests;
