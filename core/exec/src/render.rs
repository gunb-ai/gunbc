//! Terminal renderer for DAG progress display.
//!
//! Consumes [`DagLayout`] + [`DagProgress`] + [`SymbolSet`] and writes
//! visual output to any `Write` target (stdout in production, `Vec<u8>` in tests).
//!
//! # Architecture
//!
//! ```text
//! DagProgress + DagLayout + SymbolSet  ──→  TerminalRenderer  ──→  Write
//! ```
//!
//! The renderer doesn't compute layout — it receives a [`DagLayout`] and paints
//! symbols into it. If the viewport changes, the caller provides a new layout.
//!
//! # Frame Loop
//!
//! The [`FrameLoop`] trait decouples execution events from display timing.
//! For synchronous execution, `update()` + `render()` are called after each
//! observer callback. For future concurrent execution, the frame loop
//! runs on its own thread/tick.

use crate::progress::{DagPhase, DagProgress, EdgeState, NodeState};
use gunbc_ir::layout::DagLayout;
use gunbc_ir::symbols::{SemanticColor, SymbolId, SymbolSet, Tier};
use gunbc_ir::NodeId;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

// ---------------------------------------------------------------------------
// FrameLoop trait
// ---------------------------------------------------------------------------

/// Game-engine style update→render loop.
///
/// Decouples execution events from display timing. The renderer
/// implements this trait; the caller drives the loop.
pub trait FrameLoop {
    /// Process pending state changes (from observer events).
    fn update(&mut self, progress: &DagProgress, dt: Duration);
    /// Produce a visual frame from current state.
    fn render(&mut self, progress: &DagProgress);
}

// ---------------------------------------------------------------------------
// RenderMode
// ---------------------------------------------------------------------------

/// Rendering style (orthogonal to symbol tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Symbol-per-state, clean layout.
    #[default]
    Standard,
    /// Animated effects (spinners, wavefronts).
    Dynamic,
    /// Single-line summary.
    Compact,
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Universal animation state. All animation types (spinners, wavefronts,
/// node morphs) share this structure.
#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<String>,
    pub current: usize,
    pub interval: Duration,
    pub elapsed: Duration,
    pub mode: AnimationMode,
}

/// How an animation loops.
#[derive(Debug, Clone)]
pub enum AnimationMode {
    /// Loop forever (spinners, idle automaton).
    Cycle,
    /// Play once then hold last frame (node morph).
    Once,
    /// Advance position along a path (edge wavefront).
    Propagate { path_len: usize, position: usize },
}

impl Animation {
    /// Create a new cycling animation (e.g., spinner).
    pub fn cycle(frames: Vec<String>, interval: Duration) -> Self {
        Self {
            frames,
            current: 0,
            interval,
            elapsed: Duration::ZERO,
            mode: AnimationMode::Cycle,
        }
    }

    /// Create a one-shot animation (e.g., node morph).
    pub fn once(frames: Vec<String>, interval: Duration) -> Self {
        Self {
            frames,
            current: 0,
            interval,
            elapsed: Duration::ZERO,
            mode: AnimationMode::Once,
        }
    }

    /// Advance the animation by dt. Returns true if frame changed.
    ///
    /// Handles long dt values correctly: if dt spans multiple intervals,
    /// all skipped frames are consumed (the animation catches up rather
    /// than getting stuck).
    pub fn tick(&mut self, dt: Duration) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        self.elapsed += dt;
        let old = self.current;
        while self.elapsed >= self.interval {
            self.elapsed -= self.interval;
            match &mut self.mode {
                AnimationMode::Cycle => {
                    self.current = (self.current + 1) % self.frames.len();
                }
                AnimationMode::Once => {
                    if self.current < self.frames.len() - 1 {
                        self.current += 1;
                    }
                }
                AnimationMode::Propagate { path_len, position } => {
                    if *position < *path_len {
                        *position += 1;
                    }
                    self.current = (self.current + 1) % self.frames.len();
                }
            }
        }
        self.current != old
    }

    /// Get the current frame content.
    pub fn frame(&self) -> &str {
        self.frames
            .get(self.current)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Check if the animation has finished (only meaningful for Once mode).
    pub fn is_done(&self) -> bool {
        match &self.mode {
            AnimationMode::Cycle => false,
            AnimationMode::Once => self.current >= self.frames.len().saturating_sub(1),
            AnimationMode::Propagate { path_len, position } => *position >= *path_len,
        }
    }
}

// ---------------------------------------------------------------------------
// TerminalRenderer
// ---------------------------------------------------------------------------

/// Terminal renderer that paints DAG progress using symbols and ANSI colors.
///
/// Writes to any `Write` target — stdout in production, `Vec<u8>` in tests.
/// Uses carriage-return + cursor-up for live updating on a TTY.
///
/// The renderer requires at least `Tier::Unicode` — it is never instantiated
/// for ASCII-only environments. Those environments get plain text output
/// via `run_classic()` instead.
pub struct TerminalRenderer<W: Write> {
    output: W,
    symbol_set: &'static SymbolSet,
    tier: Tier,
    mode: RenderMode,
    layout: DagLayout,
    /// Number of lines written in the last frame (for cursor-up).
    last_frame_lines: usize,
    /// Whether we're writing to a TTY (controls cursor movement).
    /// Always true in production; false in tests to capture raw output.
    is_tty: bool,
    /// Whether to emit ANSI color escape codes.
    /// False when `$NO_COLOR` is set or terminal doesn't support color.
    color_enabled: bool,
    /// Spinner animation for running nodes.
    spinner: Animation,
}

impl<W: Write> TerminalRenderer<W> {
    /// Create a new terminal renderer.
    ///
    /// - `is_tty`: controls cursor movement (overwrite previous frame).
    ///   Production callers pass `true`; tests pass `false` to capture output.
    /// - `color_enabled`: whether to emit ANSI color escape codes.
    ///   Pass `profile.supports_color` from [`TerminalProfile`].
    ///
    /// # Panics (debug)
    ///
    /// Panics if `tier` is `Tier::Ascii`. The renderer requires Unicode or
    /// better — ASCII environments should use plain text output instead.
    pub fn new(
        output: W,
        symbol_set: &'static SymbolSet,
        tier: Tier,
        layout: DagLayout,
        is_tty: bool,
        color_enabled: bool,
    ) -> Self {
        debug_assert!(
            !matches!(tier, Tier::Ascii),
            "TerminalRenderer requires Tier::Unicode or Tier::Emoji — \
             ASCII environments should use plain text output"
        );

        let spinner_frames: Vec<String> = [
            SymbolId::Spinner0,
            SymbolId::Spinner1,
            SymbolId::Spinner2,
            SymbolId::Spinner3,
        ]
        .iter()
        .map(|id| symbol_set.resolve_tier(*id, tier).to_string())
        .collect();

        Self {
            output,
            symbol_set,
            tier,
            mode: RenderMode::Standard,
            layout,
            last_frame_lines: 0,
            is_tty,
            color_enabled,
            spinner: Animation::cycle(spinner_frames, Duration::from_millis(150)),
        }
    }

    /// Set the render mode.
    pub fn set_mode(&mut self, mode: RenderMode) {
        self.mode = mode;
    }

    /// Update the layout (e.g., after terminal resize).
    pub fn set_layout(&mut self, layout: DagLayout) {
        self.layout = layout;
    }

    /// Resolve a symbol for the given state.
    fn symbol(&self, id: SymbolId) -> &str {
        self.symbol_set.resolve_tier(id, self.tier)
    }

    /// Resolve a colored symbol (respects `color_enabled`).
    fn colored_symbol(&self, id: SymbolId) -> String {
        let sym = self.symbol_set.get(id);
        let glyph = sym.resolve(self.tier);
        if self.color_enabled {
            format!("{}{}{}", sym.color.ansi(), glyph, self.reset())
        } else {
            glyph.to_string()
        }
    }

    /// ANSI color start code, or empty string if color is disabled.
    fn color(&self, c: SemanticColor) -> &'static str {
        if self.color_enabled {
            c.ansi()
        } else {
            ""
        }
    }

    /// ANSI reset code, or empty string if color is disabled.
    fn reset(&self) -> &'static str {
        if self.color_enabled {
            "\x1b[0m"
        } else {
            ""
        }
    }

    /// ANSI bold code, or empty string if color is disabled.
    fn bold(&self) -> &'static str {
        if self.color_enabled {
            "\x1b[1m"
        } else {
            ""
        }
    }

    /// Semantic color for a node state.
    fn state_color(&self, state: NodeState) -> SemanticColor {
        match state {
            NodeState::Pending => SemanticColor::Dim,
            NodeState::Running => SemanticColor::Active,
            NodeState::Completed => SemanticColor::Success,
            NodeState::Failed => SemanticColor::Error,
            NodeState::Skipped => SemanticColor::Dim,
            NodeState::Intercepted => SemanticColor::Success, // intercepted = done in dry-run
        }
    }

    /// Wrap text in styled box for a node state (respects `color_enabled`).
    fn colored_box(&self, text: &str, state: NodeState) -> String {
        let color = self.state_color(state);
        // Bold for completed/failed/running, dim stays dim
        let bold = match state {
            NodeState::Completed
            | NodeState::Failed
            | NodeState::Running
            | NodeState::Intercepted => self.bold(),
            _ => "",
        };
        format!("{}{}[{}]{}", bold, self.color(color), text, self.reset())
    }

    /// Legend symbol for a node state: ✔, ✘, ◐, etc.
    fn legend_symbol(&self, state: NodeState) -> &'static str {
        match state {
            NodeState::Completed | NodeState::Intercepted => "\u{2714}", // ✔
            NodeState::Failed => "\u{2718}",                             // ✘
            NodeState::Running => "\u{25D0}",                            // ◐
            NodeState::Skipped => "\u{25CC}",                            // ◌
            NodeState::Pending => "\u{25CB}",                            // ○
        }
    }

    /// Move cursor up to overwrite previous frame (TTY only).
    fn cursor_up(&mut self) {
        if self.is_tty && self.last_frame_lines > 0 {
            let _ = write!(self.output, "\x1b[{}A\r", self.last_frame_lines);
        }
    }

    /// Render a single frame of the progress display.
    fn render_frame(&mut self, progress: &DagProgress) {
        self.cursor_up();

        let mut lines: Vec<String> = Vec::new();

        match self.mode {
            RenderMode::Compact => {
                lines.push(self.render_compact(progress));
            }
            RenderMode::Standard | RenderMode::Dynamic => {
                self.render_standard(&mut lines, progress);
            }
        }

        // Write all lines (clear each line first when overwriting in TTY mode)
        for line in &lines {
            if self.is_tty {
                let _ = write!(self.output, "\x1b[2K"); // erase entire line
            }
            let _ = writeln!(self.output, "{}", line);
        }
        // If previous frame had more lines, clear the leftover lines
        if self.is_tty && lines.len() < self.last_frame_lines {
            for _ in 0..(self.last_frame_lines - lines.len()) {
                let _ = writeln!(self.output, "\x1b[2K");
            }
            // Move cursor back up past the blank lines we just wrote
            let extra = self.last_frame_lines - lines.len();
            let _ = write!(self.output, "\x1b[{}A", extra);
        }
        let _ = self.output.flush();
        self.last_frame_lines = lines.len();
    }

    /// Render compact mode: single-line summary.
    fn render_compact(&self, progress: &DagProgress) -> String {
        let total = progress.nodes.len();
        let completed = progress
            .nodes
            .values()
            .filter(|n| matches!(n.state, NodeState::Completed | NodeState::Intercepted))
            .count();
        let failed = progress
            .nodes
            .values()
            .filter(|n| n.state == NodeState::Failed)
            .count();
        let running = progress
            .nodes
            .values()
            .filter(|n| n.state == NodeState::Running)
            .count();

        let status_sym = match &progress.phase {
            DagPhase::NotStarted => self.symbol(SymbolId::DagNotStarted),
            DagPhase::Running { .. } => self.spinner.frame(),
            DagPhase::Completed { .. } => self.symbol(SymbolId::DagCompleted),
            DagPhase::Failed { .. } => self.symbol(SymbolId::DagFailed),
        };

        let elapsed_str = format_duration(progress.elapsed());

        if failed > 0 {
            format!(
                "{} {}/{} done, {} failed, {} running [{}]",
                status_sym, completed, total, failed, running, elapsed_str
            )
        } else {
            format!(
                "{} {}/{} done, {} running [{}]",
                status_sym, completed, total, running, elapsed_str
            )
        }
    }

    /// Render standard mode: horizontal left-to-right DAG flow.
    ///
    /// Each topological level maps to a column. Parallel nodes at the same
    /// level get separate tracks (rows). Fan-outs use ─┬─ / └─ connectors.
    fn render_standard(&self, lines: &mut Vec<String>, progress: &DagProgress) {
        lines.push(self.render_dag_header(progress));

        if self.layout.levels.is_empty() {
            return;
        }

        let num_cols = self.layout.levels.len();
        let num_tracks = self.layout.tracks;
        let col_w = self.layout.box_width as usize;
        let gap_w = self.layout.gap_width as usize;

        let visible_levels = &self.layout.overflow.visible_levels;
        let start_col = visible_levels.start;
        let end_col = visible_levels.end.min(num_cols);

        // Render each track as a horizontal line
        for track in 0..num_tracks {
            let mut line = String::new();
            let last_col = end_col.saturating_sub(1);

            for (col, connector_row) in self
                .layout
                .connectors
                .iter()
                .enumerate()
                .take(last_col)
                .skip(start_col)
            {
                if let Some(node) = self.layout.grid.get(&(track, col)) {
                    let state = progress
                        .nodes
                        .get(node)
                        .map(|np| np.state)
                        .unwrap_or(NodeState::Pending);
                    let label = self
                        .layout
                        .node_letters
                        .get(node)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    line.push_str(&self.colored_box(label, state));
                    let vis_w = display_width(label) + 2; // [label]
                    let pad = col_w.saturating_sub(vis_w);
                    line.push_str(&" ".repeat(pad));
                } else {
                    line.push_str(&" ".repeat(col_w));
                }

                let cell = connector_row.get(track);
                let glyph = cell.map(|c| c.glyph).unwrap_or(' ');
                let conn = pad_connector(&format!(" {} ", glyph), gap_w);
                let edge_state = cell
                    .map(|c| self.connector_cell_state(&c.edges, progress))
                    .unwrap_or(EdgeState::Idle);
                let colored = self.color_connector(&conn, edge_state);
                line.push_str(&colored);
            }

            if end_col > start_col {
                let col = last_col;
                if let Some(node) = self.layout.grid.get(&(track, col)) {
                    let state = progress
                        .nodes
                        .get(node)
                        .map(|np| np.state)
                        .unwrap_or(NodeState::Pending);
                    let label = self
                        .layout
                        .node_letters
                        .get(node)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    line.push_str(&self.colored_box(label, state));
                    let vis_w = display_width(label) + 2; // [label]
                    let pad = col_w.saturating_sub(vis_w);
                    line.push_str(&" ".repeat(pad));
                } else {
                    line.push_str(&" ".repeat(col_w));
                }
            }

            let trimmed = line.trim_end().to_string();
            if !trimmed.is_empty() {
                lines.push(format!("  {}", trimmed));
            }
        }

        // Legend: fixed-height area showing the 3 most relevant tasks with timing.
        // Priority: running/failed first, then most recently completed (reverse topo).
        // Always reserves LEGEND_LINES to prevent jitter.
        const LEGEND_LINES: usize = 3;

        // Collect running/failed entries (highest priority)
        let mut active_entries: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();
        // Collect completed/intercepted entries in reverse topo order (most recent last)
        let mut done_entries: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();

        for level in &self.layout.levels {
            for node in level {
                let np = progress.nodes.get(node);
                let state = np.map(|n| n.state).unwrap_or(NodeState::Pending);
                let short = self
                    .layout
                    .node_letters
                    .get(node)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string());
                let label = full_label(node, &progress.snapshot.labels);
                let elapsed =
                    np.and_then(|n| n.elapsed.or_else(|| n.start_time.map(|t| t.elapsed())));

                match state {
                    NodeState::Running | NodeState::Failed => {
                        active_entries.push((state, short, label, elapsed));
                    }
                    NodeState::Completed | NodeState::Intercepted => {
                        done_entries.push((state, short, label, elapsed));
                    }
                    _ => {}
                }
            }
        }

        // Build final legend: active first, then fill remaining with most recent done
        let mut legend: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();
        legend.extend(active_entries.into_iter().take(LEGEND_LINES));
        let remaining = LEGEND_LINES.saturating_sub(legend.len());
        if remaining > 0 {
            // Take the most recently completed (last in topo order)
            legend.extend(done_entries.into_iter().rev().take(remaining));
        }

        let visible = legend.len().min(LEGEND_LINES);
        for (state, short, label, elapsed) in legend.iter().take(visible) {
            let sym = self.legend_symbol(*state);
            let color = self.state_color(*state);
            let time_str = elapsed
                .map(|d| format!(" [{}]", format_duration(d)))
                .unwrap_or_default();
            if short == label {
                lines.push(format!(
                    "  {}{} {}{}{}",
                    self.color(color),
                    sym,
                    label,
                    time_str,
                    self.reset()
                ));
            } else {
                lines.push(format!(
                    "  {}{} {}: {}{}{}",
                    self.color(color),
                    sym,
                    short,
                    label,
                    time_str,
                    self.reset()
                ));
            }
        }
        // Pad remaining legend slots with empty lines to keep frame height stable
        for _ in visible..LEGEND_LINES {
            lines.push(String::new());
        }

        // Footer
        if matches!(
            progress.phase,
            DagPhase::Completed { .. } | DagPhase::Failed { .. }
        ) {
            lines.push(self.render_footer(progress));
        }
    }

    /// Render the DAG header line.
    fn render_dag_header(&self, progress: &DagProgress) -> String {
        let elapsed = format_duration(progress.elapsed());
        match &progress.phase {
            DagPhase::NotStarted => {
                format!(
                    "{} DAG pending",
                    self.colored_symbol(SymbolId::DagNotStarted)
                )
            }
            DagPhase::Running { current_node } => {
                let label = progress
                    .snapshot
                    .labels
                    .get(current_node)
                    .map(|s| s.as_str())
                    .unwrap_or(&current_node.0);
                format!(
                    "{} Running: {} [{}]",
                    self.colored_symbol(SymbolId::DagRunning),
                    label,
                    elapsed
                )
            }
            DagPhase::Completed { elapsed: e } => {
                let icon = match self.tier {
                    Tier::Emoji => {
                        let animal = random_animal(*e);
                        format!(
                            "{}{}{}",
                            self.color(SemanticColor::Success),
                            animal,
                            self.reset()
                        )
                    }
                    _ => self.colored_symbol(SymbolId::DagCompleted),
                };
                format!("{} Completed [{}]", icon, format_duration(*e))
            }
            DagPhase::Failed { node, error } => {
                let label = progress
                    .snapshot
                    .labels
                    .get(node)
                    .map(|s| s.as_str())
                    .unwrap_or(&node.0);
                let icon = match self.tier {
                    Tier::Emoji => {
                        format!(
                            "{}\u{274C}{}",
                            self.color(SemanticColor::Error),
                            self.reset()
                        ) // ❌
                    }
                    _ => self.colored_symbol(SymbolId::DagFailed),
                };
                format!("{} Failed at {}: {} [{}]", icon, label, error, elapsed)
            }
        }
    }

    // ------------------------------------------------------------------
    // Connector helpers
    // ------------------------------------------------------------------

    /// Determine the dominant edge state for a connector cell.
    fn connector_cell_state(
        &self,
        edges: &[(NodeId, NodeId)],
        progress: &DagProgress,
    ) -> EdgeState {
        let mut has_done = false;
        let mut has_dead = false;
        for (from, to) in edges {
            if let Some(ep) = progress.edges.get(&(from.clone(), to.clone())) {
                match ep.state {
                    EdgeState::Flowing => return EdgeState::Flowing,
                    EdgeState::Dead => has_dead = true,
                    EdgeState::Done => has_done = true,
                    EdgeState::Idle => {}
                }
            }
        }
        if has_dead {
            EdgeState::Dead
        } else if has_done {
            EdgeState::Done
        } else {
            EdgeState::Idle
        }
    }

    /// Wrap a connector string in ANSI color based on edge state (respects `color_enabled`).
    fn color_connector(&self, s: &str, state: EdgeState) -> String {
        let color = match state {
            EdgeState::Idle => SemanticColor::Dim,
            EdgeState::Flowing => SemanticColor::Accent,
            EdgeState::Done => SemanticColor::Success,
            EdgeState::Dead => SemanticColor::Dim,
        };
        format!("{}{}{}", self.color(color), s, self.reset())
    }

    /// Render the footer summary.
    fn render_footer(&self, progress: &DagProgress) -> String {
        let total = progress.nodes.len();
        let completed = progress
            .nodes
            .values()
            .filter(|n| matches!(n.state, NodeState::Completed | NodeState::Intercepted))
            .count();
        let failed = progress
            .nodes
            .values()
            .filter(|n| n.state == NodeState::Failed)
            .count();
        let skipped = progress
            .nodes
            .values()
            .filter(|n| n.state == NodeState::Skipped)
            .count();

        let mut summary_parts = Vec::new();
        if completed > 0 {
            summary_parts.push(format!(
                "{} {} completed",
                self.symbol(SymbolId::Success),
                completed
            ));
        }
        if failed > 0 {
            summary_parts.push(format!(
                "{} {} failed",
                self.symbol(SymbolId::Failure),
                failed
            ));
        }
        if skipped > 0 {
            summary_parts.push(format!("{} skipped", skipped));
        }

        let elapsed = match &progress.phase {
            DagPhase::Completed { elapsed } => format_duration(*elapsed),
            _ => format_duration(progress.elapsed()),
        };

        format!(
            "{} — {}/{} [{}]",
            summary_parts.join(", "),
            completed + failed + skipped,
            total,
            elapsed
        )
    }
}

impl<W: Write> FrameLoop for TerminalRenderer<W> {
    fn update(&mut self, _progress: &DagProgress, dt: Duration) {
        self.spinner.tick(dt);
    }

    fn render(&mut self, progress: &DagProgress) {
        self.render_frame(progress);
    }
}

// ---------------------------------------------------------------------------
// Animal emojis for success
// ---------------------------------------------------------------------------

/// Animal emojis picked at random for successful DAG completion (Emoji tier only).
const ANIMAL_EMOJIS: &[&str] = &[
    "\u{1F43B}", // 🐻
    "\u{1F427}", // 🐧
    "\u{1F436}", // 🐶
    "\u{1F431}", // 🐱
    "\u{1F98A}", // 🦊
    "\u{1F43C}", // 🐼
    "\u{1F428}", // 🐨
    "\u{1F42F}", // 🐯
    "\u{1F981}", // 🦁
    "\u{1F438}", // 🐸
    "\u{1F422}", // 🐢
    "\u{1F98B}", // 🦋
    "\u{1F41D}", // 🐝
    "\u{1F433}", // 🐳
    "\u{1F99C}", // 🦜
    "\u{1F984}", // 🦄
];

/// Pick a pseudo-random animal emoji using elapsed millis as seed.
fn random_animal(elapsed: Duration) -> &'static str {
    let idx = (elapsed.as_millis() as usize) % ANIMAL_EMOJIS.len();
    ANIMAL_EMOJIS[idx]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a duration for display.
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let ms = d.subsec_millis();
    if secs == 0 {
        format!("{}ms", ms)
    } else if secs < 60 {
        format!("{}.{:01}s", secs, ms / 100)
    } else {
        let mins = secs / 60;
        let rem_secs = secs % 60;
        format!("{}m{:02}s", mins, rem_secs)
    }
}

/// Get the full label for a node (no truncation).
fn full_label(node_id: &NodeId, labels: &HashMap<NodeId, String>) -> String {
    labels
        .get(node_id)
        .map(|s| s.as_str())
        .unwrap_or(&node_id.0)
        .to_string()
}

/// Pad or truncate a connector string to exactly `w` display characters.
fn pad_connector(s: &str, w: usize) -> String {
    let display_w = display_width(s);
    if display_w >= w {
        s.chars().take(w).collect()
    } else {
        let mut result = s.to_string();
        result.push_str(&" ".repeat(w - display_w));
        result
    }
}

/// Compute the display width of a string in terminal columns.
///
/// Uses proper Unicode width rules: ASCII characters are 1 column,
/// CJK/fullwidth characters are 2 columns, combining marks are 0 columns.
/// This is correct for terminal column alignment — unlike `str::len()`
/// (bytes) or `str::chars().count()` (codepoints), which don't account
/// for display width.
fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Approximate terminal display width of a single character.
///
/// Returns 2 for CJK/fullwidth characters, 0 for combining marks
/// and zero-width characters, 1 for everything else.
fn char_width(c: char) -> usize {
    let cp = c as u32;
    // Zero-width: combining marks, zero-width joiners, variation selectors
    if (0x0300..=0x036F).contains(&cp)      // Combining Diacritical Marks
        || (0x1AB0..=0x1AFF).contains(&cp)  // Combining Diacritical Marks Extended
        || (0x1DC0..=0x1DFF).contains(&cp)  // Combining Diacritical Marks Supplement
        || (0x20D0..=0x20FF).contains(&cp)  // Combining Diacritical Marks for Symbols
        || (0xFE00..=0xFE0F).contains(&cp)  // Variation Selectors
        || (0xFE20..=0xFE2F).contains(&cp)  // Combining Half Marks
        || cp == 0x200B                      // Zero Width Space
        || cp == 0x200C                      // Zero Width Non-Joiner
        || cp == 0x200D                      // Zero Width Joiner
        || cp == 0xFEFF
    // Zero Width No-Break Space (BOM)
    {
        return 0;
    }
    // Fullwidth / CJK: most CJK unified ideographs and fullwidth forms
    if (0x1100..=0x115F).contains(&cp)      // Hangul Jamo
        || (0x2E80..=0x303E).contains(&cp)  // CJK Radicals, Kangxi, CJK Symbols
        || (0x3041..=0x33BF).contains(&cp)  // Hiragana, Katakana, Bopomofo, CJK Compat
        || (0x3400..=0x4DBF).contains(&cp)  // CJK Unified Ideographs Extension A
        || (0x4E00..=0x9FFF).contains(&cp)  // CJK Unified Ideographs
        || (0xA000..=0xA4CF).contains(&cp)  // Yi Syllables + Radicals
        || (0xAC00..=0xD7AF).contains(&cp)  // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&cp)  // CJK Compatibility Ideographs
        || (0xFE30..=0xFE6F).contains(&cp)  // CJK Compatibility Forms + Small Form Variants
        || (0xFF01..=0xFF60).contains(&cp)  // Fullwidth ASCII
        || (0xFFE0..=0xFFE6).contains(&cp)  // Fullwidth Signs
        || (0x20000..=0x2FFFF).contains(&cp) // CJK Unified Ideographs Extension B+
        || (0x30000..=0x3FFFF).contains(&cp)
    // CJK Unified Ideographs Extension G+
    {
        return 2;
    }
    1
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::{DagSnapshot, OutputSummary, ProgressObserver};
    use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};
    use gunbc_ir::symbols::STANDARD;
    use gunbc_ir::{Edge, NodeId};

    fn test_snapshot() -> DagSnapshot {
        DagSnapshot {
            node_ids: vec![
                NodeId::from("lint"),
                NodeId::from("build"),
                NodeId::from("test"),
            ],
            edges: vec![
                Edge::new("lint", "out", "build", "in"),
                Edge::new("build", "out", "test", "in"),
            ],
            topo_order: vec![
                NodeId::from("lint"),
                NodeId::from("build"),
                NodeId::from("test"),
            ],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("lint"), "lint".to_string()),
                (NodeId::from("build"), "build".to_string()),
                (NodeId::from("test"), "test".to_string()),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn empty_summary() -> OutputSummary {
        OutputSummary {
            fields: vec![],
            elapsed: Duration::from_millis(50),
        }
    }

    fn make_renderer(buf: &mut Vec<u8>, layout: DagLayout) -> TerminalRenderer<&mut Vec<u8>> {
        TerminalRenderer::new(buf, &STANDARD, Tier::Unicode, layout, false, false)
    }

    #[test]
    fn test_render_pending_state() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
        // Should contain lettered nodes
        assert!(output.contains("[A]"), "Missing [A]\n{}", output);
        assert!(output.contains("[B]"), "Missing [B]\n{}", output);
        assert!(output.contains("[C]"), "Missing [C]\n{}", output);
    }

    #[test]
    fn test_render_running_state() {
        let snap = test_snapshot();
        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Running"));
        assert!(output.contains("lint"));
    }

    #[test]
    fn test_render_completed_state() {
        let snap = test_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));
        progress.on_node_complete(&NodeId::from("lint"), empty_summary());
        progress.on_node_start(&NodeId::from("build"));
        progress.on_node_complete(&NodeId::from("build"), empty_summary());
        progress.on_node_start(&NodeId::from("test"));
        progress.on_node_complete(&NodeId::from("test"), empty_summary());
        progress.on_dag_complete(Duration::from_millis(200));

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Completed"));
    }

    #[test]
    fn test_render_failed_state() {
        let snap = test_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));
        progress.on_node_failed(&NodeId::from("lint"), "clippy error");
        progress.on_dag_complete(Duration::from_millis(100));

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Failed"));
        assert!(output.contains("clippy error"));
    }

    #[test]
    fn test_render_compact_mode() {
        let snap = test_snapshot();
        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));
        progress.on_node_complete(&NodeId::from("lint"), empty_summary());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.set_mode(RenderMode::Compact);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        // Compact mode: single line with counts
        assert!(output.contains("1/3 done"));
    }

    #[test]
    fn test_animation_cycle() {
        let mut anim = Animation::cycle(
            vec!["a".into(), "b".into(), "c".into()],
            Duration::from_millis(100),
        );

        assert_eq!(anim.frame(), "a");
        assert!(!anim.is_done());

        anim.tick(Duration::from_millis(100));
        assert_eq!(anim.frame(), "b");

        anim.tick(Duration::from_millis(100));
        assert_eq!(anim.frame(), "c");

        anim.tick(Duration::from_millis(100));
        assert_eq!(anim.frame(), "a"); // Wraps around
    }

    #[test]
    fn test_animation_once() {
        let mut anim = Animation::once(
            vec!["x".into(), "y".into(), "z".into()],
            Duration::from_millis(50),
        );

        assert_eq!(anim.frame(), "x");
        assert!(!anim.is_done());

        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.frame(), "y");

        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.frame(), "z");
        assert!(anim.is_done());

        // Stays on last frame
        anim.tick(Duration::from_millis(50));
        assert_eq!(anim.frame(), "z");
        assert!(anim.is_done());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.5s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m05s");
    }

    #[test]
    fn test_diamond_render() {
        // A → B, A → C, B → D, C → D (diamond)
        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("A"),
                NodeId::from("B"),
                NodeId::from("C"),
                NodeId::from("D"),
            ],
            edges: vec![
                Edge::new("A", "out", "B", "in"),
                Edge::new("A", "out", "C", "in"),
                Edge::new("B", "out", "D", "in"),
                Edge::new("C", "out", "D", "in"),
            ],
            topo_order: vec![
                NodeId::from("A"),
                NodeId::from("B"),
                NodeId::from("C"),
                NodeId::from("D"),
            ],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("A"), "A".to_string()),
                (NodeId::from("B"), "B".to_string()),
                (NodeId::from("C"), "C".to_string()),
                (NodeId::from("D"), "D".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("A"));
        progress.on_node_complete(&NodeId::from("A"), empty_summary());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        // Should contain all node labels
        assert!(output.contains("A"));
        assert!(output.contains("B"));
        assert!(output.contains("C"));
        assert!(output.contains("D"));
    }

    #[test]
    fn test_horizontal_linear_single_line() {
        // Linear chain: all nodes should appear on the same output line
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // All labeled boxes on one DAG line (boxes contain [A], [B], [C] with ANSI wrapping)
        let has_all_on_one_line = lines
            .iter()
            .any(|line| line.contains("[A]") && line.contains("[B]") && line.contains("[C]"));
        assert!(
            has_all_on_one_line,
            "Linear chain should render all labeled boxes on one line, got:\n{}",
            output
        );
    }

    #[test]
    fn test_horizontal_fanout_two_tracks() {
        // A → B, A → C : should produce two tracks
        let snap = DagSnapshot {
            node_ids: vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")],
            edges: vec![
                Edge::new("A", "out", "B", "in"),
                Edge::new("A", "out", "C", "in"),
            ],
            topo_order: vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("A"), "A".to_string()),
                (NodeId::from("B"), "B".to_string()),
                (NodeId::from("C"), "C".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("A"));
        progress.on_node_complete(&NodeId::from("A"), empty_summary());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        // With letter labels: A is first node, B and C are fan-out targets
        // B and C should be on different DAG lines (look for [B] and [C] with ANSI)
        let b_line = output.lines().find(|l| l.contains("[B]"));
        let c_line = output.lines().find(|l| l.contains("[C]"));
        assert!(
            b_line.is_some(),
            "B box should appear in output:\n{}",
            output
        );
        assert!(
            c_line.is_some(),
            "C box should appear in output:\n{}",
            output
        );
        assert_ne!(
            b_line.unwrap(),
            c_line.unwrap(),
            "Fan-out should put B and C boxes on different lines"
        );
    }

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("[A]"), 3);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_display_width_unicode() {
        // Box-drawing characters are 1 column each
        assert_eq!(display_width("─┬─"), 3);
        assert_eq!(display_width("└─"), 2);
        assert_eq!(display_width("│"), 1);
    }

    #[test]
    fn test_display_width_cjk() {
        // CJK characters are 2 columns each
        assert_eq!(display_width("漢字"), 4);
        assert_eq!(display_width("A漢B"), 4); // A=1 + 漢=2 + B=1
    }

    #[test]
    fn test_display_width_combining() {
        // Combining marks are 0 width
        assert_eq!(display_width("e\u{0301}"), 1); // é (e + combining acute)
    }

    #[test]
    fn test_animation_tick_catches_up() {
        let frames = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let mut anim = Animation::cycle(frames, Duration::from_millis(100));
        assert_eq!(anim.frame(), "a");

        // Skip 350ms worth of time — should advance 3 frames (a→b→c→d)
        let changed = anim.tick(Duration::from_millis(350));
        assert!(changed);
        assert_eq!(anim.frame(), "d");
    }
}
