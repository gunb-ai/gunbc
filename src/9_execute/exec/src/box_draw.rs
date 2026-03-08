//! Box rendering utilities for terminal output.
//!
//! Type definitions (`BoxStyle`, `BoxChars`) and data tables
//! (`UNICODE_BOX_CHARS`, `ASCII_BOX_CHARS`) are DSL-generated (see
//! `ir/src/generated/mod.rs`).  This file provides the runtime `TermBox`
//! builder with `Write` I/O that the DSL cannot express.

use gunbc_ir::generated::{BoxChars, ASCII_BOX_CHARS, UNICODE_BOX_CHARS};
use gunbc_ir::symbols::{SemanticColor, Tier};
use std::io::Write;

pub use gunbc_ir::generated::BoxStyle;

pub const DEFAULT_BOX_WIDTH: usize = 60;
pub const DEFAULT_MIN_BOX_WIDTH: usize = 40;
pub const ERROR_OUTPUT_MAX_LINES: usize = 50;

fn box_chars_for_tier(tier: Tier) -> &'static BoxChars {
    match tier {
        Tier::Ascii => &ASCII_BOX_CHARS,
        Tier::Unicode | Tier::Emoji => &UNICODE_BOX_CHARS,
    }
}

#[derive(Debug, Clone)]
pub struct TermBox {
    pub title: String,
    pub style: BoxStyle,
    pub width: usize,
    pub min_width: usize,
    pub color: SemanticColor,
    pub content_color: Option<SemanticColor>,
    pub tier: Tier,
    pub use_color: bool,
}

impl TermBox {
    pub fn new(title: impl Into<String>, style: BoxStyle) -> Self {
        Self {
            title: title.into(),
            style,
            width: DEFAULT_BOX_WIDTH,
            min_width: DEFAULT_MIN_BOX_WIDTH,
            color: SemanticColor::Accent,
            content_color: None,
            tier: Tier::Unicode,
            use_color: true,
        }
    }

    pub fn with_color(mut self, color: SemanticColor) -> Self {
        self.color = color;
        self
    }

    pub fn with_content_color(mut self, color: SemanticColor) -> Self {
        self.content_color = Some(color);
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_tier(mut self, tier: Tier) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_use_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn write_top<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let chars = box_chars_for_tier(self.tier);
        let title_part = format!("{} {} ", chars.horizontal, self.title);
        let eff_width = self.effective_width(&title_part);
        let (color_start, color_end) = self.border_escapes();

        if self.style == BoxStyle::Closed {
            let padding = eff_width.saturating_sub(display_width(&title_part) + 2);
            writeln!(
                w,
                "{color_start}{}{title_part}{}{}{color_end}",
                chars.top_left,
                chars.horizontal.repeat(padding),
                chars.top_right,
            )
        } else {
            let padding = eff_width.saturating_sub(display_width(&title_part) + 1);
            writeln!(
                w,
                "{color_start}{}{title_part}{}{color_end}",
                chars.top_left,
                chars.horizontal.repeat(padding),
            )
        }
    }

    pub fn write_content<W: Write>(&self, w: &mut W, content: &str) -> std::io::Result<()> {
        let chars = box_chars_for_tier(self.tier);
        let (color_start, color_end) = self.border_escapes();
        let (content_start, content_end) = self.content_escapes();

        if self.style == BoxStyle::Closed {
            let title_part = format!("{} {} ", chars.horizontal, self.title);
            let eff_width = self.effective_width(&title_part);
            let padding = eff_width.saturating_sub(3 + display_width(content));
            writeln!(
                w,
                "{color_start}{}{color_end} {content_start}{content}{content_end}{}{color_start}{}{color_end}",
                chars.vertical,
                " ".repeat(padding),
                chars.vertical,
            )
        } else {
            writeln!(
                w,
                "{color_start}{}{color_end} {content_start}{content}{content_end}",
                chars.vertical,
            )
        }
    }

    pub fn write_bottom<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let chars = box_chars_for_tier(self.tier);
        let title_part = format!("{} {} ", chars.horizontal, self.title);
        let eff_width = self.effective_width(&title_part);
        let (color_start, color_end) = self.border_escapes();

        if self.style == BoxStyle::Closed {
            writeln!(
                w,
                "{color_start}{}{}{}{color_end}",
                chars.bottom_left,
                chars.horizontal.repeat(eff_width.saturating_sub(2)),
                chars.bottom_right,
            )
        } else {
            writeln!(
                w,
                "{color_start}{}{}{color_end}",
                chars.bottom_left,
                chars.horizontal.repeat(eff_width.saturating_sub(1)),
            )
        }
    }

    pub fn render<W: Write>(&self, w: &mut W, lines: &[&str]) -> std::io::Result<()> {
        self.write_top(w)?;
        for line in lines {
            self.write_content(w, line)?;
        }
        self.write_bottom(w)
    }

    fn effective_width(&self, title_part: &str) -> usize {
        let mut width = display_width(title_part) + 10;
        if width < self.min_width {
            width = self.min_width;
        }
        if self.style == BoxStyle::Closed && width < self.width {
            width = self.width;
        }
        width
    }

    fn border_escapes(&self) -> (&str, &str) {
        if self.use_color {
            (self.color.ansi(), SemanticColor::reset())
        } else {
            ("", "")
        }
    }

    fn content_escapes(&self) -> (&str, &str) {
        if self.use_color {
            if let Some(cc) = self.content_color {
                return (cc.ansi(), SemanticColor::reset());
            }
        }
        ("", "")
    }
}

fn display_width(s: &str) -> usize {
    crate::frame_build::display_width(s)
}

pub fn error_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::OpenRight)
        .with_color(SemanticColor::Error)
        .with_content_color(SemanticColor::Dim)
        .with_tier(tier)
        .with_use_color(use_color)
}

pub fn preamble_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::Closed)
        .with_color(SemanticColor::Accent)
        .with_tier(tier)
        .with_use_color(use_color)
}

pub fn info_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::OpenRight)
        .with_color(SemanticColor::Info)
        .with_tier(tier)
        .with_use_color(use_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_box_unicode() {
        let b = TermBox::new("Title", BoxStyle::Closed)
            .with_tier(Tier::Unicode)
            .with_use_color(false);
        let mut buf = Vec::new();
        b.render(&mut buf, &["hello", "world"]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("╭"));
        assert!(output.contains("╮"));
        assert!(output.contains("│"));
        assert!(output.contains("╰"));
        assert!(output.contains("╯"));
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn open_right_box_unicode() {
        let b = TermBox::new("Error", BoxStyle::OpenRight)
            .with_tier(Tier::Unicode)
            .with_use_color(false);
        let mut buf = Vec::new();
        b.render(&mut buf, &["something failed"]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("╭"));
        assert!(!output.contains("╮"));
        assert!(output.contains("│"));
        assert!(output.contains("╰"));
        assert!(!output.contains("╯"));
        assert!(output.contains("something failed"));
    }

    #[test]
    fn ascii_box() {
        let b = TermBox::new("Title", BoxStyle::Closed)
            .with_tier(Tier::Ascii)
            .with_use_color(false);
        let mut buf = Vec::new();
        b.render(&mut buf, &["content"]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("+"));
        assert!(output.contains("|"));
        assert!(output.contains("-"));
        assert!(output.contains("content"));
    }

    #[test]
    fn colored_box() {
        let b = TermBox::new("Error", BoxStyle::OpenRight)
            .with_color(SemanticColor::Error)
            .with_content_color(SemanticColor::Dim)
            .with_tier(Tier::Unicode)
            .with_use_color(true);
        let mut buf = Vec::new();
        b.render(&mut buf, &["oops"]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains(SemanticColor::Error.ansi()));
        assert!(output.contains(SemanticColor::Dim.ansi()));
        assert!(output.contains(SemanticColor::reset()));
    }

    #[test]
    fn error_box_convenience() {
        let b = error_box("build_failed", Tier::Unicode, false);
        assert_eq!(b.style, BoxStyle::OpenRight);
        assert_eq!(b.color, SemanticColor::Error);
        assert_eq!(b.content_color, Some(SemanticColor::Dim));
    }

    #[test]
    fn preamble_box_convenience() {
        let b = preamble_box("gist", Tier::Emoji, true);
        assert_eq!(b.style, BoxStyle::Closed);
        assert_eq!(b.color, SemanticColor::Accent);
        assert!(b.use_color);
    }

    #[test]
    fn empty_content() {
        let b = TermBox::new("Empty", BoxStyle::Closed)
            .with_tier(Tier::Unicode)
            .with_use_color(false);
        let mut buf = Vec::new();
        let empty: &[&str] = &[];
        b.render(&mut buf, empty).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("╭"));
        assert!(output.contains("╰"));
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
