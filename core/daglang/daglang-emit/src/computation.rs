//! Target-independent computation model.
//!
//! Describes *what* each DAG node does, not *how* it's expressed in any language.
//! Every codegen backend consumes `Computation`, not `LoweredOp` directly.
//!
//! **Owned by**: Task 1 (dsl-codegen-tasks.md)
