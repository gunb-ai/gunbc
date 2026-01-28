//! Terminal-based progress observer for DAG execution.
//!
//! Provides human-readable progress output to stderr with inline updates.

use std::io::{self, Write};

use crate::progress::{ExecutionEvent, ExecutionObserver};

/// Terminal progress observer that outputs to stderr.
#[derive(Debug)]
pub struct TerminalObserver {
    /// Tool name prefix for output lines.
    tool_name: String,
    /// Total number of nodes in the DAG.
    total_nodes: usize,
    /// Whether we're in the middle of an inline update (need to clear line).
    inline_active: bool,
    /// The final result summary to display at the end.
    final_results: Vec<String>,
}

impl TerminalObserver {
    /// Create a new terminal observer with the given tool name.
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            total_nodes: 0,
            inline_active: false,
            final_results: Vec::new(),
        }
    }

    /// Clear the current line (for inline updates).
    fn clear_line(&self) {
        eprint!("\r\x1b[K");
    }

    /// End the current inline update and move to a new line.
    fn finish_line(&mut self) {
        if self.inline_active {
            eprintln!();
            self.inline_active = false;
        }
    }

    /// Print a progress line with the current step.
    fn print_progress(&mut self, index: usize, label: &str, suffix: &str) {
        if self.inline_active {
            self.clear_line();
        }
        eprint!(
            "{} [{}/{}] {}{}",
            self.tool_name,
            index + 1,
            self.total_nodes,
            label,
            suffix
        );
        let _ = io::stderr().flush();
        self.inline_active = true;
    }

    /// Store a result for final display.
    pub fn add_result(&mut self, key: &str, value: &str) {
        self.final_results.push(format!("{key}={value}"));
    }
}

impl ExecutionObserver for TerminalObserver {
    fn on_event(&mut self, event: ExecutionEvent<'_>) {
        match event {
            ExecutionEvent::ExecutionStarted { total_nodes } => {
                self.total_nodes = total_nodes;
            }
            ExecutionEvent::NodeStarted { label, index, .. } => {
                self.print_progress(index, &label, "...");
            }
            ExecutionEvent::NodeCompleted {
                label,
                index,
                output_summary,
                ..
            } => {
                if self.inline_active {
                    self.clear_line();
                }
                let suffix = if output_summary.is_empty() {
                    " done".to_string()
                } else {
                    format!(" done ({output_summary})")
                };
                eprintln!(
                    "{} [{}/{}] {}...{}",
                    self.tool_name,
                    index + 1,
                    self.total_nodes,
                    label,
                    suffix
                );
                self.inline_active = false;
            }
            ExecutionEvent::NodeSkipped { label, index, .. } => {
                if self.inline_active {
                    self.clear_line();
                }
                eprintln!(
                    "{} [{}/{}] {}... skipped",
                    self.tool_name,
                    index + 1,
                    self.total_nodes,
                    label
                );
                self.inline_active = false;
            }
            ExecutionEvent::NodeFailed {
                label,
                index,
                error,
                ..
            } => {
                self.finish_line();
                eprintln!(
                    "{} [{}/{}] {}... FAILED: {}",
                    self.tool_name,
                    index + 1,
                    self.total_nodes,
                    label,
                    error
                );
            }
            ExecutionEvent::ExecutionFinished { success, .. } => {
                self.finish_line();
                if success && !self.final_results.is_empty() {
                    eprintln!();
                    for result in &self.final_results {
                        eprintln!("  Result: {result}");
                    }
                }
            }
        }
    }
}
