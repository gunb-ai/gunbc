//! Frame writing layer — handles cursor I/O and medium-based rendering.
//!
//! [`TextFrameWriter`] takes a [`Frame`] and writes it to a `Write` target,
//! handling cursor movement (overwrite previous frame) and ANSI line clearing.

use gunbc_ir::render_ir::{AnsiText, CursorAction, Frame, OutputMedium, PlainText};
use gunbc_ir::symbols::{SymbolSet, Tier};
use std::io::Write;

// ---------------------------------------------------------------------------
// TermMedium — runtime medium selection
// ---------------------------------------------------------------------------

/// Runtime selection between ANSI and plain text output.
///
/// Replaces the old `color_enabled: bool` field — `Ansi` for color,
/// `Plain` for no-color.
pub enum TermMedium {
    Ansi(AnsiText),
    Plain(PlainText),
}

impl TermMedium {
    /// Create the appropriate medium for the given settings.
    pub fn new(color_enabled: bool, tier: Tier, symbol_set: &'static SymbolSet) -> Self {
        if color_enabled {
            TermMedium::Ansi(AnsiText { tier, symbol_set })
        } else {
            TermMedium::Plain(PlainText { tier, symbol_set })
        }
    }

    /// Render a single line to a string using the active medium.
    fn render_line(&self, line: &gunbc_ir::render_ir::Line) -> String {
        match self {
            TermMedium::Ansi(m) => m.render_line(line),
            TermMedium::Plain(m) => m.render_line(line),
        }
    }
}

// ---------------------------------------------------------------------------
// TextFrameWriter
// ---------------------------------------------------------------------------

/// Writes [`Frame`] IR to a `Write` target with cursor management.
///
/// Handles:
/// - Cursor-up to overwrite previous frame (TTY mode)
/// - Line clearing (TTY mode)
/// - Leftover line cleanup when frame shrinks
/// - Flushing
pub struct TextFrameWriter<W: Write> {
    medium: TermMedium,
    output: W,
    last_frame_lines: usize,
    is_tty: bool,
}

impl<W: Write> TextFrameWriter<W> {
    /// Create a new frame writer.
    pub fn new(output: W, medium: TermMedium, is_tty: bool) -> Self {
        Self {
            medium,
            output,
            last_frame_lines: 0,
            is_tty,
        }
    }

    /// Write a frame to the output, handling cursor movement and line clearing.
    pub fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        // Cursor-up to overwrite previous frame
        if self.is_tty
            && self.last_frame_lines > 0
            && frame.cursor_action == CursorAction::Overwrite
        {
            write!(self.output, "\x1b[{}A\r", self.last_frame_lines)?;
        }

        let num_lines = frame.lines.len();

        // Render each line
        for line in &frame.lines {
            if self.is_tty {
                write!(self.output, "\x1b[2K")?; // erase entire line
            }
            let rendered = self.medium.render_line(line);
            writeln!(self.output, "{}", rendered)?;
        }

        // Clear leftover lines if previous frame was taller
        if self.is_tty && num_lines < self.last_frame_lines {
            let extra = self.last_frame_lines - num_lines;
            for _ in 0..extra {
                writeln!(self.output, "\x1b[2K")?;
            }
            // Move cursor back up past the blank lines
            write!(self.output, "\x1b[{}A", extra)?;
        }

        self.output.flush()?;
        self.last_frame_lines = num_lines;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::render_ir::{CursorAction, Frame, Line, Span};
    use gunbc_ir::symbols::STANDARD;

    fn make_frame(lines: Vec<&str>, cursor_action: CursorAction) -> Frame {
        Frame {
            lines: lines
                .into_iter()
                .map(|t| Line::new(vec![Span::plain(t)]))
                .collect(),
            cursor_action,
        }
    }

    #[test]
    fn test_write_frame_plain_no_tty() {
        let mut buf = Vec::new();
        let medium = TermMedium::Plain(PlainText {
            tier: Tier::Unicode,
            symbol_set: &STANDARD,
        });
        let mut writer = TextFrameWriter::new(&mut buf, medium, false);

        let frame = make_frame(vec!["hello", "world"], CursorAction::Overwrite);
        writer.write_frame(&frame).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        // No ANSI escapes in non-TTY plain mode
        assert!(!output.contains("\x1b"), "Non-TTY should have no ANSI escapes");
    }

    #[test]
    fn test_write_frame_tty_cursor_up() {
        let mut buf = Vec::new();
        let medium = TermMedium::Plain(PlainText {
            tier: Tier::Unicode,
            symbol_set: &STANDARD,
        });
        let mut writer = TextFrameWriter::new(&mut buf, medium, true);

        // First frame
        let frame1 = make_frame(vec!["first"], CursorAction::Overwrite);
        writer.write_frame(&frame1).unwrap();

        // Second frame should cursor-up
        let frame2 = make_frame(vec!["second"], CursorAction::Overwrite);
        writer.write_frame(&frame2).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(
            output.contains("\x1b[1A"),
            "TTY should contain cursor-up escape"
        );
        assert!(output.contains("second"));
    }

    #[test]
    fn test_write_frame_tracks_line_count() {
        let mut buf = Vec::new();
        let medium = TermMedium::Plain(PlainText {
            tier: Tier::Unicode,
            symbol_set: &STANDARD,
        });
        let mut writer = TextFrameWriter::new(&mut buf, medium, false);

        let frame = make_frame(vec!["a", "b", "c"], CursorAction::Overwrite);
        writer.write_frame(&frame).unwrap();
        assert_eq!(writer.last_frame_lines, 3);
    }

    #[test]
    fn test_write_frame_clears_leftover_lines() {
        let mut buf = Vec::new();
        let medium = TermMedium::Plain(PlainText {
            tier: Tier::Unicode,
            symbol_set: &STANDARD,
        });
        let mut writer = TextFrameWriter::new(&mut buf, medium, true);

        // First frame: 3 lines
        let frame1 = make_frame(vec!["a", "b", "c"], CursorAction::Overwrite);
        writer.write_frame(&frame1).unwrap();

        // Second frame: 1 line — should clear 2 leftover
        let frame2 = make_frame(vec!["x"], CursorAction::Overwrite);
        writer.write_frame(&frame2).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // Should contain cursor-up for the leftover lines
        assert!(output.contains("\x1b[2A"), "Should cursor-up past 2 leftover lines");
    }
}
