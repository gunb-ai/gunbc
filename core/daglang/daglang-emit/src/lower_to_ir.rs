//! Computation → AbstractIR lowering.
//!
//! Converts an `EmitPlan` into target-agnostic `code_ir` constructs (Stmt, Expr, Item).
//! The bridge from "what" (Computation) to "how" (code structure) — still no
//! language-specific features.
//!
//! **Owned by**: Task 8 (dsl-codegen-tasks.md)
