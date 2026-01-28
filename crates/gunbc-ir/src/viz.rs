use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::types::PatternDecision;
use crate::{Dag, Node, NodeBody, Port};

const CHAR_WIDTH: f32 = 7.0;
const PORT_LINE_HEIGHT: f32 = 14.0;
const CENTER_LINE_HEIGHT: f32 = 15.0;
const PORT_GAP: f32 = 6.0;
const COL_INNER_PAD: f32 = 8.0;
const COL_GAP: f32 = 12.0;
const MIN_COL_WIDTH: f32 = 90.0;
const MIN_CENTER_WIDTH: f32 = 140.0;
const MIN_EMPTY_COL_WIDTH: f32 = 30.0;
const NODE_GAP_Y: f32 = 24.0;
const LAYER_GAP_X: f32 = 80.0;
const MARGIN_X: f32 = 40.0;
const MARGIN_Y: f32 = 30.0;
const NODE_PAD_Y: f32 = 16.0;
const MAX_PORT_CHARS: usize = 28;
const MAX_GUARD_CHARS: usize = 36;
const MAX_CENTER_CHARS: usize = 32;

#[derive(Debug, Clone)]
struct RenderPort {
    name: String,
    lines: Vec<String>,
    height: f32,
    top_y: f32,
    anchor_y: f32,
}

#[derive(Debug, Clone)]
struct RenderNode {
    id: String,
    is_subdag: bool,
    inputs: Vec<RenderPort>,
    outputs: Vec<RenderPort>,
    center_lines: Vec<String>,
    in_col_width: f32,
    center_width: f32,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct LayoutItem {
    id: String,
    width: f32,
    height: f32,
}

fn decision_label(d: &PatternDecision) -> String {
    match d {
        PatternDecision::Instantiated => "Instantiated".to_string(),
        PatternDecision::NotApplicable { reason } => format!("NotApplicable: {}", reason),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    if max_chars <= 3 {
        return "...".to_string();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max_chars - 3 {
            break;
        }
        out.push(ch);
    }
    out.push_str("...");
    out
}

fn sanitize_id(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "id".to_string()
    } else {
        out
    }
}

fn port_lines(port: &Port, show_guards: bool) -> Vec<String> {
    let base = format!("{}:{}", port.name.0, port.type_id.0);
    let mut lines = vec![truncate(&base, MAX_PORT_CHARS)];
    if show_guards {
        if let Some(guard) = &port.guard {
            let guard_line = format!("if {}", guard);
            lines.push(truncate(&guard_line, MAX_GUARD_CHARS));
        }
    }
    lines
}

fn ports_render(ports: &[Port], show_guards: bool) -> (Vec<RenderPort>, f32, f32) {
    let mut rendered = Vec::new();
    let mut max_line_chars = 0usize;
    let mut total_height = 0.0;

    for p in ports {
        let lines = port_lines(p, show_guards);
        let height = lines.len() as f32 * PORT_LINE_HEIGHT;
        let mut line_max = 0usize;
        for line in &lines {
            line_max = line_max.max(line.chars().count());
        }
        max_line_chars = max_line_chars.max(line_max);
        rendered.push(RenderPort {
            name: p.name.0.clone(),
            lines,
            height,
            top_y: 0.0,
            anchor_y: 0.0,
        });
    }

    if !rendered.is_empty() {
        total_height = rendered.iter().map(|p| p.height).sum::<f32>()
            + PORT_GAP * (rendered.len().saturating_sub(1) as f32);
    }

    (rendered, max_line_chars as f32, total_height)
}

fn center_lines<T>(node: &Node<T>, is_subdag: bool) -> Vec<String> {
    let mut lines = vec![truncate(&node.id.0, MAX_CENTER_CHARS)];
    if is_subdag {
        lines.push("SubDag".to_string());
    }
    lines
}

fn layout_positions(items: &[LayoutItem], edges: &[(String, String)]) -> (BTreeMap<String, (f32, f32)>, f32, f32) {
    let mut ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
    ids.sort();

    let mut widths: HashMap<String, f32> = HashMap::new();
    let mut heights: HashMap<String, f32> = HashMap::new();
    for item in items {
        widths.insert(item.id.clone(), item.width);
        heights.insert(item.id.clone(), item.height);
    }

    let mut unique_edges: BTreeSet<(String, String)> = BTreeSet::new();
    for (from, to) in edges {
        if widths.contains_key(from) && widths.contains_key(to) && from != to {
            unique_edges.insert((from.clone(), to.clone()));
        }
    }

    let mut incoming: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut outgoing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();

    for id in &ids {
        indegree.insert(id.clone(), 0);
    }

    for (from, to) in &unique_edges {
        outgoing.entry(from.clone()).or_default().push(to.clone());
        incoming.entry(to.clone()).or_default().push(from.clone());
        if let Some(val) = indegree.get_mut(to) {
            *val += 1;
        }
    }

    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(id, &deg)| if deg == 0 { Some(id.clone()) } else { None })
        .collect();

    let mut order = Vec::new();
    let mut indegree_work = indegree.clone();
    while let Some(id) = ready.iter().next().cloned() {
        ready.remove(&id);
        order.push(id.clone());
        if let Some(nexts) = outgoing.get(&id) {
            for to in nexts {
                if let Some(val) = indegree_work.get_mut(to) {
                    *val = val.saturating_sub(1);
                    if *val == 0 {
                        ready.insert(to.clone());
                    }
                }
            }
        }
    }

    let has_cycle = order.len() != ids.len();
    if has_cycle {
        order = ids.clone();
    }

    let mut rank: BTreeMap<String, usize> = BTreeMap::new();
    if !has_cycle {
        for id in &order {
            let mut best = 0usize;
            if let Some(preds) = incoming.get(id) {
                for pred in preds {
                    let pred_rank = rank.get(pred).cloned().unwrap_or(0);
                    best = best.max(pred_rank + 1);
                }
            }
            rank.insert(id.clone(), best);
        }
    } else {
        for (i, id) in order.iter().enumerate() {
            rank.insert(id.clone(), i);
        }
    }

    let mut ranks: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for id in &ids {
        let r = rank.get(id).cloned().unwrap_or(0);
        ranks.entry(r).or_default().push(id.clone());
    }
    for ids in ranks.values_mut() {
        ids.sort();
    }

    let mut rank_width: BTreeMap<usize, f32> = BTreeMap::new();
    let mut rank_height: BTreeMap<usize, f32> = BTreeMap::new();

    for (r, nodes) in &ranks {
        let mut max_w: f32 = 0.0;
        let mut total_h = 0.0;
        for (i, id) in nodes.iter().enumerate() {
            let w = *widths.get(id).unwrap_or(&0.0);
            let h = *heights.get(id).unwrap_or(&0.0);
            max_w = max_w.max(w);
            total_h += h;
            if i + 1 < nodes.len() {
                total_h += NODE_GAP_Y;
            }
        }
        rank_width.insert(*r, max_w);
        rank_height.insert(*r, total_h);
    }

    let mut max_rank_height: f32 = 0.0;
    for h in rank_height.values() {
        max_rank_height = max_rank_height.max(*h);
    }

    let mut rank_x: BTreeMap<usize, f32> = BTreeMap::new();
    let mut x = MARGIN_X;
    for (i, (r, w)) in rank_width.iter().enumerate() {
        rank_x.insert(*r, x);
        x += *w;
        if i + 1 < rank_width.len() {
            x += LAYER_GAP_X;
        }
    }

    let mut positions: BTreeMap<String, (f32, f32)> = BTreeMap::new();
    for (r, nodes) in &ranks {
        let total_h = *rank_height.get(r).unwrap_or(&0.0);
        let mut y = MARGIN_Y + (max_rank_height - total_h) / 2.0;
        for id in nodes {
            let h = *heights.get(id).unwrap_or(&0.0);
            let x = *rank_x.get(r).unwrap_or(&MARGIN_X);
            positions.insert(id.clone(), (x, y));
            y += h + NODE_GAP_Y;
        }
    }

    let total_width = if rank_width.is_empty() {
        MARGIN_X * 2.0
    } else {
        let sum_widths: f32 = rank_width.values().sum();
        let gaps = if rank_width.len() > 1 {
            LAYER_GAP_X * (rank_width.len() as f32 - 1.0)
        } else {
            0.0
        };
        MARGIN_X + sum_widths + gaps + MARGIN_X
    };

    let total_height = max_rank_height + MARGIN_Y * 2.0;

    (positions, total_width, total_height)
}

fn build_render_nodes<T>(dag: &Dag<T>, show_guards: bool) -> Vec<RenderNode> {
    let mut nodes: Vec<&Node<T>> = dag.nodes.iter().collect();
    nodes.sort_by_key(|n| n.id.0.clone());

    let mut rendered = Vec::new();
    for n in nodes {
        let is_subdag = matches!(n.body, NodeBody::SubDag(_));
        let (mut inputs, in_chars, in_height) = ports_render(&n.inputs, show_guards);
        let (mut outputs, out_chars, out_height) = ports_render(&n.outputs, false);

        let center_lines = center_lines(n, is_subdag);
        let center_max_chars = center_lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as f32;

        let in_col_width = if inputs.is_empty() {
            MIN_EMPTY_COL_WIDTH
        } else {
            (in_chars * CHAR_WIDTH + COL_INNER_PAD * 2.0).max(MIN_COL_WIDTH)
        };
        let out_col_width = if outputs.is_empty() {
            MIN_EMPTY_COL_WIDTH
        } else {
            (out_chars * CHAR_WIDTH + COL_INNER_PAD * 2.0).max(MIN_COL_WIDTH)
        };
        let center_width = (center_max_chars * CHAR_WIDTH + COL_INNER_PAD * 2.0).max(MIN_CENTER_WIDTH);

        let center_height = center_lines.len() as f32 * CENTER_LINE_HEIGHT;
        let body_height = in_height.max(out_height).max(center_height);
        let height = body_height + NODE_PAD_Y * 2.0;

        let width = in_col_width + center_width + out_col_width + COL_GAP * 2.0;

        let in_start = if in_height > 0.0 {
            (body_height - in_height) / 2.0 + NODE_PAD_Y
        } else {
            (body_height / 2.0) + NODE_PAD_Y
        };
        let out_start = if out_height > 0.0 {
            (body_height - out_height) / 2.0 + NODE_PAD_Y
        } else {
            (body_height / 2.0) + NODE_PAD_Y
        };

        let mut y = in_start;
        for port in &mut inputs {
            port.top_y = y;
            port.anchor_y = y + port.height / 2.0;
            y += port.height + PORT_GAP;
        }

        let mut y = out_start;
        for port in &mut outputs {
            port.top_y = y;
            port.anchor_y = y + port.height / 2.0;
            y += port.height + PORT_GAP;
        }

        rendered.push(RenderNode {
            id: n.id.0.clone(),
            is_subdag,
            inputs,
            outputs,
            center_lines,
            in_col_width,
            center_width,
            width,
            height,
            x: 0.0,
            y: 0.0,
        });
    }

    rendered
}

fn svg_header(width: f32, height: f32, title: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\" role=\"img\" aria-label=\"{}\">\n",
        width,
        height,
        width,
        height,
        xml_escape(title)
    ));
    out.push_str("<defs>\n");
    out.push_str("  <linearGradient id=\"bg\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\">\n");
    out.push_str("    <stop offset=\"0%\" stop-color=\"#f8fafc\"/>\n");
    out.push_str("    <stop offset=\"100%\" stop-color=\"#eef2ff\"/>\n");
    out.push_str("  </linearGradient>\n");
    out.push_str("  <filter id=\"shadow\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\">\n");
    out.push_str("    <feDropShadow dx=\"0\" dy=\"4\" stdDeviation=\"4\" flood-color=\"#0f172a\" flood-opacity=\"0.12\"/>\n");
    out.push_str("  </filter>\n");
    out.push_str("  <marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"8\" markerHeight=\"8\" orient=\"auto-start-reverse\">\n");
    out.push_str("    <path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#64748b\"/>\n");
    out.push_str("  </marker>\n");
    out.push_str("</defs>\n");
    out.push_str("<style>\n");
    out.push_str("  @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;600&amp;family=Space+Mono&amp;display=swap');\n");
    out.push_str("  .bg { fill: url(#bg); }\n");
    out.push_str("  .node-rect { fill: #f8fafc; stroke: #334155; stroke-width: 1.2; rx: 10; filter: url(#shadow); }\n");
    out.push_str("  .node-rect.observe { stroke-dasharray: 5 4; }\n");
    out.push_str("  .node-rect.writes { stroke: #c2410c; stroke-width: 1.8; }\n");
    out.push_str("  .node-title { font-family: 'Space Grotesk', sans-serif; font-size: 13px; font-weight: 600; fill: #0f172a; }\n");
    out.push_str("  .node-sub { font-family: 'Space Grotesk', sans-serif; font-size: 11px; fill: #475569; }\n");
    out.push_str("  .port-text { font-family: 'Space Mono', monospace; font-size: 11px; fill: #334155; }\n");
    out.push_str("  .port-guard { font-family: 'Space Mono', monospace; font-size: 10px; fill: #64748b; }\n");
    out.push_str("  .edge { fill: none; stroke: #64748b; stroke-width: 1.3; marker-end: url(#arrow); }\n");
    out.push_str("</style>\n");
    out.push_str(&format!("<rect class=\"bg\" width=\"{:.0}\" height=\"{:.0}\"/>\n", width, height));
    out
}

fn svg_footer() -> String {
    "</svg>\n".to_string()
}

fn edge_path(x1: f32, y1: f32, x2: f32, y2: f32) -> String {
    let dx = (x2 - x1).abs().max(40.0) * 0.5;
    let c1x = x1 + dx;
    let c2x = x2 - dx;
    format!(
        "M {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}",
        x1, y1, c1x, y1, c2x, y2, x2, y2
    )
}

/// Node-level SVG: ports + types. SubDAG nodes are shown as a single node by default.
pub fn dag_to_svg<T>(dag: &Dag<T>, show_guards: bool) -> String {
    let mut nodes = build_render_nodes(dag, show_guards);

    let mut node_edges: Vec<(String, String)> = Vec::new();
    for e in &dag.edges {
        node_edges.push((e.from_node.0.clone(), e.to_node.0.clone()));
    }

    let layout_items: Vec<LayoutItem> = nodes
        .iter()
        .map(|n| LayoutItem {
            id: n.id.clone(),
            width: n.width,
            height: n.height,
        })
        .collect();

    let (positions, width, height) = layout_positions(&layout_items, &node_edges);

    for node in &mut nodes {
        if let Some((x, y)) = positions.get(&node.id) {
            node.x = *x;
            node.y = *y;
        }
    }

    let mut port_positions: HashMap<(String, String, bool), (f32, f32)> = HashMap::new();
    for node in &nodes {
        let in_x = node.x;
        let out_x = node.x + node.width;
        for port in &node.inputs {
            let key = (node.id.clone(), port.name.clone(), false);
            port_positions.insert(key, (in_x, node.y + port.anchor_y));
        }
        for port in &node.outputs {
            let key = (node.id.clone(), port.name.clone(), true);
            port_positions.insert(key, (out_x, node.y + port.anchor_y));
        }
    }

    let mut edges = dag.edges.clone();
    edges.sort_by_key(|e| (
        e.from_node.0.clone(),
        e.from_port.0.clone(),
        e.to_node.0.clone(),
        e.to_port.0.clone(),
    ));

    let mut out = svg_header(width, height, "gunbc dag");

    out.push_str("<g class=\"edges\">\n");
    for e in &edges {
        let from_key = (e.from_node.0.clone(), e.from_port.0.clone(), true);
        let to_key = (e.to_node.0.clone(), e.to_port.0.clone(), false);
        if let (Some((x1, y1)), Some((x2, y2))) = (port_positions.get(&from_key), port_positions.get(&to_key)) {
            let path = edge_path(*x1, *y1, *x2, *y2);
            out.push_str(&format!("  <path class=\"edge\" d=\"{}\"/>\n", path));
        }
    }
    out.push_str("</g>\n");

    out.push_str("<g class=\"nodes\">\n");
    for node in &nodes {
        // All nodes are pure by default. SubDags get a different style.
        let rect_class = if node.is_subdag { "node-rect subdag" } else { "node-rect" };

        out.push_str(&format!(
            "  <g id=\"node_{}\" transform=\"translate({:.1} {:.1})\">\n",
            sanitize_id(&node.id),
            node.x,
            node.y
        ));
        out.push_str(&format!(
            "    <rect class=\"{}\" width=\"{:.1}\" height=\"{:.1}\"/>\n",
            rect_class, node.width, node.height
        ));

        let in_x = COL_INNER_PAD;
        let center_x = node.in_col_width + COL_GAP;
        let out_x = node.in_col_width + COL_GAP + node.center_width + COL_GAP + COL_INNER_PAD;

        for port in &node.inputs {
            let mut y = port.top_y;
            for (i, line) in port.lines.iter().enumerate() {
                let class = if i == 0 { "port-text" } else { "port-guard" };
                out.push_str(&format!(
                    "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" dominant-baseline=\"hanging\">{}</text>\n",
                    class,
                    in_x,
                    y,
                    xml_escape(line)
                ));
                y += PORT_LINE_HEIGHT;
            }
        }

        for port in &node.outputs {
            let mut y = port.top_y;
            for (i, line) in port.lines.iter().enumerate() {
                let class = if i == 0 { "port-text" } else { "port-guard" };
                out.push_str(&format!(
                    "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" dominant-baseline=\"hanging\">{}</text>\n",
                    class,
                    out_x,
                    y,
                    xml_escape(line)
                ));
                y += PORT_LINE_HEIGHT;
            }
        }

        let center_mid_x = center_x + node.center_width / 2.0;
        let center_start_y = (node.height - (node.center_lines.len() as f32 * CENTER_LINE_HEIGHT)) / 2.0;
        for (i, line) in node.center_lines.iter().enumerate() {
            let class = if i == 0 { "node-title" } else { "node-sub" };
            let y = center_start_y + (i as f32) * CENTER_LINE_HEIGHT;
            out.push_str(&format!(
                "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"hanging\">{}</text>\n",
                class,
                center_mid_x,
                y,
                xml_escape(line)
            ));
        }

        if node.is_subdag {
            out.push_str(&format!(
                "    <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"none\" stroke=\"#6366f1\" stroke-dasharray=\"4 3\" rx=\"8\"/>\n",
                4.0,
                4.0,
                node.width - 8.0,
                node.height - 8.0
            ));
        }

        out.push_str("  </g>\n");
    }
    out.push_str("</g>\n");

    out.push_str(&svg_footer());
    out
}

/// Derive a "group" name from a node ID.
/// If the ID contains "/", returns the prefix before the first "/".
/// Otherwise returns the full ID.
fn derive_group(node_id: &str) -> String {
    if let Some(pos) = node_id.find('/') {
        node_id[..pos].to_string()
    } else {
        node_id.to_string()
    }
}

/// Tool-level SVG: compress node graph into groups; label with pattern decisions.
pub fn tools_to_svg<T>(dag: &Dag<T>) -> String {
    let mut tool_nodes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut tool_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let mut node_tool: HashMap<String, String> = HashMap::new();

    for n in &dag.nodes {
        let tool = derive_group(&n.id.0);
        tool_nodes.entry(tool.clone()).or_default();
        node_tool.insert(n.id.0.clone(), tool);
    }

    for e in &dag.edges {
        let from_tool = node_tool.get(&e.from_node.0).cloned();
        let to_tool = node_tool.get(&e.to_node.0).cloned();
        if let (Some(from_t), Some(to_t)) = (from_tool, to_tool) {
            if from_t != to_t {
                tool_edges.insert((from_t, to_t));
            }
        }
    }

    for pd in &dag.metadata.pattern_decisions {
        tool_nodes
            .entry(pd.node.0.clone())
            .or_default()
            .push(format!("{}={}", pd.pattern, decision_label(&pd.decision)));
    }
    for decisions in tool_nodes.values_mut() {
        decisions.sort();
    }

    let mut render_nodes: Vec<(String, Vec<String>, f32, f32)> = Vec::new();
    for (tool, decisions) in &tool_nodes {
        let mut lines = vec![truncate(tool, MAX_CENTER_CHARS)];
        for d in decisions {
            lines.push(truncate(d, MAX_CENTER_CHARS));
        }
        let max_chars = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as f32;
        let width = (max_chars * CHAR_WIDTH + COL_INNER_PAD * 2.0).max(MIN_CENTER_WIDTH);
        let height = (lines.len() as f32 * CENTER_LINE_HEIGHT).max(30.0) + NODE_PAD_Y * 2.0;
        render_nodes.push((tool.clone(), lines, width, height));
    }

    let layout_items: Vec<LayoutItem> = render_nodes
        .iter()
        .map(|(id, _lines, width, height)| LayoutItem {
            id: id.clone(),
            width: *width,
            height: *height,
        })
        .collect();

    let edge_list: Vec<(String, String)> = tool_edges.into_iter().collect();
    let mut size_map: HashMap<String, (f32, f32)> = HashMap::new();
    for (id, _lines, w, h) in &render_nodes {
        size_map.insert(id.clone(), (*w, *h));
    }
    let (positions, width, height) = layout_positions(&layout_items, &edge_list);

    let mut out = svg_header(width, height, "gunbc tools");

    out.push_str("<g class=\"edges\">\n");
    for (from, to) in &edge_list {
        if let (Some((x1, y1)), Some((x2, y2))) = (positions.get(from), positions.get(to)) {
            let (from_w, from_h) = size_map.get(from).cloned().unwrap_or((0.0, 0.0));
            let (_to_w, to_h) = size_map.get(to).cloned().unwrap_or((0.0, 0.0));
            let start_x = x1 + from_w;
            let start_y = y1 + from_h / 2.0;
            let end_x = *x2;
            let end_y = *y2 + to_h / 2.0;
            let path = edge_path(start_x, start_y, end_x, end_y);
            out.push_str(&format!("  <path class=\"edge\" d=\"{}\"/>\n", path));
        }
    }
    out.push_str("</g>\n");

    out.push_str("<g class=\"nodes\">\n");
    for (tool, lines, w, h) in &render_nodes {
        if let Some((x, y)) = positions.get(tool) {
            out.push_str(&format!(
                "  <g id=\"tool_{}\" transform=\"translate({:.1} {:.1})\">\n",
                sanitize_id(tool),
                x,
                y
            ));
            out.push_str(&format!(
                "    <rect class=\"node-rect\" width=\"{:.1}\" height=\"{:.1}\"/>\n",
                w, h
            ));
            let mid_x = w / 2.0;
            let start_y = (h - (lines.len() as f32 * CENTER_LINE_HEIGHT)) / 2.0;
            for (i, line) in lines.iter().enumerate() {
                let class = if i == 0 { "node-title" } else { "node-sub" };
                let ty = start_y + (i as f32) * CENTER_LINE_HEIGHT;
                out.push_str(&format!(
                    "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"hanging\">{}</text>\n",
                    class,
                    mid_x,
                    ty,
                    xml_escape(line)
                ));
            }
            out.push_str("  </g>\n");
        }
    }
    out.push_str("</g>\n");

    out.push_str(&svg_footer());
    out
}
