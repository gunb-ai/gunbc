//! Makefile-flavored `StructuredRenderer` implementation.
//!
//! Renders `StructuredBlock` IR to Makefile syntax:
//! - Targets use tab-indented recipe lines
//! - Categories use `#` comment headers with provenance
//! - Sections use `#` banner comments

use crate::render_ir::{Category, StructuredBlock, StructuredRenderer, Target, TextMedium};

/// Renders structured IR to Makefile-compatible text.
///
/// Uses `\t` for recipe indentation (Makefile requirement) and `#` for comments.
pub struct MakefileStructuredRenderer<M> {
    medium: M,
}

impl<M: TextMedium> MakefileStructuredRenderer<M> {
    pub fn new(medium: M) -> Self {
        Self { medium }
    }
}

impl<M: TextMedium> StructuredRenderer<M> for MakefileStructuredRenderer<M> {
    fn medium(&self) -> &M {
        &self.medium
    }

    fn render_target(&self, target: &Target) -> String {
        let mut out = String::new();
        // # comment
        if let Some(ref comment) = target.comment {
            out.push_str("# ");
            out.push_str(comment);
            out.push('\n');
        }
        // target: dep1 dep2
        out.push_str(&target.name);
        out.push(':');
        for dep in &target.deps {
            out.push(' ');
            out.push_str(dep);
        }
        out.push('\n');
        // \trecipe1
        // \trecipe2
        for line in &target.body {
            out.push('\t');
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
        out
    }

    fn render_category(&self, category: &Category) -> String {
        let mut out = String::new();
        let source = category.source.as_deref().unwrap_or("unknown");
        out.push_str(&format!(
            "# --- {} (from {}) ---\n",
            category.name, source
        ));
        if let Some(ref rationale) = category.rationale {
            out.push_str(&format!("# {}\n", rationale));
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
                out.push_str(&format!("# {}\n", title));
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
    use crate::render_ir::PlainText;
    use crate::symbols::{Tier, STANDARD};

    fn plain() -> PlainText {
        PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        }
    }

    #[test]
    fn test_render_target_no_deps() {
        let r = MakefileStructuredRenderer::new(plain());
        let target = Target {
            name: "clean".to_string(),
            deps: vec![],
            body: vec!["@cargo clean".to_string()],
            comment: None,
        };
        assert_eq!(r.render_target(&target), "clean:\n\t@cargo clean\n\n");
    }

    #[test]
    fn test_render_target_with_deps() {
        let r = MakefileStructuredRenderer::new(plain());
        let target = Target {
            name: "build".to_string(),
            deps: vec!["codegen".to_string(), "testgen".to_string()],
            body: vec!["@cargo build".to_string()],
            comment: None,
        };
        assert_eq!(
            r.render_target(&target),
            "build: codegen testgen\n\t@cargo build\n\n"
        );
    }

    #[test]
    fn test_render_category() {
        let r = MakefileStructuredRenderer::new(plain());
        let cat = Category {
            name: "Build artifacts".to_string(),
            source: Some("cargo".to_string()),
            items: vec!["/target/".to_string()],
            rationale: Some("Reproducible".to_string()),
        };
        let out = r.render_category(&cat);
        assert!(out.contains("# --- Build artifacts (from cargo) ---"));
        assert!(out.contains("# Reproducible"));
        assert!(out.contains("/target/"));
    }

    #[test]
    fn test_render_target_with_comment() {
        let r = MakefileStructuredRenderer::new(plain());
        let target = Target {
            name: "build".to_string(),
            deps: vec!["codegen".to_string()],
            body: vec!["@cargo build".to_string()],
            comment: Some("Full build transaction".to_string()),
        };
        assert_eq!(
            r.render_target(&target),
            "# Full build transaction\nbuild: codegen\n\t@cargo build\n\n"
        );
    }

    #[test]
    fn test_render_blank() {
        let r = MakefileStructuredRenderer::new(plain());
        assert_eq!(r.render_block(&StructuredBlock::Blank), "\n");
    }

    #[test]
    fn test_render_raw() {
        let r = MakefileStructuredRenderer::new(plain());
        assert_eq!(
            r.render_block(&StructuredBlock::Raw(".PHONY: all\n".to_string())),
            ".PHONY: all\n"
        );
    }
}
