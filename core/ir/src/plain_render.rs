//! Plain-text `StructuredRenderer` implementation.
//!
//! Renders `StructuredBlock` IR to plain text:
//! - Sections use `===` / `---` separators
//! - Categories use `---` headers
//! - Targets rendered as labeled blocks (not Makefile syntax)

use crate::render_ir::{Category, StructuredBlock, StructuredRenderer, Target, TextMedium};
use std::fmt::Write;

/// Renders structured IR to plain text (reports, pragma files, etc.).
pub struct PlainStructuredRenderer<M> {
    medium: M,
}

impl<M: TextMedium> PlainStructuredRenderer<M> {
    pub fn new(medium: M) -> Self {
        Self { medium }
    }
}

impl<M: TextMedium> StructuredRenderer<M> for PlainStructuredRenderer<M> {
    fn medium(&self) -> &M {
        &self.medium
    }

    fn render_target(&self, target: &Target) -> String {
        let mut out = String::new();
        out.push_str(&target.name);
        if !target.deps.is_empty() {
            out.push_str(": ");
            out.push_str(&target.deps.join(", "));
        }
        out.push('\n');
        for line in &target.body {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        out
    }

    fn render_category(&self, category: &Category) -> String {
        let mut out = String::new();
        let source = category.source.as_deref().unwrap_or("unknown");
        writeln!(out, "# --- {} (from {}) ---", category.name, source).unwrap();
        if let Some(ref rationale) = category.rationale {
            writeln!(out, "# {}", rationale).unwrap();
        }
        for item in &category.items {
            out.push_str(item);
            out.push('\n');
        }
        out.push('\n');
        out
    }

    fn render_block(&self, block: &StructuredBlock) -> String {
        match block {
            StructuredBlock::Target(t) => self.render_target(t),
            StructuredBlock::Category(c) => self.render_category(c),
            StructuredBlock::Section { title, content } => {
                let mut out = String::new();
                out.push_str(title);
                out.push('\n');
                let sep = "=".repeat(title.len());
                out.push_str(&sep);
                out.push('\n');
                out.push_str(&self.medium.render_block(content));
                out.push_str("\n\n");
                out
            }
            StructuredBlock::Content(block) => {
                let mut out = self.medium.render_block(block);
                out.push('\n');
                out
            }
            StructuredBlock::Blank => "\n".to_string(),
            StructuredBlock::Raw(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_ir::{Block, PlainText};
    use crate::symbols::{Tier, STANDARD};
    use std::borrow::Cow;

    fn plain() -> PlainText {
        PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        }
    }

    #[test]
    fn test_render_category() {
        let r = PlainStructuredRenderer::new(plain());
        let cat = Category {
            name: Cow::Borrowed("Crate exemptions"),
            source: Some(Cow::Borrowed("policy")),
            items: vec![Cow::Borrowed("core/codegen/src/lib.rs:1")],
            rationale: Some(Cow::Borrowed("Crate-level allowance")),
        };
        let out = r.render_category(&cat);
        assert!(out.contains("# --- Crate exemptions (from policy) ---"));
        assert!(out.contains("# Crate-level allowance"));
        assert!(out.contains("core/codegen/src/lib.rs:1"));
    }

    #[test]
    fn test_render_section() {
        use crate::render_ir::{Line, Span};
        let r = PlainStructuredRenderer::new(plain());
        let block = StructuredBlock::Section {
            title: "CI Report".to_string(),
            content: Block::new(vec![Line::new(vec![Span::plain("Build: PASS")])]),
        };
        let out = r.render_block(&block);
        assert!(out.contains("CI Report\n========="));
        assert!(out.contains("Build: PASS"));
    }

    #[test]
    fn test_render_blank() {
        let r = PlainStructuredRenderer::new(plain());
        assert_eq!(r.render_block(&StructuredBlock::Blank), "\n");
    }

    #[test]
    fn test_render_raw() {
        let r = PlainStructuredRenderer::new(plain());
        assert_eq!(
            r.render_block(&StructuredBlock::Raw("[section]\n".to_string())),
            "[section]\n"
        );
    }
}
