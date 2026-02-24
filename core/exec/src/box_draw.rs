//! Box rendering utilities for terminal output.
//!
//! Provides consistent box drawing for preambles, errors, and other structured
//! output. Ported from `gunb.ai/tools/terminal/box.go`.
//!
//! Box styles:
//! - [`BoxStyle::Closed`]: Fully closed `╭─╮ │ │ ╰─╯`
//! - [`BoxStyle::OpenRight`]: Open on right `╭─ │  ╰─`
//!
//! Tier-aware rendering:
//! - Unicode/Emoji: `╭─╮`, `│`, `╰─╯` (rounded box drawing characters)
//! - ASCII: `+-+`, `|`, `+-+`

use gunbc_ir::symbols::{SemanticColor, Tier};
use std::io::Write;

/// Default box width in columns.
pub const DEFAULT_BOX_WIDTH: usize = 60;

/// Minimum box width in columns.
pub const DEFAULT_MIN_BOX_WIDTH: usize = 40;

/// Maximum lines of captured output shown in error boxes.
pub const ERROR_OUTPUT_MAX_LINES: usize = 50;

/// Controls whether a box has a closed right edge or is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxStyle {
    /// Fully closed box: `╭─╮ │ │ ╰─╯`
    Closed,
    /// Box open on the right: `╭─ │  ╰─`
    OpenRight,
}

/// Box drawing character set for a specific tier.
struct BoxChars {
    top_left: &'static str,
    top_right: &'static str,
    bottom_left: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

impl BoxChars {
    fn for_tier(tier: Tier) -> Self {
        match tier {
            Tier::Ascii => Self {
                top_left: "+",
                top_right: "+",
                bottom_left: "+",
                bottom_right: "+",
                horizontal: "-",
                vertical: "|",
            },
            Tier::Unicode | Tier::Emoji => Self {
                top_left: "╭",
                top_right: "╮",
                bottom_left: "╰",
                bottom_right: "╯",
                horizontal: "─",
                vertical: "│",
            },
        }
    }
}

/// A terminal box with title, style, and color.
///
/// Renders bordered content to a `Write` sink. The box has a title on the
/// top line and content lines below. Colors are applied via ANSI escape codes
/// when `use_color` is true.
#[derive(Debug, Clone)]
pub struct TermBox {
    /// Title appears after the top-left corner character.
    pub title: String,
    /// Controls whether the right edge is closed or open.
    pub style: BoxStyle,
    /// Total box width (minimum auto-calculated from title).
    pub width: usize,
    /// Minimum box width.
    pub min_width: usize,
    /// Semantic color for the border.
    pub color: SemanticColor,
    /// Optional semantic color for content text (inside the box).
    pub content_color: Option<SemanticColor>,
    /// Encoding tier for box-drawing characters.
    pub tier: Tier,
    /// Whether to emit ANSI color codes.
    pub use_color: bool,
}

impl TermBox {
    /// Create a new box with default settings.
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

    /// Set the border color.
    pub fn with_color(mut self, color: SemanticColor) -> Self {
        self.color = color;
        self
    }

    /// Set the content text color.
    pub fn with_content_color(mut self, color: SemanticColor) -> Self {
        self.content_color = Some(color);
        self
    }

    /// Set the box width.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the encoding tier.
    pub fn with_tier(mut self, tier: Tier) -> Self {
        self.tier = tier;
        self
    }

    /// Set whether to use ANSI color codes.
    pub fn with_use_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    /// Write the top line of the box.
    ///
    /// Closed: `╭─ Title ─────╮`
    /// Open:   `╭─ Title ─────`
    pub fn write_top<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let chars = BoxChars::for_tier(self.tier);
        let title_part = format!("{} {} ", chars.horizontal, self.title);
        let eff_width = self.effective_width(&title_part);

        let (color_start, color_end) = self.border_escapes();

        if self.style == BoxStyle::Closed {
            // ╭─ Title ─────╮
            let padding = eff_width.saturating_sub(display_width(&title_part) + 2);
            writeln!(
                w,
                "{color_start}{}{title_part}{}{}{color_end}",
                chars.top_left,
                chars.horizontal.repeat(padding),
                chars.top_right,
            )
        } else {
            // ╭─ Title ─────
            let padding = eff_width.saturating_sub(display_width(&title_part) + 1);
            writeln!(
                w,
                "{color_start}{}{title_part}{}{color_end}",
                chars.top_left,
                chars.horizontal.repeat(padding),
            )
        }
    }

    /// Write a content line with the left border.
    ///
    /// Closed: `│ content     │`
    /// Open:   `│ content`
    pub fn write_content<W: Write>(&self, w: &mut W, content: &str) -> std::io::Result<()> {
        let chars = BoxChars::for_tier(self.tier);
        let (color_start, color_end) = self.border_escapes();
        let (content_start, content_end) = self.content_escapes();

        if self.style == BoxStyle::Closed {
            let title_part = format!("{} {} ", chars.horizontal, self.title);
            let eff_width = self.effective_width(&title_part);
            // Content format: 1(│) + 1(space) + text + padding + 1(│) = eff_width
            let padding = eff_width.saturating_sub(3 + display_width(content));
            writeln!(
                w,
                "{color_start}{}{color_end} {content_start}{content}{content_end}{}{color_start}{}{color_end}",
                chars.vertical,
                " ".repeat(padding),
                chars.vertical,
            )
        } else {
            // │ content
            writeln!(
                w,
                "{color_start}{}{color_end} {content_start}{content}{content_end}",
                chars.vertical,
            )
        }
    }

    /// Write the bottom line of the box.
    ///
    /// Closed: `╰─────────────╯`
    /// Open:   `╰─────────────`
    pub fn write_bottom<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let chars = BoxChars::for_tier(self.tier);
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

    /// Render a complete box with the given content lines.
    pub fn render<W: Write>(&self, w: &mut W, lines: &[&str]) -> std::io::Result<()> {
        self.write_top(w)?;
        for line in lines {
            self.write_content(w, line)?;
        }
        self.write_bottom(w)
    }

    /// Calculate effective width considering title and minimums.
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

    /// Get ANSI escape pair for the border color.
    fn border_escapes(&self) -> (&str, &str) {
        if self.use_color {
            (self.color.ansi(), SemanticColor::reset())
        } else {
            ("", "")
        }
    }

    /// Get ANSI escape pair for the content color.
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

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Create an error box (open-right, red border, dim content).
pub fn error_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::OpenRight)
        .with_color(SemanticColor::Error)
        .with_content_color(SemanticColor::Dim)
        .with_tier(tier)
        .with_use_color(use_color)
}

/// Create a preamble/header box (closed, soft blue border).
pub fn preamble_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::Closed)
        .with_color(SemanticColor::Accent)
        .with_tier(tier)
        .with_use_color(use_color)
}

/// Create an info box (open-right, info color border).
pub fn info_box(title: impl Into<String>, tier: Tier, use_color: bool) -> TermBox {
    TermBox::new(title, BoxStyle::OpenRight)
        .with_color(SemanticColor::Info)
        .with_tier(tier)
        .with_use_color(use_color)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_box_unicode() {
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
    fn test_open_right_box_unicode() {
        let b = TermBox::new("Error", BoxStyle::OpenRight)
            .with_tier(Tier::Unicode)
            .with_use_color(false);
        let mut buf = Vec::new();
        b.render(&mut buf, &["something failed"]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("╭"));
        assert!(
            !output.contains("╮"),
            "open-right should not have top-right corner"
        );
        assert!(output.contains("│"));
        assert!(output.contains("╰"));
        assert!(
            !output.contains("╯"),
            "open-right should not have bottom-right corner"
        );
        assert!(output.contains("something failed"));
    }

    #[test]
    fn test_ascii_box() {
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
    fn test_colored_box() {
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
    fn test_error_box_convenience() {
        let b = error_box("build_failed", Tier::Unicode, false);
        assert_eq!(b.style, BoxStyle::OpenRight);
        assert_eq!(b.color, SemanticColor::Error);
        assert_eq!(b.content_color, Some(SemanticColor::Dim));
    }

    #[test]
    fn test_preamble_box_convenience() {
        let b = preamble_box("gist", Tier::Emoji, true);
        assert_eq!(b.style, BoxStyle::Closed);
        assert_eq!(b.color, SemanticColor::Accent);
        assert!(b.use_color);
    }

    #[test]
    fn test_empty_content() {
        let b = TermBox::new("Empty", BoxStyle::Closed)
            .with_tier(Tier::Unicode)
            .with_use_color(false);
        let mut buf = Vec::new();
        let empty: &[&str] = &[];
        b.render(&mut buf, empty).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should have top and bottom but no content lines
        assert!(output.contains("╭"));
        assert!(output.contains("╰"));
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2, "empty box should have just top and bottom");
    }
}
