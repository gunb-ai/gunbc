//! Python stub backend for test rendering.
//!
//! This stub exists at Phase 0 to validate the `TestRenderer` trait surface.
//! Every method compiles but panics at runtime. If a method signature can't
//! be cleanly implemented for Python, the abstraction is wrong.

use super::render::TestRenderer;
use super::test_ir::*;
use gunbc_ir::ValueExpr;

pub struct PythonRenderer;

impl TestRenderer for PythonRenderer {
    fn extension(&self) -> &str {
        "py"
    }

    fn render_value(&self, _expr: &ValueExpr) -> String {
        todo!("Python value rendering not yet implemented")
    }

    fn render_file(&self, _file: &TestFile) -> String {
        todo!("Python file rendering not yet implemented")
    }

    fn render_expr(&self, _expr: &Expr) -> String {
        todo!("Python expression rendering not yet implemented")
    }

    fn render_stmt(&self, _stmt: &Stmt, _indent: usize) -> String {
        todo!("Python statement rendering not yet implemented")
    }

    fn render_assert(&self, _assert: &Assert, _indent: usize) -> String {
        todo!("Python assertion rendering not yet implemented")
    }

    fn render_import(&self, _import: &Import) -> String {
        todo!("Python import rendering not yet implemented")
    }
}
