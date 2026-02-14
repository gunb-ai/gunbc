//! CI context for DAG execution.
//!
//! This module provides runtime CI context that enables DAG nodes to
//! automatically emit workflow commands (groups, annotations, etc.)
//! when executing in a CI environment.
//!
//! # Architecture
//!
//! ```text
//! CiContext
//!     ├── provider: Box<dyn CiProvider>  ← Formats commands
//!     ├── group_stack: Vec<String>       ← Tracks nesting
//!     └── writer: Box<dyn Write>         ← Output destination
//!
//! DAG Execution:
//!     start_group("build") → ::group::build
//!         start_group("test") → ::group::build/test
//!         end_group() → ::endgroup::
//!     end_group() → ::endgroup::
//! ```
//!
//! # Example
//!
//! ```ignore
//! use gunbc_exec::CiContext;
//!
//! let mut ci = CiContext::detect();
//!
//! ci.start_group("build", false);
//! // ... execute build node ...
//! ci.end_group();
//!
//! ci.error("Test failed", Some(("src/lib.rs", 42)));
//! ```

use crate::execute::LogEntry;
use crate::progress::{DagSnapshot, OutputSummary, ProgressObserver};
use gunbc_ir::transport::ci::{
    detect_provider, detect_provider_strict, AnnotationLevel, CiProvider, FileLocation,
    WorkflowCommand,
};
use gunbc_ir::{NodeId, Value};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

/// Runtime CI context for DAG execution.
///
/// Tracks the current CI provider and group nesting to emit
/// properly formatted workflow commands.
pub struct CiContext {
    /// Active CI provider (auto-detected or explicit)
    provider: Box<dyn CiProvider>,
    /// Current group nesting path (for hierarchical names)
    group_stack: Vec<String>,
    /// Output writer (stdout by default)
    writer: Box<dyn Write + Send>,
    /// Whether to actually emit commands (can be disabled for testing)
    enabled: bool,
}

impl CiContext {
    /// Create a new CI context with the given provider.
    pub fn new(provider: Box<dyn CiProvider>) -> Self {
        Self {
            provider,
            group_stack: Vec::new(),
            writer: Box::new(io::stdout()),
            enabled: true,
        }
    }

    /// Auto-detect CI environment and create context.
    pub fn detect() -> Self {
        let env: HashMap<String, String> = std::env::vars().collect();
        let provider = detect_provider_strict(&env).unwrap_or_else(|err| {
            panic!("strict CI provider detection failed: {err}");
        });
        Self::new(provider)
    }

    /// Create a disabled context (for testing or when CI output is unwanted).
    pub fn disabled() -> Self {
        let env: HashMap<String, String> = std::env::vars().collect();
        Self {
            provider: detect_provider(&env),
            group_stack: Vec::new(),
            writer: Box::new(io::sink()),
            enabled: false,
        }
    }

    /// Set a custom output writer.
    pub fn with_writer(mut self, writer: Box<dyn Write + Send>) -> Self {
        self.writer = writer;
        self
    }

    /// Enable or disable command emission.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if CI output is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the current provider ID.
    pub fn provider_id(&self) -> &'static str {
        self.provider.id()
    }

    /// Get the current provider name.
    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    /// Get the current nesting depth.
    pub fn depth(&self) -> usize {
        self.group_stack.len()
    }

    /// Get the full group path (joined with "/").
    pub fn current_path(&self) -> String {
        self.group_stack.join("/")
    }

    // ========================================================================
    // Command Emission
    // ========================================================================

    /// Emit a raw workflow command.
    pub fn emit(&mut self, cmd: WorkflowCommand) {
        if !self.enabled {
            return;
        }
        assert!(
            self.provider.supports(&cmd),
            "CI provider '{}' does not support workflow command {:?} in strict mode",
            self.provider.id(),
            cmd
        );
        let formatted = self.provider.format(&cmd);
        writeln!(self.writer, "{}", formatted).ok();
    }

    /// Start a collapsible group.
    ///
    /// Groups are hierarchical - nested groups get names like "parent/child".
    pub fn start_group(&mut self, name: &str, collapsed: bool) {
        let full_name = self.full_group_name(name);
        self.group_stack.push(name.to_string());
        self.emit(WorkflowCommand::GroupStart {
            name: full_name,
            collapsed,
        });
    }

    /// End the current group.
    ///
    /// Returns the name of the closed group, or None if no group was open.
    pub fn end_group(&mut self) -> Option<String> {
        if let Some(name) = self.group_stack.pop() {
            // Reconstruct full name for the end marker
            let full_name = if self.group_stack.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", self.group_stack.join("/"), name)
            };
            self.emit(WorkflowCommand::GroupEnd { name: full_name });
            Some(name)
        } else {
            None
        }
    }

    /// Emit an error annotation.
    pub fn error(&mut self, message: &str, location: Option<(&str, u32)>) {
        self.annotation(AnnotationLevel::Error, message, location);
    }

    /// Emit a warning annotation.
    pub fn warning(&mut self, message: &str, location: Option<(&str, u32)>) {
        self.annotation(AnnotationLevel::Warning, message, location);
    }

    /// Emit a notice annotation.
    pub fn notice(&mut self, message: &str, location: Option<(&str, u32)>) {
        self.annotation(AnnotationLevel::Notice, message, location);
    }

    /// Emit a debug message.
    pub fn debug(&mut self, message: &str) {
        self.annotation(AnnotationLevel::Debug, message, None);
    }

    /// Emit an annotation with the given level.
    pub fn annotation(
        &mut self,
        level: AnnotationLevel,
        message: &str,
        location: Option<(&str, u32)>,
    ) {
        let loc = location.map(|(file, line)| FileLocation::new(file).with_line(line));
        self.emit(WorkflowCommand::Annotation {
            level,
            message: message.to_string(),
            title: None,
            location: loc,
        });
    }

    /// Set an output variable.
    pub fn set_output(&mut self, key: &str, value: &str) {
        self.emit(WorkflowCommand::SetOutput {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    /// Mask a secret value.
    pub fn mask(&mut self, value: &str) {
        self.emit(WorkflowCommand::MaskValue {
            value: value.to_string(),
        });
    }

    /// Write to job summary.
    pub fn summary(&mut self, markdown: &str) {
        self.emit(WorkflowCommand::Summary {
            markdown: markdown.to_string(),
        });
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Get the full hierarchical group name.
    fn full_group_name(&self, name: &str) -> String {
        if self.group_stack.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.group_stack.join("/"), name)
        }
    }

    /// Execute a closure within a group.
    ///
    /// Automatically starts and ends the group around the closure.
    pub fn with_group<F, R>(&mut self, name: &str, collapsed: bool, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.start_group(name, collapsed);
        let result = f(self);
        self.end_group();
        result
    }
}

impl ProgressObserver for CiContext {
    fn on_dag_start(&mut self, _snapshot: &DagSnapshot) {
        // No-op: CiContext doesn't need the snapshot
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        if node_id.0 != "report" {
            self.start_group(&node_id.0, false);
        }
    }

    fn on_node_complete(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        if node_id.0 != "report" {
            self.end_group();
        }
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.error(&format!("Node '{}' failed: {}", node_id.0, error), None);
        if node_id.0 != "report" {
            self.end_group();
        }
    }

    fn on_node_skipped(&mut self, _node_id: &NodeId) {
        // No-op: no group was opened for skipped nodes
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        if node_id.0 != "report" {
            self.end_group();
        }
    }

    fn on_dag_complete(&mut self, _elapsed: Duration) {
        // No-op: CiContext doesn't need DAG completion events
    }

    fn on_secret_output(&mut self, _node_id: &NodeId, secret_value: &str) {
        self.mask(secret_value);
    }

    fn on_failure_diagnostics(&mut self, _node_id: &NodeId, inputs: &HashMap<String, Value>) {
        println!("  inputs at failure:");
        for (port, value) in inputs {
            crate::display::print_value(port, value);
        }
    }

    fn on_boundary_output(&mut self, _node_id: &NodeId, entry: &LogEntry) {
        for (port, value) in &entry.outputs {
            crate::display::print_value(port, value);
        }
    }

    fn requires_sequential(&self) -> bool {
        true
    }
}

impl std::fmt::Debug for CiContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CiContext")
            .field("provider", &self.provider.id())
            .field("group_stack", &self.group_stack)
            .field("enabled", &self.enabled)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::ci::{GitLabCiProvider, PlainTextProvider};
    use std::sync::{Arc, Mutex};

    /// Capture output for testing.
    fn capture_context() -> (CiContext, Arc<Mutex<Vec<u8>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = CaptureWriter(Arc::clone(&buffer));
        let ctx = CiContext::new(Box::new(PlainTextProvider)).with_writer(Box::new(writer));
        (ctx, buffer)
    }

    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn get_output(buffer: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8(buffer.lock().unwrap().clone()).unwrap()
    }

    #[test]
    fn test_ci_context_detect() {
        let ctx = CiContext::detect();
        // Should have a valid provider
        assert!(!ctx.provider_id().is_empty());
    }

    #[test]
    fn test_ci_context_group() {
        let (mut ctx, buffer) = capture_context();

        ctx.start_group("build", false);
        ctx.end_group();

        let output = get_output(&buffer);
        assert!(output.contains("=== build ==="));
        assert!(output.contains("=== /build ==="));
    }

    #[test]
    fn test_ci_context_nested_groups() {
        let (mut ctx, buffer) = capture_context();

        ctx.start_group("build", false);
        ctx.start_group("test", false);
        ctx.end_group();
        ctx.end_group();

        let output = get_output(&buffer);
        assert!(output.contains("=== build ==="));
        assert!(output.contains("=== build/test ==="));
    }

    #[test]
    fn test_ci_context_error() {
        let (mut ctx, buffer) = capture_context();

        ctx.error("test failed", Some(("src/lib.rs", 42)));

        let output = get_output(&buffer);
        assert!(output.contains("[ERROR]"));
        assert!(output.contains("test failed"));
        assert!(output.contains("src/lib.rs:42"));
    }

    #[test]
    fn test_ci_context_with_group() {
        let (mut ctx, buffer) = capture_context();

        let result = ctx.with_group("test", false, |ci| {
            ci.debug("inside group");
            42
        });

        assert_eq!(result, 42);
        let output = get_output(&buffer);
        assert!(output.contains("=== test ==="));
        assert!(output.contains("=== /test ==="));
    }

    #[test]
    fn test_ci_context_disabled() {
        let mut ctx = CiContext::disabled();
        ctx.start_group("test", false);
        ctx.error("should not appear", None);
        ctx.end_group();
        // No assertions needed - just verify it doesn't crash
    }

    #[test]
    fn test_ci_context_depth() {
        let (mut ctx, _) = capture_context();

        assert_eq!(ctx.depth(), 0);
        ctx.start_group("a", false);
        assert_eq!(ctx.depth(), 1);
        ctx.start_group("b", false);
        assert_eq!(ctx.depth(), 2);
        ctx.end_group();
        assert_eq!(ctx.depth(), 1);
    }

    #[test]
    fn test_ci_context_current_path() {
        let (mut ctx, _) = capture_context();

        assert_eq!(ctx.current_path(), "");
        ctx.start_group("a", false);
        assert_eq!(ctx.current_path(), "a");
        ctx.start_group("b", false);
        assert_eq!(ctx.current_path(), "a/b");
    }

    #[test]
    #[should_panic(expected = "does not support workflow command")]
    fn test_ci_context_strict_rejects_unsupported_commands() {
        let mut ctx = CiContext::new(Box::new(GitLabCiProvider::new()));
        ctx.error("unsupported in strict mode", None);
    }
}
