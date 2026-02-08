//! Render types — animation and render mode.
//!
//! # Architecture
//!
//! ```text
//! DagProgress + DagLayout  →  build_frame() [pure]  →  Frame IR
//! Frame IR  →  FrameWriter [I/O]  →  Write
//! ```
//!
//! Frame **building** (pure, returns `Frame` IR) is in [`frame_build`](super::frame_build).
//! Frame **writing** (I/O, handles cursor control) is in [`frame_write`](super::frame_write).
//! This module provides [`Animation`] for spinner state and [`RenderMode`] for
//! selecting between standard/compact/dynamic display.

use std::time::Duration;

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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_build::{build_frame, display_width, format_duration};
    use crate::frame_write::FrameWriter;
    use crate::progress::{DagProgress, DagSnapshot, OutputSummary, ProgressObserver};
    use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};
    use gunbc_ir::symbols::{Tier, STANDARD};
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

    /// Build a frame and write it to a buffer using PlainText medium.
    fn render_to_string(progress: &DagProgress, layout: &gunbc_ir::layout::DagLayout) -> String {
        render_to_string_with_mode(progress, layout, RenderMode::Standard)
    }

    fn render_to_string_with_mode(
        progress: &DagProgress,
        layout: &gunbc_ir::layout::DagLayout,
        mode: RenderMode,
    ) -> String {
        let frame = build_frame(progress, layout, mode, "◐", Tier::Unicode, &STANDARD);
        let mut buf = Vec::new();
        let mut writer = FrameWriter::new(false, Tier::Unicode, &STANDARD, false);
        writer.write_frame(&frame, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_render_pending_state() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let output = render_to_string(&progress, &layout);
        assert!(!output.is_empty());
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

        let output = render_to_string(&progress, &layout);
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

        let output = render_to_string(&progress, &layout);
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

        let output = render_to_string(&progress, &layout);
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

        let output = render_to_string_with_mode(&progress, &layout, RenderMode::Compact);
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

        let output = render_to_string(&progress, &layout);
        assert!(output.contains("A"));
        assert!(output.contains("B"));
        assert!(output.contains("C"));
        assert!(output.contains("D"));
    }

    #[test]
    fn test_horizontal_linear_single_line() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let output = render_to_string(&progress, &layout);
        let lines: Vec<&str> = output.lines().collect();
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

        let output = render_to_string(&progress, &layout);
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
        assert_eq!(display_width("─┬─"), 3);
        assert_eq!(display_width("└─"), 2);
        assert_eq!(display_width("│"), 1);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("漢字"), 4);
        assert_eq!(display_width("A漢B"), 4);
    }

    #[test]
    fn test_display_width_combining() {
        assert_eq!(display_width("e\u{0301}"), 1);
    }

    #[test]
    fn test_animation_tick_catches_up() {
        let frames = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let mut anim = Animation::cycle(frames, Duration::from_millis(100));
        assert_eq!(anim.frame(), "a");

        let changed = anim.tick(Duration::from_millis(350));
        assert!(changed);
        assert_eq!(anim.frame(), "d");
    }
}
