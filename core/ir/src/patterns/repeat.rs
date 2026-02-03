//! Repetition patterns: Retry, While, and Poll.
//!
//! These patterns provide higher-order constructs for repeated execution:
//!
//! - **Retry**: Re-execute on failure, with configurable backoff
//! - **While**: Re-execute while a condition holds
//! - **Poll**: Re-execute periodically until success or timeout
//!
//! All patterns follow the "template DAG + instance DAG" principle:
//! - The body is a template DAG
//! - Each iteration creates a new instance with a unique iteration index
//! - The execution trace remains acyclic (indexed by iteration)

use crate::dag::{Dag, Edge, Port};
use crate::node::Node;
use crate::patterns::PatternOp;
use std::time::Duration;

// ============================================================================
// Common Types
// ============================================================================

/// Policy for repetition constructs.
#[derive(Debug, Clone)]
pub struct RepeatPolicy {
    /// Maximum number of attempts (including initial)
    pub max_attempts: usize,
    /// Initial delay between attempts
    pub initial_delay: Duration,
    /// Maximum delay (for capped exponential backoff)
    pub max_delay: Duration,
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    /// Total timeout for all attempts
    pub timeout: Option<Duration>,
}

impl Default for RepeatPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff: BackoffStrategy::Exponential { factor: 2.0 },
            timeout: None,
        }
    }
}

impl RepeatPolicy {
    /// Create a policy with fixed delay between attempts.
    pub fn fixed(max_attempts: usize, delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay: delay,
            max_delay: delay,
            backoff: BackoffStrategy::Fixed,
            timeout: None,
        }
    }

    /// Create a policy with exponential backoff.
    pub fn exponential(max_attempts: usize, initial: Duration, factor: f64) -> Self {
        Self {
            max_attempts,
            initial_delay: initial,
            max_delay: Duration::from_secs(300), // 5 minutes default cap
            backoff: BackoffStrategy::Exponential { factor },
            timeout: None,
        }
    }

    /// Set a total timeout for all attempts.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum delay (caps exponential backoff).
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }
}

/// Backoff strategy for delays between attempts.
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between attempts
    Fixed,
    /// Exponential backoff: delay * factor^attempt
    Exponential { factor: f64 },
    /// Linear backoff: initial_delay + (increment * attempt)
    Linear { increment: Duration },
}

/// Classifier for determining if a failure is retryable.
#[derive(Debug, Clone, Default)]
pub enum FailureClassifier {
    /// Always retry on any failure
    #[default]
    Always,
    /// Never retry (useful for testing)
    Never,
    /// Retry only on specific error patterns (stored as string patterns)
    OnPatterns(Vec<String>),
}

// ============================================================================
// Retry Pattern
// ============================================================================

/// Builder for the Retry pattern.
///
/// Retry re-executes a body DAG on failure, up to a maximum number of attempts.
///
/// # Example
///
/// ```ignore
/// let retry_node = RetryBuilder::new("fetch_with_retry")
///     .with_body(fetch_dag)
///     .with_policy(RepeatPolicy::exponential(3, Duration::from_secs(1), 2.0))
///     .build();
/// ```
pub struct RetryBuilder<T> {
    name: String,
    body_dag: Option<Dag<T>>,
    policy: RepeatPolicy,
    classifier: FailureClassifier,
    // Port configuration
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> RetryBuilder<T> {
    /// Create a new retry builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            body_dag: None,
            policy: RepeatPolicy::default(),
            classifier: FailureClassifier::default(),
            input_port_name: "input".to_string(),
            input_port_type: "Any".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "Any".to_string(),
        }
    }

    /// Set the body DAG to retry on failure.
    pub fn with_body(mut self, dag: Dag<T>) -> Self {
        self.body_dag = Some(dag);
        self
    }

    /// Set the retry policy.
    pub fn with_policy(mut self, policy: RepeatPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set the failure classifier.
    pub fn with_classifier(mut self, classifier: FailureClassifier) -> Self {
        self.classifier = classifier;
        self
    }

    /// Configure the input port.
    pub fn with_input(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self
    }

    /// Configure the output port.
    pub fn with_output(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Build the retry pattern as a SubDag node.
    pub fn build(self) -> Node<T>
    where
        T: From<PatternOp>,
    {
        let body_dag = self.body_dag.expect("body DAG is required");

        let mut dag = Dag::new();

        // Controller node: manages retry state
        dag.add_node(Node::opaque(
            "controller",
            vec![
                Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str()),
                Port::optional("last_error", "Error"),
            ],
            vec![
                Port::scalar("body_input", self.input_port_type.as_str()),
                Port::scalar("attempt", "Int"),
                Port::scalar("should_retry", "Bool"),
            ],
            T::from(PatternOp::RetryController {
                input_port: self.input_port_name.clone(),
                policy: self.policy.clone(),
                classifier: self.classifier.clone(),
            }),
        ));

        // Body subdag
        dag.add_node(Node::subdag(
            "body",
            vec![Port::scalar("input", self.input_port_type.as_str())],
            vec![
                Port::optional("result", self.output_port_type.as_str()),
                Port::optional("error", "Error"),
            ],
            body_dag,
        ));

        // Result collector: captures success or final failure
        dag.add_node(Node::opaque(
            "collector",
            vec![
                Port::optional("result", self.output_port_type.as_str()),
                Port::optional("error", "Error"),
                Port::scalar("attempt", "Int"),
            ],
            vec![
                Port::scalar(self.output_port_name.as_str(), self.output_port_type.as_str()),
                Port::scalar("attempts_made", "Int"),
                Port::optional("final_error", "Error"),
            ],
            T::from(PatternOp::RetryCollector {
                output_port: self.output_port_name.clone(),
            }),
        ));

        // Wire internal nodes
        dag.add_edge(Edge::new("controller", "body_input", "body", "input"));
        dag.add_edge(Edge::new("controller", "attempt", "collector", "attempt"));
        dag.add_edge(Edge::new("body", "result", "collector", "result"));
        dag.add_edge(Edge::new("body", "error", "collector", "error"));

        // Create outer node
        Node::subdag(
            self.name.as_str(),
            vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())],
            vec![
                Port::scalar(self.output_port_name.as_str(), self.output_port_type.as_str()),
                Port::scalar("attempts_made", "Int"),
                Port::optional("final_error", "Error"),
            ],
            dag,
        )
    }
}

// ============================================================================
// While Pattern
// ============================================================================

/// Builder for the While pattern.
///
/// While re-executes a body DAG as long as a condition holds.
/// The condition is checked before each iteration.
///
/// # Example
///
/// ```ignore
/// let while_node = WhileBuilder::new("process_queue")
///     .with_condition(check_queue_dag)
///     .with_body(process_dag)
///     .build();
/// ```
pub struct WhileBuilder<T> {
    name: String,
    condition_dag: Option<Dag<T>>,
    body_dag: Option<Dag<T>>,
    max_iterations: Option<usize>,
    // Loop-carried state
    state_type: Option<String>,
    state_port_name: String,
}

impl<T: Clone> WhileBuilder<T> {
    /// Create a new while builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            condition_dag: None,
            body_dag: None,
            max_iterations: None,
            state_type: None,
            state_port_name: "state".to_string(),
        }
    }

    /// Set the condition DAG.
    ///
    /// The condition DAG should output a Bool indicating whether to continue.
    pub fn with_condition(mut self, dag: Dag<T>) -> Self {
        self.condition_dag = Some(dag);
        self
    }

    /// Set the body DAG executed each iteration.
    pub fn with_body(mut self, dag: Dag<T>) -> Self {
        self.body_dag = Some(dag);
        self
    }

    /// Set a maximum iteration limit (safety bound).
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = Some(max);
        self
    }

    /// Configure loop-carried state.
    ///
    /// The body's output state becomes the next iteration's input state.
    pub fn with_state(mut self, type_id: impl Into<String>) -> Self {
        self.state_type = Some(type_id.into());
        self
    }

    /// Configure the state port name.
    pub fn with_state_port(mut self, name: impl Into<String>) -> Self {
        self.state_port_name = name.into();
        self
    }

    /// Build the while pattern as a SubDag node.
    pub fn build(self) -> Node<T>
    where
        T: From<PatternOp>,
    {
        let condition_dag = self.condition_dag.expect("condition DAG is required");
        let body_dag = self.body_dag.expect("body DAG is required");

        let mut dag = Dag::new();

        let state_type = self.state_type.as_deref().unwrap_or("Unit");

        // Initial state input
        dag.add_node(Node::opaque(
            "init",
            vec![Port::scalar(self.state_port_name.as_str(), state_type)],
            vec![Port::scalar("state_out", state_type)],
            T::from(PatternOp::WhileInit {
                input_port: self.state_port_name.clone(),
            }),
        ));

        // Condition subdag
        dag.add_node(Node::subdag(
            "condition",
            vec![Port::scalar("state", state_type)],
            vec![Port::scalar("continue", "Bool")],
            condition_dag,
        ));

        // Body subdag
        dag.add_node(Node::subdag(
            "body",
            vec![
                Port::scalar("state", state_type),
                Port::scalar("iteration", "Int"),
            ],
            vec![Port::scalar("next_state", state_type)],
            body_dag,
        ));

        // Iteration controller
        dag.add_node(Node::opaque(
            "controller",
            vec![
                Port::scalar("continue", "Bool"),
                Port::scalar("next_state", state_type),
            ],
            vec![
                Port::scalar("final_state", state_type),
                Port::scalar("iterations", "Int"),
            ],
            T::from(PatternOp::WhileController {
                max_iterations: self.max_iterations,
            }),
        ));

        // Wire internal nodes
        dag.add_edge(Edge::new("init", "state_out", "condition", "state"));
        dag.add_edge(Edge::new("init", "state_out", "body", "state"));
        dag.add_edge(Edge::new("condition", "continue", "controller", "continue"));
        dag.add_edge(Edge::new("body", "next_state", "controller", "next_state"));

        // Create outer node
        Node::subdag(
            self.name.as_str(),
            vec![Port::scalar(self.state_port_name.as_str(), state_type)],
            vec![
                Port::scalar("final_state", state_type),
                Port::scalar("iterations", "Int"),
            ],
            dag,
        )
    }
}

// ============================================================================
// Poll Pattern
// ============================================================================

/// Builder for the Poll pattern.
///
/// Poll re-executes a body DAG at intervals until success or timeout.
///
/// # Example
///
/// ```ignore
/// let poll_node = PollBuilder::new("wait_for_ready")
///     .with_body(check_ready_dag)
///     .with_interval(Duration::from_secs(5))
///     .with_timeout(Duration::from_secs(300))
///     .build();
/// ```
pub struct PollBuilder<T> {
    name: String,
    body_dag: Option<Dag<T>>,
    interval: Duration,
    timeout: Duration,
    // Port configuration
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> PollBuilder<T> {
    /// Create a new poll builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            body_dag: None,
            interval: Duration::from_secs(1),
            timeout: Duration::from_secs(60),
            input_port_name: "input".to_string(),
            input_port_type: "Any".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "Any".to_string(),
        }
    }

    /// Set the body DAG to poll.
    pub fn with_body(mut self, dag: Dag<T>) -> Self {
        self.body_dag = Some(dag);
        self
    }

    /// Set the polling interval.
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set the total timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Configure the input port.
    pub fn with_input(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self
    }

    /// Configure the output port.
    pub fn with_output(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Build the poll pattern as a SubDag node.
    pub fn build(self) -> Node<T>
    where
        T: From<PatternOp>,
    {
        let body_dag = self.body_dag.expect("body DAG is required");

        let mut dag = Dag::new();

        // Timer node: manages intervals and timeout
        dag.add_node(Node::opaque(
            "timer",
            vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())],
            vec![
                Port::scalar("body_input", self.input_port_type.as_str()),
                Port::scalar("poll_count", "Int"),
                Port::scalar("elapsed_ms", "Int"),
            ],
            T::from(PatternOp::PollTimer {
                input_port: self.input_port_name.clone(),
                interval: self.interval,
                timeout: self.timeout,
            }),
        ));

        // Body subdag
        dag.add_node(Node::subdag(
            "body",
            vec![Port::scalar("input", self.input_port_type.as_str())],
            vec![
                Port::optional("result", self.output_port_type.as_str()),
                Port::scalar("success", "Bool"),
            ],
            body_dag,
        ));

        // Collector: captures result or timeout
        dag.add_node(Node::opaque(
            "collector",
            vec![
                Port::optional("result", self.output_port_type.as_str()),
                Port::scalar("success", "Bool"),
                Port::scalar("poll_count", "Int"),
                Port::scalar("elapsed_ms", "Int"),
            ],
            vec![
                Port::optional(self.output_port_name.as_str(), self.output_port_type.as_str()),
                Port::scalar("success", "Bool"),
                Port::scalar("polls", "Int"),
                Port::scalar("elapsed_ms", "Int"),
            ],
            T::from(PatternOp::PollCollector {
                output_port: self.output_port_name.clone(),
            }),
        ));

        // Wire internal nodes
        dag.add_edge(Edge::new("timer", "body_input", "body", "input"));
        dag.add_edge(Edge::new("timer", "poll_count", "collector", "poll_count"));
        dag.add_edge(Edge::new("timer", "elapsed_ms", "collector", "elapsed_ms"));
        dag.add_edge(Edge::new("body", "result", "collector", "result"));
        dag.add_edge(Edge::new("body", "success", "collector", "success"));

        // Create outer node
        Node::subdag(
            self.name.as_str(),
            vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())],
            vec![
                Port::optional(self.output_port_name.as_str(), self.output_port_type.as_str()),
                Port::scalar("success", "Bool"),
                Port::scalar("polls", "Int"),
                Port::scalar("elapsed_ms", "Int"),
            ],
            dag,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    type TestOp = PatternOp;

    // ============ Retry Tests ============

    #[test]
    fn test_retry_builder_basic() {
        let body: Dag<TestOp> = Dag::new();

        let node = RetryBuilder::new("test_retry")
            .with_body(body)
            .with_policy(RepeatPolicy::fixed(3, Duration::from_secs(1)))
            .build();

        assert_eq!(node.id.0, "test_retry");
        assert!(node.is_subdag());
    }

    #[test]
    fn test_retry_subdag_structure() {
        let body: Dag<TestOp> = Dag::new();

        let node = RetryBuilder::new("test")
            .with_body(body)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 3);
                let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(names.contains(&"controller"));
                assert!(names.contains(&"body"));
                assert!(names.contains(&"collector"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_retry_policy_exponential() {
        let policy = RepeatPolicy::exponential(5, Duration::from_millis(100), 2.0);
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
        match policy.backoff {
            BackoffStrategy::Exponential { factor } => assert_eq!(factor, 2.0),
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_retry_policy_with_timeout() {
        let policy = RepeatPolicy::default().with_timeout(Duration::from_secs(60));
        assert_eq!(policy.timeout, Some(Duration::from_secs(60)));
    }

    // ============ While Tests ============

    #[test]
    fn test_while_builder_basic() {
        let condition: Dag<TestOp> = Dag::new();
        let body: Dag<TestOp> = Dag::new();

        let node = WhileBuilder::new("test_while")
            .with_condition(condition)
            .with_body(body)
            .build();

        assert_eq!(node.id.0, "test_while");
        assert!(node.is_subdag());
    }

    #[test]
    fn test_while_subdag_structure() {
        let condition: Dag<TestOp> = Dag::new();
        let body: Dag<TestOp> = Dag::new();

        let node = WhileBuilder::new("test")
            .with_condition(condition)
            .with_body(body)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 4);
                let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(names.contains(&"init"));
                assert!(names.contains(&"condition"));
                assert!(names.contains(&"body"));
                assert!(names.contains(&"controller"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_while_with_state() {
        let condition: Dag<TestOp> = Dag::new();
        let body: Dag<TestOp> = Dag::new();

        let node = WhileBuilder::new("test")
            .with_condition(condition)
            .with_body(body)
            .with_state("Counter")
            .build();

        // Check that state type is propagated
        assert_eq!(node.inputs[0].type_id.0, "Counter");
    }

    // ============ Poll Tests ============

    #[test]
    fn test_poll_builder_basic() {
        let body: Dag<TestOp> = Dag::new();

        let node = PollBuilder::new("test_poll")
            .with_body(body)
            .with_interval(Duration::from_secs(5))
            .with_timeout(Duration::from_secs(60))
            .build();

        assert_eq!(node.id.0, "test_poll");
        assert!(node.is_subdag());
    }

    #[test]
    fn test_poll_subdag_structure() {
        let body: Dag<TestOp> = Dag::new();

        let node = PollBuilder::new("test")
            .with_body(body)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 3);
                let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(names.contains(&"timer"));
                assert!(names.contains(&"body"));
                assert!(names.contains(&"collector"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_poll_outputs() {
        let body: Dag<TestOp> = Dag::new();

        let node = PollBuilder::new("test")
            .with_body(body)
            .with_output("result", "Status")
            .build();

        // Check outputs include success flag and metrics
        let output_names: Vec<_> = node.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"result"));
        assert!(output_names.contains(&"success"));
        assert!(output_names.contains(&"polls"));
        assert!(output_names.contains(&"elapsed_ms"));
    }

    // ============ Interface Validation Tests ============

    fn make_retry_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "op",
            vec![Port::scalar("input", "Any")],
            vec![
                Port::optional("result", "Any"),
                Port::optional("error", "Error"),
            ],
            PatternOp::RetryCollector { output_port: "result".into() },
        ));
        dag
    }

    #[test]
    fn test_retry_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = RetryBuilder::new("retry")
            .with_body(make_retry_body())
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "retry interface errors: {:?}", errors);
    }

    fn make_while_condition() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "check",
            vec![Port::scalar("state", "Unit")],
            vec![Port::scalar("continue", "Bool")],
            PatternOp::WhileController { max_iterations: None },
        ));
        dag
    }

    fn make_while_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "step",
            vec![
                Port::scalar("state", "Unit"),
                Port::scalar("iteration", "Int"),
            ],
            vec![Port::scalar("next_state", "Unit")],
            PatternOp::WhileInit { input_port: "state".into() },
        ));
        dag
    }

    #[test]
    fn test_while_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = WhileBuilder::new("while")
            .with_condition(make_while_condition())
            .with_body(make_while_body())
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "while interface errors: {:?}", errors);
    }

    fn make_poll_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "check",
            vec![Port::scalar("input", "Any")],
            vec![
                Port::optional("result", "Any"),
                Port::scalar("success", "Bool"),
            ],
            PatternOp::PollCollector { output_port: "result".into() },
        ));
        dag
    }

    #[test]
    fn test_poll_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = PollBuilder::new("poll")
            .with_body(make_poll_body())
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "poll interface errors: {:?}", errors);
    }
}
