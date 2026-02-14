//! Pure frame building — produces [`Frame`] IR from progress state.
//!
//! All functions in this module are pure: no I/O, no `Write` trait.
//! The caller passes the resulting [`Frame`] to a
//! [`FrameWriter`](super::frame_write::FrameWriter) for actual output.

use crate::progress::{DagPhase, DagProgress, EdgeState, GroupProgress, NodeState};
use crate::render::RenderMode;
use gunbc_ir::layout::DagLayout;
use gunbc_ir::render_ir::{CursorAction, Frame, Line, Span, SpanStyle};
use gunbc_ir::symbols::{SemanticColor, SymbolId, SymbolSet, Tier};
use gunbc_ir::NodeId;
use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build a complete frame from progress state (pure — no I/O).
pub fn build_frame(
    progress: &DagProgress,
    layout: &DagLayout,
    mode: RenderMode,
    spinner_frame: &str,
    tier: Tier,
    symbol_set: &'static SymbolSet,
) -> Frame {
    let mut lines = Vec::new();

    match mode {
        RenderMode::Compact => {
            lines.push(build_compact_line(
                progress,
                spinner_frame,
                symbol_set,
                tier,
            ));
        }
        RenderMode::Standard | RenderMode::Dynamic => {
            lines.push(build_dag_header(progress, spinner_frame, tier, symbol_set));
            // No DAG grid — show only the grouped stage panel (gunb.ai style).
            lines.extend(build_legend_lines(
                progress,
                layout,
                spinner_frame,
                symbol_set,
                tier,
            ));
            if let Some(footer) = build_footer_line(progress, symbol_set, tier) {
                lines.push(footer);
            }
        }
    }

    Frame {
        lines,
        cursor_action: CursorAction::Overwrite,
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

/// Build the DAG header line (e.g., "◐ Running: lint [1.2s]").
fn build_dag_header(
    progress: &DagProgress,
    spinner_frame: &str,
    tier: Tier,
    symbol_set: &'static SymbolSet,
) -> Line {
    let elapsed = format_duration(progress.elapsed());
    match &progress.phase {
        DagPhase::NotStarted => Line::new(vec![
            symbol_span(SymbolId::DagNotStarted, symbol_set, tier),
            Span::plain(" DAG pending"),
        ]),
        DagPhase::Running { current_node } => {
            let label = progress
                .snapshot
                .labels
                .get(current_node)
                .map(|s| s.as_str())
                .unwrap_or(&current_node.0);
            let icon = if spinner_frame.is_empty() {
                symbol_span(SymbolId::DagRunning, symbol_set, tier)
            } else {
                Span::styled(
                    spinner_frame.to_string(),
                    SpanStyle {
                        color: Some(SemanticColor::Active),
                        ..Default::default()
                    },
                )
            };
            Line::new(vec![
                icon,
                Span::plain(format!(" Running: {} [{}]", label, elapsed)),
            ])
        }
        DagPhase::Completed { elapsed: e } => {
            let icon_spans = match tier {
                Tier::Emoji => {
                    let animal = random_animal(*e);
                    vec![Span::styled(
                        animal.to_string(),
                        SpanStyle {
                            color: Some(SemanticColor::Success),
                            ..Default::default()
                        },
                    )]
                }
                _ => vec![symbol_span(SymbolId::DagCompleted, symbol_set, tier)],
            };
            let mut spans = icon_spans;
            spans.push(Span::plain(format!(" Completed [{}]", format_duration(*e))));
            Line::new(spans)
        }
        DagPhase::Failed { node, error } => {
            let label = progress
                .snapshot
                .labels
                .get(node)
                .map(|s| s.as_str())
                .unwrap_or(&node.0);
            let icon_spans = match tier {
                Tier::Emoji => {
                    vec![Span::styled(
                        "\u{274C}".to_string(), // ❌
                        SpanStyle {
                            color: Some(SemanticColor::Error),
                            ..Default::default()
                        },
                    )]
                }
                _ => vec![symbol_span(SymbolId::DagFailed, symbol_set, tier)],
            };
            let mut spans = icon_spans;
            spans.push(Span::plain(format!(
                " Failed at {}: {} [{}]",
                label, error, elapsed
            )));
            Line::new(spans)
        }
    }
}

// ---------------------------------------------------------------------------
// Standard mode lines
// ---------------------------------------------------------------------------

/// Build the DAG grid lines for standard/dynamic mode.
fn build_standard_lines(
    progress: &DagProgress,
    layout: &DagLayout,
    _symbol_set: &'static SymbolSet,
    _tier: Tier,
) -> Vec<Line> {
    if layout.levels.is_empty() {
        return Vec::new();
    }

    let num_cols = layout.levels.len();
    let num_tracks = layout.tracks;
    let col_w = layout.box_width as usize;
    let gap_w = layout.gap_width as usize;

    let visible_levels = &layout.overflow.visible_levels;
    let start_col = visible_levels.start;
    let end_col = visible_levels.end.min(num_cols);

    let mut lines = Vec::new();

    for track in 0..num_tracks {
        let mut spans = Vec::new();
        let last_col = end_col.saturating_sub(1);

        for (col, connector_row) in layout
            .connectors
            .iter()
            .enumerate()
            .take(last_col)
            .skip(start_col)
        {
            if let Some(node) = layout.grid.get(&(track, col)) {
                let state = progress
                    .nodes
                    .get(node)
                    .map(|np| np.state)
                    .unwrap_or(NodeState::Pending);
                let label = layout
                    .node_letters
                    .get(node)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                spans.extend(node_box_spans(label, state));
                let vis_w = display_width(label) + 2; // [label]
                let pad = col_w.saturating_sub(vis_w);
                if pad > 0 {
                    spans.push(Span::plain(" ".repeat(pad)));
                }
            } else {
                spans.push(Span::plain(" ".repeat(col_w)));
            }

            let cell = connector_row.get(track);
            let glyph = cell.map(|c| c.glyph).unwrap_or(' ');
            let conn = pad_connector(&format!(" {} ", glyph), gap_w);
            let edge_state = cell
                .map(|c| connector_cell_state(&c.edges, progress))
                .unwrap_or(EdgeState::Idle);
            spans.push(connector_span(&conn, edge_state));
        }

        if end_col > start_col {
            let col = last_col;
            if let Some(node) = layout.grid.get(&(track, col)) {
                let state = progress
                    .nodes
                    .get(node)
                    .map(|np| np.state)
                    .unwrap_or(NodeState::Pending);
                let label = layout
                    .node_letters
                    .get(node)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                spans.extend(node_box_spans(label, state));
                let vis_w = display_width(label) + 2;
                let pad = col_w.saturating_sub(vis_w);
                if pad > 0 {
                    spans.push(Span::plain(" ".repeat(pad)));
                }
            } else {
                spans.push(Span::plain(" ".repeat(col_w)));
            }
        }

        // Check if any spans contain non-whitespace
        let has_content = spans
            .iter()
            .any(|s| s.text.chars().any(|c| !c.is_whitespace()));
        if has_content {
            // Prepend 2-space indent
            spans.insert(0, Span::plain("  "));
            lines.push(Line::new(spans));
        }
    }

    lines
}

// ---------------------------------------------------------------------------
// Compact mode
// ---------------------------------------------------------------------------

/// Build a single-line compact summary.
fn build_compact_line(
    progress: &DagProgress,
    spinner_frame: &str,
    symbol_set: &'static SymbolSet,
    tier: Tier,
) -> Line {
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

    let status_span = match &progress.phase {
        DagPhase::NotStarted => symbol_span(SymbolId::DagNotStarted, symbol_set, tier),
        DagPhase::Running { .. } => Span::plain(spinner_frame.to_string()),
        DagPhase::Completed { .. } => symbol_span(SymbolId::DagCompleted, symbol_set, tier),
        DagPhase::Failed { .. } => symbol_span(SymbolId::DagFailed, symbol_set, tier),
    };

    let elapsed_str = format_duration(progress.elapsed());

    let text = if failed > 0 {
        format!(
            " {}/{} done, {} failed, {} running [{}]",
            completed, total, failed, running, elapsed_str
        )
    } else {
        format!(
            " {}/{} done, {} running [{}]",
            completed, total, running, elapsed_str
        )
    };

    Line::new(vec![status_span, Span::plain(text)])
}

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Build lower panel lines under the DAG grid.
///
/// When groups are present, render a stage panel with per-stage progress bars.
/// Otherwise, render the legacy per-node legend.
fn build_legend_lines(
    progress: &DagProgress,
    layout: &DagLayout,
    spinner_frame: &str,
    _symbol_set: &'static SymbolSet,
    _tier: Tier,
) -> Vec<Line> {
    const LEGEND_LINES: usize = 3;
    const STAGE_PANEL_MAX_GROUPS: usize = 12;
    const STAGE_PANEL_MAX_LINES: usize = 20;

    if !progress.snapshot.groups.is_empty() {
        return build_grouped_stage_panel(
            progress,
            spinner_frame,
            STAGE_PANEL_MAX_GROUPS,
            STAGE_PANEL_MAX_LINES,
        );
    }

    let mut active_entries: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();
    let mut done_entries: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();

    for level in &layout.levels {
        for node in level {
            let np = progress.nodes.get(node);
            let state = np.map(|n| n.state).unwrap_or(NodeState::Pending);
            let short = layout
                .node_letters
                .get(node)
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            let label = full_label(node, &progress.snapshot.labels);
            let elapsed = np.and_then(|n| n.elapsed.or_else(|| n.start_time.map(|t| t.elapsed())));

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

    let mut legend: Vec<(NodeState, String, String, Option<Duration>)> = Vec::new();
    legend.extend(active_entries.into_iter().take(LEGEND_LINES));
    let remaining = LEGEND_LINES.saturating_sub(legend.len());
    if remaining > 0 {
        legend.extend(done_entries.into_iter().rev().take(remaining));
    }

    let mut lines = Vec::new();
    let visible = legend.len().min(LEGEND_LINES);

    for (state, short, label, elapsed) in legend.iter().take(visible) {
        let sym = legend_char(*state);
        let icon_color = state_color(*state);
        let name_color = match state {
            NodeState::Completed | NodeState::Intercepted | NodeState::Skipped => {
                Some(SemanticColor::Dim)
            }
            _ => None, // Running/Failed use default terminal color
        };
        let time_str = elapsed
            .map(|d| format!(" [{}]", format_duration(d)))
            .unwrap_or_default();
        let text = if short == label {
            format!(" {}{}", label, time_str)
        } else {
            format!(" {}: {}{}", short, label, time_str)
        };

        lines.push(Line::new(vec![
            Span::plain("  "),
            Span::styled(
                sym.to_string(),
                SpanStyle {
                    color: Some(icon_color),
                    ..Default::default()
                },
            ),
            Span::styled(
                text,
                SpanStyle {
                    color: name_color,
                    ..Default::default()
                },
            ),
        ]));
    }

    // Pad remaining slots with empty lines to prevent jitter
    for _ in visible..LEGEND_LINES {
        lines.push(Line::new(vec![Span::plain("")]));
    }

    lines
}

/// Build a grouped stage panel with progress bars and inline expansion.
///
/// Matches `gunb.ai`'s group rendering style:
/// `spinner › GroupName [done/total] (running-task-name)`
///
/// Failed groups always expand. Running groups auto-expand when they contain
/// long-running nodes.
fn build_grouped_stage_panel(
    progress: &DagProgress,
    spinner_frame: &str,
    max_groups: usize,
    max_lines: usize,
) -> Vec<Line> {
    /// Auto-expand groups after 30s (matches gunb.ai threshold).
    const LONG_RUNNING_EXPAND_AFTER: Duration = Duration::from_secs(30);
    const DETAILS_PER_GROUP: usize = 4;

    let mut lines: Vec<Line> = Vec::new();

    let mut rows: Vec<(usize, GroupProgress)> = progress
        .snapshot
        .groups
        .iter()
        .enumerate()
        .map(|(idx, group)| (idx, group.progress(progress)))
        .collect();
    if rows.is_empty() {
        while lines.len() < max_lines {
            lines.push(Line::new(vec![Span::plain("")]));
        }
        return lines;
    }

    // Show all groups in topology order (up to max_groups).
    // Active/failing groups are always included, then fill remaining slots.
    let mut selected: Vec<(usize, GroupProgress)> = rows
        .iter()
        .filter(|(_, gp)| gp.is_failed() || gp.running > 0)
        .cloned()
        .collect();
    for row in rows.drain(..) {
        if selected.len() >= max_groups {
            break;
        }
        if selected.iter().any(|(idx, _)| *idx == row.0) {
            continue;
        }
        selected.push(row);
    }
    selected.truncate(max_groups);

    let is_final = spinner_frame.is_empty();

    for (group_idx, gp) in selected {
        if lines.len() >= max_lines {
            break;
        }
        let group = &progress.snapshot.groups[group_idx];
        let done = gp.completed + gp.failed + gp.skipped;

        // Icon color: colored icon, but name uses a SEPARATE style (matching gunb.ai).
        // - Failed:    red ✘,   default name
        // - Running:   orange spinner, default name
        // - Completed: green ✔, dim name
        // - Pending:   dim ○,   dim name
        let (icon, icon_color, name_color) = if gp.is_failed() {
            ("\u{2718}".to_string(), SemanticColor::Error, None) // ✘, red icon, default name
        } else if gp.running > 0 && !is_final {
            (spinner_frame.to_string(), SemanticColor::Active, None) // spinner, orange, default name
        } else if gp.running > 0 || gp.is_done() {
            ("\u{2714}".to_string(), SemanticColor::Success, Some(SemanticColor::Dim)) // ✔, green, dim name
        } else {
            ("\u{25CB}".to_string(), SemanticColor::Dim, Some(SemanticColor::Dim)) // ○, dim, dim name
        };

        // Build suffix with running task names or failed task names inline
        let suffix = if gp.failed > 0 {
            let failed_names = group_failed_task_names(progress, group_idx);
            if failed_names.is_empty() {
                format!(" ({} failed)", gp.failed)
            } else {
                format!(" [failed: {}]", failed_names.join(", "))
            }
        } else if gp.running > 0 {
            let running_names = group_running_task_names(progress, group_idx);
            if running_names.is_empty() {
                format!(" ({} running)", gp.running)
            } else if running_names.len() <= 2 {
                format!(" ({})", running_names.join(", "))
            } else {
                format!(" ({}, +{} more)", running_names[0], running_names.len() - 1)
            }
        } else {
            String::new()
        };

        // Main group line: icon › GroupName [done/total] suffix
        // No progress bar — just the count, like gunb.ai.
        let mut spans = vec![
            Span::plain("  "),
            Span::styled(
                icon,
                SpanStyle {
                    color: Some(icon_color),
                    ..Default::default()
                },
            ),
            Span::plain(" \u{203A} "), // › (uncolored, neutral separator)
            Span::styled(
                format!("{:<18} [{}/{}]", group.name, done, gp.total),
                SpanStyle {
                    color: name_color,
                    ..Default::default()
                },
            ),
        ];
        if !suffix.is_empty() {
            let suffix_color = if gp.failed > 0 {
                SemanticColor::Error
            } else {
                SemanticColor::Dim
            };
            spans.push(Span::styled(
                suffix,
                SpanStyle {
                    color: Some(suffix_color),
                    ..Default::default()
                },
            ));
        }
        lines.push(Line::new(spans));

        let expand = gp.is_failed()
            || group_has_long_running_node(progress, group_idx, LONG_RUNNING_EXPAND_AFTER);
        if !expand {
            continue;
        }

        for detail_line in grouped_detail_lines(progress, group_idx, DETAILS_PER_GROUP) {
            if lines.len() >= max_lines {
                break;
            }
            lines.push(detail_line);
        }
    }

    // Keep fixed height to reduce flicker while stages update.
    while lines.len() < max_lines {
        lines.push(Line::new(vec![Span::plain("")]));
    }
    lines
}

/// Get the names of running tasks in a group.
fn group_running_task_names(progress: &DagProgress, group_idx: usize) -> Vec<String> {
    let group = &progress.snapshot.groups[group_idx];
    group
        .node_ids
        .iter()
        .filter(|node_id| {
            progress
                .nodes
                .get(*node_id)
                .map(|np| np.state == NodeState::Running)
                .unwrap_or(false)
        })
        .map(|node_id| full_label(node_id, &progress.snapshot.labels))
        .collect()
}

/// Get the names of failed tasks in a group.
fn group_failed_task_names(progress: &DagProgress, group_idx: usize) -> Vec<String> {
    let group = &progress.snapshot.groups[group_idx];
    group
        .node_ids
        .iter()
        .filter(|node_id| {
            progress
                .nodes
                .get(*node_id)
                .map(|np| np.state == NodeState::Failed)
                .unwrap_or(false)
        })
        .map(|node_id| full_label(node_id, &progress.snapshot.labels))
        .collect()
}

fn grouped_detail_lines(progress: &DagProgress, group_idx: usize, limit: usize) -> Vec<Line> {
    /// Maximum pending tasks to show before truncating.
    const MAX_PENDING_DISPLAY: usize = 4;

    let group = &progress.snapshot.groups[group_idx];
    let mut entries: Vec<(NodeState, String, Option<Duration>)> = Vec::new();
    let mut pending_entries: Vec<String> = Vec::new();

    for node_id in &group.node_ids {
        let Some(np) = progress.nodes.get(node_id) else {
            continue;
        };

        let label = full_label(node_id, &progress.snapshot.labels);
        match np.state {
            NodeState::Failed | NodeState::Running => {
                let elapsed = np.elapsed.or_else(|| np.start_time.map(|t| t.elapsed()));
                entries.push((np.state, label, elapsed));
            }
            NodeState::Pending => {
                pending_entries.push(label);
            }
            _ => {}
        }
    }

    // Sort: failed first, then running
    entries.sort_by_key(|(state, _, _)| match state {
        NodeState::Failed => 0,
        NodeState::Running => 1,
        _ => 2,
    });

    let mut lines = Vec::new();

    // Show failed and running entries with split icon/name coloring (gunb.ai style):
    // Icon gets the status color; name is default for running/failed, dim for completed.
    for (state, label, elapsed) in entries.iter().take(limit) {
        let time_str = elapsed
            .map(|d| format!(" ({})", format_duration(d)))
            .unwrap_or_default();
        let name_color = match state {
            NodeState::Completed | NodeState::Intercepted | NodeState::Skipped => {
                Some(SemanticColor::Dim)
            }
            _ => None, // Running / Failed use default terminal color
        };
        lines.push(Line::new(vec![
            Span::plain("    "),
            Span::styled(
                legend_char(*state).to_string(),
                SpanStyle {
                    color: Some(state_color(*state)),
                    ..Default::default()
                },
            ),
            Span::styled(
                format!(" {}{}", label, time_str),
                SpanStyle {
                    color: name_color,
                    ..Default::default()
                },
            ),
        ]));
    }

    if entries.len() > limit {
        lines.push(Line::new(vec![
            Span::plain("    "),
            Span::styled(
                format!("… {} more active", entries.len() - limit),
                SpanStyle {
                    color: Some(SemanticColor::Dim),
                    ..Default::default()
                },
            ),
        ]));
    }

    // Show pending tasks (up to MAX_PENDING_DISPLAY, then truncate)
    let pending_to_show = pending_entries.len().min(MAX_PENDING_DISPLAY);
    for label in pending_entries.iter().take(pending_to_show) {
        if lines.len() >= limit + MAX_PENDING_DISPLAY {
            break;
        }
        lines.push(Line::new(vec![
            Span::plain("    "),
            Span::styled(
                legend_char(NodeState::Pending).to_string(),
                SpanStyle {
                    color: Some(SemanticColor::Dim),
                    ..Default::default()
                },
            ),
            Span::styled(
                format!(" {}", label),
                SpanStyle {
                    color: Some(SemanticColor::Dim),
                    ..Default::default()
                },
            ),
        ]));
    }

    if pending_entries.len() > MAX_PENDING_DISPLAY {
        lines.push(Line::new(vec![
            Span::plain("    "),
            Span::styled(
                format!(
                    "… and {} more pending",
                    pending_entries.len() - MAX_PENDING_DISPLAY
                ),
                SpanStyle {
                    color: Some(SemanticColor::Dim),
                    ..Default::default()
                },
            ),
        ]));
    }

    lines
}

fn group_has_long_running_node(
    progress: &DagProgress,
    group_idx: usize,
    threshold: Duration,
) -> bool {
    let group = &progress.snapshot.groups[group_idx];
    group.node_ids.iter().any(|node_id| {
        let Some(np) = progress.nodes.get(node_id) else {
            return false;
        };
        if np.state != NodeState::Running {
            return false;
        }
        np.start_time
            .map(|start| start.elapsed() >= threshold)
            .unwrap_or(false)
    })
}

fn stage_progress_bar(done: usize, total: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if total == 0 {
        return "-".repeat(width);
    }
    let filled = ((done * width) + (total / 2)) / total;
    format!("{}{}", "#".repeat(filled), "-".repeat(width - filled))
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

/// Build the footer summary line (only for completed/failed phases).
fn build_footer_line(
    progress: &DagProgress,
    symbol_set: &'static SymbolSet,
    tier: Tier,
) -> Option<Line> {
    if !matches!(
        progress.phase,
        DagPhase::Completed { .. } | DagPhase::Failed { .. }
    ) {
        return None;
    }

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

    let mut parts = Vec::new();
    if completed > 0 {
        let sym = symbol_set.resolve_tier(SymbolId::Success, tier);
        parts.push(format!("{} {} completed", sym, completed));
    }
    if failed > 0 {
        let sym = symbol_set.resolve_tier(SymbolId::Failure, tier);
        parts.push(format!("{} {} failed", sym, failed));
    }
    if skipped > 0 {
        parts.push(format!("{} skipped", skipped));
    }

    let elapsed = match &progress.phase {
        DagPhase::Completed { elapsed } => format_duration(*elapsed),
        _ => format_duration(progress.elapsed()),
    };

    Some(Line::new(vec![Span::plain(format!(
        "{} — {}/{} [{}]",
        parts.join(", "),
        completed + failed + skipped,
        total,
        elapsed
    ))]))
}

// ---------------------------------------------------------------------------
// Span constructors
// ---------------------------------------------------------------------------

/// Create a span that renders a symbol with its semantic color.
fn symbol_span(id: SymbolId, symbol_set: &'static SymbolSet, _tier: Tier) -> Span {
    let sym = symbol_set.get(id);
    Span::styled(
        String::new(),
        SpanStyle {
            symbol: Some(id),
            color: Some(sym.color),
            ..Default::default()
        },
    )
}

/// Create spans for a node box: `[` + styled label + `]`.
fn node_box_spans(label: &str, state: NodeState) -> Vec<Span> {
    let color = state_color(state);
    let bold = matches!(
        state,
        NodeState::Completed | NodeState::Failed | NodeState::Running | NodeState::Intercepted
    );

    vec![Span::styled(
        format!("[{}]", label),
        SpanStyle {
            color: Some(color),
            bold,
            ..Default::default()
        },
    )]
}

/// Create a span for a connector string with edge-state coloring.
fn connector_span(text: &str, edge_state: EdgeState) -> Span {
    let color = match edge_state {
        EdgeState::Idle => SemanticColor::Dim,
        EdgeState::Flowing => SemanticColor::Accent,
        EdgeState::Done => SemanticColor::Success,
        EdgeState::Dead => SemanticColor::Dim,
    };
    Span::styled(
        text.to_string(),
        SpanStyle {
            color: Some(color),
            ..Default::default()
        },
    )
}

/// Map node state to its semantic color.
pub fn state_color(state: NodeState) -> SemanticColor {
    match state {
        NodeState::Pending => SemanticColor::Dim,
        NodeState::Running => SemanticColor::Active,
        NodeState::Completed => SemanticColor::Success,
        NodeState::Failed => SemanticColor::Error,
        NodeState::Skipped => SemanticColor::Dim,
        NodeState::Intercepted => SemanticColor::Success,
    }
}

/// Legend character for a node state.
fn legend_char(state: NodeState) -> &'static str {
    match state {
        NodeState::Completed | NodeState::Intercepted => "\u{2714}", // ✔
        NodeState::Failed => "\u{2718}",                             // ✘
        NodeState::Running => "\u{25D0}",                            // ◐
        NodeState::Skipped => "\u{25CC}",                            // ◌
        NodeState::Pending => "\u{25CB}",                            // ○
    }
}

// ---------------------------------------------------------------------------
// Connector helpers
// ---------------------------------------------------------------------------

/// Determine the dominant edge state for a connector cell.
fn connector_cell_state(edges: &[(NodeId, NodeId)], progress: &DagProgress) -> EdgeState {
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

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Format a duration for display.
pub fn format_duration(d: Duration) -> String {
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
pub fn full_label(node_id: &NodeId, labels: &HashMap<NodeId, String>) -> String {
    labels
        .get(node_id)
        .map(|s| s.as_str())
        .unwrap_or(&node_id.0)
        .to_string()
}

/// Pad or truncate a connector string to exactly `w` display characters.
pub fn pad_connector(s: &str, w: usize) -> String {
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
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Approximate terminal display width of a single character.
///
/// Returns 2 for CJK/fullwidth characters, 0 for combining marks
/// and zero-width characters, 1 for everything else.
pub fn char_width(c: char) -> usize {
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
pub fn random_animal(elapsed: Duration) -> &'static str {
    let idx = (elapsed.as_millis() as usize) % ANIMAL_EMOJIS.len();
    ANIMAL_EMOJIS[idx]
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
    use std::time::Instant;

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
            groups: vec![],
        }
    }

    fn empty_summary() -> OutputSummary {
        OutputSummary {
            fields: vec![],
            elapsed: Duration::from_millis(50),
        }
    }

    #[test]
    fn test_build_frame_standard_has_header() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());
        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        assert!(
            !frame.lines.is_empty(),
            "Frame should have at least a header line"
        );
        assert_eq!(frame.cursor_action, CursorAction::Overwrite);
    }

    #[test]
    fn test_build_frame_compact_single_line_content() {
        let snap = test_snapshot();
        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));
        progress.on_node_complete(&NodeId::from("lint"), empty_summary());

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Compact,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        assert_eq!(frame.lines.len(), 1, "Compact mode should produce one line");
        let text: String = frame.lines[0].spans.iter().map(|s| &s.text[..]).collect();
        assert!(text.contains("1/3 done"), "Should contain count: {}", text);
    }

    #[test]
    fn test_build_frame_standard_contains_node_boxes() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());
        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        // Collect all text from all spans
        let all_text: String = frame
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.text.as_str()))
            .collect();
        assert!(all_text.contains("[A]"), "Missing [A] in: {}", all_text);
        assert!(all_text.contains("[B]"), "Missing [B] in: {}", all_text);
        assert!(all_text.contains("[C]"), "Missing [C] in: {}", all_text);
    }

    #[test]
    fn test_state_color_mapping() {
        assert_eq!(state_color(NodeState::Pending), SemanticColor::Dim);
        assert_eq!(state_color(NodeState::Running), SemanticColor::Active);
        assert_eq!(state_color(NodeState::Completed), SemanticColor::Success);
        assert_eq!(state_color(NodeState::Failed), SemanticColor::Error);
    }

    #[test]
    fn test_node_box_spans_bold_for_running() {
        let spans = node_box_spans("A", NodeState::Running);
        assert!(
            spans.iter().any(|s| s.style.bold),
            "Running node should be bold"
        );
    }

    #[test]
    fn test_node_box_spans_not_bold_for_pending() {
        let spans = node_box_spans("A", NodeState::Pending);
        assert!(
            !spans.iter().any(|s| s.style.bold),
            "Pending node should not be bold"
        );
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.5s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m05s");
    }

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("[A]"), 3);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("漢字"), 4);
        assert_eq!(display_width("A漢B"), 4);
    }

    #[test]
    fn test_footer_only_on_terminal_phase() {
        let snap = test_snapshot();
        let progress = DagProgress::new(snap.clone());
        // NotStarted phase → no footer
        assert!(build_footer_line(&progress, &STANDARD, Tier::Unicode).is_none());
    }

    #[test]
    fn test_footer_on_completed_phase() {
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

        let footer = build_footer_line(&progress, &STANDARD, Tier::Unicode);
        assert!(footer.is_some(), "Completed phase should have a footer");
    }

    // -------------------------------------------------------------------
    // Phase 6: Grouped stage-panel tests
    // -------------------------------------------------------------------

    #[test]
    fn test_grouped_legend_shows_group_names() {
        use crate::progress::StageGroup;
        use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};

        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            edges: vec![
                Edge::new("prepare_build", "out", "execute_build", "in"),
                Edge::new("execute_build", "out", "parse_build", "in"),
            ],
            topo_order: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("prepare_build"), "prepare_build".to_string()),
                (NodeId::from("execute_build"), "execute_build".to_string()),
                (NodeId::from("parse_build"), "parse_build".to_string()),
            ]
            .into_iter()
            .collect(),
            groups: vec![StageGroup {
                name: "build".into(),
                node_ids: vec![
                    NodeId::from("prepare_build"),
                    NodeId::from("execute_build"),
                    NodeId::from("parse_build"),
                ],
            }],
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("prepare_build"));
        progress.on_node_complete(&NodeId::from("prepare_build"), empty_summary());
        progress.on_node_start(&NodeId::from("execute_build"));

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        // Collect all text from the frame
        let all_text: String = frame
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.text.as_str()))
            .collect();
        // Grouped legend should show "build" group name, not individual node names
        assert!(
            all_text.contains("build"),
            "Grouped legend should contain group name 'build', got:\n{}",
            all_text
        );
        assert!(
            all_text.contains("Stages:"),
            "Grouped panel should contain 'Stages:', got:\n{}",
            all_text
        );
        assert!(
            all_text.contains("["), // progress bar bracket
            "Grouped panel should contain a progress bar, got:\n{}",
            all_text
        );
    }

    #[test]
    fn test_ungrouped_legend_shows_node_names() {
        let snap = test_snapshot(); // No groups
        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("lint"));
        progress.on_node_complete(&NodeId::from("lint"), empty_summary());
        progress.on_node_start(&NodeId::from("build"));

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);

        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        let all_text: String = frame
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.text.as_str()))
            .collect();
        // Ungrouped legend should show individual node names
        assert!(
            all_text.contains("build"),
            "Ungrouped legend should contain node name 'build', got:\n{}",
            all_text
        );
    }

    #[test]
    fn test_grouped_stage_panel_auto_expands_long_running_node() {
        use crate::progress::StageGroup;
        use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};

        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            edges: vec![
                Edge::new("prepare_build", "out", "execute_build", "in"),
                Edge::new("execute_build", "out", "parse_build", "in"),
            ],
            topo_order: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("prepare_build"), "prepare_build".to_string()),
                (NodeId::from("execute_build"), "execute_build".to_string()),
                (NodeId::from("parse_build"), "parse_build".to_string()),
            ]
            .into_iter()
            .collect(),
            groups: vec![StageGroup {
                name: "build".into(),
                node_ids: vec![
                    NodeId::from("prepare_build"),
                    NodeId::from("execute_build"),
                    NodeId::from("parse_build"),
                ],
            }],
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("prepare_build"));
        progress.on_node_complete(&NodeId::from("prepare_build"), empty_summary());
        progress.on_node_start(&NodeId::from("execute_build"));
        if let Some(np) = progress.nodes.get_mut(&NodeId::from("execute_build")) {
            np.start_time = Some(Instant::now() - Duration::from_secs(25));
        }

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);
        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        let all_text: String = frame
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.text.as_str()))
            .collect();
        assert!(
            all_text.contains("execute_build"),
            "Long-running group should auto-expand running node detail, got:\n{}",
            all_text
        );
    }

    #[test]
    fn test_grouped_stage_panel_expands_failed_node_detail() {
        use crate::progress::StageGroup;
        use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};

        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            edges: vec![
                Edge::new("prepare_build", "out", "execute_build", "in"),
                Edge::new("execute_build", "out", "parse_build", "in"),
            ],
            topo_order: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("prepare_build"), "prepare_build".to_string()),
                (NodeId::from("execute_build"), "execute_build".to_string()),
                (NodeId::from("parse_build"), "parse_build".to_string()),
            ]
            .into_iter()
            .collect(),
            groups: vec![StageGroup {
                name: "build".into(),
                node_ids: vec![
                    NodeId::from("prepare_build"),
                    NodeId::from("execute_build"),
                    NodeId::from("parse_build"),
                ],
            }],
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("prepare_build"));
        progress.on_node_complete(&NodeId::from("prepare_build"), empty_summary());
        progress.on_node_start(&NodeId::from("execute_build"));
        progress.on_node_failed(&NodeId::from("execute_build"), "boom");

        let vp = Viewport::new(80, 24, ViewportUnit::Chars);
        let layout = compute_layout(&snap.topo_order, &snap.edges, &snap.labels, &vp);
        let frame = build_frame(
            &progress,
            &layout,
            RenderMode::Standard,
            "◐",
            Tier::Unicode,
            &STANDARD,
        );

        let all_text: String = frame
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.text.as_str()))
            .collect();
        assert!(
            all_text.contains("✘"),
            "Failed stage should render failure marker, got:\n{}",
            all_text
        );
        assert!(
            all_text.contains("execute_build"),
            "Failed stage should include failed node detail, got:\n{}",
            all_text
        );
    }

    #[test]
    fn test_stage_progress_bar_renders_expected_fill() {
        assert_eq!(stage_progress_bar(0, 4, 10), "----------");
        assert_eq!(stage_progress_bar(2, 4, 10), "#####-----");
        assert_eq!(stage_progress_bar(4, 4, 10), "##########");
    }
}
