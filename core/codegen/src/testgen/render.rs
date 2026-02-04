//! Rendering trait for converting test IR to target-language source text.
//!
//! Each backend (Rust, Python, TypeScript) implements `TestRenderer` to
//! produce syntactically valid source for that language. The trait surface
//! is validated by language stubs at Phase 0 — if a method signature
//! can't be cleanly implemented for a language, the abstraction is wrong.

use super::test_ir::*;
use gunbc_ir::ValueExpr;

/// Render a `TestFile` to source text in a target language.
pub trait TestRenderer {
    /// File extension for the generated file (e.g., "rs", "py", "ts").
    fn extension(&self) -> &str;

    /// Render a value literal to source text.
    ///
    /// This is the core function that replaces `value_to_rust_literal` and
    /// `value_to_code`. It must handle every `ValueExpr` variant — the
    /// compiler enforces this since `ValueExpr` has no catch-all.
    fn render_value(&self, expr: &ValueExpr) -> String;

    /// Render a full test file to source text.
    fn render_file(&self, file: &TestFile) -> String;

    /// Render a single expression to source text.
    fn render_expr(&self, expr: &Expr) -> String;

    /// Render a single statement to source text.
    fn render_stmt(&self, stmt: &Stmt, indent: usize) -> String;

    /// Render an assertion to source text.
    fn render_assert(&self, assert: &Assert, indent: usize) -> String;

    /// Render an import to source text.
    fn render_import(&self, import: &Import) -> String;
}
