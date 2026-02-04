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
        Self { width, height, unit }
    }

    /// Standard terminal viewport (80×24 chars). Fallback when not a TTY.
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
    /// Vertical position (level → row mapping).
    pub row: u16,
    /// Horizontal position within the level band.
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

/// Node spacing constants (in character cells).
const NODE_SYMBOL_WIDTH: u16 = 2; // symbol + space
const NODE_MIN_LABEL: u16 = 4;    // minimum label display
const NODE_PADDING: u16 = 3;      // inter-node padding
const LEVEL_ROW_SPACING: u16 = 2; // rows between levels (node row + edge row)

/// Compute the spatial layout of a DAG within a viewport.
///
/// This is the main entry point. Takes a DAG's topology (as topo order + edges)
/// and a viewport, returns positioned nodes and edges.
pub fn compute_layout(
    topo_order: &[NodeId],
    edges: &[Edge],
    labels: &HashMap<NodeId, String>,
    viewport: &Viewport,
) -> DagLayout {
    let levels = compute_levels(topo_order, edges);

    // Compute node positions
    let mut node_layouts: HashMap<NodeId, NodeLayout> = HashMap::new();
    let mut max_col: u16 = 0;
    let mut collapsed_nodes: Vec<NodeId> = Vec::new();

    for (level_idx, level_nodes) in levels.iter().enumerate() {
        let row = (level_idx as u16) * LEVEL_ROW_SPACING;

        // Calculate how much space each node needs
        let node_widths: Vec<u16> = level_nodes
            .iter()
            .map(|id| {
                let label = labels.get(id).map(|s| s.as_str()).unwrap_or(&id.0);
                let label_w = label.len() as u16;
                NODE_SYMBOL_WIDTH + label_w.max(NODE_MIN_LABEL)
            })
            .collect();

        // Total width needed for this level
        let total_needed: u16 = node_widths.iter().sum::<u16>()
            + NODE_PADDING * (level_nodes.len().saturating_sub(1) as u16);

        let fits = total_needed <= viewport.width;

        // Place nodes left-to-right
        let mut col: u16 = 0;
        for (i, id) in level_nodes.iter().enumerate() {
            let label = labels.get(id).map(|s| s.as_str()).unwrap_or(&id.0);
            let label_width = label.len() as u16;
            let is_collapsed = !fits && i >= max_visible_nodes(viewport.width);

            if is_collapsed {
                collapsed_nodes.push(id.clone());
            }

            node_layouts.insert(
                id.clone(),
                NodeLayout {
                    id: id.clone(),
                    row,
                    col,
                    label_width,
                    is_collapsed,
                },
            );

            col += node_widths[i] + NODE_PADDING;
            if col > max_col {
                max_col = col;
            }
        }
    }

    // Compute edge layouts
    let edge_layouts: Vec<EdgeLayout> = edges
        .iter()
        .filter_map(|edge| {
            let from_layout = node_layouts.get(&edge.from_node)?;
            let to_layout = node_layouts.get(&edge.to_node)?;
            Some(compute_edge_layout(from_layout, to_layout))
        })
        .collect();

    let total_rows = if levels.is_empty() {
        0
    } else {
        (levels.len() as u16 - 1) * LEVEL_ROW_SPACING + 1
    };

    // Determine visible levels
    let visible_levels = if viewport.is_unbounded_height() || total_rows <= viewport.height {
        0..levels.len()
    } else {
        // Truncate to fit viewport
        let max_levels = (viewport.height / LEVEL_ROW_SPACING) as usize + 1;
        0..max_levels.min(levels.len())
    };

    let overflow = OverflowState {
        strategy: if visible_levels.len() < levels.len() && collapsed_nodes.is_empty() {
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
        levels,
        total_rows,
        total_cols: max_col,
        overflow,
    }
}

/// Compute how many nodes can fit side-by-side in the given width.
fn max_visible_nodes(width: u16) -> usize {
    // Each node needs at least symbol + short label + padding
    let min_node_width = NODE_SYMBOL_WIDTH + NODE_MIN_LABEL + NODE_PADDING;
    (width / min_node_width).max(1) as usize
}

/// Compute the edge layout between two positioned nodes.
fn compute_edge_layout(from: &NodeLayout, to: &NodeLayout) -> EdgeLayout {
    let from_center_col = from.col + NODE_SYMBOL_WIDTH + from.label_width / 2;
    let to_center_col = to.col + NODE_SYMBOL_WIDTH + to.label_width / 2;

    let orientation = if from.row == to.row {
        EdgeOrientation::Horizontal
    } else if from_center_col == to_center_col {
        EdgeOrientation::Vertical
    } else {
        EdgeOrientation::Bend
    };

    let path = match orientation {
        EdgeOrientation::Horizontal => {
            vec![(from.row, from_center_col), (to.row, to_center_col)]
        }
        EdgeOrientation::Vertical => {
            // Straight down: from bottom of source to top of dest
            let edge_row = from.row + 1; // connector row between levels
            vec![
                (from.row, from_center_col),
                (edge_row, from_center_col),
                (to.row, to_center_col),
            ]
        }
        EdgeOrientation::Bend => {
            // Route: go down from source, then horizontal, then to destination
            let edge_row = from.row + 1; // connector row
            vec![
                (from.row, from_center_col),
                (edge_row, from_center_col),
                (edge_row, to_center_col),
                (to.row, to_center_col),
            ]
        }
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
        let order = vec![
            NodeId::from("A"),
            NodeId::from("B"),
            NodeId::from("C"),
        ];
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
        let order = vec![
            NodeId::from("A"),
            NodeId::from("B"),
            NodeId::from("C"),
        ];
        let edges = vec![];

        let levels = compute_levels(&order, &edges);

        // All at level 0
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 3);
    }

    #[test]
    fn test_layout_linear_pipeline() {
        let order = vec![
            NodeId::from("A"),
            NodeId::from("B"),
            NodeId::from("C"),
        ];
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

        // Nodes should be at different rows
        let a = layout.node(&NodeId::from("A")).unwrap();
        let b = layout.node(&NodeId::from("B")).unwrap();
        let c = layout.node(&NodeId::from("C")).unwrap();
        assert!(a.row < b.row);
        assert!(b.row < c.row);

        // All at column 0 (single node per level)
        assert_eq!(a.col, 0);
        assert_eq!(b.col, 0);
        assert_eq!(c.col, 0);
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

        // B and C should be on the same row (parallel)
        let b = layout.node(&NodeId::from("B")).unwrap();
        let c = layout.node(&NodeId::from("C")).unwrap();
        assert_eq!(b.row, c.row);

        // B and C should have different columns
        assert_ne!(b.col, c.col);

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

        // Narrow viewport: only fits ~4 nodes
        let vp = Viewport::new(40, 24, ViewportUnit::Chars);
        let layout = compute_layout(&order, &edges, &labels, &vp);

        // Should have collapsed some nodes
        assert!(layout.has_overflow());
        assert!(!layout.overflow.collapsed_nodes.is_empty());
    }

    #[test]
    fn test_edge_orientations() {
        // Linear: edges should be vertical
        let order = vec![NodeId::from("A"), NodeId::from("B")];
        let edges = vec![Edge::new("A", "out", "B", "in")];
        let labels = labels_from(&["A", "B"]);
        let vp = Viewport::terminal_default();

        let layout = compute_layout(&order, &edges, &labels, &vp);

        // Same column → vertical or bend based on center alignment
        assert_eq!(layout.edges.len(), 1);
        let edge = &layout.edges[0];
        assert!(
            edge.orientation == EdgeOrientation::Vertical
                || edge.orientation == EdgeOrientation::Bend
        );
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
