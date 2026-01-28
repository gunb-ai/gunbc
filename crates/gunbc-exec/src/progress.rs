//! Progress reporting abstractions for DAG execution.
//!
//! This module provides an observer pattern for tracking execution progress.
//! The core abstraction is the `ExecutionObserver` trait, which receives events
//! as nodes are executed.

use std::collections::HashMap;

use crate::Value;

/// Events emitted during DAG execution.
#[derive(Debug)]
pub enum ExecutionEvent<'a> {
    /// Execution has started.
    ExecutionStarted { total_nodes: usize },
    /// A node has started executing.
    NodeStarted {
        node_id: &'a str,
        label: String,
        index: usize,
    },
    /// A node has completed successfully.
    NodeCompleted {
        node_id: &'a str,
        label: String,
        index: usize,
        output_summary: String,
    },
    /// A node was skipped due to guard conditions.
    NodeSkipped {
        node_id: &'a str,
        label: String,
        index: usize,
    },
    /// A node failed during execution.
    NodeFailed {
        node_id: &'a str,
        label: String,
        index: usize,
        error: &'a str,
    },
    /// Execution has finished.
    ExecutionFinished {
        success: bool,
        nodes_completed: usize,
        nodes_skipped: usize,
        nodes_failed: usize,
    },
}

/// Observer trait for receiving execution events.
pub trait ExecutionObserver {
    /// Called when an execution event occurs.
    fn on_event(&mut self, event: ExecutionEvent<'_>);
}

/// No-op observer that discards all events.
#[derive(Debug, Default)]
pub struct NullObserver;

impl ExecutionObserver for NullObserver {
    fn on_event(&mut self, _event: ExecutionEvent<'_>) {}
}

/// Derive a human-readable label from a node ID.
///
/// Examples:
/// - `gist/call_gist_real` → "Call gist real"
/// - `filter_files` → "Filter files"
/// - `checkAuth` → "Check auth"
pub fn node_label(node_id: &str) -> String {
    // Take the last segment after any '/'
    let segment = node_id.rsplit('/').next().unwrap_or(node_id);
    humanize_identifier(segment)
}

/// Convert a snake_case or camelCase identifier to human-readable words.
///
/// Examples:
/// - `call_gist_real` → "Call gist real"
/// - `checkAuth` → "Check auth"
/// - `filterFiles` → "Filter files"
pub fn humanize_identifier(id: &str) -> String {
    let mut words = Vec::new();
    let mut current_word = String::new();

    for ch in id.chars() {
        if ch == '_' {
            // snake_case separator
            if !current_word.is_empty() {
                words.push(current_word);
                current_word = String::new();
            }
        } else if ch.is_uppercase() {
            // camelCase boundary
            if !current_word.is_empty() {
                words.push(current_word);
                current_word = String::new();
            }
            current_word.push(ch.to_ascii_lowercase());
        } else {
            current_word.push(ch);
        }
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    if words.is_empty() {
        return id.to_string();
    }

    // Capitalize first word
    if let Some(first) = words.first_mut() {
        if let Some(c) = first.chars().next() {
            *first = c.to_uppercase().chain(first.chars().skip(1)).collect();
        }
    }

    words.join(" ")
}

/// Summarize a single value for display.
///
/// - Strings are truncated and have control characters escaped
/// - Secrets are redacted
/// - Collections show their size
pub fn summarize_value(value: &Value, max_len: usize) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => truncate_string(s, max_len),
        Value::StrList(v) => format!("[{} items]", v.len()),
        Value::MapStrStr(m) => format!("{{{} entries}}", m.len()),
        Value::Secret(_) => "<REDACTED>".to_string(),
        Value::Skipped => "<skipped>".to_string(),
        Value::Unit => "()".to_string(),
    }
}

/// Truncate and sanitize a string for display.
fn truncate_string(s: &str, max_len: usize) -> String {
    // Escape common control sequences and problematic characters
    let sanitized: String = s
        .chars()
        .flat_map(|c| match c {
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            '"' => vec!['"'],
            c if c.is_control() => vec![], // Drop other control chars
            c => vec![c],
        })
        .collect();

    if sanitized.len() <= max_len {
        format!("\"{sanitized}\"")
    } else {
        format!("\"{}...\"", &sanitized[..max_len.saturating_sub(4)])
    }
}

/// Summarize all outputs from a node for display.
pub fn summarize_outputs(outputs: &HashMap<String, Value>, max_len: usize) -> String {
    if outputs.is_empty() {
        return String::new();
    }

    // For single output, just show the value
    if outputs.len() == 1 {
        let (key, value) = outputs.iter().next().unwrap();
        let summary = summarize_value(value, max_len);
        return format!("{key}={summary}");
    }

    // For multiple outputs, show key=value pairs
    let parts: Vec<String> = outputs
        .iter()
        .map(|(k, v)| format!("{}={}", k, summarize_value(v, 40)))
        .collect();

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_snake_case() {
        assert_eq!(humanize_identifier("call_gist_real"), "Call gist real");
        assert_eq!(humanize_identifier("filter_files"), "Filter files");
        assert_eq!(humanize_identifier("check_auth"), "Check auth");
    }

    #[test]
    fn test_humanize_camel_case() {
        assert_eq!(humanize_identifier("callGistReal"), "Call gist real");
        assert_eq!(humanize_identifier("filterFiles"), "Filter files");
        assert_eq!(humanize_identifier("checkAuth"), "Check auth");
    }

    #[test]
    fn test_humanize_mixed() {
        assert_eq!(humanize_identifier("call_gistReal"), "Call gist real");
    }

    #[test]
    fn test_node_label_with_path() {
        assert_eq!(node_label("gist/call_gist_real"), "Call gist real");
        assert_eq!(node_label("auth/check_token"), "Check token");
    }

    #[test]
    fn test_node_label_simple() {
        assert_eq!(node_label("filter_files"), "Filter files");
    }

    #[test]
    fn test_summarize_value_string() {
        let val = Value::Str("hello world".to_string());
        assert_eq!(summarize_value(&val, 50), "\"hello world\"");
    }

    #[test]
    fn test_summarize_value_truncates() {
        let val = Value::Str("a".repeat(100));
        let summary = summarize_value(&val, 20);
        assert!(summary.len() <= 22); // 20 + quotes
        assert!(summary.ends_with("...\""));
    }

    #[test]
    fn test_summarize_value_escapes_newlines() {
        let val = Value::Str("line1\nline2".to_string());
        assert_eq!(summarize_value(&val, 50), "\"line1\\nline2\"");
    }

    #[test]
    fn test_summarize_value_secret() {
        let val = Value::Secret(gunbc_ir::types::Secret("secret".to_string()));
        assert_eq!(summarize_value(&val, 50), "<REDACTED>");
    }
}
