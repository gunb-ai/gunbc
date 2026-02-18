//! AbstractIR → Go (ManagedIR) lowering.
//!
//! Adds Go-specific constructs: multi-return errors, short declarations,
//! goroutines, package/import management, go.mod generation.
//!
//! **Owned by**: Task 10 (dsl-codegen-tasks.md)
