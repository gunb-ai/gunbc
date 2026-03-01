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
//! # Overlap detection
//!
//! Freshness steps that subsume a service operation declare it via
//! `FreshnessStep::subsumes`. When composing, the operation keys from
//! freshness steps and tool DAG nodes are validated for overlap using
//! `gunbc_ir::validate_no_operation_overlap` — the same general invariant
//! that prevents duplicate upserts anywhere in the system.
//!
//! Recursion is prevented via the `GUNBC_FRESHNESS_ACTIVE` environment variable:
//! freshness steps set it on child processes, and [`compose_with_freshness`]
//! skips injection when it's set.

use crate::{ExecError, Executable};
use gunbc_ir::{validate_no_operation_overlap, Dag, Edge, Node, OperationKey, Port, Value};
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
    /// The service operation this step subsumes, if any.
    ///
    /// When a freshness step performs the same work as a service operation
    /// (e.g., the "clippy" freshness step runs `cargo clippy`, which is the
    /// same as `cargo.Build.Clippy`), this field declares that identity.
    /// The composition layer uses this to detect and reject duplicate work:
    /// if a freshness step subsumes an operation that the tool DAG also
    /// contains, the composition is invalid.
    ///
    /// Derived from the domain model — not a manually maintained mapping.
    pub subsumes: Option<OperationKey>,
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
/// `Err(ExecError)` if any freshness step's `subsumes` operation key overlaps
/// with an `operation_key` already present in the tool DAG. This is the general
/// idempotency/upsert invariant: composing the same operation twice without
/// modification is a structural error.
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

    // Build the freshness sub-DAG: sequential chain of steps.
    // Each freshness step that subsumes an operation stamps its operation_key
    // on the corresponding node — this is what the overlap validation reads.
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

    // Validate: no operation should appear in both the freshness sub-DAG
    // and the tool DAG. This uses the general IR-level overlap detection,
    // not a freshness-specific mapping table.
    let duplicates = validate_no_operation_overlap(&wrapped);
    if !duplicates.is_empty() {
        let details: Vec<_> = duplicates.iter().map(|d| d.to_string()).collect();
        return Err(ExecError::new(format!(
            "composition rejected — duplicate operations detected (same operation \
             in both freshness chain and tool DAG is redundant work):\n  {}\n\
             Fix: use FreshnessScope::GenerationOnly for tools that include \
             build/clippy/test operations, or remove the overlapping freshness steps.",
            details.join("\n  ")
        )));
    }

    Ok(wrapped)
}

/// Build a sequential sub-DAG from freshness steps.
///
/// Each step depends on the previous step's "done" output, enforcing
/// sequential execution: codegen-dag → testgen → pragma → clippy → test-compile.
///
/// The sub-DAG's boundary output is "done: Bool" from the last step.
///
/// Each step's `subsumes` operation key is stamped on the node as
/// `operation_key`, enabling the general overlap detection in the IR.
fn build_freshness_subdag<T: Clone>(steps: Vec<FreshnessStep>) -> Dag<WithFreshness<T>> {
    let mut dag = Dag::new();

    let mut prev_id: Option<String> = None;

    for step in steps {
        let id = step.id.clone();
        let operation_key = step.subsumes.clone();

        let inputs = if prev_id.is_some() {
            vec![Port::new("_prev", "Bool")]
        } else {
            vec![]
        };

        let mut node = Node::opaque(
            id.as_str(),
            inputs,
            vec![Port::new("done", "Bool")],
            WithFreshness::Freshness(step),
        );

        // Stamp the operation key from subsumes — this is what connects
        // the freshness step to the domain model's operation identity.
        if let Some(key) = operation_key {
            node = node.with_operation_key(key);
        }

        dag.nodes.push(node);

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

    /// Create a tool DAG with a node that has the given operation key.
    fn dag_with_operation(service: &str, operation: &str) -> Dag<String> {
        let mut dag = Dag::new();
        dag.nodes.push(
            Node::opaque(
                format!("execute_transport_{service}_{operation}"),
                vec![],
                vec![Port::new("success", "Bool")],
                "noop".to_string(),
            )
            .with_operation_key(OperationKey::new(service, operation)),
        );
        dag
    }

    fn step(id: &str) -> FreshnessStep {
        FreshnessStep {
            id: id.to_string(),
            command: vec!["echo".to_string()],
            subsumes: None,
        }
    }

    fn step_subsumes(id: &str, service: &str, operation: &str) -> FreshnessStep {
        FreshnessStep {
            id: id.to_string(),
            command: vec!["echo".to_string()],
            subsumes: Some(OperationKey::new(service, operation)),
        }
    }

    #[test]
    fn overlap_detected_for_clippy() {
        let dag = dag_with_operation("cargo.Build", "Clippy");
        let result = compose_with_freshness(
            dag,
            Some(vec![step_subsumes("clippy", "cargo.Build", "Clippy")]),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cargo.Build.Clippy"));
    }

    #[test]
    fn overlap_detected_for_test_compile() {
        let dag = dag_with_operation("cargo.Build", "Test");
        let result = compose_with_freshness(
            dag,
            Some(vec![step_subsumes("test-compile", "cargo.Build", "Test")]),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cargo.Build.Test")
        );
    }

    #[test]
    fn overlap_detected_for_release_check() {
        let dag = dag_with_operation("cargo.Build", "Build");
        let result = compose_with_freshness(
            dag,
            Some(vec![step_subsumes(
                "release-check",
                "cargo.Build",
                "Build",
            )]),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cargo.Build.Build"));
    }

    #[test]
    fn no_overlap_for_generation_steps() {
        let dag = dag_with_operation("cargo.Build", "Clippy");
        let gen_steps = vec![step("codegen"), step("testgen"), step("pragma")];
        let result = compose_with_freshness(dag, Some(gen_steps));
        assert!(result.is_ok());
    }

    #[test]
    fn no_overlap_when_dag_has_no_operation_keys() {
        let mut dag = Dag::new();
        dag.nodes.push(Node::opaque(
            "some_other_node",
            vec![],
            vec![Port::new("done", "Bool")],
            "noop".to_string(),
        ));
        let steps = vec![
            step_subsumes("clippy", "cargo.Build", "Clippy"),
            step_subsumes("test-compile", "cargo.Build", "Test"),
            step_subsumes("release-check", "cargo.Build", "Build"),
        ];
        let result = compose_with_freshness(dag, Some(steps));
        assert!(result.is_ok());
    }

    #[test]
    fn no_steps_returns_ok() {
        let dag: Dag<String> = Dag::new();
        let result = compose_with_freshness(dag, None);
        assert!(result.is_ok());
    }

    #[test]
    fn steps_without_subsumes_never_overlap() {
        let dag = dag_with_operation("cargo.Build", "Clippy");
        // Step has same display ID but no subsumes — no overlap
        let result = compose_with_freshness(dag, Some(vec![step("clippy")]));
        assert!(result.is_ok());
    }
}
