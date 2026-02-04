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
use std::collections::{HashMap, HashSet};
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

/// Frame scheduling policy.
#[derive(Debug, Clone, Default)]
pub enum FramePolicy {
    /// Always show latest state. Skip intermediate frames.
    #[default]
    Latest,
    /// Show every state transition, queue if execution outpaces display.
    Sequential { max_queue: usize },
    /// Adaptive: slow for interesting transitions, catch up when idle.
    Adaptive { min_fps: u8, max_fps: u8 },
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
    pub fn tick(&mut self, dt: Duration) -> bool {
        if self.frames.is_empty() {
            return false;
        }
        self.elapsed += dt;
        if self.elapsed >= self.interval {
            self.elapsed -= self.interval;
            let old = self.current;
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
            self.current != old
        } else {
            false
        }
    }

    /// Get the current frame content.
    pub fn frame(&self) -> &str {
        self.frames.get(self.current).map(|s| s.as_str()).unwrap_or("")
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
/// Uses carriage-return + cursor-up for live updating when TTY is available,
/// falls back to append-only when piped.
pub struct TerminalRenderer<W: Write> {
    output: W,
    symbol_set: &'static SymbolSet,
    tier: Tier,
    mode: RenderMode,
    layout: DagLayout,
    /// Number of lines written in the last frame (for cursor-up).
    last_frame_lines: usize,
    /// Whether we're writing to a TTY (supports cursor movement).
    is_tty: bool,
    /// Spinner animation for running nodes.
    spinner: Animation,
}

impl<W: Write> TerminalRenderer<W> {
    /// Create a new terminal renderer.
    pub fn new(
        output: W,
        symbol_set: &'static SymbolSet,
        tier: Tier,
        layout: DagLayout,
    ) -> Self {
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
            is_tty: false,
            spinner: Animation::cycle(spinner_frames, Duration::from_millis(150)),
        }
    }

    /// Set whether we're writing to a TTY.
    pub fn set_tty(&mut self, is_tty: bool) {
        self.is_tty = is_tty;
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

    /// Resolve a colored symbol.
    fn colored_symbol(&self, id: SymbolId) -> String {
        self.symbol_set.get(id).resolve_colored(self.tier)
    }

    /// ANSI color code for a node state (for coloring boxes).
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

    /// Wrap text in ANSI bold + color for a node state.
    fn colored_box(&self, text: &str, state: NodeState) -> String {
        let color = self.state_color(state);
        // Bold for completed/failed/running, dim stays dim
        let bold = match state {
            NodeState::Completed | NodeState::Failed | NodeState::Running | NodeState::Intercepted => "\x1b[1m",
            _ => "",
        };
        format!("{}{}[{}]\x1b[0m", bold, color.ansi(), text)
    }

    /// Legend symbol for a node state: ✔, ✘, ◐, etc.
    fn legend_symbol(&self, state: NodeState) -> &'static str {
        match self.tier {
            Tier::Ascii => match state {
                NodeState::Completed | NodeState::Intercepted => "ok",
                NodeState::Failed => "FAIL",
                NodeState::Running => "...",
                NodeState::Skipped => "skip",
                NodeState::Pending => "-",
            },
            _ => match state {
                NodeState::Completed | NodeState::Intercepted => "\u{2714}", // ✔
                NodeState::Failed => "\u{2718}",                             // ✘
                NodeState::Running => "\u{25D0}",                            // ◐
                NodeState::Skipped => "\u{25CC}",                            // ◌
                NodeState::Pending => "\u{25CB}",                            // ○
            },
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
                let _ = write!(self.output, "\x1b[2K\n");
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
            .filter(|n| {
                matches!(
                    n.state,
                    NodeState::Completed | NodeState::Intercepted
                )
            })
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
        let (node_track, num_tracks) = self.assign_tracks(progress);

        // Build (track, column) → NodeId grid
        let mut grid: HashMap<(usize, usize), NodeId> = HashMap::new();
        for (col, level) in self.layout.levels.iter().enumerate() {
            for node in level {
                if let Some(&track) = node_track.get(node) {
                    grid.insert((track, col), node.clone());
                }
            }
        }

        // Build parent → children and child → parents adjacency
        let mut children_of: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut parents_of: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for edge in &progress.snapshot.edges {
            children_of
                .entry(edge.from_node.clone())
                .or_default()
                .push(edge.to_node.clone());
            parents_of
                .entry(edge.to_node.clone())
                .or_default()
                .push(edge.from_node.clone());
        }

        // Assign single-letter labels (A, B, C, ...) in topological order
        let mut node_letter: HashMap<NodeId, String> = HashMap::new();
        let mut letter_idx: usize = 0;
        for level in &self.layout.levels {
            let mut sorted = level.clone();
            sorted.sort_by_key(|n| node_track.get(n).copied().unwrap_or(0));
            for node in &sorted {
                let letter = letter_for_index(letter_idx);
                node_letter.insert(node.clone(), letter);
                letter_idx += 1;
            }
        }

        // Column dimensions: all boxes are uniform [X] = letter + 2 brackets
        let max_letter_w = node_letter.values().map(|l| l.len()).max().unwrap_or(1);
        let col_w = max_letter_w + 2; // [X] = 1 + letter + 1

        // Gap widths: 5 for fan-out or merge columns, 3 otherwise
        let mut gap_widths: Vec<usize> = vec![3; num_cols.saturating_sub(1)];
        for (col, gw) in gap_widths.iter_mut().enumerate() {
            for track in 0..num_tracks {
                // Fan-out: node at (track, col) has >1 children
                if let Some(node) = grid.get(&(track, col)) {
                    if children_of.get(node).map(|c| c.len()).unwrap_or(0) > 1 {
                        *gw = 5;
                        break;
                    }
                }
                // Merge: node at (track, col+1) has parents on different tracks
                if let Some(node) = grid.get(&(track, col + 1)) {
                    if let Some(pars) = parents_of.get(node) {
                        let par_tracks: HashSet<usize> = pars
                            .iter()
                            .filter_map(|p| node_track.get(p).copied())
                            .collect();
                        if par_tracks.len() > 1 {
                            *gw = 5;
                            break;
                        }
                    }
                }
            }
        }

        // Render each track as a horizontal line
        for track in 0..num_tracks {
            let mut line = String::new();
            #[allow(clippy::needless_range_loop)]
            for col in 0..num_cols {
                if let Some(node) = grid.get(&(track, col)) {
                    let state = progress
                        .nodes
                        .get(node)
                        .map(|np| np.state)
                        .unwrap_or(NodeState::Pending);
                    let letter = node_letter.get(node).map(|s| s.as_str()).unwrap_or("?");
                    line.push_str(&self.colored_box(letter, state));
                    let vis_w = letter.len() + 2; // [X]
                    let pad = col_w.saturating_sub(vis_w);
                    line.push_str(&" ".repeat(pad));
                } else {
                    line.push_str(&" ".repeat(col_w));
                }

                if col < num_cols - 1 {
                    let conn = self.horizontal_connector(
                        track,
                        col,
                        &grid,
                        &children_of,
                        &parents_of,
                        &node_track,
                        num_tracks,
                        gap_widths[col],
                    );
                    let edge_state = self.connector_state(
                        track, col, &grid, &children_of, &node_track, progress,
                    );
                    let colored = self.color_connector(&conn, edge_state);
                    line.push_str(&colored);
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
        let mut active_entries: Vec<(NodeState, &str, String, Option<Duration>)> = Vec::new();
        // Collect completed/intercepted entries in reverse topo order (most recent last)
        let mut done_entries: Vec<(NodeState, &str, String, Option<Duration>)> = Vec::new();

        for level in &self.layout.levels {
            for node in level {
                let np = progress.nodes.get(node);
                let state = np.map(|n| n.state).unwrap_or(NodeState::Pending);
                let letter = node_letter.get(node).map(|s| s.as_str()).unwrap_or("?");
                let label = full_label(node, &progress.snapshot.labels);
                let elapsed = np.and_then(|n| {
                    n.elapsed.or_else(|| n.start_time.map(|t| t.elapsed()))
                });

                match state {
                    NodeState::Running | NodeState::Failed => {
                        active_entries.push((state, letter, label, elapsed));
                    }
                    NodeState::Completed | NodeState::Intercepted => {
                        done_entries.push((state, letter, label, elapsed));
                    }
                    _ => {}
                }
            }
        }

        // Build final legend: active first, then fill remaining with most recent done
        let mut legend: Vec<(NodeState, &str, String, Option<Duration>)> = Vec::new();
        legend.extend(active_entries.into_iter().take(LEGEND_LINES));
        let remaining = LEGEND_LINES.saturating_sub(legend.len());
        if remaining > 0 {
            // Take the most recently completed (last in topo order)
            legend.extend(done_entries.into_iter().rev().take(remaining));
        }

        let visible = legend.len().min(LEGEND_LINES);
        for (state, letter, label, elapsed) in legend.iter().take(visible) {
            let sym = self.legend_symbol(*state);
            let color = self.state_color(*state);
            let time_str = elapsed
                .map(|d| format!(" [{}]", format_duration(d)))
                .unwrap_or_default();
            lines.push(format!(
                "  {}{} {}: {}{}\x1b[0m",
                color.ansi(), sym, letter, label, time_str
            ));
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
                format!("{} DAG pending", self.colored_symbol(SymbolId::DagNotStarted))
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
                        format!("{}{}\x1b[0m", SemanticColor::Success.ansi(), animal)
                    }
                    _ => self.colored_symbol(SymbolId::DagCompleted),
                };
                format!(
                    "{} Completed [{}]",
                    icon,
                    format_duration(*e)
                )
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
                        format!("{}\u{274C}\x1b[0m", SemanticColor::Error.ansi()) // ❌
                    }
                    _ => self.colored_symbol(SymbolId::DagFailed),
                };
                format!(
                    "{} Failed at {}: {} [{}]",
                    icon,
                    label,
                    error,
                    elapsed
                )
            }
        }
    }

    // ------------------------------------------------------------------
    // Horizontal layout helpers
    // ------------------------------------------------------------------

    /// Assign each node to a horizontal track (row in the output).
    ///
    /// Track 0 is the main path. When a node fans out, the first child stays
    /// on the parent's track; additional children get new tracks below.
    fn assign_tracks(
        &self,
        progress: &DagProgress,
    ) -> (HashMap<NodeId, usize>, usize) {
        let mut parents_of: HashMap<&NodeId, Vec<&NodeId>> = HashMap::new();
        for edge in &progress.snapshot.edges {
            parents_of
                .entry(&edge.to_node)
                .or_default()
                .push(&edge.from_node);
        }

        let mut node_track: HashMap<NodeId, usize> = HashMap::new();
        let mut next_track: usize = 0;

        for level in &self.layout.levels {
            let mut claimed = HashSet::new();

            // Sort: nodes whose parents are on lower tracks go first
            let mut sorted = level.clone();
            sorted.sort_by_key(|n| {
                parents_of
                    .get(n)
                    .and_then(|ps| {
                        ps.iter()
                            .filter_map(|p| node_track.get(*p))
                            .min()
                            .copied()
                    })
                    .unwrap_or(usize::MAX)
            });

            for node in &sorted {
                let inherited = parents_of.get(node).and_then(|parents| {
                    let mut tracks: Vec<usize> = parents
                        .iter()
                        .filter_map(|p| node_track.get(*p).copied())
                        .collect();
                    tracks.sort();
                    tracks.dedup();
                    tracks.into_iter().find(|t| !claimed.contains(t))
                });

                if let Some(track) = inherited {
                    node_track.insert(node.clone(), track);
                    claimed.insert(track);
                } else {
                    while claimed.contains(&next_track) {
                        next_track += 1;
                    }
                    node_track.insert(node.clone(), next_track);
                    claimed.insert(next_track);
                    next_track += 1;
                }
            }
        }

        let num_tracks = node_track
            .values()
            .max()
            .copied()
            .map(|m| m + 1)
            .unwrap_or(1);
        (node_track, num_tracks)
    }

    /// Compute the connector string between column `col` and `col+1` for a track.
    #[allow(clippy::too_many_arguments)]
    fn horizontal_connector(
        &self,
        track: usize,
        col: usize,
        grid: &HashMap<(usize, usize), NodeId>,
        children_of: &HashMap<NodeId, Vec<NodeId>>,
        parents_of: &HashMap<NodeId, Vec<NodeId>>,
        node_track: &HashMap<NodeId, usize>,
        num_tracks: usize,
        gap: usize,
    ) -> String {
        let has_here = grid.contains_key(&(track, col));
        let has_next = grid.contains_key(&(track, col + 1));

        // Check if the next node is a merge (has parents on multiple tracks)
        let next_is_merge = has_next && {
            let next_node = &grid[&(track, col + 1)];
            parents_of.get(next_node).map(|pars| {
                let par_tracks: HashSet<usize> = pars
                    .iter()
                    .filter_map(|p| node_track.get(p).copied())
                    .collect();
                par_tracks.len() > 1
            }).unwrap_or(false)
        };

        if has_here {
            let node = &grid[&(track, col)];
            let mut child_tracks: Vec<usize> = children_of
                .get(node)
                .map(|cs| cs.iter().filter_map(|c| node_track.get(c).copied()).collect())
                .unwrap_or_default();
            // Deduplicate: multi-port edges to the same child produce duplicate tracks
            child_tracks.sort();
            child_tracks.dedup();

            if child_tracks.is_empty() {
                " ".repeat(gap)
            } else if child_tracks.len() > 1 && !child_tracks.iter().all(|&t| t == track) {
                // Fan-out: children on multiple different tracks
                self.fanout_top_str(gap)
            } else if has_next || child_tracks.contains(&track) {
                // Straight through on same track (or next node on this track)
                if next_is_merge {
                    self.merge_top_str(gap)
                } else {
                    self.arrow_str(gap)
                }
            } else if child_tracks.iter().all(|&t| t < track) {
                // All children above — merge up ─┘
                self.merge_up_str(gap)
            } else {
                // Children below — merge down ─┬
                self.merge_down_str(gap)
            }
        } else if has_next {
            // No node here but node at next col — check if it's a merge target
            let next_node = &grid[&(track, col + 1)];

            // Is this a fan-out branch from a parent on another track?
            let is_branch = (0..num_tracks).any(|t| {
                t != track
                    && grid
                        .get(&(t, col))
                        .and_then(|p| children_of.get(p))
                        .map(|cs| cs.contains(next_node))
                        .unwrap_or(false)
            });

            // Is this a merge branch from a node below merging up?
            let is_merge_branch = parents_of.get(next_node).map(|pars| {
                pars.iter().any(|p| {
                    node_track.get(p).copied().unwrap_or(0) > track
                })
            }).unwrap_or(false);

            if is_branch {
                self.fanout_branch_str(
                    track, col, grid, children_of, node_track, num_tracks, gap,
                )
            } else if is_merge_branch && next_is_merge {
                // Merge branch entering from below
                self.merge_branch_str(
                    track, col, grid, parents_of, node_track, num_tracks, gap,
                )
            } else {
                " ".repeat(gap)
            }
        } else {
            // Empty cell — check for vertical pass-through
            if self.needs_vertical(track, col, grid, children_of, node_track) {
                self.vertical_str(gap)
            } else {
                " ".repeat(gap)
            }
        }
    }

    /// Check if a vertical pass-through │ is needed at (track, col gap).
    fn needs_vertical(
        &self,
        track: usize,
        col: usize,
        grid: &HashMap<(usize, usize), NodeId>,
        children_of: &HashMap<NodeId, Vec<NodeId>>,
        node_track: &HashMap<NodeId, usize>,
    ) -> bool {
        for src_track in 0..track {
            if let Some(parent) = grid.get(&(src_track, col)) {
                if let Some(children) = children_of.get(parent) {
                    let child_tracks: Vec<usize> = children
                        .iter()
                        .filter_map(|c| node_track.get(c).copied())
                        .collect();
                    if child_tracks.iter().any(|&t| t > track) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Arrow connector padded to `w`: ` → ` or ` > `.
    fn arrow_str(&self, w: usize) -> String {
        let s = match self.tier {
            Tier::Ascii => " > ",
            _ => " → ",
        };
        pad_connector(s, w)
    }

    /// Fan-out top: ` ─┬─ ` or ` -+- `.
    fn fanout_top_str(&self, w: usize) -> String {
        let s = match self.tier {
            Tier::Ascii => " -+- ",
            _ => " \u{2500}\u{252C}\u{2500} ", // ─┬─
        };
        pad_connector(s, w)
    }

    /// Fan-out branch: `  └─ ` or `  '- `, with `├` for middle branches.
    #[allow(clippy::too_many_arguments)]
    fn fanout_branch_str(
        &self,
        track: usize,
        col: usize,
        grid: &HashMap<(usize, usize), NodeId>,
        children_of: &HashMap<NodeId, Vec<NodeId>>,
        _node_track: &HashMap<NodeId, usize>,
        num_tracks: usize,
        w: usize,
    ) -> String {
        let has_more_below = (track + 1..num_tracks).any(|t| {
            grid.get(&(t, col + 1))
                .map(|next| {
                    (0..track).any(|st| {
                        grid.get(&(st, col))
                            .and_then(|p| children_of.get(p))
                            .map(|cs| cs.contains(next))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if has_more_below {
            match self.tier {
                Tier::Ascii => pad_connector("  |- ", w),
                _ => pad_connector("  \u{251C}\u{2500} ", w), // ├─
            }
        } else {
            match self.tier {
                Tier::Ascii => pad_connector("  '- ", w),
                _ => pad_connector("  \u{2514}\u{2500} ", w), // └─
            }
        }
    }

    /// Vertical pass-through: `  │  ` or `  |  `.
    fn vertical_str(&self, w: usize) -> String {
        match self.tier {
            Tier::Ascii => pad_connector("  |  ", w),
            _ => pad_connector("  \u{2502}  ", w), // │
        }
    }

    /// Merge-down connector (node's children are only on tracks below).
    fn merge_down_str(&self, w: usize) -> String {
        match self.tier {
            Tier::Ascii => pad_connector(" -+  ", w),
            _ => pad_connector(" \u{2500}\u{252C}  ", w), // ─┬
        }
    }

    /// Merge-up connector: node's child is on a track above. ` ─┘ `
    fn merge_up_str(&self, w: usize) -> String {
        match self.tier {
            Tier::Ascii => pad_connector(" -'  ", w),
            _ => pad_connector(" \u{2500}\u{2518}  ", w), // ─┘
        }
    }

    /// Merge-top connector: node on this track receives edges from below. ` ─┴─ `
    fn merge_top_str(&self, w: usize) -> String {
        match self.tier {
            Tier::Ascii => pad_connector(" -+- ", w),
            _ => pad_connector(" \u{2500}\u{2534}\u{2500} ", w), // ─┴─
        }
    }

    /// Merge-branch connector: lower track merges into node above. `  └─ ` or `  ├─ `
    #[allow(clippy::too_many_arguments)]
    fn merge_branch_str(
        &self,
        track: usize,
        _col: usize,
        _grid: &HashMap<(usize, usize), NodeId>,
        _parents_of: &HashMap<NodeId, Vec<NodeId>>,
        _node_track: &HashMap<NodeId, usize>,
        num_tracks: usize,
        w: usize,
    ) -> String {
        // Check if there are more merge branches below this one
        let is_last = track >= num_tracks - 1;
        if is_last {
            match self.tier {
                Tier::Ascii => pad_connector("  '- ", w),
                _ => pad_connector("  \u{2514}\u{2500} ", w), // └─
            }
        } else {
            match self.tier {
                Tier::Ascii => pad_connector("  '- ", w),
                _ => pad_connector("  \u{2514}\u{2500} ", w), // └─
            }
        }
    }

    /// Look up the edge state between two nodes from progress.
    fn connector_edge_state(
        &self,
        from: &NodeId,
        to: &NodeId,
        progress: &DagProgress,
    ) -> EdgeState {
        progress
            .edges
            .get(&(from.clone(), to.clone()))
            .map(|ep| ep.state)
            .unwrap_or(EdgeState::Idle)
    }

    /// Determine the dominant edge state for a connector between col and col+1 on a track.
    fn connector_state(
        &self,
        track: usize,
        col: usize,
        grid: &HashMap<(usize, usize), NodeId>,
        children_of: &HashMap<NodeId, Vec<NodeId>>,
        node_track: &HashMap<NodeId, usize>,
        progress: &DagProgress,
    ) -> EdgeState {
        // Find the source node at (track, col) or earlier
        if let Some(src) = grid.get(&(track, col)) {
            // Find target at (track, col+1) or find child on this track
            if let Some(dst) = grid.get(&(track, col + 1)) {
                return self.connector_edge_state(src, dst, progress);
            }
            // Check if src has children on other tracks
            if let Some(children) = children_of.get(src) {
                for child in children {
                    if let Some(&ct) = node_track.get(child) {
                        if ct == track {
                            return self.connector_edge_state(src, child, progress);
                        }
                    }
                }
                // Any child — use first
                if let Some(child) = children.first() {
                    return self.connector_edge_state(src, child, progress);
                }
            }
        }
        // Branch target — find which parent fans out to the node on col+1
        if let Some(dst) = grid.get(&(track, col + 1)) {
            // Find parent on an earlier track at this column
            for parent_track in 0..track + 1 {
                if let Some(parent) = grid.get(&(parent_track, col)) {
                    if let Some(children) = children_of.get(parent) {
                        if children.contains(dst) {
                            return self.connector_edge_state(parent, dst, progress);
                        }
                    }
                }
            }
        }
        EdgeState::Idle
    }

    /// Wrap a connector string in ANSI color based on edge state.
    fn color_connector(&self, s: &str, state: EdgeState) -> String {
        let color = match state {
            EdgeState::Idle => SemanticColor::Dim,
            EdgeState::Flowing => SemanticColor::Accent,
            EdgeState::Done => SemanticColor::Success,
            EdgeState::Dead => SemanticColor::Dim,
        };
        format!("{}{}\x1b[0m", color.ansi(), s)
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

        format!("{} — {}/{} [{}]", summary_parts.join(", "), completed + failed + skipped, total, elapsed)
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

/// Convert an index to a letter label: 0→A, 1→B, ..., 25→Z, 26→AA, etc.
fn letter_for_index(idx: usize) -> String {
    if idx < 26 {
        return String::from((b'A' + idx as u8) as char);
    }
    let mut result = String::new();
    let mut n = idx;
    loop {
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    result
}

/// Get the full label for a node (no truncation).
fn full_label(
    node_id: &NodeId,
    labels: &HashMap<NodeId, String>,
) -> String {
    labels
        .get(node_id)
        .map(|s| s.as_str())
        .unwrap_or(&node_id.0)
        .to_string()
}

/// Shorten a node label to fit within `max_width`.
#[cfg(test)]
fn short_label(
    node_id: &NodeId,
    labels: &HashMap<NodeId, String>,
    max_width: usize,
) -> String {
    let full = labels
        .get(node_id)
        .map(|s| s.as_str())
        .unwrap_or(&node_id.0);
    if full.len() <= max_width {
        return full.to_string();
    }
    // Strip common verb prefixes
    for prefix in &["prepare_", "execute_", "generate_", "parse_", "write_"] {
        if let Some(rest) = full.strip_prefix(prefix) {
            if rest.len() <= max_width {
                return rest.to_string();
            }
        }
    }
    // Truncate
    if max_width > 1 {
        format!("{}~", &full[..max_width - 1])
    } else {
        full[..max_width].to_string()
    }
}

/// Pad or truncate a connector string to exactly `w` display characters.
fn pad_connector(s: &str, w: usize) -> String {
    let display_w = s.chars().count();
    if display_w >= w {
        s.chars().take(w).collect()
    } else {
        let mut result = s.to_string();
        result.push_str(&" ".repeat(w - display_w));
        result
    }
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
        TerminalRenderer::new(buf, &STANDARD, Tier::Ascii, layout)
    }

    #[test]
    fn test_render_pending_state() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
        // Should contain letter-boxed nodes
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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

        let mut buf = Vec::new();
        {
            let mut renderer = make_renderer(&mut buf, layout);
            renderer.render(&progress);
        }

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // All letter-boxes on one DAG line (boxes contain [A], [B], [C] with ANSI wrapping)
        let has_all_on_one_line = lines.iter().any(|line| {
            line.contains("[A]") && line.contains("[B]") && line.contains("[C]")
        });
        assert!(
            has_all_on_one_line,
            "Linear chain should render all letter-boxes on one line, got:\n{}",
            output
        );
    }

    #[test]
    fn test_horizontal_fanout_two_tracks() {
        // A → B, A → C : should produce two tracks
        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("A"),
                NodeId::from("B"),
                NodeId::from("C"),
            ],
            edges: vec![
                Edge::new("A", "out", "B", "in"),
                Edge::new("A", "out", "C", "in"),
            ],
            topo_order: vec![
                NodeId::from("A"),
                NodeId::from("B"),
                NodeId::from("C"),
            ],
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
        let layout = compute_layout(
            &snap.topo_order,
            &snap.edges,
            &snap.labels,
            &vp,
        );

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
        assert!(b_line.is_some(), "B box should appear in output:\n{}", output);
        assert!(c_line.is_some(), "C box should appear in output:\n{}", output);
        assert_ne!(
            b_line.unwrap(),
            c_line.unwrap(),
            "Fan-out should put B and C boxes on different lines"
        );
    }

    #[test]
    fn test_short_label_truncation() {
        let labels: HashMap<NodeId, String> = [
            (NodeId::from("prepare_scan_workspace"), "prepare_scan_workspace".to_string()),
            (NodeId::from("short"), "short".to_string()),
        ]
        .into_iter()
        .collect();

        // Short label fits
        assert_eq!(
            short_label(&NodeId::from("short"), &labels, 10),
            "short"
        );

        // Long label with prefix strip: "prepare_" removed → "scan_workspace" (14 chars ≤ 15)
        let result = short_label(&NodeId::from("prepare_scan_workspace"), &labels, 15);
        assert_eq!(result, "scan_workspace");

        // Truncation with ~
        let result = short_label(&NodeId::from("prepare_scan_workspace"), &labels, 8);
        assert_eq!(result, "prepare~");
    }
}
