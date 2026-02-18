//! Rust exec-runtime codegen (Layer 1 fast path).
//!
//! Generates a Rust crate that builds `Dag<Op>` and calls `gunbc-exec` to run it.
//! This is the bootstrap path — bypasses the language DAG for immediate results.
//!
//! **Owned by**: Task 3 (dsl-codegen-tasks.md)
