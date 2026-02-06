//! TypeScript stub backend for code rendering.
//!
//! This stub exists to validate the `CodeRenderer` trait surface.
//! Every method compiles but panics at runtime. If a method signature can't
//! be cleanly implemented for TypeScript, the abstraction is wrong.

use gunbc_ir::code_ir::*;
use gunbc_ir::render_ir::{CodeRenderer, OutputMedium, TextMedium};
use gunbc_ir::ValueExpr;

pub struct TypeScriptCodeRenderer<M: OutputMedium> {
    medium: M,
}

impl<M: OutputMedium> TypeScriptCodeRenderer<M> {
    pub fn new(medium: M) -> Self {
        Self { medium }
    }
}

impl<M: TextMedium> CodeRenderer<M> for TypeScriptCodeRenderer<M> {
    fn medium(&self) -> &M {
        &self.medium
    }

    fn render_value(&self, _expr: &ValueExpr) -> String {
        todo!("TypeScript value rendering not yet implemented")
    }

    fn render_file(&self, _file: &TestFile) -> String {
        todo!("TypeScript file rendering not yet implemented")
    }

    fn render_source_file(&self, _file: &SourceFile) -> String {
        todo!("TypeScript source file rendering not yet implemented")
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

    fn render_item(&self, _item: &Item, _indent: usize) -> String {
        todo!("TypeScript item rendering not yet implemented")
    }
}
