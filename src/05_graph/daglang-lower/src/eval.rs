//! Re-exports from `daglang-eval` for backward compatibility.
//!
//! All evaluation functions and types now live in `daglang-eval`.
//! This module re-exports them so existing `daglang_lower::eval::*`
//! imports continue to work.

pub use daglang_eval::eval::*;
