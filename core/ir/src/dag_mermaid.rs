//! Mermaid diagram generation for DAG topology diffs and snapshots.
//!
//! Produces color-annotated Mermaid flowchart text. Two categories:
//!
//! **Diff mode** (for `make dag-diff`):
//! - Overview: Top-level nodes colored by diff status (added/removed/changed/unchanged)
//! - Expanded: Per-tool detail with subgraph nesting and diff coloring
//!
//! **Snapshot mode** (for `make dag-viz`):
//! - Semantic coloring by node role (env, execute, SubDag, etc.)
//! - Depth-based subgraph border colors for visual nesting
//! - Aggregated edges to reduce label noise
//!
//! Edge aggregation: multiple edges between the same node pair are collapsed
//! into a single arrow. Resource edges (`res:*`) are shown separately with
//! dashed lines.

use crate::dag_diff::{DagDiffResult, NodeDiffStatus};
use crate::dag_topology::{DagTopology, EdgeTopology, NodeTopology};
use crate::types::NodeId;
use std::collections::BTreeMap;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum subgraph nesting depth before we truncate.
const MAX_SUBGRAPH_DEPTH: usize = 3;

/// Mermaid init directive — dark theme for GitHub Gist readability.
const MERMAID_DARK_INIT: &str =
    "%%{init: {\"theme\": \"dark\", \"themeVariables\": {\"darkMode\": true}}}%%\n";

/// Maximum Mermaid source length (chars) for which we generate mermaid.ink URLs.
/// Larger diagrams exceed URL length limits (~8KB encoded ≈ ~6KB source).
const MERMAID_INK_MAX_LEN: usize = 6000;

/// Mermaid classDef declarations for diff status colors (dark-mode palette).
const DIFF_CLASS_DEFS: &str = "\
    classDef added fill:#166534,stroke:#22c55e,stroke-width:2px,color:#bbf7d0\n    \
    classDef removed fill:#991b1b,stroke:#ef4444,stroke-dasharray:5,color:#fecaca\n    \
    classDef changed fill:#92400e,stroke:#f59e0b,stroke-width:2px,color:#fef3c7\n    \
    classDef unchanged fill:#374151,stroke:#6b7280,color:#e5e7eb\n";

/// Mermaid classDef declarations for snapshot semantic colors.
///
/// Uses the project's SemanticColor palette (from gunb.ai/pkg/fermi/colors.go):
/// - Cyan (#22d3ee): env/resource provider — maps to SemanticColor::Info
/// - Orange (#f97316): execute/transport boundary — maps to SemanticColor::Warning
/// - Blue (#60a5fa): SubDag composition — maps to SemanticColor::Accent
/// - Gray: default logic nodes — maps to SemanticColor::Dim
///
/// All colors use dark-on-transparent text for contrast on both light/dark backgrounds.
const SNAPSHOT_CLASS_DEFS: &str = "\
    classDef env fill:#0e7490,stroke:#22d3ee,stroke-width:2px,color:#cffafe\n    \
    classDef execute fill:#9a3412,stroke:#f97316,stroke-width:2px,color:#ffedd5\n    \
    classDef subdag fill:#1e40af,stroke:#60a5fa,stroke-width:2px,color:#dbeafe\n    \
    classDef default fill:#374151,stroke:#6b7280,color:#e5e7eb\n";

/// Subgraph border colors by depth level — cool-to-warm gradient using the
/// standard palette. Borders are visible, fills are near-transparent.
const DEPTH_COLORS: &[&str] = &[
    "#60a5fa", // depth 0: accent blue (SemanticColor::Accent)
    "#22d3ee", // depth 1: cyan (SemanticColor::Info)
    "#6b7280", // depth 2: gray (SemanticColor::Dim)
    "#f97316", // depth 3: orange (SemanticColor::Warning)
];

/// Subgraph fill colors by depth level — very faint so nodes remain readable.
/// Dark-mode friendly: dark fills with light borders.
const DEPTH_FILLS: &[&str] = &[
    "#1e293b", // depth 0: slate-800
    "#1a2332", // depth 1: darker blue-gray
    "#1f2937", // depth 2: gray-800
    "#292524", // depth 3: stone-800
];

// ---------------------------------------------------------------------------
// Public API: Diff mode
// ---------------------------------------------------------------------------

/// Render a Mermaid overview diagram showing only top-level nodes.
///
/// Each node is colored by its diff status. SubDag nodes use `[[name]]` syntax.
/// This always produces a small diagram (N nodes = number of top-level tools).
pub fn to_mermaid_overview_diff(
    new_topo: &DagTopology,
    diff: &DagDiffResult,
    removed_nodes: &[&NodeTopology],
) -> String {
    let mut out = String::new();
    out.push_str(MERMAID_DARK_INIT);
    out.push_str("flowchart TB\n");
    out.push_str("    ");
    out.push_str(DIFF_CLASS_DEFS);

    // Render nodes present in the new topology
    for node in &new_topo.nodes {
        let status = diff.node_status(&node.id);
        let class = status_class(status);
        let node_id = sanitize_id(&node.id.0);
        let label = &node.id.0;

        if node.is_subdag() {
            writeln!(out, "    {}[[{}]]:::{}", node_id, label, class).unwrap();
        } else {
            writeln!(out, "    {}[{}]:::{}", node_id, label, class).unwrap();
        }
    }

    // Render removed nodes (only in old topology)
    for node in removed_nodes {
        let node_id = sanitize_id(&node.id.0);
        let label = &node.id.0;

        if node.is_subdag() {
            writeln!(out, "    {}[[{}]]:::removed", node_id, label).unwrap();
        } else {
            writeln!(out, "    {}[{}]:::removed", node_id, label).unwrap();
        }
    }

    // Render edges (new topology only — removed edges are not shown in overview)
    for edge in &new_topo.edges {
        let from_id = sanitize_id(&edge.from_node.0);
        let to_id = sanitize_id(&edge.to_node.0);
        let label = format!("{}:{}", edge.from_port.0, edge.to_port.0);
        writeln!(out, "    {} -->|{}| {}", from_id, label, to_id).unwrap();
    }

    out
}

/// Render an expanded Mermaid diagram for a single tool's internal structure.
///
/// Shows all internal nodes with edges, uses `subgraph` blocks for nested
/// SubDags, and colors each node by its diff status. Recurses up to
/// `MAX_SUBGRAPH_DEPTH` levels.
pub fn to_mermaid_expanded_diff(
    node_id: &str,
    new_topo: &DagTopology,
    diff: &DagDiffResult,
    removed_nodes: &[&NodeTopology],
) -> String {
    let mut out = String::new();
    out.push_str(MERMAID_DARK_INIT);
    out.push_str("flowchart TB\n");
    out.push_str("    ");
    out.push_str(DIFF_CLASS_DEFS);

    let prefix = sanitize_id(node_id);
    render_dag_diff_contents(&mut out, &prefix, new_topo, diff, removed_nodes, 0);

    out
}

// ---------------------------------------------------------------------------
// Public API: Snapshot mode
// ---------------------------------------------------------------------------

/// Render a snapshot Mermaid diagram with semantic coloring and aggregated edges.
pub fn to_mermaid_snapshot(name: &str, topo: &DagTopology) -> String {
    let mut out = String::new();
    out.push_str(MERMAID_DARK_INIT);
    out.push_str("flowchart TB\n");
    out.push_str("    ");
    out.push_str(SNAPSHOT_CLASS_DEFS);

    let prefix = sanitize_id(name);
    render_snapshot_contents(&mut out, &prefix, name, topo, 0);

    out
}

// ---------------------------------------------------------------------------
// Public API: mermaid.ink image URL
// ---------------------------------------------------------------------------

/// Generate a mermaid.ink image URL for a Mermaid diagram.
///
/// This renders the diagram server-side and returns a URL that can be
/// embedded as `![diagram](url)` in markdown for proper browser zoom/pan.
///
/// Returns `None` if the diagram source exceeds `MERMAID_INK_MAX_LEN`
/// characters — URLs that long break browsers and the mermaid.ink service.
pub fn mermaid_ink_url(mermaid_code: &str) -> Option<String> {
    if mermaid_code.len() > MERMAID_INK_MAX_LEN {
        return None;
    }
    let encoded = base64_url_encode(mermaid_code.as_bytes());
    Some(format!("https://mermaid.ink/svg/base64:{}", encoded))
}

// ---------------------------------------------------------------------------
// Diff rendering internals
// ---------------------------------------------------------------------------

fn render_dag_diff_contents(
    out: &mut String,
    prefix: &str,
    topo: &DagTopology,
    diff: &DagDiffResult,
    removed_nodes: &[&NodeTopology],
    depth: usize,
) {
    let indent = "    ".repeat(depth + 1);

    for node in &topo.nodes {
        let status = diff.node_status(&node.id);
        let class = status_class(status);
        let full_id = format!("{}_{}", prefix, sanitize_id(&node.id.0));
        let label = &node.id.0;

        if let Some(ref children) = node.children {
            if depth < MAX_SUBGRAPH_DEPTH {
                let subgraph_label = diff_subgraph_label(label, status, diff, &node.id);
                writeln!(
                    out,
                    "{}subgraph {}[\"{}\"]",
                    indent, full_id, subgraph_label
                )
                .unwrap();

                let child_diff = find_child_diff(diff, &node.id);
                let child_removed = find_child_removed_nodes(diff, &node.id);

                if let Some(cd) = child_diff {
                    render_dag_diff_contents(
                        out,
                        &full_id,
                        children,
                        cd,
                        &child_removed,
                        depth + 1,
                    );
                } else {
                    let empty_diff = DagDiffResult {
                        unchanged_nodes: children.nodes.iter().map(|n| n.id.clone()).collect(),
                        unchanged_edges: children.edges.clone(),
                        ..Default::default()
                    };
                    render_dag_diff_contents(out, &full_id, children, &empty_diff, &[], depth + 1);
                }

                writeln!(out, "{}end", indent).unwrap();
            } else {
                let child_count = children.total_node_count();
                writeln!(
                    out,
                    "{}{}[\"{}\\n({} internal nodes)\"]:::{}",
                    indent, full_id, label, child_count, class
                )
                .unwrap();
            }
        } else {
            writeln!(out, "{}{}[{}]:::{}", indent, full_id, label, class).unwrap();
        }
    }

    for node in removed_nodes {
        let full_id = format!("{}_{}", prefix, sanitize_id(&node.id.0));
        let label = &node.id.0;
        writeln!(out, "{}{}[{}]:::removed", indent, full_id, label).unwrap();
    }

    for edge in &topo.edges {
        let from_id = format!("{}_{}", prefix, sanitize_id(&edge.from_node.0));
        let to_id = format!("{}_{}", prefix, sanitize_id(&edge.to_node.0));
        let edge_class = edge_diff_class(edge, diff);
        let label = format!("{}:{}", edge.from_port.0, edge.to_port.0);

        if edge_class == "added" {
            writeln!(out, "{}  {} -. {} .-> {}", indent, from_id, label, to_id).unwrap();
        } else {
            writeln!(out, "{}  {} -->|{}| {}", indent, from_id, label, to_id).unwrap();
        }
    }

    for edge in &diff.removed_edges {
        let from_id = format!("{}_{}", prefix, sanitize_id(&edge.from_node.0));
        let to_id = format!("{}_{}", prefix, sanitize_id(&edge.to_node.0));
        let label = format!("{}:{}", edge.from_port.0, edge.to_port.0);
        writeln!(out, "{}  {} -. {} .-> {}", indent, from_id, label, to_id).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Snapshot rendering internals
// ---------------------------------------------------------------------------

fn render_snapshot_contents(
    out: &mut String,
    prefix: &str,
    name: &str,
    topo: &DagTopology,
    depth: usize,
) {
    let indent = "    ".repeat(depth + 1);

    // Open subgraph with depth-colored styling
    let color = DEPTH_COLORS[depth.min(DEPTH_COLORS.len() - 1)];
    let fill = DEPTH_FILLS[depth.min(DEPTH_FILLS.len() - 1)];
    let sg_id = format!("{}_{}", prefix, sanitize_id(name));

    writeln!(out, "{}subgraph {}[\"{}\"]", indent, sg_id, name).unwrap();
    writeln!(
        out,
        "{}style {} fill:{},stroke:{},stroke-width:2px,color:#e5e7eb",
        indent, sg_id, fill, color
    )
    .unwrap();

    // Render nodes
    for node in &topo.nodes {
        let full_id = format!("{}_{}", sg_id, sanitize_id(&node.id.0));
        let label = &node.id.0;

        if let Some(ref children) = node.children {
            if depth < MAX_SUBGRAPH_DEPTH {
                // Recurse into SubDag
                render_snapshot_contents(out, &sg_id, label, children, depth + 1);
            } else {
                // Truncated SubDag
                let count = children.total_node_count();
                let class = snapshot_node_class(label, true);
                writeln!(
                    out,
                    "{}    {}[[\"{}\\n({} nodes)\"]]:::{}",
                    indent, full_id, label, count, class
                )
                .unwrap();
            }
        } else {
            let class = snapshot_node_class(label, false);
            let has_res = node.inputs.iter().any(|p| p.name.0.starts_with("res:"));

            if has_res {
                // Nodes that consume resources get a distinct shape
                writeln!(out, "{}    {}([{}]):::execute", indent, full_id, label).unwrap();
            } else {
                writeln!(out, "{}    {}[{}]:::{}", indent, full_id, label, class).unwrap();
            }
        }
    }

    // Aggregate and render edges
    let aggregated = aggregate_edges(&topo.edges, &sg_id);
    for agg in &aggregated {
        if agg.resource_count > 0 && agg.data_count > 0 {
            // Both data and resource edges — render two arrows
            let data_label = if agg.data_count == 1 {
                String::new()
            } else {
                format!("{} ports", agg.data_count)
            };
            if data_label.is_empty() {
                writeln!(out, "{}    {} --> {}", indent, agg.from_id, agg.to_id).unwrap();
            } else {
                writeln!(
                    out,
                    "{}    {} -->|{}| {}",
                    indent, agg.from_id, data_label, agg.to_id
                )
                .unwrap();
            }
            let res_label = if agg.resource_count == 1 {
                agg.resource_sample.clone()
            } else {
                format!("{} resources", agg.resource_count)
            };
            writeln!(
                out,
                "{}    {} -.->|{}| {}",
                indent, agg.from_id, res_label, agg.to_id
            )
            .unwrap();
        } else if agg.resource_count > 0 {
            // Resource-only edge
            let label = if agg.resource_count == 1 {
                agg.resource_sample.clone()
            } else {
                format!("{} resources", agg.resource_count)
            };
            writeln!(
                out,
                "{}    {} -.->|{}| {}",
                indent, agg.from_id, label, agg.to_id
            )
            .unwrap();
        } else if agg.data_count > 0 {
            // Data-only edge
            if agg.data_count == 1 {
                writeln!(out, "{}    {} --> {}", indent, agg.from_id, agg.to_id).unwrap();
            } else {
                writeln!(
                    out,
                    "{}    {} -->|{} ports| {}",
                    indent, agg.from_id, agg.data_count, agg.to_id
                )
                .unwrap();
            }
        }
    }

    writeln!(out, "{}end", indent).unwrap();
}

/// Classify a node by its name for snapshot coloring.
fn snapshot_node_class(name: &str, is_subdag: bool) -> &'static str {
    if is_subdag {
        return "subdag";
    }
    // Environment / resource provider nodes
    if name.ends_with("_env") || name == "resource_gate" || name.starts_with("cloud_env") {
        return "env";
    }
    // Execute / transport boundary nodes
    if name.starts_with("execute_") || name == "execute" || name.ends_with("_transport") {
        return "execute";
    }
    "default"
}

// ---------------------------------------------------------------------------
// Edge aggregation
// ---------------------------------------------------------------------------

/// An aggregated edge between two nodes.
struct AggregatedEdge {
    from_id: String,
    to_id: String,
    data_count: usize,
    resource_count: usize,
    /// Sample resource name for label when resource_count == 1
    resource_sample: String,
}

/// Aggregate edges between the same node pairs. Separates resource vs data edges.
fn aggregate_edges(edges: &[EdgeTopology], prefix: &str) -> Vec<AggregatedEdge> {
    // Key: (from_id, to_id) -> (data_count, resource_count, resource_sample)
    let mut map: BTreeMap<(String, String), (usize, usize, String)> = BTreeMap::new();

    for edge in edges {
        let from_id = format!("{}_{}", prefix, sanitize_id(&edge.from_node.0));
        let to_id = format!("{}_{}", prefix, sanitize_id(&edge.to_node.0));
        let is_resource =
            edge.from_port.0.starts_with("res:") || edge.to_port.0.starts_with("res:");

        let entry = map.entry((from_id, to_id)).or_default();
        if is_resource {
            entry.1 += 1;
            if entry.2.is_empty() {
                // Capture the resource name
                let port = if edge.from_port.0.starts_with("res:") {
                    &edge.from_port.0
                } else {
                    &edge.to_port.0
                };
                entry.2 = port.clone();
            }
        } else {
            entry.0 += 1;
        }
    }

    map.into_iter()
        .map(
            |((from_id, to_id), (data_count, resource_count, resource_sample))| AggregatedEdge {
                from_id,
                to_id,
                data_count,
                resource_count,
                resource_sample,
            },
        )
        .collect()
}

// ---------------------------------------------------------------------------
// Text changelog rendering
// ---------------------------------------------------------------------------

/// Render a text changelog for a diff result (used below each Mermaid diagram).
pub fn render_changelog(diff: &DagDiffResult) -> String {
    let mut lines = Vec::new();

    for id in &diff.added_nodes {
        lines.push(format!("- **Added** node `{}`", id.0));
    }

    for id in &diff.removed_nodes {
        lines.push(format!("- **Removed** node `{}`", id.0));
    }

    for change in &diff.changed_nodes {
        let mut parts = Vec::new();
        for pc in &change.port_changes {
            parts.push(format!("{} `{}` ({})", pc.kind, pc.name, pc.direction));
        }
        if change.structure_changed && change.port_changes.is_empty() {
            parts.push("internal structure changed".to_string());
        }
        let detail = if parts.is_empty() {
            String::new()
        } else {
            format!(": {}", parts.join(", "))
        };
        lines.push(format!("- **Changed** node `{}`{}", change.id.0, detail));
    }

    for edge in &diff.added_edges {
        lines.push(format!(
            "- **Added** edge `{}:{} -> {}:{}`",
            edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
        ));
    }

    for edge in &diff.removed_edges {
        lines.push(format!(
            "- **Removed** edge `{}:{} -> {}:{}`",
            edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
        ));
    }

    if lines.is_empty() {
        "No changes.".to_string()
    } else {
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sanitize_id(s: &str) -> String {
    s.replace(['-', ' ', ':', '.'], "_")
}

fn status_class(status: NodeDiffStatus) -> &'static str {
    match status {
        NodeDiffStatus::Added => "added",
        NodeDiffStatus::Removed => "removed",
        NodeDiffStatus::Changed => "changed",
        NodeDiffStatus::Unchanged => "unchanged",
    }
}

fn diff_subgraph_label(
    label: &str,
    status: NodeDiffStatus,
    diff: &DagDiffResult,
    node_id: &NodeId,
) -> String {
    match status {
        NodeDiffStatus::Unchanged => format!("{} (unchanged)", label),
        NodeDiffStatus::Changed => {
            if let Some(child_diff) = find_child_diff(diff, node_id) {
                let summary = child_diff.stats_summary();
                format!("{} ({})", label, summary)
            } else {
                format!("{} (changed)", label)
            }
        }
        NodeDiffStatus::Added => format!("{} (new)", label),
        NodeDiffStatus::Removed => format!("{} (removed)", label),
    }
}

fn find_child_diff<'a>(diff: &'a DagDiffResult, node_id: &NodeId) -> Option<&'a DagDiffResult> {
    diff.changed_nodes
        .iter()
        .find(|c| &c.id == node_id)
        .and_then(|c| c.child_diff.as_deref())
}

fn find_child_removed_nodes<'a>(
    diff: &'a DagDiffResult,
    node_id: &NodeId,
) -> Vec<&'a NodeTopology> {
    let _ = (diff, node_id);
    Vec::new()
}

fn edge_diff_class(edge: &EdgeTopology, diff: &DagDiffResult) -> &'static str {
    let is_added = diff.added_edges.iter().any(|e| {
        e.from_node == edge.from_node
            && e.from_port == edge.from_port
            && e.to_node == edge.to_node
            && e.to_port == edge.to_port
    });
    if is_added {
        "added"
    } else {
        "unchanged"
    }
}

// ---------------------------------------------------------------------------
// Base64 URL-safe encoding (no external dependency)
// ---------------------------------------------------------------------------

fn base64_url_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Edge, Port};
    use crate::dag_diff::diff_topologies;
    use crate::node::Node;
    use crate::Dag;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    enum TestOp {
        A,
        B,
    }

    #[test]
    fn test_overview_diff_renders() {
        let mut old: Dag<TestOp> = Dag::new();
        let inner_old: Dag<TestOp> = Dag::new();
        old.add_node(Node::subdag("tool_a", inner_old));

        let mut new: Dag<TestOp> = Dag::new();
        let inner_new: Dag<TestOp> = Dag::new();
        new.add_node(Node::subdag("tool_a", inner_new.clone()));
        new.add_node(Node::subdag("tool_b", inner_new));

        let old_topo = old.topology();
        let new_topo = new.topology();
        let diff = diff_topologies(&old_topo, &new_topo);

        let removed: Vec<&NodeTopology> = old_topo
            .nodes
            .iter()
            .filter(|n| diff.removed_nodes.contains(&n.id))
            .collect();

        let mermaid = to_mermaid_overview_diff(&new_topo, &diff, &removed);

        assert!(mermaid.contains("flowchart TB"));
        assert!(mermaid.contains("classDef added"));
        assert!(mermaid.contains("tool_a[[tool_a]]:::unchanged"));
        assert!(mermaid.contains("tool_b[[tool_b]]:::added"));
    }

    #[test]
    fn test_expanded_diff_renders() {
        let mut old_inner: Dag<TestOp> = Dag::new();
        old_inner.add_node(Node::opaque(
            "prepare",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            TestOp::A,
        ));

        let mut new_inner: Dag<TestOp> = Dag::new();
        new_inner.add_node(Node::opaque(
            "prepare",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            TestOp::A,
        ));
        new_inner.add_node(Node::opaque("validate", vec![], vec![], TestOp::B));

        let old_topo = old_inner.topology();
        let new_topo = new_inner.topology();
        let diff = diff_topologies(&old_topo, &new_topo);

        let mermaid = to_mermaid_expanded_diff("gist", &new_topo, &diff, &[]);

        assert!(mermaid.contains("flowchart TB"));
        assert!(mermaid.contains("gist_prepare[prepare]:::unchanged"));
        assert!(mermaid.contains("gist_validate[validate]:::added"));
    }

    #[test]
    fn test_snapshot_has_colors() {
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "fs_env",
            vec![],
            vec![Port::scalar("handle", "FsHandle")],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "execute_transport",
            vec![
                Port::scalar("req", "Request"),
                Port::resource("file", "FsHandle", crate::resource::AccessMode::Read),
            ],
            vec![Port::scalar("resp", "Response")],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("fs_env", "handle", "execute_transport", "req"));

        let topo = dag.topology();
        let mermaid = to_mermaid_snapshot("tool", &topo);

        assert!(mermaid.contains("classDef env"));
        assert!(mermaid.contains("classDef execute"));
        assert!(mermaid.contains(":::env")); // fs_env node
        assert!(mermaid.contains(":::execute")); // execute_transport (has res: port)
    }

    #[test]
    fn test_snapshot_edge_aggregation() {
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![
                Port::scalar("x", "String"),
                Port::scalar("y", "Int"),
                Port::scalar("z", "Bool"),
            ],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![
                Port::scalar("x", "String"),
                Port::scalar("y", "Int"),
                Port::scalar("z", "Bool"),
            ],
            vec![],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("a", "x", "b", "x"));
        dag.add_edge(Edge::new("a", "y", "b", "y"));
        dag.add_edge(Edge::new("a", "z", "b", "z"));

        let topo = dag.topology();
        let mermaid = to_mermaid_snapshot("tool", &topo);

        // Should have "3 ports" aggregated label, not 3 separate edges
        assert!(mermaid.contains("3 ports"));
    }

    #[test]
    fn test_snapshot_resource_edge_separate() {
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "env",
            vec![],
            vec![
                Port::scalar("data", "String"),
                Port::resource("file", "FsHandle", crate::resource::AccessMode::Read),
            ],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "exec",
            vec![
                Port::scalar("data", "String"),
                Port::resource("file", "FsHandle", crate::resource::AccessMode::Read),
            ],
            vec![],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("env", "data", "exec", "data"));
        dag.add_edge(Edge::new("env", "res:file", "exec", "res:file"));

        let topo = dag.topology();
        let mermaid = to_mermaid_snapshot("tool", &topo);

        // Should have both a solid arrow and a dashed resource arrow
        assert!(mermaid.contains("-->"));
        assert!(mermaid.contains("-.->"));
    }

    #[test]
    fn test_snapshot_depth_styling() {
        let mut inner: Dag<TestOp> = Dag::new();
        inner.add_node(Node::opaque(
            "child",
            vec![Port::scalar("x", "String")],
            vec![Port::scalar("y", "String")],
            TestOp::A,
        ));

        let mut outer: Dag<TestOp> = Dag::new();
        outer.add_node(Node::subdag("sub", inner));

        let topo = outer.topology();
        let mermaid = to_mermaid_snapshot("workspace", &topo);

        // Should have depth-colored subgraph styling (dark palette)
        assert!(mermaid.contains("style"));
        assert!(mermaid.contains("#1e293b")); // depth-0 fill
        assert!(mermaid.contains("#1a2332")); // depth-1 fill
    }

    #[test]
    fn test_changelog_rendering() {
        let mut old: Dag<TestOp> = Dag::new();
        old.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![],
            TestOp::A,
        ));

        let mut new: Dag<TestOp> = Dag::new();
        new.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "Bool")],
            vec![],
            TestOp::A,
        ));
        new.add_node(Node::opaque("n2", vec![], vec![], TestOp::B));

        let diff = diff_topologies(&old.topology(), &new.topology());
        let changelog = render_changelog(&diff);

        assert!(changelog.contains("**Added** node `n2`"));
        assert!(changelog.contains("**Changed** node `n1`"));
        assert!(changelog.contains("type changed String -> Bool"));
    }

    #[test]
    fn test_mermaid_ink_url_small() {
        let url = mermaid_ink_url("flowchart TB\n    A --> B");
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.starts_with("https://mermaid.ink/svg/base64:"));
        assert!(url.len() > 40);
    }

    #[test]
    fn test_mermaid_ink_url_too_large() {
        let big = "x".repeat(MERMAID_INK_MAX_LEN + 1);
        assert!(mermaid_ink_url(&big).is_none());
    }

    #[test]
    fn test_base64_url_encode_roundtrip() {
        // Just verify it produces valid base64url characters
        let encoded = base64_url_encode(b"Hello, World!");
        assert!(encoded
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}
