//! Viewport and DagLayout: spatial mapping from DAG topology to coordinates.
//!
//! This module provides the spatial layer between the logical DAG and any
//! renderer (terminal, CI, web). It answers: "given a DAG's topology and a
//! bounded rendering region, where does each node and edge go?"
//!
//! # Architecture
//!
//! ```text
//! Dag<T> + Viewport  ──→  compute_layout()  ──→  DagLayout
//! ```
//!
//! - [`Viewport`]: Bounded rendering region (terminal, CI log, web container)
//! - [`DagLayout`]: Spatial positions for every node and edge
//! - [`compute_layout`]: Transforms topology into positions
//!
//! The layout knows nothing about symbols, colors, or animation.
//! It's purely about spatial coordinates.

use crate::dag::Edge;
use crate::types::NodeId;
use std::collections::HashMap;
use std::ops::Range;

// ---------------------------------------------------------------------------
// Viewport
// ---------------------------------------------------------------------------

/// A bounded region to render into. Every renderer provides one.
///
/// - **Terminal**: character cells, queried from terminal size
/// - **CI**: wide (120 cols), unbounded height (logs scroll forever)
/// - **Web**: CSS pixels from container dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
    pub unit: ViewportUnit,
}

/// What the viewport dimensions measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportUnit {
    /// Terminal: character cells
    Chars,
    /// Web: CSS pixels
    Pixels,
}

impl Viewport {
    /// Create a new viewport.
    pub fn new(width: u16, height: u16, unit: ViewportUnit) -> Self {
        Self {
            width,
            height,
            unit,
        }
    }

    /// Standard terminal viewport (80×24 chars).
    pub fn terminal_default() -> Self {
        Self {
            width: 80,
            height: 24,
            unit: ViewportUnit::Chars,
        }
    }

    /// CI viewport: wide, unbounded height (logs scroll forever).
    pub fn ci() -> Self {
        Self {
            width: 120,
            height: u16::MAX,
            unit: ViewportUnit::Chars,
        }
    }

    /// Check if the viewport has unbounded height (CI / pipe mode).
    pub fn is_unbounded_height(&self) -> bool {
        self.height == u16::MAX
    }
}

// ---------------------------------------------------------------------------
// Layout types
// ---------------------------------------------------------------------------

/// Spatial layout of a DAG within a viewport.
///
/// Derived from topology + constraints. Recomputed on viewport change.
/// Renderer-agnostic: just positions, no symbols or colors.
#[derive(Debug, Clone)]
pub struct DagLayout {
    pub viewport: Viewport,
    pub nodes: HashMap<NodeId, NodeLayout>,
    pub edges: Vec<EdgeLayout>,
    /// Nodes grouped by topological level (parallel nodes share a level).
    pub levels: Vec<Vec<NodeId>>,
    /// Number of horizontal tracks (rows) used by the layout.
    pub tracks: usize,
    /// Mapping of node → track index.
    pub node_tracks: HashMap<NodeId, usize>,
    /// Mapping of node → letter label (A, B, ...).
    pub node_letters: HashMap<NodeId, String>,
    /// Grid lookup: (track, column) → node.
    pub grid: HashMap<(usize, usize), NodeId>,
    /// Connector cells between columns (gap index → track index).
    pub connectors: Vec<Vec<ConnectorCell>>,
    /// Box width in characters (uniform for all nodes).
    pub box_width: u16,
    /// Connector gap width in characters.
    pub gap_width: u16,
    /// Actual height used (may be less than viewport.height).
    pub total_rows: u16,
    /// Actual width used.
    pub total_cols: u16,
    /// Overflow state if DAG doesn't fit in viewport.
    pub overflow: OverflowState,
}

/// Spatial position of a single node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLayout {
    pub id: NodeId,
    /// Vertical position (track index).
    pub row: u16,
    /// Horizontal position (level/column index).
    pub col: u16,
    /// Display width of the node label.
    pub label_width: u16,
    /// True if viewport forced this node to collapse.
    pub is_collapsed: bool,
}

/// Spatial route for an edge between two nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeLayout {
    pub from: NodeId,
    pub to: NodeId,
    /// (row, col) waypoints for edge routing.
    pub path: Vec<(u16, u16)>,
    pub orientation: EdgeOrientation,
}

/// A single connector cell between two columns on a specific track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorCell {
    pub glyph: char,
    pub edges: Vec<(NodeId, NodeId)>,
}

/// How an edge is oriented in the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOrientation {
    /// Same-level: ──── between nodes
    Horizontal,
    /// Cross-level: │ downward
    Vertical,
    /// Corner: └── or ┌──
    Bend,
}

/// What to do when the DAG doesn't fit in the viewport.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OverflowStrategy {
    /// Collapse low-priority nodes. Show completed nodes as a count.
    #[default]
    Collapse,
    /// Truncate: show first N levels that fit, add "... +M more" footer.
    Truncate,
    /// Scroll: focus viewport on the active region.
    Scroll { focus: NodeId },
}

/// Tracks what got truncated/collapsed due to viewport constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowState {
    pub strategy: OverflowStrategy,
    /// Nodes hidden by collapse.
    pub collapsed_nodes: Vec<NodeId>,
    /// Which levels are currently visible.
    pub visible_levels: Range<usize>,
}

impl Default for OverflowState {
    fn default() -> Self {
        Self {
            strategy: OverflowStrategy::Collapse,
            collapsed_nodes: Vec::new(),
            visible_levels: 0..0,
        }
    }
}

// ---------------------------------------------------------------------------
// Level computation
// ---------------------------------------------------------------------------

/// Compute topological levels from node ordering and edges.
///
/// `level[node] = max(level[predecessor] + 1)` for each predecessor.
/// Nodes at the same level can execute in parallel (diamond pattern).
pub fn compute_levels(topo_order: &[NodeId], edges: &[Edge]) -> Vec<Vec<NodeId>> {
    let mut level_of: HashMap<&NodeId, usize> = HashMap::new();

    // Initialize all nodes at level 0
    for id in topo_order {
        level_of.insert(id, 0);
    }

    // For each node in topo order, push its successors down
    for id in topo_order {
        let my_level = level_of[id];
        for edge in edges {
            if edge.from_node == *id {
                let entry = level_of.entry(&edge.to_node).or_insert(0);
                if my_level + 1 > *entry {
                    *entry = my_level + 1;
                }
            }
        }
    }

    // Group nodes by level
    if topo_order.is_empty() {
        return Vec::new();
    }
    let max_level = level_of.values().copied().max().unwrap_or(0);
    let mut levels: Vec<Vec<NodeId>> = vec![Vec::new(); max_level + 1];
    for id in topo_order {
        let lvl = level_of[id];
        levels[lvl].push(id.clone());
    }

    levels
}

// ---------------------------------------------------------------------------
// Layout computation
// ---------------------------------------------------------------------------

/// Connector gap width between columns (in character cells).
const DEFAULT_GAP_WIDTH: u16 = 3;

#[derive(Debug, Clone, Default)]
struct ConnectorBits {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    arrow: bool,
    edges: Vec<(NodeId, NodeId)>,
}

impl ConnectorBits {
    fn add_edge(&mut self, from: &NodeId, to: &NodeId) {
        if !self.edges.iter().any(|(f, t)| f == from && t == to) {
            self.edges.push((from.clone(), to.clone()));
        }
    }

    fn glyph(&self) -> char {
        let up = self.up;
        let down = self.down;
        let left = self.left;
        let right = self.right;

        match (up, down, left, right) {
            (true, true, true, true) => '\u{253C}',    // ┼
            (true, true, true, false) => '\u{2524}',   // ┤
            (true, true, false, true) => '\u{251C}',   // ├
            (true, false, true, true) => '\u{2534}',   // ┴
            (false, true, true, true) => '\u{252C}',   // ┬
            (false, true, false, true) => '\u{250C}',  // ┌
            (false, true, true, false) => '\u{2510}',  // ┐
            (true, false, false, true) => '\u{2514}',  // └
            (true, false, true, false) => '\u{2518}',  // ┘
            (true, true, false, false) => '\u{2502}',  // │
            (true, false, false, false) => '\u{2502}', // │
            (false, true, false, false) => '\u{2502}', // │
            (false, false, true, true) => {
                if self.arrow {
                    '\u{2192}'
                } else {
                    '\u{2500}'
                } // → or ─
            }
            (false, false, true, false) => '\u{2500}', // ─
            (false, false, false, true) => '\u{2500}', // ─
            _ => ' ',
        }
    }
}

/// Compute the spatial layout of a DAG within a viewport.
///
/// This is the main entry point. Takes a DAG's topology (as topo order + edges)
/// and a viewport, returns positioned nodes, connectors, and edge routes.
pub fn compute_layout(
    topo_order: &[NodeId],
    edges: &[Edge],
    _labels: &HashMap<NodeId, String>,
    viewport: &Viewport,
) -> DagLayout {
    let levels = compute_levels(topo_order, edges);
    let num_cols = levels.len();

    let mut level_of: HashMap<NodeId, usize> = HashMap::new();
    for (col, level) in levels.iter().enumerate() {
        for node in level {
            level_of.insert(node.clone(), col);
        }
    }

    let mut topo_index: HashMap<NodeId, usize> = HashMap::new();
    for (idx, node) in topo_order.iter().enumerate() {
        topo_index.insert(node.clone(), idx);
    }

    let mut parents_of: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for edge in edges {
        parents_of
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge.from_node.clone());
    }

    // Order nodes within each level using parent-track barycenter, then topo order.
    // This is deterministic and derived from dependency structure (not renderer heuristics).
    let mut node_tracks: HashMap<NodeId, usize> = HashMap::new();
    let mut ordered_levels: Vec<Vec<NodeId>> = Vec::new();

    for (col, level) in levels.iter().enumerate() {
        let mut nodes = level.clone();
        if col == 0 {
            nodes.sort_by_key(|n| topo_index.get(n).copied().unwrap_or(usize::MAX));
        } else {
            nodes.sort_by(|a, b| {
                let a_score = avg_parent_track(a, &parents_of, &node_tracks);
                let b_score = avg_parent_track(b, &parents_of, &node_tracks);
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        topo_index
                            .get(a)
                            .copied()
                            .unwrap_or(usize::MAX)
                            .cmp(&topo_index.get(b).copied().unwrap_or(usize::MAX))
                    })
            });
        }

        for (idx, node) in nodes.iter().enumerate() {
            node_tracks.insert(node.clone(), idx);
        }
        ordered_levels.push(nodes);
    }

    let tracks = ordered_levels
        .iter()
        .map(|lvl| lvl.len())
        .max()
        .unwrap_or(0);

    // Letters for node boxes (A, B, ...).
    let mut node_letters: HashMap<NodeId, String> = HashMap::new();
    for (idx, node) in topo_order.iter().enumerate() {
        node_letters.insert(node.clone(), index_to_letters(idx));
    }
    let max_letter_width = node_letters
        .values()
        .map(|s| s.len() as u16)
        .max()
        .unwrap_or(1);
    let box_width = max_letter_width + 2; // [A]
    let gap_width = DEFAULT_GAP_WIDTH;
    let max_tracks_visible = if viewport.is_unbounded_height() {
        tracks
    } else {
        viewport.height as usize
    };

    // Build node layouts and grid lookup.
    let mut node_layouts: HashMap<NodeId, NodeLayout> = HashMap::new();
    let mut grid: HashMap<(usize, usize), NodeId> = HashMap::new();
    let mut collapsed_nodes: Vec<NodeId> = Vec::new();
    for (col, level) in ordered_levels.iter().enumerate() {
        for node in level {
            let track = node_tracks.get(node).copied().unwrap_or(0);
            let letter_w = node_letters.get(node).map(|s| s.len() as u16).unwrap_or(1);
            let is_collapsed = track >= max_tracks_visible;
            if is_collapsed {
                collapsed_nodes.push(node.clone());
            }
            node_layouts.insert(
                node.clone(),
                NodeLayout {
                    id: node.clone(),
                    row: track as u16,
                    col: col as u16,
                    label_width: letter_w,
                    is_collapsed,
                },
            );
            grid.insert((track, col), node.clone());
        }
    }

    // Connector bits between columns (gap index → track index).
    let mut connector_bits: Vec<Vec<ConnectorBits>> =
        vec![vec![ConnectorBits::default(); tracks]; num_cols.saturating_sub(1)];

    // Route edges using a deterministic policy:
    // - Vertical move in the first gap to reach the target track.
    // - Horizontal moves along the target track for remaining gaps.
    for edge in edges {
        let from = &edge.from_node;
        let to = &edge.to_node;
        let col_from = *level_of.get(from).unwrap_or(&0);
        let col_to = *level_of.get(to).unwrap_or(&col_from);
        if col_to <= col_from {
            continue;
        }
        let track_from = *node_tracks.get(from).unwrap_or(&0);
        let track_to = *node_tracks.get(to).unwrap_or(&0);
        let first_gap = col_from;
        let last_gap = col_to - 1;

        if track_from == track_to {
            for gap in first_gap..=last_gap {
                if let Some(cell) = connector_bits
                    .get_mut(gap)
                    .and_then(|row| row.get_mut(track_from))
                {
                    cell.left = true;
                    cell.right = true;
                    if gap == last_gap {
                        cell.arrow = true;
                    }
                    cell.add_edge(from, to);
                }
            }
            continue;
        }

        // First gap: vertical move between tracks.
        if let Some(col_cells) = connector_bits.get_mut(first_gap) {
            let (min_t, max_t) = if track_from < track_to {
                (track_from, track_to)
            } else {
                (track_to, track_from)
            };
            for track in min_t..=max_t {
                if let Some(cell) = col_cells.get_mut(track) {
                    if track == track_from {
                        cell.left = true;
                        if track_to > track_from {
                            cell.down = true;
                        } else {
                            cell.up = true;
                        }
                    } else if track == track_to {
                        cell.right = true;
                        if track_to > track_from {
                            cell.up = true;
                        } else {
                            cell.down = true;
                        }
                    } else {
                        cell.up = true;
                        cell.down = true;
                    }
                    cell.add_edge(from, to);
                }
            }
        }

        // Remaining gaps: horizontal along target track.
        if col_to > col_from + 1 {
            for gap in (first_gap + 1)..=last_gap {
                if let Some(cell) = connector_bits
                    .get_mut(gap)
                    .and_then(|row| row.get_mut(track_to))
                {
                    cell.left = true;
                    cell.right = true;
                    if gap == last_gap {
                        cell.arrow = true;
                    }
                    cell.add_edge(from, to);
                }
            }
        }
    }

    let connectors: Vec<Vec<ConnectorCell>> = connector_bits
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|bits| ConnectorCell {
                    glyph: bits.glyph(),
                    edges: bits.edges,
                })
                .collect()
        })
        .collect();

    let edge_layouts: Vec<EdgeLayout> = edges
        .iter()
        .filter_map(|edge| {
            let from_layout = node_layouts.get(&edge.from_node)?;
            let to_layout = node_layouts.get(&edge.to_node)?;
            Some(compute_edge_layout_horizontal(from_layout, to_layout))
        })
        .collect();

    let total_rows = tracks as u16;
    let total_cols = if num_cols == 0 {
        0
    } else {
        (num_cols as u16) * box_width + ((num_cols as u16).saturating_sub(1) * gap_width)
    };

    // Visible columns based on horizontal viewport width.
    let visible_levels = if num_cols == 0 {
        0..0
    } else {
        let max_levels = if viewport.width == 0 {
            0
        } else {
            ((viewport.width + gap_width) / (box_width + gap_width)).max(1) as usize
        };
        if total_cols <= viewport.width {
            0..num_cols
        } else {
            0..max_levels.min(num_cols)
        }
    };

    let overflow = OverflowState {
        strategy: if visible_levels.len() < num_cols && collapsed_nodes.is_empty() {
            OverflowStrategy::Truncate
        } else {
            OverflowStrategy::Collapse
        },
        collapsed_nodes,
        visible_levels,
    };

    DagLayout {
        viewport: *viewport,
        nodes: node_layouts,
        edges: edge_layouts,
        levels: ordered_levels,
        tracks,
        node_tracks,
        node_letters,
        grid,
        connectors,
        box_width,
        gap_width,
        total_rows,
        total_cols,
        overflow,
    }
}

fn avg_parent_track(
    node: &NodeId,
    parents_of: &HashMap<NodeId, Vec<NodeId>>,
    node_tracks: &HashMap<NodeId, usize>,
) -> f64 {
    let parents = match parents_of.get(node) {
        Some(p) if !p.is_empty() => p,
        _ => return f64::MAX,
    };
    let mut total = 0.0;
    let mut count = 0.0;
    for parent in parents {
        if let Some(track) = node_tracks.get(parent) {
            total += *track as f64;
            count += 1.0;
        }
    }
    if count == 0.0 {
        f64::MAX
    } else {
        total / count
    }
}

fn index_to_letters(mut idx: usize) -> String {
    // 0 -> A, 1 -> B, ... 25 -> Z, 26 -> AA
    let mut chars = Vec::new();
    loop {
        let rem = idx % 26;
        chars.push((b'A' + rem as u8) as char);
        if idx < 26 {
            break;
        }
        idx = (idx / 26) - 1;
    }
    chars.iter().rev().collect()
}

fn compute_edge_layout_horizontal(from: &NodeLayout, to: &NodeLayout) -> EdgeLayout {
    let orientation = if from.row == to.row {
        EdgeOrientation::Horizontal
    } else {
        EdgeOrientation::Bend
    };
    let path = if from.row == to.row {
        vec![(from.row, from.col), (to.row, to.col)]
    } else {
        vec![(from.row, from.col), (to.row, from.col), (to.row, to.col)]
    };
    EdgeLayout {
        from: from.id.clone(),
        to: to.id.clone(),
        path,
        orientation,
    }
}

impl DagLayout {
    /// Update the viewport and recompute overflow state.
    ///
    /// Call this on terminal resize (SIGWINCH) — same topology,
    /// new spatial mapping.
    pub fn resize(&mut self, viewport: Viewport) {
        // Re-derive the layout with the new viewport
        // For now, just update the viewport. Full recompute would
        // recalculate node positions, but we keep it simple:
        // the caller should call compute_layout() again.
        self.viewport = viewport;
    }

    /// Get the layout for a specific node.
    pub fn node(&self, id: &NodeId) -> Option<&NodeLayout> {
        self.nodes.get(id)
    }

    /// Get edges originating from a specific node.
    pub fn edges_from(&self, id: &NodeId) -> Vec<&EdgeLayout> {
        self.edges.iter().filter(|e| e.from == *id).collect()
    }

    /// Get edges terminating at a specific node.
    pub fn edges_to(&self, id: &NodeId) -> Vec<&EdgeLayout> {
        self.edges.iter().filter(|e| e.to == *id).collect()
    }

    /// Check if the layout has any overflow (nodes/levels that didn't fit).
    pub fn has_overflow(&self) -> bool {
        !self.overflow.collapsed_nodes.is_empty()
            || self.overflow.visible_levels.len() < self.levels.len()
    }

    /// Get the number of topological levels in the layout.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Get nodes at a specific level.
    pub fn nodes_at_level(&self, level: usize) -> &[NodeId] {
        self.levels.get(level).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the maximum parallelism (widest level).
    pub fn max_parallelism(&self) -> usize {
        self.levels.iter().map(|l| l.len()).max().unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn labels_from(ids: &[&str]) -> HashMap<NodeId, String> {
        ids.iter()
            .map(|id| (NodeId::from(*id), id.to_string()))
            .collect()
    }

    #[test]
    fn test_compute_levels_linear() {
        // A → B → C
        let order = vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")];
        let edges = vec![
            Edge::new("A", "out", "B", "in"),
            Edge::new("B", "out", "C", "in"),
        ];

        let levels = compute_levels(&order, &edges);

        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![NodeId::from("A")]);
        assert_eq!(levels[1], vec![NodeId::from("B")]);
        assert_eq!(levels[2], vec![NodeId::from("C")]);
    }

    #[test]
    fn test_compute_levels_diamond() {
        // A → B, A → C, B → D, C → D
        let order = vec![
            NodeId::from("A"),
            NodeId::from("B"),
            NodeId::from("C"),
            NodeId::from("D"),
        ];
        let edges = vec![
            Edge::new("A", "out", "B", "in"),
            Edge::new("A", "out", "C", "in"),
            Edge::new("B", "out", "D", "in"),
            Edge::new("C", "out", "D", "in"),
        ];

        let levels = compute_levels(&order, &edges);

        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![NodeId::from("A")]);
        // B and C at same level (parallel)
        assert_eq!(levels[1].len(), 2);
        assert!(levels[1].contains(&NodeId::from("B")));
        assert!(levels[1].contains(&NodeId::from("C")));
        assert_eq!(levels[2], vec![NodeId::from("D")]);
    }

    #[test]
    fn test_compute_levels_independent() {
        // A, B, C (no edges)
        let order = vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")];
        let edges = vec![];

        let levels = compute_levels(&order, &edges);

        // All at level 0
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 3);
    }

    #[test]
    fn test_layout_linear_pipeline() {
        let order = vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")];
        let edges = vec![
            Edge::new("A", "out", "B", "in"),
            Edge::new("B", "out", "C", "in"),
        ];
        let labels = labels_from(&["A", "B", "C"]);
        let vp = Viewport::terminal_default();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        assert_eq!(layout.level_count(), 3);
        assert_eq!(layout.max_parallelism(), 1);
        assert!(!layout.has_overflow());

        // Nodes should share the same track (row 0)
        let a = layout.node(&NodeId::from("A")).unwrap();
        let b = layout.node(&NodeId::from("B")).unwrap();
        let c = layout.node(&NodeId::from("C")).unwrap();
        assert_eq!(a.row, 0);
        assert_eq!(b.row, 0);
        assert_eq!(c.row, 0);

        // Columns should advance by level
        assert_eq!(a.col, 0);
        assert_eq!(b.col, 1);
        assert_eq!(c.col, 2);
    }

    #[test]
    fn test_layout_diamond() {
        let order = vec![
            NodeId::from("A"),
            NodeId::from("B"),
            NodeId::from("C"),
            NodeId::from("D"),
        ];
        let edges = vec![
            Edge::new("A", "out", "B", "in"),
            Edge::new("A", "out", "C", "in"),
            Edge::new("B", "out", "D", "in"),
            Edge::new("C", "out", "D", "in"),
        ];
        let labels = labels_from(&["A", "B", "C", "D"]);
        let vp = Viewport::terminal_default();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        assert_eq!(layout.level_count(), 3);
        assert_eq!(layout.max_parallelism(), 2);

        // B and C should be in the same column (parallel level)
        let b = layout.node(&NodeId::from("B")).unwrap();
        let c = layout.node(&NodeId::from("C")).unwrap();
        assert_eq!(b.col, c.col);

        // B and C should have different rows (tracks)
        assert_ne!(b.row, c.row);

        // Edges to D should be bends (different columns)
        let d_edges = layout.edges_to(&NodeId::from("D"));
        assert_eq!(d_edges.len(), 2);
    }

    #[test]
    fn test_layout_ci_viewport() {
        let order = vec![NodeId::from("lint"), NodeId::from("build")];
        let edges = vec![Edge::new("lint", "out", "build", "in")];
        let labels = labels_from(&["lint", "build"]);
        let vp = Viewport::ci();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        assert!(vp.is_unbounded_height());
        assert!(!layout.has_overflow());
    }

    #[test]
    fn test_layout_overflow_many_parallel() {
        // 10 parallel nodes in a narrow viewport
        let ids: Vec<String> = (0..10).map(|i| format!("node_{i}")).collect();
        let order: Vec<NodeId> = ids.iter().map(|s| NodeId::from(s.as_str())).collect();
        let edges = vec![]; // All independent = all at level 0
        let labels: HashMap<NodeId, String> = ids
            .iter()
            .map(|s| (NodeId::from(s.as_str()), s.clone()))
            .collect();

        // Short viewport height: only fits 3 tracks
        let vp = Viewport::new(80, 3, ViewportUnit::Chars);
        let layout = compute_layout(&order, &edges, &labels, &vp);

        // Should have collapsed some nodes
        assert!(layout.has_overflow());
        assert!(!layout.overflow.collapsed_nodes.is_empty());
    }

    #[test]
    fn test_edge_orientations() {
        // Linear: edges should be horizontal in the left-to-right layout
        let order = vec![NodeId::from("A"), NodeId::from("B")];
        let edges = vec![Edge::new("A", "out", "B", "in")];
        let labels = labels_from(&["A", "B"]);
        let vp = Viewport::terminal_default();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        // Same track → horizontal
        assert_eq!(layout.edges.len(), 1);
        let edge = &layout.edges[0];
        assert_eq!(edge.orientation, EdgeOrientation::Horizontal);
    }

    #[test]
    fn test_viewport_defaults() {
        let term = Viewport::terminal_default();
        assert_eq!(term.width, 80);
        assert_eq!(term.height, 24);
        assert_eq!(term.unit, ViewportUnit::Chars);
        assert!(!term.is_unbounded_height());

        let ci = Viewport::ci();
        assert_eq!(ci.width, 120);
        assert!(ci.is_unbounded_height());
    }

    #[test]
    fn test_empty_dag_layout() {
        let order: Vec<NodeId> = vec![];
        let edges: Vec<Edge> = vec![];
        let labels = HashMap::new();
        let vp = Viewport::terminal_default();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        assert_eq!(layout.level_count(), 0);
        assert_eq!(layout.max_parallelism(), 0);
        assert_eq!(layout.total_rows, 0);
    }
}
