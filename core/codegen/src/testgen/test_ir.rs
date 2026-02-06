//! Re-exports of Code IR types from `gunbc_ir::code_ir`.
//!
//! These types were moved to `gunbc-ir` in Phase 2. This module
//! re-exports them for backward compatibility within the testgen module.

pub use gunbc_ir::code_ir::{
    Assert, EnumDef, Expr, FnDef, HelperFn, ImplBlock, Import, Item, MatchArm, SourceFile, Stmt,
    StructDef, TestFile, TestFn, TestSection,
};
