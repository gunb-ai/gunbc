//! AbstractIR → C (CStyleIR) lowering.
//!
//! Adds C-specific constructs: explicit memory management, tagged union Value types,
//! function pointers, arena allocation, char*/length string handling.
//!
//! **Owned by**: Task 11 (dsl-codegen-tasks.md)
