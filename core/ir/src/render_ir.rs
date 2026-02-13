//! Unified Emission Model — Phase 1: Content IR + OutputMedium + Domain Renderer Stubs.
//!
//! This module defines the medium-agnostic content IR primitives and the
//! `OutputMedium` trait hierarchy that will unify all rendering systems.
//!
//! # Architecture
//!
//! - **Content IR**: `Span`, `Line`, `Block` — medium-agnostic content
//! - **OutputMedium**: Root trait with `render_span`/`render_line`/`render_block`/`compose`
//! - **TextMedium**: Marker for string-producing media (`AnsiText`, `PlainText`, `HtmlText`)
//! - **GraphicsMedium**: Marker for graphics-producing media (stubs only in Phase 1)
//! - **Domain renderers**: `CodeRenderer`, `MarkupRenderer`, `StructuredRenderer`,
//!   `FrameRenderer`, `DocumentRenderer` (trait definitions only, no impls)
//! - **Document layer**: `Document`, `FileHeader`, `Frame`, `CursorAction`
//! - **Data IR**: `DataValue`, `DataNode` for structured format sharing

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::symbols::{SemanticColor, SymbolId, SymbolSet, Tier};

// ---------------------------------------------------------------------------
// Content IR primitives
// ---------------------------------------------------------------------------

/// Visual style applied to a span of text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpanStyle {
    pub color: Option<SemanticColor>,
    pub bold: bool,
    pub italic: bool,
    /// When present, the symbol is rendered before the span text.
    pub symbol: Option<SymbolId>,
}

/// A styled fragment of text — the atomic unit of content IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: SpanStyle,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: SpanStyle::default(),
        }
    }

    pub fn styled(text: impl Into<String>, style: SpanStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

/// A line of spans with an indentation level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub spans: Vec<Span>,
    pub indent: usize,
}

impl Line {
    pub fn new(spans: Vec<Span>) -> Self {
        Self { spans, indent: 0 }
    }

    pub fn indented(spans: Vec<Span>, indent: usize) -> Self {
        Self { spans, indent }
    }
}

/// A block of lines — the basic content unit for rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub lines: Vec<Line>,
}

impl Block {
    pub fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
}

// ---------------------------------------------------------------------------
// OutputMedium hierarchy
// ---------------------------------------------------------------------------

/// Root trait for output media. The medium owns rendering context (tier, symbol set)
/// and converts content IR into a concrete output type.
pub trait OutputMedium {
    type Output;

    fn render_span(&self, span: &Span) -> Self::Output;
    fn render_line(&self, line: &Line) -> Self::Output;
    fn render_block(&self, block: &Block) -> Self::Output;
    fn compose(&self, parts: Vec<Self::Output>) -> Self::Output;
}

/// Marker trait for text-producing media (output is `String`).
pub trait TextMedium: OutputMedium<Output = String> {}

/// Marker trait for graphics-producing media (output is `RenderSurface`).
pub trait GraphicsMedium: OutputMedium<Output = RenderSurface> {}

// ---------------------------------------------------------------------------
// Text medium implementations
// ---------------------------------------------------------------------------

/// ANSI terminal output with color codes and symbol resolution.
pub struct AnsiText {
    pub tier: Tier,
    pub symbol_set: &'static SymbolSet,
}

impl OutputMedium for AnsiText {
    type Output = String;

    fn render_span(&self, span: &Span) -> String {
        let mut out = String::new();
        let needs_reset = span.style.color.is_some() || span.style.bold || span.style.italic;

        if span.style.bold {
            out.push_str("\x1b[1m");
        }
        if span.style.italic {
            out.push_str("\x1b[3m");
        }
        if let Some(color) = span.style.color {
            out.push_str(color.ansi());
        }
        if let Some(sym_id) = span.style.symbol {
            out.push_str(self.symbol_set.resolve_tier(sym_id, self.tier));
        }
        out.push_str(&span.text);
        if needs_reset {
            out.push_str(SemanticColor::reset());
        }
        out
    }

    fn render_line(&self, line: &Line) -> String {
        let indent = "    ".repeat(line.indent);
        let content: String = line.spans.iter().map(|s| self.render_span(s)).collect();
        format!("{indent}{content}")
    }

    fn render_block(&self, block: &Block) -> String {
        block
            .lines
            .iter()
            .map(|l| self.render_line(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compose(&self, parts: Vec<String>) -> String {
        parts.join("")
    }
}

impl TextMedium for AnsiText {}

/// Plain text output — no ANSI escapes, no colors. Symbols resolved at configured tier.
pub struct PlainText {
    pub tier: Tier,
    pub symbol_set: &'static SymbolSet,
}

impl OutputMedium for PlainText {
    type Output = String;

    fn render_span(&self, span: &Span) -> String {
        let mut out = String::new();
        if let Some(sym_id) = span.style.symbol {
            out.push_str(self.symbol_set.resolve_tier(sym_id, self.tier));
        }
        out.push_str(&span.text);
        out
    }

    fn render_line(&self, line: &Line) -> String {
        let indent = "    ".repeat(line.indent);
        let content: String = line.spans.iter().map(|s| self.render_span(s)).collect();
        format!("{indent}{content}")
    }

    fn render_block(&self, block: &Block) -> String {
        block
            .lines
            .iter()
            .map(|l| self.render_line(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compose(&self, parts: Vec<String>) -> String {
        parts.join("")
    }
}

impl TextMedium for PlainText {}

/// HTML output — wraps styled spans in `<span class="...">` elements.
pub struct HtmlText {
    pub tier: Tier,
    pub symbol_set: &'static SymbolSet,
}

impl OutputMedium for HtmlText {
    type Output = String;

    fn render_span(&self, span: &Span) -> String {
        let mut classes = Vec::new();
        if let Some(color) = span.style.color {
            classes.push(color.css_class());
        }
        if span.style.bold {
            classes.push("bold");
        }
        if span.style.italic {
            classes.push("italic");
        }

        let mut content = String::new();
        if let Some(sym_id) = span.style.symbol {
            content.push_str(self.symbol_set.resolve_tier(sym_id, self.tier));
        }
        content.push_str(&span.text);

        if classes.is_empty() {
            content
        } else {
            format!("<span class=\"{}\">{content}</span>", classes.join(" "))
        }
    }

    fn render_line(&self, line: &Line) -> String {
        let indent = "    ".repeat(line.indent);
        let content: String = line.spans.iter().map(|s| self.render_span(s)).collect();
        format!("{indent}{content}")
    }

    fn render_block(&self, block: &Block) -> String {
        block
            .lines
            .iter()
            .map(|l| self.render_line(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compose(&self, parts: Vec<String>) -> String {
        parts.join("")
    }
}

impl TextMedium for HtmlText {}

// ---------------------------------------------------------------------------
// Graphics stubs
// ---------------------------------------------------------------------------

/// A rendered graphics surface (stub — no real implementation in Phase 1).
#[derive(Debug, Clone, Default)]
pub struct RenderSurface {
    pub elements: Vec<GraphicsElement>,
}

/// A graphics primitive (stub — no real implementation in Phase 1).
#[derive(Debug, Clone)]
pub enum GraphicsElement {
    Glyph {
        x: f64,
        y: f64,
        text: String,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Path {
        points: Vec<(f64, f64)>,
    },
}

// ---------------------------------------------------------------------------
// Document layer
// ---------------------------------------------------------------------------

/// Generated file header metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub generator_name: Cow<'static, str>,
    pub regenerate_command: Cow<'static, str>,
    pub comment_prefix: Cow<'static, str>,
}

impl FileHeader {
    /// Render the standard "Generated by" / "DO NOT EDIT" header.
    pub fn render(&self) -> String {
        crate::language::traits::comment::generated_header(
            &self.generator_name,
            &self.regenerate_command,
            &self.comment_prefix,
        )
    }
}

/// What a frame does to existing output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorAction {
    Overwrite,
    Append,
    Clear,
}

/// A single frame of streaming output.
#[derive(Debug, Clone)]
pub struct Frame {
    pub lines: Vec<Line>,
    pub cursor_action: CursorAction,
}

/// A complete generated document.
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub header: Option<FileHeader>,
    pub body: DocumentBody,
}

/// The body of a generated document.
#[derive(Debug, Clone)]
pub enum DocumentBody {
    Code(crate::code_ir::SourceFile),
    Markup(Vec<MarkupNode>),
    Structured(Vec<StructuredBlock>),
    Frames(Vec<Frame>),
    Data(DataValue),
    Raw(String),
}

// ---------------------------------------------------------------------------
// Structured layer IR types
// ---------------------------------------------------------------------------

/// A build target (e.g., Makefile target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub name: Cow<'static, str>,
    pub deps: Vec<Cow<'static, str>>,
    pub body: Vec<Cow<'static, str>>,
    /// Optional comment line(s) rendered above the target definition.
    pub comment: Option<Cow<'static, str>>,
}

/// A categorized group of items (e.g., review category).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub name: Cow<'static, str>,
    pub source: Option<Cow<'static, str>>,
    pub items: Vec<Cow<'static, str>>,
    pub rationale: Option<Cow<'static, str>>,
}

/// A block in structured output.
#[derive(Debug, Clone)]
pub enum StructuredBlock {
    Target(Target),
    Category(Category),
    Section { title: String, content: Block },
    Content(Block),
    Blank,
    Raw(String),
}

// ---------------------------------------------------------------------------
// Markup layer IR types
// ---------------------------------------------------------------------------

/// A node in markup output (e.g., Markdown/HTML).
#[derive(Debug, Clone)]
pub enum MarkupNode {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(Vec<Span>),
    List {
        ordered: bool,
        items: Vec<Vec<Span>>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    ThematicBreak,
    BlockQuote(Vec<MarkupNode>),
}

// ---------------------------------------------------------------------------
// Data IR
// ---------------------------------------------------------------------------

/// A structured data value for format-agnostic data sharing (JSON, TOML, YAML, etc.).
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<DataValue>),
    Map(BTreeMap<String, DataValue>),
}

/// A data value with an optional comment annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct DataNode {
    pub value: DataValue,
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// Domain renderer traits (definitions only — no impls in Phase 1)
// ---------------------------------------------------------------------------

/// Renders code constructs (files, expressions, statements, imports).
pub trait CodeRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_value(&self, expr: &crate::ValueExpr) -> M::Output;
    fn render_file(&self, file: &crate::code_ir::TestFile) -> M::Output;
    fn render_source_file(&self, file: &crate::code_ir::SourceFile) -> M::Output;
    fn render_expr(&self, expr: &crate::code_ir::Expr) -> M::Output;
    fn render_stmt(&self, stmt: &crate::code_ir::Stmt, indent: usize) -> M::Output;
    fn render_assert(&self, assert: &crate::code_ir::Assert, indent: usize) -> M::Output;
    fn render_import(&self, import: &crate::code_ir::Import) -> M::Output;
    fn render_item(&self, item: &crate::code_ir::Item, indent: usize) -> M::Output;
}

/// Renders markup content (headings, paragraphs, lists, code blocks).
pub trait MarkupRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_node(&self, node: &MarkupNode) -> M::Output;
    fn render_document(&self, nodes: &[MarkupNode]) -> M::Output;
}

/// Renders structured content (targets, categories, sections).
pub trait StructuredRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_target(&self, target: &Target) -> M::Output;
    fn render_category(&self, category: &Category) -> M::Output;
    fn render_block(&self, block: &StructuredBlock) -> M::Output;
}

/// Renders streaming frames to an output sink.
pub trait FrameRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_frame(&mut self, frame: &Frame, sink: &mut dyn std::io::Write)
        -> std::io::Result<()>;
}

/// Renders a complete document.
pub trait DocumentRenderer<M: OutputMedium> {
    fn render(&self, document: &Document) -> M::Output;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::STANDARD;

    fn sample_span() -> Span {
        Span::styled(
            "hello",
            SpanStyle {
                color: Some(SemanticColor::Success),
                bold: true,
                italic: false,
                symbol: None,
            },
        )
    }

    #[test]
    fn test_ansi_text_span_round_trip() {
        let medium = AnsiText {
            tier: Tier::Emoji,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(rendered.contains("\x1b[1m"), "should contain bold code");
        assert!(
            rendered.contains("\x1b[32m"),
            "should contain green (success) code"
        );
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("\x1b[0m"), "should contain reset code");
    }

    #[test]
    fn test_plain_text_span_no_escapes() {
        let medium = PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(
            !rendered.contains("\x1b"),
            "should not contain ANSI escapes"
        );
        assert_eq!(rendered, "hello");
    }

    #[test]
    fn test_html_text_span_css_class() {
        let medium = HtmlText {
            tier: Tier::Emoji,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(rendered.contains("sym-success"), "should contain CSS class");
        assert!(rendered.contains("bold"), "should contain bold class");
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("<span"), "should be wrapped in span tag");
    }

    #[test]
    fn test_symbol_resolution() {
        let medium = AnsiText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let span = Span::styled(
            "ok",
            SpanStyle {
                color: None,
                bold: false,
                italic: false,
                symbol: Some(SymbolId::Success),
            },
        );
        let rendered = medium.render_span(&span);
        assert!(
            rendered.contains("OK"),
            "should resolve Success symbol at ASCII tier"
        );
        assert!(rendered.contains("ok"));
    }

    #[test]
    fn test_line_indent() {
        let medium = PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let line = Line::indented(vec![Span::plain("text")], 2);
        let rendered = medium.render_line(&line);
        assert!(
            rendered.starts_with("        "),
            "indent=2 should produce 8 spaces"
        );
        assert!(rendered.ends_with("text"));
    }

    #[test]
    fn test_block_join() {
        let medium = PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let block = Block::new(vec![
            Line::new(vec![Span::plain("line1")]),
            Line::new(vec![Span::plain("line2")]),
        ]);
        let rendered = medium.render_block(&block);
        assert_eq!(rendered, "line1\nline2");
    }

    #[test]
    fn test_compose() {
        let medium = AnsiText {
            tier: Tier::Emoji,
            symbol_set: &STANDARD,
        };
        let result = medium.compose(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_graphics_stubs_exist() {
        let surface = RenderSurface::default();
        assert!(surface.elements.is_empty());

        let _glyph = GraphicsElement::Glyph {
            x: 0.0,
            y: 0.0,
            text: "A".to_string(),
        };
        let _rect = GraphicsElement::Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let _path = GraphicsElement::Path {
            points: vec![(0.0, 0.0), (1.0, 1.0)],
        };
    }

    #[test]
    fn test_graphics_medium_trait_bound() {
        fn _check<M: GraphicsMedium>() {}
        // Compiles — that's the test.
    }

    #[test]
    fn test_domain_traits_generic() {
        fn _check_code<M: OutputMedium, R: CodeRenderer<M>>() {}
        fn _check_markup<M: OutputMedium, R: MarkupRenderer<M>>() {}
        fn _check_structured<M: OutputMedium, R: StructuredRenderer<M>>() {}
        fn _check_frame<M: OutputMedium, R: FrameRenderer<M>>() {}
        fn _check_document<M: OutputMedium, R: DocumentRenderer<M>>() {}
        // All compile — that's the test.
    }

    #[test]
    fn test_document_construction() {
        let doc = Document {
            path: PathBuf::from("output/test.rs"),
            header: Some(FileHeader {
                generator_name: Cow::Borrowed("gunbc-test"),
                regenerate_command: Cow::Borrowed("make test"),
                comment_prefix: Cow::Borrowed("//"),
            }),
            body: DocumentBody::Raw("fn main() {}".to_string()),
        };
        assert_eq!(doc.path, PathBuf::from("output/test.rs"));
        assert!(doc.header.is_some());
        matches!(doc.body, DocumentBody::Raw(_));
    }

    #[test]
    fn test_data_value_nested() {
        let mut inner = BTreeMap::new();
        inner.insert(
            "list".to_string(),
            DataValue::List(vec![DataValue::Int(1), DataValue::Int(2)]),
        );
        inner.insert("flag".to_string(), DataValue::Bool(true));
        inner.insert("name".to_string(), DataValue::Str("test".to_string()));

        let map = DataValue::Map(inner);
        match &map {
            DataValue::Map(m) => {
                assert_eq!(m.len(), 3);
                assert_eq!(m.get("flag"), Some(&DataValue::Bool(true)));
                match m.get("list") {
                    Some(DataValue::List(items)) => assert_eq!(items.len(), 2),
                    other => panic!("expected List, got {:?}", other),
                }
            }
            _ => panic!("expected Map"),
        }
    }
}
