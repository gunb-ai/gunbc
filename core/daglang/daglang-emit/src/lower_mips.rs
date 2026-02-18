//! CStyleIR → MIPS (RegisterIR) lowering.
//!
//! Lowers C-level constructs to MIPS register-level instructions: register allocation,
//! stack frame layout, calling conventions, syscall sequences.
//!
//! **Owned by**: Task 15 (dsl-codegen-tasks.md)
