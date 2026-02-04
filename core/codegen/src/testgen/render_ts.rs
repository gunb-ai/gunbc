//! TypeScript stub backend for test rendering.
//!
//! This stub exists at Phase 0 to validate the `TestRenderer` trait surface.
//! Every method compiles but panics at runtime. If a method signature can't
//! be cleanly implemented for TypeScript, the abstraction is wrong.

use super::render::TestRenderer;
use super::test_ir::*;
use gunbc_ir::ValueExpr;

pub struct TypeScriptRenderer;

impl TestRenderer for TypeScriptRenderer {
    fn extension(&self) -> &str {
        "ts"
    }

    fn render_value(&self, _expr: &ValueExpr) -> String {
        todo!("TypeScript value rendering not yet implemented")
    }

    fn render_file(&self, _file: &TestFile) -> String {
        todo!("TypeScript file rendering not yet implemented")
    }

    fn render_expr(&self, _expr: &Expr) -> String {
        todo!("TypeScript expression rendering not yet implemented")
    }

    fn render_stmt(&self, _stmt: &Stmt, _indent: usize) -> String {
        todo!("TypeScript statement rendering not yet implemented")
    }

    fn render_assert(&self, _assert: &Assert, _indent: usize) -> String {
        todo!("TypeScript assertion rendering not yet implemented")
    }

    fn render_import(&self, _import: &Import) -> String {
        todo!("TypeScript import rendering not yet implemented")
    }
}
