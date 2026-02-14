//! Deduced freshness chain: type-safe DAG composition for repo freshness.
//!
//! Instead of every tool manually declaring `LintCheck` variants, this module
//! provides a generic `WithFreshness<T>` wrapper that separates freshness
//! concerns from tool concerns entirely. Tools never know about freshness —
//! the composition happens at the binary boundary via [`compose_with_freshness`].
//!
//! # Architecture
//!
//! The freshness chain is **deduced**, not manually modeled:
//!
//! 1. At binary startup, check if the repo is fresh (fast manifest check)
//! 2. If stale, build a freshness sub-DAG (codegen → testgen → pragma → clippy → test)
//! 3. Compose the freshness sub-DAG with the tool's DAG using [`WithFreshness<T>`]
//! 4. Execute the combined DAG — freshness nodes display inline
//!
//! Recursion is prevented via the `GUNBC_FRESHNESS_ACTIVE` environment variable:
//! freshness steps set it on child processes, and [`compose_with_freshness`]
//! skips injection when it's set.

use crate::{ExecError, Executable};
use gunbc_ir::{Dag, Edge, Node, Port, Value};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Environment variable that prevents recursive freshness checking.
///
/// When a freshness step runs a sub-binary (e.g., `cargo run --bin gunbc-codegen-dag`),
/// it sets this variable. The sub-binary's `compose_with_freshness` sees it and
/// skips freshness injection — no hardcoded skip list needed.
pub const FRESHNESS_ACTIVE_ENV: &str = "GUNBC_FRESHNESS_ACTIVE";

/// A single step in the freshness chain.
///
/// Each step represents a shell command that must succeed before the tool DAG
/// can execute. Steps are chained sequentially in the freshness sub-DAG.
#[derive(Clone)]
pub struct FreshnessStep {
    /// Display ID for this step (e.g., "codegen-dag", "clippy").
    pub id: String,
    /// The shell command to run (program + args).
    pub command: Vec<String>,
}

impl fmt::Debug for FreshnessStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FreshnessStep({})", self.id)
    }
}

impl Executable for FreshnessStep {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        if self.command.is_empty() {
            return Err(ExecError::new("freshness step has no command"));
        }

        let program = &self.command[0];
        let args = &self.command[1..];

        // Freshness steps run external tooling (codegen, clippy, etc.) as child
        // processes. This is the execution boundary — analogous to a transport
        // executor — so direct Command::new is correct here.
        #[allow(clippy::disallowed_methods)]
        let status = std::process::Command::new(program)
            .args(args)
            .env(FRESHNESS_ACTIVE_ENV, "1")
            .status()
            .map_err(|e| {
                ExecError::new(format!(
                    "freshness step '{}' failed to start: {}",
                    self.id, e
                ))
            })?;

        if !status.success() {
            let code = status.code().unwrap_or(-1);
            return Err(ExecError::new(format!(
                "freshness step '{}' failed (exit {})",
                self.id, code
            )));
        }

        Ok(HashMap::from([("done".into(), Value::Bool(true))]))
    }
}

/// Wrapper that composes freshness operations with arbitrary tool DAGs.
///
/// Tools define their own op types without knowing about freshness. The
/// execution layer wraps them via [`compose_with_freshness`], which converts
/// `Dag<T>` into `Dag<WithFreshness<T>>` by:
///
/// 1. Wrapping existing nodes as `WithFreshness::Tool(op)` via `map_ops`
/// 2. Adding freshness nodes as `WithFreshness::Freshness(step)`
/// 3. Wiring freshness completion to tool root node inputs
#[derive(Clone)]
pub enum WithFreshness<T> {
    /// A regular tool operation (unchanged from the original DAG).
    Tool(T),
    /// A freshness step (injected by compose_with_freshness).
    Freshness(FreshnessStep),
}

impl<T: fmt::Debug> fmt::Debug for WithFreshness<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tool(op) => write!(f, "Tool({:?})", op),
            Self::Freshness(step) => write!(f, "Freshness({:?})", step),
        }
    }
}

impl<T: Executable> Executable for WithFreshness<T> {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Self::Tool(op) => op.execute(inputs),
            Self::Freshness(step) => step.execute(inputs),
        }
    }
}

/// Compose a tool DAG with an optional freshness chain.
///
/// This is the single entry point for freshness integration. It replaces
/// the old `inject_lint_guard` + per-tool `wire_lint_guard` pattern.
///
/// # Arguments
///
/// * `dag` - The tool's DAG (e.g., gist graph, build graph)
/// * `steps` - Freshness steps to prepend, or `None` if repo is already fresh.
///   Obtain this from `gunbc_lib_transport::check_and_plan_freshness()`.
///
/// # Returns
///
/// A `Dag<WithFreshness<T>>` where:
/// - All original tool nodes are wrapped as `WithFreshness::Tool(op)`
/// - If steps were provided, a "freshness" sub-DAG is prepended and wired
///   as a blocking dependency to all tool root nodes
///
/// # Display
///
/// The freshness sub-DAG renders as a grouped stage in the terminal display:
/// ```text
/// › freshness [5/5]
///    ✓ codegen-dag
///    ✓ testgen
///    ✓ pragma
///    ✓ clippy
///    ✓ test-compile
/// ```
pub fn compose_with_freshness<T: Clone>(
    dag: Dag<T>,
    steps: Option<Vec<FreshnessStep>>,
) -> Dag<WithFreshness<T>> {
    // Wrap all existing tool nodes
    let mut wrapped = dag.map_ops(&mut WithFreshness::Tool);

    let steps = match steps {
        Some(s) if !s.is_empty() => s,
        _ => return wrapped,
    };

    // Build the freshness sub-DAG: sequential chain of steps
    let freshness_subdag = build_freshness_subdag(steps);

    // Find root nodes in the wrapped DAG (nodes that no edge targets)
    let targets: HashSet<_> = wrapped.edges.iter().map(|e| e.to_node.clone()).collect();
    let roots: Vec<_> = wrapped
        .nodes
        .iter()
        .filter(|n| !targets.contains(&n.id))
        .map(|n| n.id.clone())
        .collect();

    // Add _freshness input port to each root node
    for node in &mut wrapped.nodes {
        if roots.contains(&node.id) {
            node.inputs.push(Port::new("_freshness", "Bool"));
        }
    }

    // Add the freshness sub-DAG as a SubDag node
    // SubDag children get prefixed IDs (e.g., "freshness/codegen-dag")
    // which the display system groups automatically under "freshness"
    wrapped
        .nodes
        .push(Node::subdag("freshness", freshness_subdag));

    // Wire freshness.done → each root._freshness
    for root_id in &roots {
        wrapped.edges.push(Edge::new(
            "freshness",
            "done",
            root_id.0.as_str(),
            "_freshness",
        ));
    }

    wrapped
}

/// Build a sequential sub-DAG from freshness steps.
///
/// Each step depends on the previous step's "done" output, enforcing
/// sequential execution: codegen-dag → testgen → pragma → clippy → test-compile.
///
/// The sub-DAG's boundary output is "done: Bool" from the last step.
fn build_freshness_subdag<T: Clone>(steps: Vec<FreshnessStep>) -> Dag<WithFreshness<T>> {
    let mut dag = Dag::new();

    let mut prev_id: Option<String> = None;

    for step in steps {
        let id = step.id.clone();

        let inputs = if prev_id.is_some() {
            vec![Port::new("_prev", "Bool")]
        } else {
            vec![]
        };

        dag.nodes.push(Node::opaque(
            id.as_str(),
            inputs,
            vec![Port::new("done", "Bool")],
            WithFreshness::Freshness(step),
        ));

        // Chain: previous step's done → this step's _prev
        if let Some(ref prev) = prev_id {
            dag.edges
                .push(Edge::new(prev.as_str(), "done", id.as_str(), "_prev"));
        }

        prev_id = Some(id);
    }

    dag
}
