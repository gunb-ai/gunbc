//! Generated from DSL type definitions. Do not edit.

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum SemanticColor {
    Default,
    Success,
    Warning,
    Error,
    Info,
    Dim,
    Active,
    Accent,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Tier {
    Emoji,
    Unicode,
    Ascii,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum SymbolId {
    NodePending,
    NodeRunning,
    NodeCompleted,
    NodeFailed,
    NodeSkipped,
    NodeIntercepted,
    EdgeIdle,
    EdgeFlowing,
    EdgeDone,
    EdgeDead,
    DagNotStarted,
    DagRunning,
    DagCompleted,
    DagFailed,
    BoundaryMarker,
    Spinner0,
    Spinner1,
    Spinner2,
    Spinner3,
    Spinner4,
    Spinner5,
    Spinner6,
    Spinner7,
    Spinner8,
    Spinner9,
    Success,
    Failure,
    Warning,
    Info,
    DataList,
    DataMap,
    DataSecret,
    DataUrl,
    DataTimer,
    ConnectorHorizontal,
    ConnectorVertical,
    ConnectorTeeDown,
    ConnectorTeeUp,
    ConnectorCornerBottomLeft,
    ConnectorCornerTopLeft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    pub id: SymbolId,
    pub emoji: &'static str,
    pub unicode: &'static str,
    pub ascii: &'static str,
    pub color: SemanticColor,
}

pub static STANDARD_SYMBOLS: &[SymbolEntry] = &[
    SymbolEntry { id: SymbolId::NodePending, emoji: "⏳", unicode: "○", ascii: "[ ]", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::NodeRunning, emoji: "🔄", unicode: "◐", ascii: "[~]", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::NodeCompleted, emoji: "✅", unicode: "●", ascii: "[x]", color: SemanticColor::Success },
    SymbolEntry { id: SymbolId::NodeFailed, emoji: "❌", unicode: "✗", ascii: "[!]", color: SemanticColor::Error },
    SymbolEntry { id: SymbolId::NodeSkipped, emoji: "⏭️", unicode: "◌", ascii: "[-]", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::NodeIntercepted, emoji: "🔮", unicode: "◇", ascii: "[m]", color: SemanticColor::Info },
    SymbolEntry { id: SymbolId::EdgeIdle, emoji: "─", unicode: "─", ascii: "-", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::EdgeFlowing, emoji: "⚡", unicode: "═", ascii: "=", color: SemanticColor::Accent },
    SymbolEntry { id: SymbolId::EdgeDone, emoji: "─", unicode: "─", ascii: "-", color: SemanticColor::Success },
    SymbolEntry { id: SymbolId::EdgeDead, emoji: "┄", unicode: "┄", ascii: ".", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::DagNotStarted, emoji: "🔲", unicode: "□", ascii: "[ ]", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::DagRunning, emoji: "🚀", unicode: "▶", ascii: "[>]", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::DagCompleted, emoji: "🏁", unicode: "■", ascii: "[X]", color: SemanticColor::Success },
    SymbolEntry { id: SymbolId::DagFailed, emoji: "💥", unicode: "■", ascii: "[!]", color: SemanticColor::Error },
    SymbolEntry { id: SymbolId::BoundaryMarker, emoji: "🌐", unicode: "◈", ascii: "[B]", color: SemanticColor::Info },
    SymbolEntry { id: SymbolId::Spinner0, emoji: "⠋", unicode: "⠋", ascii: "|", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner1, emoji: "⠙", unicode: "⠙", ascii: "/", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner2, emoji: "⠹", unicode: "⠹", ascii: "-", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner3, emoji: "⠸", unicode: "⠸", ascii: "\\", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner4, emoji: "⠼", unicode: "⠼", ascii: "|", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner5, emoji: "⠴", unicode: "⠴", ascii: "/", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner6, emoji: "⠦", unicode: "⠦", ascii: "-", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner7, emoji: "⠧", unicode: "⠧", ascii: "\\", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner8, emoji: "⠇", unicode: "⠇", ascii: "|", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Spinner9, emoji: "⠏", unicode: "⠏", ascii: "/", color: SemanticColor::Active },
    SymbolEntry { id: SymbolId::Success, emoji: "✅", unicode: "✓", ascii: "OK", color: SemanticColor::Success },
    SymbolEntry { id: SymbolId::Failure, emoji: "❌", unicode: "✗", ascii: "FAIL", color: SemanticColor::Error },
    SymbolEntry { id: SymbolId::Warning, emoji: "⚠️", unicode: "⚠", ascii: "WARN", color: SemanticColor::Warning },
    SymbolEntry { id: SymbolId::Info, emoji: "ℹ️", unicode: "ℹ", ascii: "INFO", color: SemanticColor::Info },
    SymbolEntry { id: SymbolId::DataList, emoji: "📋", unicode: "≡", ascii: "[L]", color: SemanticColor::Default },
    SymbolEntry { id: SymbolId::DataMap, emoji: "🗂️", unicode: "⊞", ascii: "[M]", color: SemanticColor::Default },
    SymbolEntry { id: SymbolId::DataSecret, emoji: "🔒", unicode: "▪", ascii: "[*]", color: SemanticColor::Warning },
    SymbolEntry { id: SymbolId::DataUrl, emoji: "🔗", unicode: "↗", ascii: "[U]", color: SemanticColor::Info },
    SymbolEntry { id: SymbolId::DataTimer, emoji: "⏱️", unicode: "⏱", ascii: "[T]", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorHorizontal, emoji: "──", unicode: "──", ascii: "--", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorVertical, emoji: "│", unicode: "│", ascii: "|", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorTeeDown, emoji: "┬", unicode: "┬", ascii: "+", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorTeeUp, emoji: "┴", unicode: "┴", ascii: "+", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorCornerBottomLeft, emoji: "└", unicode: "└", ascii: "+", color: SemanticColor::Dim },
    SymbolEntry { id: SymbolId::ConnectorCornerTopLeft, emoji: "┌", unicode: "┌", ascii: "+", color: SemanticColor::Dim }
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiMapping {
    pub color: SemanticColor,
    pub code: &'static str,
}

pub static ANSI_MAPPINGS: &[AnsiMapping] = &[
    AnsiMapping { color: SemanticColor::Default, code: "\x1b[0m" },
    AnsiMapping { color: SemanticColor::Success, code: "\x1b[38;5;34m" },
    AnsiMapping { color: SemanticColor::Warning, code: "\x1b[38;5;208m" },
    AnsiMapping { color: SemanticColor::Error, code: "\x1b[38;5;196m" },
    AnsiMapping { color: SemanticColor::Info, code: "\x1b[38;5;39m" },
    AnsiMapping { color: SemanticColor::Dim, code: "\x1b[2m" },
    AnsiMapping { color: SemanticColor::Active, code: "\x1b[38;5;208m" },
    AnsiMapping { color: SemanticColor::Accent, code: "\x1b[38;5;75m" }
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanStyle {
    pub color: Option<SemanticColor>,
    pub bold: bool,
    pub italic: bool,
    pub symbol: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum RenderMode {
    Standard,
    Dynamic,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: SpanStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub spans: Vec<Span>,
    pub indent: i64,
    pub max_width: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum CursorAction {
    Overwrite,
    Append,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub lines: Vec<Line>,
    pub cursor_action: CursorAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum ViewportUnit {
    Chars,
    Pixels,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Viewport {
    pub width: i64,
    pub height: i64,
    pub unit: ViewportUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum BoxStyle {
    Closed,
    OpenRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxChars {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
}

pub static UNICODE_BOX_CHARS: BoxChars = BoxChars { top_left: "╭", top_right: "╮", bottom_left: "╰", bottom_right: "╯", horizontal: "─", vertical: "│" };

pub static ASCII_BOX_CHARS: BoxChars = BoxChars { top_left: "+", top_right: "+", bottom_left: "+", bottom_right: "+", horizontal: "-", vertical: "|" };

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxConfig {
    pub title: String,
    pub style: BoxStyle,
    pub width: i64,
    pub min_width: i64,
    pub color: SemanticColor,
    pub content_color: Option<SemanticColor>,
    pub tier: Tier,
}

impl SymbolId {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NodePending => "⏳",
            Self::NodeRunning => "🔄",
            Self::NodeCompleted => "✅",
            Self::NodeFailed => "❌",
            Self::NodeSkipped => "⏭️",
            Self::NodeIntercepted => "🔮",
            Self::EdgeIdle => "─",
            Self::EdgeFlowing => "⚡",
            Self::EdgeDone => "─",
            Self::EdgeDead => "┄",
            Self::DagNotStarted => "🔲",
            Self::DagRunning => "🚀",
            Self::DagCompleted => "🏁",
            Self::DagFailed => "💥",
            Self::BoundaryMarker => "🌐",
            Self::Spinner0 => "⠋",
            Self::Spinner1 => "⠙",
            Self::Spinner2 => "⠹",
            Self::Spinner3 => "⠸",
            Self::Spinner4 => "⠼",
            Self::Spinner5 => "⠴",
            Self::Spinner6 => "⠦",
            Self::Spinner7 => "⠧",
            Self::Spinner8 => "⠇",
            Self::Spinner9 => "⠏",
            Self::Success => "✅",
            Self::Failure => "❌",
            Self::Warning => "⚠️",
            Self::Info => "ℹ️",
            Self::DataList => "📋",
            Self::DataMap => "🗂️",
            Self::DataSecret => "🔒",
            Self::DataUrl => "🔗",
            Self::DataTimer => "⏱️",
            Self::ConnectorHorizontal => "──",
            Self::ConnectorVertical => "│",
            Self::ConnectorTeeDown => "┬",
            Self::ConnectorTeeUp => "┴",
            Self::ConnectorCornerBottomLeft => "└",
            Self::ConnectorCornerTopLeft => "┌",
        }
    }
}

impl SymbolId {
    pub fn unicode(&self) -> &'static str {
        match self {
            Self::NodePending => "○",
            Self::NodeRunning => "◐",
            Self::NodeCompleted => "●",
            Self::NodeFailed => "✗",
            Self::NodeSkipped => "◌",
            Self::NodeIntercepted => "◇",
            Self::EdgeIdle => "─",
            Self::EdgeFlowing => "═",
            Self::EdgeDone => "─",
            Self::EdgeDead => "┄",
            Self::DagNotStarted => "□",
            Self::DagRunning => "▶",
            Self::DagCompleted => "■",
            Self::DagFailed => "■",
            Self::BoundaryMarker => "◈",
            Self::Spinner0 => "⠋",
            Self::Spinner1 => "⠙",
            Self::Spinner2 => "⠹",
            Self::Spinner3 => "⠸",
            Self::Spinner4 => "⠼",
            Self::Spinner5 => "⠴",
            Self::Spinner6 => "⠦",
            Self::Spinner7 => "⠧",
            Self::Spinner8 => "⠇",
            Self::Spinner9 => "⠏",
            Self::Success => "✓",
            Self::Failure => "✗",
            Self::Warning => "⚠",
            Self::Info => "ℹ",
            Self::DataList => "≡",
            Self::DataMap => "⊞",
            Self::DataSecret => "▪",
            Self::DataUrl => "↗",
            Self::DataTimer => "⏱",
            Self::ConnectorHorizontal => "──",
            Self::ConnectorVertical => "│",
            Self::ConnectorTeeDown => "┬",
            Self::ConnectorTeeUp => "┴",
            Self::ConnectorCornerBottomLeft => "└",
            Self::ConnectorCornerTopLeft => "┌",
        }
    }
}

impl SymbolId {
    pub fn ascii(&self) -> &'static str {
        match self {
            Self::NodePending => "[ ]",
            Self::NodeRunning => "[~]",
            Self::NodeCompleted => "[x]",
            Self::NodeFailed => "[!]",
            Self::NodeSkipped => "[-]",
            Self::NodeIntercepted => "[m]",
            Self::EdgeIdle => "-",
            Self::EdgeFlowing => "=",
            Self::EdgeDone => "-",
            Self::EdgeDead => ".",
            Self::DagNotStarted => "[ ]",
            Self::DagRunning => "[>]",
            Self::DagCompleted => "[X]",
            Self::DagFailed => "[!]",
            Self::BoundaryMarker => "[B]",
            Self::Spinner0 => "|",
            Self::Spinner1 => "/",
            Self::Spinner2 => "-",
            Self::Spinner3 => "\\",
            Self::Spinner4 => "|",
            Self::Spinner5 => "/",
            Self::Spinner6 => "-",
            Self::Spinner7 => "\\",
            Self::Spinner8 => "|",
            Self::Spinner9 => "/",
            Self::Success => "OK",
            Self::Failure => "FAIL",
            Self::Warning => "WARN",
            Self::Info => "INFO",
            Self::DataList => "[L]",
            Self::DataMap => "[M]",
            Self::DataSecret => "[*]",
            Self::DataUrl => "[U]",
            Self::DataTimer => "[T]",
            Self::ConnectorHorizontal => "--",
            Self::ConnectorVertical => "|",
            Self::ConnectorTeeDown => "+",
            Self::ConnectorTeeUp => "+",
            Self::ConnectorCornerBottomLeft => "+",
            Self::ConnectorCornerTopLeft => "+",
        }
    }
}

impl SymbolId {
    pub fn color(&self) -> SemanticColor {
        match self {
            Self::NodePending => SemanticColor::Dim,
            Self::NodeRunning => SemanticColor::Active,
            Self::NodeCompleted => SemanticColor::Success,
            Self::NodeFailed => SemanticColor::Error,
            Self::NodeSkipped => SemanticColor::Dim,
            Self::NodeIntercepted => SemanticColor::Info,
            Self::EdgeIdle => SemanticColor::Dim,
            Self::EdgeFlowing => SemanticColor::Accent,
            Self::EdgeDone => SemanticColor::Success,
            Self::EdgeDead => SemanticColor::Dim,
            Self::DagNotStarted => SemanticColor::Dim,
            Self::DagRunning => SemanticColor::Active,
            Self::DagCompleted => SemanticColor::Success,
            Self::DagFailed => SemanticColor::Error,
            Self::BoundaryMarker => SemanticColor::Info,
            Self::Spinner0 => SemanticColor::Active,
            Self::Spinner1 => SemanticColor::Active,
            Self::Spinner2 => SemanticColor::Active,
            Self::Spinner3 => SemanticColor::Active,
            Self::Spinner4 => SemanticColor::Active,
            Self::Spinner5 => SemanticColor::Active,
            Self::Spinner6 => SemanticColor::Active,
            Self::Spinner7 => SemanticColor::Active,
            Self::Spinner8 => SemanticColor::Active,
            Self::Spinner9 => SemanticColor::Active,
            Self::Success => SemanticColor::Success,
            Self::Failure => SemanticColor::Error,
            Self::Warning => SemanticColor::Warning,
            Self::Info => SemanticColor::Info,
            Self::DataList => SemanticColor::Default,
            Self::DataMap => SemanticColor::Default,
            Self::DataSecret => SemanticColor::Warning,
            Self::DataUrl => SemanticColor::Info,
            Self::DataTimer => SemanticColor::Dim,
            Self::ConnectorHorizontal => SemanticColor::Dim,
            Self::ConnectorVertical => SemanticColor::Dim,
            Self::ConnectorTeeDown => SemanticColor::Dim,
            Self::ConnectorTeeUp => SemanticColor::Dim,
            Self::ConnectorCornerBottomLeft => SemanticColor::Dim,
            Self::ConnectorCornerTopLeft => SemanticColor::Dim,
        }
    }
}

impl SemanticColor {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Default => "\x1b[0m",
            Self::Success => "\x1b[38;5;34m",
            Self::Warning => "\x1b[38;5;208m",
            Self::Error => "\x1b[38;5;196m",
            Self::Info => "\x1b[38;5;39m",
            Self::Dim => "\x1b[2m",
            Self::Active => "\x1b[38;5;208m",
            Self::Accent => "\x1b[38;5;75m",
        }
    }
}

