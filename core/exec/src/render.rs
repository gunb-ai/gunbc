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
use gunbc_ir::layout::{DagLayout, EdgeOrientation};
use gunbc_ir::symbols::{SemanticColor, SymbolId, SymbolSet, Tier};
use gunbc_ir::NodeId;
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

    /// Get the symbol ID for a node state.
    fn node_symbol_id(&self, state: NodeState) -> SymbolId {
        match state {
            NodeState::Pending => SymbolId::NodePending,
            NodeState::Running => SymbolId::NodeRunning,
            NodeState::Completed => SymbolId::NodeCompleted,
            NodeState::Failed => SymbolId::NodeFailed,
            NodeState::Skipped => SymbolId::NodeSkipped,
            NodeState::Intercepted => SymbolId::NodeIntercepted,
        }
    }

    /// Get the symbol ID for an edge state.
    fn edge_symbol_id(&self, state: EdgeState) -> SymbolId {
        match state {
            EdgeState::Idle => SymbolId::EdgeIdle,
            EdgeState::Flowing => SymbolId::EdgeFlowing,
            EdgeState::Done => SymbolId::EdgeDone,
            EdgeState::Dead => SymbolId::EdgeDead,
        }
    }

    /// Get the semantic color for a node state.
    fn node_color(&self, state: NodeState) -> SemanticColor {
        match state {
            NodeState::Pending => SemanticColor::Dim,
            NodeState::Running => SemanticColor::Active,
            NodeState::Completed => SemanticColor::Success,
            NodeState::Failed => SemanticColor::Error,
            NodeState::Skipped => SemanticColor::Dim,
            NodeState::Intercepted => SemanticColor::Info,
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

        // Write all lines
        for line in &lines {
            let _ = writeln!(self.output, "{}", line);
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

    /// Render standard mode: one line per level, nodes and edges.
    fn render_standard(&self, lines: &mut Vec<String>, progress: &DagProgress) {
        // Header: DAG status
        let header = self.render_dag_header(progress);
        lines.push(header);

        // Render each level
        for (level_idx, level_nodes) in self.layout.levels.iter().enumerate() {
            // Node line
            let node_line = self.render_level_nodes(level_nodes, progress);
            lines.push(node_line);

            // Edge line (between this level and next, if not last)
            if level_idx < self.layout.levels.len() - 1 {
                let edge_line = self.render_level_edges(level_nodes, progress);
                if !edge_line.trim().is_empty() {
                    lines.push(edge_line);
                }
            }
        }

        // Footer: summary
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
                format!(
                    "{} Completed [{}]",
                    self.colored_symbol(SymbolId::DagCompleted),
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
                format!(
                    "{} Failed at {}: {} [{}]",
                    self.colored_symbol(SymbolId::DagFailed),
                    label,
                    error,
                    elapsed
                )
            }
        }
    }

    /// Render nodes at a single level.
    fn render_level_nodes(&self, level_nodes: &[NodeId], progress: &DagProgress) -> String {
        let mut parts: Vec<String> = Vec::new();

        for node_id in level_nodes {
            let node_layout = match self.layout.nodes.get(node_id) {
                Some(nl) => nl,
                None => continue,
            };

            if node_layout.is_collapsed {
                continue;
            }

            let state = progress
                .nodes
                .get(node_id)
                .map(|np| np.state)
                .unwrap_or(NodeState::Pending);

            let sym_id = self.node_symbol_id(state);
            let sym = self.colored_symbol(sym_id);
            let label = progress
                .snapshot
                .labels
                .get(node_id)
                .map(|s| s.as_str())
                .unwrap_or(&node_id.0);

            // Add elapsed time for completed/failed nodes
            let elapsed_str = progress
                .nodes
                .get(node_id)
                .and_then(|np| np.elapsed)
                .map(|d| format!(" ({})", format_duration(d)))
                .unwrap_or_default();

            let color = self.node_color(state);
            let label_colored = format!("{}{}{}", color.ansi(), label, SemanticColor::reset());

            parts.push(format!("  {} {}{}", sym, label_colored, elapsed_str));
        }

        // Handle collapsed nodes
        let collapsed_count = level_nodes
            .iter()
            .filter(|id| {
                self.layout
                    .nodes
                    .get(*id)
                    .map(|nl| nl.is_collapsed)
                    .unwrap_or(false)
            })
            .count();
        if collapsed_count > 0 {
            parts.push(format!("  ... +{} more", collapsed_count));
        }

        parts.join("  ")
    }

    /// Render edges between levels.
    fn render_level_edges(&self, level_nodes: &[NodeId], progress: &DagProgress) -> String {
        let mut parts: Vec<String> = Vec::new();

        for node_id in level_nodes {
            let outgoing: Vec<&gunbc_ir::layout::EdgeLayout> = self.layout.edges_from(node_id);
            if outgoing.is_empty() {
                continue;
            }

            for edge_layout in &outgoing {
                let edge_state = progress
                    .edges
                    .get(&(edge_layout.from.clone(), edge_layout.to.clone()))
                    .map(|ep| ep.state)
                    .unwrap_or(EdgeState::Idle);

                let connector = match edge_layout.orientation {
                    EdgeOrientation::Vertical => {
                        self.symbol(SymbolId::ConnectorVertical)
                    }
                    EdgeOrientation::Horizontal => {
                        self.symbol(SymbolId::ConnectorHorizontal)
                    }
                    EdgeOrientation::Bend => {
                        self.symbol(SymbolId::ConnectorCornerBottomLeft)
                    }
                };

                let edge_sym = self.symbol(self.edge_symbol_id(edge_state));
                parts.push(format!("  {} {}", connector, edge_sym));
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("")
        }
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
        // Should contain node labels
        assert!(output.contains("lint"));
        assert!(output.contains("build"));
        assert!(output.contains("test"));
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
}
