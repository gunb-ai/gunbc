//! Frame writing layer — handles cursor I/O and medium-based rendering.
//!
//! [`FrameWriter`] dispatches between [`AnsiFrameRenderer`] and [`PlainFrameRenderer`],
//! each implementing [`FrameRenderer<M>`] for their respective medium.

use gunbc_ir::render_ir::{AnsiText, CursorAction, Frame, FrameRenderer, OutputMedium, PlainText};
use gunbc_ir::symbols::{SymbolSet, Tier};
use std::io::Write;

// ---------------------------------------------------------------------------
// AnsiFrameRenderer
// ---------------------------------------------------------------------------

/// Renders frames using ANSI escape codes for styled output.
pub struct AnsiFrameRenderer {
    medium: AnsiText,
    last_frame_lines: usize,
    is_tty: bool,
}

impl AnsiFrameRenderer {
    pub fn new(medium: AnsiText, is_tty: bool) -> Self {
        Self {
            medium,
            last_frame_lines: 0,
            is_tty,
        }
    }
}

impl FrameRenderer<AnsiText> for AnsiFrameRenderer {
    fn medium(&self) -> &AnsiText {
        &self.medium
    }

    fn render_frame(&mut self, frame: &Frame, sink: &mut dyn Write) -> std::io::Result<()> {
        render_frame_common(
            &self.medium,
            frame,
            sink,
            self.is_tty,
            &mut self.last_frame_lines,
        )
    }
}

// ---------------------------------------------------------------------------
// PlainFrameRenderer
// ---------------------------------------------------------------------------

/// Renders frames using plain text (no ANSI content escapes).
/// Cursor control still uses raw ANSI (terminal protocol, not content).
pub struct PlainFrameRenderer {
    medium: PlainText,
    last_frame_lines: usize,
    is_tty: bool,
}

impl PlainFrameRenderer {
    pub fn new(medium: PlainText, is_tty: bool) -> Self {
        Self {
            medium,
            last_frame_lines: 0,
            is_tty,
        }
    }
}

impl FrameRenderer<PlainText> for PlainFrameRenderer {
    fn medium(&self) -> &PlainText {
        &self.medium
    }

    fn render_frame(&mut self, frame: &Frame, sink: &mut dyn Write) -> std::io::Result<()> {
        render_frame_common(
            &self.medium,
            frame,
            sink,
            self.is_tty,
            &mut self.last_frame_lines,
        )
    }
}

// ---------------------------------------------------------------------------
// Shared cursor I/O logic
// ---------------------------------------------------------------------------

fn render_frame_common<M: OutputMedium<Output = String>>(
    medium: &M,
    frame: &Frame,
    sink: &mut dyn Write,
    is_tty: bool,
    last_frame_lines: &mut usize,
) -> std::io::Result<()> {
    // Cursor-up to overwrite previous frame
    if is_tty && *last_frame_lines > 0 && frame.cursor_action == CursorAction::Overwrite {
        write!(sink, "\x1b[{}A\r", *last_frame_lines)?;
    }

    let num_lines = frame.lines.len();

    // Render each line
    for line in &frame.lines {
        if is_tty {
            write!(sink, "\x1b[2K")?; // erase entire line
        }
        let rendered = medium.render_line(line);
        writeln!(sink, "{}", rendered)?;
    }

    // Clear leftover lines if previous frame was taller
    if is_tty && num_lines < *last_frame_lines {
        let extra = *last_frame_lines - num_lines;
        for _ in 0..extra {
            writeln!(sink, "\x1b[2K")?;
        }
        // Move cursor back up past the blank lines
        write!(sink, "\x1b[{}A", extra)?;
    }

    sink.flush()?;
    *last_frame_lines = num_lines;
    Ok(())
}

// ---------------------------------------------------------------------------
// FrameWriter — enum dispatch wrapper
// ---------------------------------------------------------------------------

/// Runtime dispatch between ANSI and plain frame renderers.
///
/// The set of media is finite and known at compile time, so enum dispatch
/// avoids `Box<dyn>` overhead while keeping a simple API.
pub enum FrameWriter {
    Ansi(AnsiFrameRenderer),
    Plain(PlainFrameRenderer),
}

impl FrameWriter {
    /// Create the appropriate renderer for the given settings.
    pub fn new(
        color_enabled: bool,
        tier: Tier,
        symbol_set: &'static SymbolSet,
        is_tty: bool,
    ) -> Self {
        if color_enabled {
            Self::Ansi(AnsiFrameRenderer::new(
                AnsiText { tier, symbol_set },
                is_tty,
            ))
        } else {
            Self::Plain(PlainFrameRenderer::new(
                PlainText { tier, symbol_set },
                is_tty,
            ))
        }
    }

    /// Write a frame to the given sink.
    pub fn write_frame(&mut self, frame: &Frame, sink: &mut dyn Write) -> std::io::Result<()> {
        match self {
            Self::Ansi(r) => r.render_frame(frame, sink),
            Self::Plain(r) => r.render_frame(frame, sink),
        }
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
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, false);

        let frame = make_frame(vec!["hello", "world"], CursorAction::Overwrite);
        writer.write_frame(&frame, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        // No ANSI escapes in non-TTY plain mode
        assert!(
            !output.contains("\x1b"),
            "Non-TTY should have no ANSI escapes"
        );
    }

    #[test]
    fn test_write_frame_tty_cursor_up() {
        let mut buf = Vec::new();
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, true);

        // First frame
        let frame1 = make_frame(vec!["first"], CursorAction::Overwrite);
        writer.write_frame(&frame1, &mut buf).unwrap();

        // Second frame should cursor-up
        let frame2 = make_frame(vec!["second"], CursorAction::Overwrite);
        writer.write_frame(&frame2, &mut buf).unwrap();

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
        let mut renderer = PlainFrameRenderer::new(
            PlainText {
                tier: Tier::Unicode,
                symbol_set: &STANDARD,
            },
            false,
        );

        let frame = make_frame(vec!["a", "b", "c"], CursorAction::Overwrite);
        renderer.render_frame(&frame, &mut buf).unwrap();
        assert_eq!(renderer.last_frame_lines, 3);
    }

    #[test]
    fn test_write_frame_clears_leftover_lines() {
        let mut buf = Vec::new();
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, true);

        // First frame: 3 lines
        let frame1 = make_frame(vec!["a", "b", "c"], CursorAction::Overwrite);
        writer.write_frame(&frame1, &mut buf).unwrap();

        // Second frame: 1 line — should clear 2 leftover
        let frame2 = make_frame(vec!["x"], CursorAction::Overwrite);
        writer.write_frame(&frame2, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // Should contain cursor-up for the leftover lines
        assert!(
            output.contains("\x1b[2A"),
            "Should cursor-up past 2 leftover lines"
        );
    }
}
