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
        run_freshness_step(self)?;
        Ok(HashMap::from([("done".into(), Value::Bool(true))]))
    }
}

fn format_freshness_step_failure_detail(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return format!("\n{stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();
    if !stdout.is_empty() {
        return format!("\n{stdout}");
    }

    String::new()
}

/// Run a single freshness step command with fail-closed error reporting.
///
/// Child stdout/stderr are captured so progress rendering remains clean.
/// On failure, stderr (or stdout when stderr is empty) is included in the error.
pub fn run_freshness_step(step: &FreshnessStep) -> Result<(), ExecError> {
    if step.command.is_empty() {
        return Err(ExecError::new("freshness step has no command"));
    }

    let program = &step.command[0];
    let args = &step.command[1..];

    // Freshness steps run external tooling (codegen, clippy, etc.) as child
    // processes. Capture stdout/stderr so child output doesn't interleave
    // with the parent's progress display (gunb.ai's SetTaskOutput pattern:
    // failure-first, only render child output when the step fails).
    #[allow(clippy::disallowed_methods)]
    let output = std::process::Command::new(program)
        .args(args)
        .env(FRESHNESS_ACTIVE_ENV, "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| {
            ExecError::new(format!(
                "freshness step '{}' failed to start: {}",
                step.id, e
            ))
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let detail = format_freshness_step_failure_detail(&output);
        return Err(ExecError::new(format!(
            "freshness step '{}' failed (exit {}){detail}",
            step.id, code
        )));
    }

    Ok(())
}

/// Run a sequence of freshness steps in order.
pub fn run_freshness_steps(steps: &[FreshnessStep]) -> Result<(), ExecError> {
    for step in steps {
        run_freshness_step(step)?;
    }
    Ok(())
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

/// Overlap surface: maps freshness step IDs to cargo operation keywords
/// found in tool DAG node IDs. If a freshness step's ID is a key here,
/// and any tool DAG node ID contains the corresponding keyword preceded
/// by a cargo transport prefix, the composition would schedule redundant work.
///
/// This is the single source of truth for the freshness/tool overlap boundary.
/// Add entries here when new freshness steps are introduced that correspond
/// to cargo operations the tool DAG might already perform.
const FRESHNESS_CARGO_OVERLAP: &[(&str, &str)] = &[
    ("clippy", "Clippy"),
    ("test-compile", "Test"),
    ("release-check", "Build"),
];

/// Marker in tool DAG node IDs that indicates a cargo transport operation.
const CARGO_TRANSPORT_MARKER: &str = "transport_services_cargo";

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
/// `Ok(Dag<WithFreshness<T>>)` where:
/// - All original tool nodes are wrapped as `WithFreshness::Tool(op)`
/// - If steps were provided, a "freshness" sub-DAG is prepended and wired
///   as a blocking dependency to all tool root nodes
///
/// `Err(ExecError)` if any freshness step would duplicate work already
/// performed by the tool DAG's cargo operations (overlap detection).
///
/// # Display
///
/// The freshness sub-DAG renders as a grouped stage in the terminal display:
/// ```text
/// › freshness [4/4]
///    ✓ codegen
///    ✓ codegen-dag
///    ✓ testgen
///    ✓ pragma
/// ```
pub fn compose_with_freshness<T: Clone>(
    dag: Dag<T>,
    steps: Option<Vec<FreshnessStep>>,
) -> Result<Dag<WithFreshness<T>>, ExecError> {
    // Wrap all existing tool nodes
    let mut wrapped = dag.map_ops(&mut WithFreshness::Tool);

    let steps = match steps {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(wrapped),
    };

    // Detect redundant work: if a freshness step covers a cargo operation
    // that the tool DAG already performs, the composition is invalid.
    detect_freshness_overlap(&wrapped, &steps)?;

    // Build the freshness sub-DAG: sequential chain of steps
    let freshness_subdag = build_freshness_subdag(steps);

    // Find opaque root nodes (nodes that no edge targets and are not SubDags).
    // SubDag nodes derive ports from inner structure; injecting _freshness
    // would violate the entrypoint contract and cause validation failures.
    let targets: HashSet<_> = wrapped.edges.iter().map(|e| e.to_node.clone()).collect();
    let opaque_roots: Vec<_> = wrapped
        .nodes
        .iter()
        .filter(|n| !targets.contains(&n.id) && n.is_opaque())
        .map(|n| n.id.clone())
        .collect();

    for node in &mut wrapped.nodes {
        if opaque_roots.contains(&node.id) {
            node.inputs.push(Port::new("_freshness", "Bool"));
        }
    }

    wrapped
        .nodes
        .push(Node::subdag("freshness", freshness_subdag));

    for root_id in &opaque_roots {
        wrapped.edges.push(Edge::new(
            "freshness",
            "done",
            root_id.0.as_str(),
            "_freshness",
        ));
    }

    Ok(wrapped)
}

/// Detect freshness steps that would duplicate cargo operations already
/// present in the tool DAG. Returns an error listing the overlapping steps.
fn detect_freshness_overlap<T>(
    dag: &Dag<WithFreshness<T>>,
    steps: &[FreshnessStep],
) -> Result<(), ExecError> {
    // Collect cargo operation keywords present in the tool DAG
    let dag_cargo_ops: HashSet<&str> = FRESHNESS_CARGO_OVERLAP
        .iter()
        .filter(|(_, op_keyword)| {
            dag.nodes.iter().any(|n| {
                let id = n.id.0.as_str();
                id.contains(CARGO_TRANSPORT_MARKER) && id.contains(op_keyword)
            })
        })
        .map(|(_, op_keyword)| *op_keyword)
        .collect();

    if dag_cargo_ops.is_empty() {
        return Ok(());
    }

    // Check if any freshness step overlaps with the DAG's cargo operations
    let mut overlaps = Vec::new();
    for step in steps {
        for &(step_id, op_keyword) in FRESHNESS_CARGO_OVERLAP {
            if step.id == step_id && dag_cargo_ops.contains(op_keyword) {
                overlaps.push(format!(
                    "freshness step '{}' → cargo operation '{}'",
                    step_id, op_keyword
                ));
            }
        }
    }

    if overlaps.is_empty() {
        return Ok(());
    }

    Err(ExecError::new(format!(
        "freshness/tool overlap detected — the following freshness steps would \
         duplicate cargo operations already in the tool DAG:\n  {}\n\
         Fix: use FreshnessScope::GenerationOnly for tools that include \
         build/clippy/test operations, or split the freshness chain in \
         freshness_policy.rs to exclude overlapping steps.",
        overlaps.join("\n  ")
    )))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dag_with_cargo_node(op_keyword: &str) -> Dag<String> {
        let node_id = format!(
            "execute_{CARGO_TRANSPORT_MARKER}_cargo_Build_{op_keyword}"
        );
        let mut dag = Dag::new();
        dag.nodes.push(Node::opaque(
            node_id.as_str(),
            vec![],
            vec![Port::new("success", "Bool")],
            "noop".to_string(),
        ));
        dag
    }

    fn step(id: &str) -> FreshnessStep {
        FreshnessStep {
            id: id.to_string(),
            command: vec!["echo".to_string()],
        }
    }

    #[test]
    fn overlap_detected_for_clippy() {
        let dag = dag_with_cargo_node("Clippy");
        let result = compose_with_freshness(dag, Some(vec![step("clippy")]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("clippy"));
    }

    #[test]
    fn overlap_detected_for_test_compile() {
        let dag = dag_with_cargo_node("Test");
        let result = compose_with_freshness(dag, Some(vec![step("test-compile")]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test-compile"));
    }

    #[test]
    fn overlap_detected_for_release_check() {
        let dag = dag_with_cargo_node("Build");
        let result = compose_with_freshness(dag, Some(vec![step("release-check")]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("release-check"));
    }

    #[test]
    fn no_overlap_for_generation_steps() {
        let dag = dag_with_cargo_node("Clippy");
        let gen_steps = vec![step("codegen"), step("testgen"), step("pragma")];
        let result = compose_with_freshness(dag, Some(gen_steps));
        assert!(result.is_ok());
    }

    #[test]
    fn no_overlap_when_dag_has_no_cargo_nodes() {
        let mut dag = Dag::new();
        dag.nodes.push(Node::opaque(
            "some_other_node",
            vec![],
            vec![Port::new("done", "Bool")],
            "noop".to_string(),
        ));
        let all_steps = vec![step("clippy"), step("test-compile"), step("release-check")];
        let result = compose_with_freshness(dag, Some(all_steps));
        assert!(result.is_ok());
    }

    #[test]
    fn no_steps_returns_ok() {
        let dag: Dag<String> = Dag::new();
        let result = compose_with_freshness(dag, None);
        assert!(result.is_ok());
    }
}
