//! Test rendering trait and re-export of `CodeRenderer`.

pub use gunbc_ir::render_ir::CodeRenderer;

use super::test_ir::{Assert, Expr, Import, Stmt, TestFile};
use gunbc_ir::ValueExpr;

/// Trait for rendering test IR to source text.
pub trait TestRenderer {
    /// File extension for this language (e.g., "rs", "py", "ts").
    fn extension(&self) -> &str;

    /// Render a `ValueExpr` to source text.
    fn render_value(&self, expr: &ValueExpr) -> String;

    /// Render a complete `TestFile` to source text.
    fn render_file(&self, file: &TestFile) -> String;

    /// Render an `Expr` to source text.
    fn render_expr(&self, expr: &Expr) -> String;

    /// Render a `Stmt` to source text at the given indentation level.
    fn render_stmt(&self, stmt: &Stmt, indent: usize) -> String;

    /// Render an `Assert` to source text at the given indentation level.
    fn render_assert(&self, assert: &Assert, indent: usize) -> String;

    /// Render an `Import` to source text.
    fn render_import(&self, import: &Import) -> String;
}
