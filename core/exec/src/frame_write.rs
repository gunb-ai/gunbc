//! Frame writing layer — handles cursor I/O and medium-based rendering.
//!
//! [`FrameWriter`] dispatches between [`AnsiFrameRenderer`] and [`PlainFrameRenderer`],
//! each implementing [`FrameRenderer<M>`] for their respective medium.
//!
//! [`CursorGuard`] provides RAII cursor hide/show to ensure the cursor is
//! always restored, even on panics.

use gunbc_ir::render_ir::{AnsiText, CursorAction, Frame, FrameRenderer, OutputMedium, PlainText};
use gunbc_ir::symbols::{SymbolSet, Tier, CURSOR_HIDE, CURSOR_SHOW};
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
    let overwrite = is_tty && frame.cursor_action == CursorAction::Overwrite;
    let num_lines = frame.lines.len();

    // Buffer the entire frame so it's written as a single atomic chunk.
    // Rust's stderr is unbuffered — without this, each write!() is a separate
    // syscall and the terminal renders intermediate states (visible flicker).
    let mut buf: Vec<u8> = Vec::with_capacity(4096);

    // Move cursor up to the start of the previous frame.
    if overwrite && *last_frame_lines > 0 {
        write!(buf, "\x1b[{}A\r", *last_frame_lines)?;
    }

    // Render each line with per-line clearing (matches gunb.ai).
    // \x1b[2K clears the current line before writing, so old content is
    // overwritten in-place with no visible gap between frames.
    for line in &frame.lines {
        let rendered = medium.render_line(line);
        if overwrite {
            write!(buf, "\x1b[2K")?;
        }
        writeln!(buf, "{}", rendered)?;
    }

    // If the new frame is shorter than the previous one, clear leftover lines
    // so stale content doesn't remain below the new frame.
    if overwrite {
        let leftover = last_frame_lines.saturating_sub(num_lines);
        for _ in 0..leftover {
            writeln!(buf, "\x1b[2K")?;
        }
        // After clearing leftover lines, move cursor back up to the end of the
        // actual frame content so the next overwrite starts from the right place.
        if leftover > 0 {
            write!(buf, "\x1b[{}A", leftover)?;
        }
    }

    // Single atomic write — terminal processes the entire frame at once.
    sink.write_all(&buf)?;
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

    /// Get the number of lines the last rendered frame occupied.
    /// Used to seed a subsequent writer for seamless transitions.
    pub fn last_frame_lines(&self) -> usize {
        match self {
            Self::Ansi(r) => r.last_frame_lines,
            Self::Plain(r) => r.last_frame_lines,
        }
    }

    /// Seed the writer with a known previous frame height so the first
    /// frame it renders will cursor-up over that many lines.
    pub fn seed_last_frame_lines(&mut self, lines: usize) {
        match self {
            Self::Ansi(r) => r.last_frame_lines = lines,
            Self::Plain(r) => r.last_frame_lines = lines,
        }
    }
}

// ---------------------------------------------------------------------------
// CursorGuard — RAII cursor hide/show
// ---------------------------------------------------------------------------

/// RAII guard that hides the terminal cursor on creation and restores it on drop.
///
/// This ensures the cursor is always restored, even if the progress render
/// loop panics. Matches `gunb.ai`'s cursor management pattern.
///
/// Only hides the cursor when `is_tty` is true. When not a TTY, this is a no-op.
pub struct CursorGuard {
    is_tty: bool,
}

impl CursorGuard {
    /// Create a new cursor guard. If `is_tty`, immediately hides the cursor.
    pub fn new(is_tty: bool) -> Self {
        if is_tty {
            // Hide cursor — write directly to stderr (progress renders to stderr)
            let _ = write!(std::io::stderr(), "{}", CURSOR_HIDE);
            let _ = std::io::stderr().flush();
        }
        Self { is_tty }
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        if self.is_tty {
            // Restore cursor — critical for terminal cleanup
            let _ = write!(std::io::stderr(), "{}", CURSOR_SHOW);
            let _ = std::io::stderr().flush();
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
    fn test_write_frame_tty_cursor_up_and_clear() {
        let mut buf = Vec::new();
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, true);

        // First frame
        let frame1 = make_frame(vec!["first"], CursorAction::Overwrite);
        writer.write_frame(&frame1, &mut buf).unwrap();

        // Second frame should cursor-up and per-line clear
        let frame2 = make_frame(vec!["second"], CursorAction::Overwrite);
        writer.write_frame(&frame2, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(
            output.contains("\x1b[1A\r"),
            "TTY should contain cursor-up: {output}"
        );
        assert!(
            output.contains("\x1b[2K"),
            "TTY should contain per-line clear: {output}"
        );
        assert!(
            !output.contains("\x1b[J"),
            "TTY should NOT contain bulk clear-to-end: {output}"
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
    fn test_write_frame_shorter_frame_clears_leftover_lines() {
        let mut buf = Vec::new();
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, true);

        // First frame: 3 lines
        let frame1 = make_frame(vec!["a", "b", "c"], CursorAction::Overwrite);
        writer.write_frame(&frame1, &mut buf).unwrap();

        // Second frame: 1 line — leftover lines are cleared individually
        let frame2 = make_frame(vec!["x"], CursorAction::Overwrite);
        writer.write_frame(&frame2, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        // Should cursor-up 3 lines (previous frame height)
        assert!(
            output.contains("\x1b[3A\r"),
            "Should cursor-up 3 lines: {output}"
        );
        // Should NOT use bulk clear-to-end
        assert!(
            !output.contains("\x1b[J"),
            "Should NOT use bulk clear: {output}"
        );
        // Per-line clears for content (1) + leftover (2) = at least 3 \x1b[2K
        let clear_count = output.matches("\x1b[2K").count();
        assert!(
            clear_count >= 3,
            "Expected at least 3 per-line clears (1 content + 2 leftover), got {clear_count}"
        );
        assert!(output.contains("x\n"));
    }
}
