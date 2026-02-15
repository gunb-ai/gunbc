//! Interactive HTML DAG viewer.
//!
//! Generates a self-contained HTML file using Cytoscape.js that provides:
//! - Left-to-right DAG layout (dagre)
//! - Click-to-expand/collapse SubDag nodes ("fog of war")
//! - Dark theme matching SemanticColor palette
//! - Zoom, pan, hover tooltips
//!
//! The HTML embeds the full `DagTopology` as JSON and converts it to
//! Cytoscape elements client-side.
//!
//! Uses `gunbc_ir::render_html_document` for the HTML boilerplate, composing
//! the `<head>` (CDN scripts + CSS) and `<body>` (UI + JS) from separate
//! template fragments.

use gunbc_ir::dag_topology::DagTopology;
use gunbc_ir::render_html_document;

/// Head fragment: CDN scripts + CSS.
const HEAD_FRAGMENT: &str = include_str!("viewer_head.html");

/// Body fragment: UI markup + JavaScript (contains `{TITLE}` and `{TOPOLOGY_JSON}` placeholders).
const BODY_FRAGMENT: &str = include_str!("viewer_body.html");

/// Render an interactive HTML viewer for the given topology.
///
/// Returns a complete, self-contained HTML string that can be written
/// to a file and opened in a browser.
pub fn render_viewer(title: &str, topo: &DagTopology) -> String {
    let json = serde_json::to_string(topo).unwrap_or_else(|e| {
        panic!("Failed to serialize DagTopology: {}", e);
    });

    let body = BODY_FRAGMENT
        .replacen("{TITLE}", title, 1)
        .replacen("{TOPOLOGY_JSON}", &json, 1);

    render_html_document(title, HEAD_FRAGMENT, &body)
}
