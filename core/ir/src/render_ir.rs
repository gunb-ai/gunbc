//! Unified Emission Model — Content IR + OutputMedium + Domain Renderer Stubs.
//!
//! Pure types (`SpanStyle`, `Span`, `Line`, `Frame`, `CursorAction`,
//! `RenderMode`, `ViewportUnit`, `Viewport`) are DSL-generated — see
//! `dsl/std/render.dag`.  This file provides runtime glue: constructors,
//! traits, medium implementations, and the document layer.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::symbols::{SemanticColor, SymbolSet, Tier};

// ---------------------------------------------------------------------------
// Re-exports from generated code (DSL is the source of truth)
// ---------------------------------------------------------------------------

pub use crate::generated::{CursorAction, Frame, Line, RenderMode, Span, SpanStyle, Viewport, ViewportUnit};

// ---------------------------------------------------------------------------
// Default impl for SpanStyle (cannot be derived in generated code)
// ---------------------------------------------------------------------------

impl Default for SpanStyle {
    fn default() -> Self {
        Self {
            color: None,
            bold: false,
            italic: false,
            symbol: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

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

impl Line {
    pub fn new(spans: Vec<Span>) -> Self {
        Self {
            spans,
            indent: 0,
            max_width: None,
        }
    }

    pub fn indented(spans: Vec<Span>, indent: usize) -> Self {
        Self {
            spans,
            indent: indent as i64,
            max_width: None,
        }
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

pub trait OutputMedium {
    type Output;

    fn render_span(&self, span: &Span) -> Self::Output;
    fn render_line(&self, line: &Line) -> Self::Output;
    fn render_block(&self, block: &Block) -> Self::Output;
    fn compose(&self, parts: Vec<Self::Output>) -> Self::Output;
}

pub trait TextMedium: OutputMedium<Output = String> {}
pub trait GraphicsMedium: OutputMedium<Output = RenderSurface> {}

// ---------------------------------------------------------------------------
// Text medium implementations
// ---------------------------------------------------------------------------

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
        let indent = "    ".repeat(line.indent.max(0) as usize);
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
        let indent = "    ".repeat(line.indent.max(0) as usize);
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
        let indent = "    ".repeat(line.indent.max(0) as usize);
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

#[derive(Debug, Clone, Default)]
pub struct RenderSurface {
    pub elements: Vec<GraphicsElement>,
}

#[derive(Debug, Clone)]
pub enum GraphicsElement {
    Glyph { x: f64, y: f64, text: String },
    Rect { x: f64, y: f64, width: f64, height: f64 },
    Path { points: Vec<(f64, f64)> },
}

// ---------------------------------------------------------------------------
// Document layer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub generator_name: Cow<'static, str>,
    pub regenerate_command: Cow<'static, str>,
    pub comment_prefix: Cow<'static, str>,
}

impl FileHeader {
    pub fn render(&self) -> String {
        crate::language::traits::comment::generated_header(
            &self.generator_name,
            &self.regenerate_command,
            &self.comment_prefix,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub header: Option<FileHeader>,
    pub body: DocumentBody,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub name: Cow<'static, str>,
    pub deps: Vec<Cow<'static, str>>,
    pub body: Vec<Cow<'static, str>>,
    pub comment: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub name: Cow<'static, str>,
    pub source: Option<Cow<'static, str>>,
    pub items: Vec<Cow<'static, str>>,
    pub rationale: Option<Cow<'static, str>>,
}

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

#[derive(Debug, Clone)]
pub enum MarkupNode {
    Heading { level: u8, text: String },
    Paragraph(Vec<Span>),
    List { ordered: bool, items: Vec<Vec<Span>> },
    CodeBlock { language: Option<String>, code: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    ThematicBreak,
    BlockQuote(Vec<MarkupNode>),
}

// ---------------------------------------------------------------------------
// Data IR
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<DataValue>),
    Map(BTreeMap<String, DataValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataNode {
    pub value: DataValue,
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// Domain renderer traits
// ---------------------------------------------------------------------------

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

pub trait MarkupRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_node(&self, node: &MarkupNode) -> M::Output;
    fn render_document(&self, nodes: &[MarkupNode]) -> M::Output;
}

pub trait StructuredRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_target(&self, target: &Target) -> M::Output;
    fn render_category(&self, category: &Category) -> M::Output;
    fn render_block(&self, block: &StructuredBlock) -> M::Output;
}

pub trait FrameRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_frame(&mut self, frame: &Frame, sink: &mut dyn std::io::Write)
        -> std::io::Result<()>;
}

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
    fn ansi_text_span_round_trip() {
        let medium = AnsiText {
            tier: Tier::Emoji,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(rendered.contains("\x1b[1m"));
        assert!(rendered.contains("\x1b[38;5;34m"));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("\x1b[0m"));
    }

    #[test]
    fn plain_text_span_no_escapes() {
        let medium = PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(!rendered.contains("\x1b"));
        assert_eq!(rendered, "hello");
    }

    #[test]
    fn html_text_span_css_class() {
        let medium = HtmlText {
            tier: Tier::Emoji,
            symbol_set: &STANDARD,
        };
        let rendered = medium.render_span(&sample_span());
        assert!(rendered.contains("sym-success"));
        assert!(rendered.contains("bold"));
        assert!(rendered.contains("<span"));
    }

    #[test]
    fn line_indent() {
        let medium = PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        };
        let line = Line::indented(vec![Span::plain("text")], 2);
        let rendered = medium.render_line(&line);
        assert!(rendered.starts_with("        "));
        assert!(rendered.ends_with("text"));
    }

    #[test]
    fn block_join() {
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
    fn data_value_nested() {
        let mut inner = BTreeMap::new();
        inner.insert(
            "list".to_string(),
            DataValue::List(vec![DataValue::Int(1), DataValue::Int(2)]),
        );
        inner.insert("flag".to_string(), DataValue::Bool(true));
        let map = DataValue::Map(inner);
        match &map {
            DataValue::Map(m) => assert_eq!(m.len(), 2),
            _ => panic!("expected Map"),
        }
    }
}
