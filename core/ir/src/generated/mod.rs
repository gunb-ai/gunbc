//! Generated from DSL type definitions. Do not edit.

#[inline]
pub fn code_point_i64(c: char) -> i64 { c as u32 as i64 }

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

pub fn resolve_symbol(id: SymbolId, tier: Tier) -> String {
    todo!("generated from DSL");
}

pub fn symbol_color(id: SymbolId) -> SemanticColor {
    todo!("generated from DSL");
}

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

pub fn ansi_code(c: SemanticColor) -> String {
    todo!("generated from DSL");
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum DisplayWidth {
    ZeroWidth,
    Narrow,
    Wide,
}

pub fn display_width_columns(w: DisplayWidth) -> i64 {
    match w {
    DisplayWidth::ZeroWidth => {
        0
    }
    DisplayWidth::Narrow => {
        1
    }
    DisplayWidth::Wide => {
        2
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeBlock {
    pub name: &'static str,
    pub start: i64,
    pub end_inclusive: i64,
    pub default_width: DisplayWidth,
}

pub static ZERO_WIDTH_BLOCKS: &[UnicodeBlock] = &[
    UnicodeBlock { name: "Combining Diacritical Marks", start: 768, end_inclusive: 879, default_width: DisplayWidth::ZeroWidth },
    UnicodeBlock { name: "Combining Diacritical Marks Extended", start: 6832, end_inclusive: 6911, default_width: DisplayWidth::ZeroWidth },
    UnicodeBlock { name: "Combining Diacritical Marks Supplement", start: 7616, end_inclusive: 7679, default_width: DisplayWidth::ZeroWidth },
    UnicodeBlock { name: "Combining Marks for Symbols", start: 8400, end_inclusive: 8447, default_width: DisplayWidth::ZeroWidth },
    UnicodeBlock { name: "Variation Selectors", start: 65024, end_inclusive: 65039, default_width: DisplayWidth::ZeroWidth },
    UnicodeBlock { name: "Combining Half Marks", start: 65056, end_inclusive: 65071, default_width: DisplayWidth::ZeroWidth }
];

pub static ZERO_WIDTH_CODEPOINTS: &[i64] = &[
    8203,
    8204,
    8205,
    65279
];

pub static WIDE_BLOCKS: &[UnicodeBlock] = &[
    UnicodeBlock { name: "Hangul Jamo", start: 4352, end_inclusive: 4447, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Radicals and Symbols", start: 11904, end_inclusive: 12350, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Hiragana / Katakana / CJK Compat", start: 12353, end_inclusive: 13247, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Extension A", start: 13312, end_inclusive: 19903, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Unified Ideographs", start: 19968, end_inclusive: 40959, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Yi Syllables and Radicals", start: 40960, end_inclusive: 42191, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Hangul Syllables", start: 44032, end_inclusive: 55215, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Compatibility Ideographs", start: 63744, end_inclusive: 64255, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Compatibility Forms", start: 65072, end_inclusive: 65135, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Fullwidth ASCII", start: 65281, end_inclusive: 65376, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Fullwidth Signs", start: 65504, end_inclusive: 65510, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Extension B+", start: 131072, end_inclusive: 196607, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "CJK Extension G+", start: 196608, end_inclusive: 262143, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Misc Symbols and Dingbats", start: 9728, end_inclusive: 10175, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Symbols / Pictographs / Emoticons", start: 127744, end_inclusive: 129535, default_width: DisplayWidth::Wide },
    UnicodeBlock { name: "Symbols Extended", start: 129536, end_inclusive: 131071, default_width: DisplayWidth::Wide }
];

pub fn in_block(cp: i64, block: UnicodeBlock) -> bool {
    cp >= block.start && cp <= block.end_inclusive
}

pub fn char_display_width(c: char) -> DisplayWidth {
    let cp = code_point_i64(c);
    if {
    let mut __contains_0 = false;
    for __elem_1 in ZERO_WIDTH_CODEPOINTS.iter().cloned() {
        if __elem_1 == cp {
    __contains_0 = true;
    break;
};
    }
    __contains_0
} {
    return ZeroWidth;
};
    if {
    let mut __any_2 = false;
    for __elem_3 in ZERO_WIDTH_BLOCKS.iter().cloned() {
        if in_block(cp, __elem_3) {
    __any_2 = true;
    break;
};
    }
    __any_2
} {
    return ZeroWidth;
};
    if {
    let mut __any_4 = false;
    for __elem_5 in WIDE_BLOCKS.iter().cloned() {
        if in_block(cp, __elem_5) {
    __any_4 = true;
    break;
};
    }
    __any_4
} {
    return Wide;
};
    return Narrow;
}

pub fn char_width(c: char) -> i64 {
    display_width_columns(char_display_width(c))
}

pub fn string_display_width(s: String) -> i64 {
    {
    let mut __sum_2 = 0;
    for __elem_3 in {
    let mut __mapped_0 = vec!();
    for __elem_1 in s.chars() {
        __mapped_0.push(char_width(__elem_1));
    }
    __mapped_0
} {
        __sum_2 = __sum_2 + __elem_3;
    }
    __sum_2
}
}

pub fn truncate_text(text: String, max_width: i64) -> String {
    todo!("generated from DSL");
}

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

pub fn span_width(span: Span) -> i64 {
    string_display_width(span.text)
}

pub fn truncate_spans(spans: Vec<Span>, budget: i64) -> Vec<Span> {
    todo!("generated from DSL");
}

pub fn constrain_line(line: Line, max_width: i64) -> Line {
    let indent_width = line.indent * 4;
    let budget = max_width - indent_width;
    if budget <= 0 {
    return Line { spans: vec!(), indent: line.indent, max_width: Some(max_width) };
};
    Line { spans: truncate_spans(line.spans, budget), indent: line.indent, max_width: Some(max_width) }
}

pub fn constrain_frame(frame: Frame, viewport: Viewport) -> Frame {
    Frame { lines: {
    let mut __mapped_0 = vec!();
    for __elem_1 in frame.lines {
        __mapped_0.push(constrain_line(__elem_1, viewport.width));
    }
    __mapped_0
}, cursor_action: frame.cursor_action }
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

pub fn box_chars_for_tier(tier: Tier) -> BoxChars {
    match tier {
    Tier::Ascii => {
        ASCII_BOX_CHARS.clone()
    }
    _ => {
        UNICODE_BOX_CHARS.clone()
    }
}
}

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

pub fn effective_width(config: BoxConfig, title_part: String) -> i64 {
    let base = string_display_width(title_part) + 10;
    let w1 = if base < config.min_width {
    config.min_width
} else {
    base
};
    match config.style {
    BoxStyle::Closed => {
        if w1 < config.width {
    config.width
} else {
    w1
}
    }
    _ => {
        w1
    }
}
}

pub fn box_top_line(config: BoxConfig) -> Line {
    let chars = box_chars_for_tier(config.tier);
    let title_part = format!("{} {} ", chars.horizontal, config.title);
    let eff = effective_width(config, title_part);
    let title_width = string_display_width(title_part);
    match config.style {
    BoxStyle::Closed => {
         {  }
    }
    BoxStyle::OpenRight => {
         {  }
    }
}
}

pub fn box_content_line(config: BoxConfig, content: String) -> Line {
    let chars = box_chars_for_tier(config.tier);
    let border_span = Span { text: format!("{} ", chars.vertical), style: SpanStyle { color: Some(config.color), bold: false, italic: false, symbol: None } };
    let content_style = match config.content_color {
    None => {
        SpanStyle { color: None, bold: false, italic: false, symbol: None }
    }
    c => {
        SpanStyle { color: Some(c), bold: false, italic: false, symbol: None }
    }
};
    let content_span = Span { text: content, style: content_style };
    Line { spans: vec!(border_span, content_span), indent: 0, max_width: None }
}

pub fn box_bottom_line(config: BoxConfig) -> Line {
    let chars = box_chars_for_tier(config.tier);
    let title_part = format!("{} {} ", chars.horizontal, config.title);
    let eff = effective_width(config, title_part);
    match config.style {
    BoxStyle::Closed => {
         {  }
    }
    BoxStyle::OpenRight => {
         {  }
    }
}
}

pub fn error_box(title: String, tier: Tier) -> BoxConfig {
    BoxConfig { title: title, style: OpenRight, width: 60, min_width: 40, color: Error, content_color: Some(Dim), tier: tier }
}

pub fn preamble_box(title: String, tier: Tier) -> BoxConfig {
    BoxConfig { title: title, style: Closed, width: 60, min_width: 40, color: Accent, content_color: None, tier: tier }
}

pub fn info_box(title: String, tier: Tier) -> BoxConfig {
    BoxConfig { title: title, style: OpenRight, width: 60, min_width: 40, color: Info, content_color: None, tier: tier }
}

pub fn repeat_char(c: String, n: i64) -> String {
    todo!("generated from DSL");
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

