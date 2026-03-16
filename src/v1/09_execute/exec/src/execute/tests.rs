use super::*;
use gunbc_ir::build::*;
use gunbc_ir::Edge;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Test operation: produces a fixed value, or passes through inputs if `pass_through` is set.
#[derive(Debug, Clone)]
struct TestOp {
    port: String,
    value: Value,
    pass_through: bool,
}

impl TestOp {
    fn produce(port: &str, value: Value) -> Self {
        Self {
            port: port.to_string(),
            value,
            pass_through: false,
        }
    }

    fn echo() -> Self {
        Self {
            port: String::new(),
            value: Value::Unit,
            pass_through: true,
        }
    }
}

impl Executable for TestOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if self.pass_through {
            return Ok(inputs);
        }
        let mut out = HashMap::new();
        out.insert(self.port.clone(), self.value.clone());
        Ok(out)
    }
}

// Backward-compat alias used in existing tests
type Produce = TestOp;

fn file_response(path: &str, operation: FileOp) -> Value {
    Value::Response(TransportResponse::File(gunbc_ir::transport::FileResponse {
        path: path.to_string(),
        operation,
        success: true,
        content: None,
        bytes: None,
        exists: None,
        error: None,
    }))
}

#[test]
fn test_execute_runs_ready_nodes_in_parallel() {
    if execution_max_concurrency() == 1 {
        return;
    }

    #[derive(Debug, Clone)]
    struct BlockingOp {
        port: String,
        value: Value,
        sleep_ms: u64,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    }

    impl BlockingOp {
        fn new(
            port: &str,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                port: port.to_string(),
                value,
                sleep_ms,
                active,
                peak,
            }
        }
    }

    impl Executable for BlockingOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            loop {
                let observed = self.peak.load(Ordering::SeqCst);
                if current <= observed {
                    break;
                }
                if self
                    .peak
                    .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(self.sleep_ms));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut dag: Dag<BlockingOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("a", "Int")],
        BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![],
        vec![port("b", "Int")],
        BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "C",
        vec![port("a", "Int"), port("b", "Int")],
        vec![port("out", "Int")],
        BlockingOp::new("out", Value::Int(3), 0, active.clone(), peak.clone()),
    ));
    dag.add_edge(edge("A", "a", "C", "a"));
    dag.add_edge(edge("B", "b", "C", "b"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();
    assert_eq!(log.entries.len(), 3);
    assert!(
        peak.load(Ordering::SeqCst) >= 2,
        "expected at least 2 concurrent nodes, saw {}",
        peak.load(Ordering::SeqCst)
    );
}

#[test]
fn test_execute_resource_conflicts_serialize_parallel_writes() {
    if execution_max_concurrency() == 1 {
        return;
    }

    #[derive(Debug, Clone)]
    struct BlockingOp {
        port: String,
        value: Value,
        sleep_ms: u64,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    }

    impl BlockingOp {
        fn new(
            port: &str,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                port: port.to_string(),
                value,
                sleep_ms,
                active,
                peak,
            }
        }
    }

    impl Executable for BlockingOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            loop {
                let observed = self.peak.load(Ordering::SeqCst);
                if current <= observed {
                    break;
                }
                if self
                    .peak
                    .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(self.sleep_ms));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut dag: Dag<BlockingOp> = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs", "FilesystemHandle")],
        BlockingOp::new("fs", Value::Unit, 0, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_a",
        vec![resource(
            "file:shared.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("a", "Int")],
        BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_b",
        vec![resource(
            "file:shared.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("b", "Int")],
        BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
    ));
    dag.add_edge(edge("fs_env", "fs", "writer_a", "res:file:shared.txt"));
    dag.add_edge(edge("fs_env", "fs", "writer_b", "res:file:shared.txt"));

    let _ = execute_dag(&dag, ExecuteConfig::default()).expect("execution should succeed");
    assert_eq!(
        peak.load(Ordering::SeqCst),
        1,
        "conflicting write nodes should be serialized by admission control"
    );
}

#[test]
fn test_execute_resource_reads_can_run_in_parallel() {
    if execution_max_concurrency() == 1 {
        return;
    }

    #[derive(Debug, Clone)]
    struct BlockingOp {
        port: String,
        value: Value,
        sleep_ms: u64,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    }

    impl BlockingOp {
        fn new(
            port: &str,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                port: port.to_string(),
                value,
                sleep_ms,
                active,
                peak,
            }
        }
    }

    impl Executable for BlockingOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            loop {
                let observed = self.peak.load(Ordering::SeqCst);
                if current <= observed {
                    break;
                }
                if self
                    .peak
                    .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(self.sleep_ms));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut dag: Dag<BlockingOp> = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs", "FilesystemHandle")],
        BlockingOp::new("fs", Value::Unit, 0, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "reader_a",
        vec![resource(
            "file:shared.txt",
            "FilesystemHandle",
            AccessMode::Read,
        )],
        vec![port("a", "Int")],
        BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "reader_b",
        vec![resource(
            "file:shared.txt",
            "FilesystemHandle",
            AccessMode::Read,
        )],
        vec![port("b", "Int")],
        BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
    ));
    dag.add_edge(edge("fs_env", "fs", "reader_a", "res:file:shared.txt"));
    dag.add_edge(edge("fs_env", "fs", "reader_b", "res:file:shared.txt"));

    let _ = execute_dag(&dag, ExecuteConfig::default()).expect("execution should succeed");
    assert!(
        peak.load(Ordering::SeqCst) >= 2,
        "read/read nodes should be allowed to run in parallel"
    );
}

#[test]
fn test_execute_resource_coarse_file_conflicts_with_specific_file() {
    if execution_max_concurrency() == 1 {
        return;
    }

    #[derive(Debug, Clone)]
    struct BlockingOp {
        port: String,
        value: Value,
        sleep_ms: u64,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    }

    impl BlockingOp {
        fn new(
            port: &str,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                port: port.to_string(),
                value,
                sleep_ms,
                active,
                peak,
            }
        }
    }

    impl Executable for BlockingOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            loop {
                let observed = self.peak.load(Ordering::SeqCst);
                if current <= observed {
                    break;
                }
                if self
                    .peak
                    .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(self.sleep_ms));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut dag: Dag<BlockingOp> = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs", "FilesystemHandle")],
        BlockingOp::new("fs", Value::Unit, 0, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_all_files",
        vec![resource("file", "FilesystemHandle", AccessMode::Write)],
        vec![port("a", "Int")],
        BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_specific_file",
        vec![resource(
            "file:shared.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("b", "Int")],
        BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
    ));
    dag.add_edge(edge("fs_env", "fs", "writer_all_files", "res:file"));
    dag.add_edge(edge(
        "fs_env",
        "fs",
        "writer_specific_file",
        "res:file:shared.txt",
    ));

    let _ = execute_dag(&dag, ExecuteConfig::default()).expect("execution should succeed");
    assert_eq!(
        peak.load(Ordering::SeqCst),
        1,
        "coarse res:file lock should serialize conflicting specific file writes"
    );
}

#[test]
fn test_execute_resource_distinct_file_writes_can_run_in_parallel() {
    if execution_max_concurrency() == 1 {
        return;
    }

    #[derive(Debug, Clone)]
    struct BlockingOp {
        port: String,
        value: Value,
        sleep_ms: u64,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    }

    impl BlockingOp {
        fn new(
            port: &str,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                port: port.to_string(),
                value,
                sleep_ms,
                active,
                peak,
            }
        }
    }

    impl Executable for BlockingOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            loop {
                let observed = self.peak.load(Ordering::SeqCst);
                if current <= observed {
                    break;
                }
                if self
                    .peak
                    .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(self.sleep_ms));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut dag: Dag<BlockingOp> = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs", "FilesystemHandle")],
        BlockingOp::new("fs", Value::Unit, 0, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_a",
        vec![resource(
            "file:a.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("a", "Int")],
        BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
    ));
    dag.add_node(Node::opaque(
        "writer_b",
        vec![resource(
            "file:b.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("b", "Int")],
        BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
    ));
    dag.add_edge(edge("fs_env", "fs", "writer_a", "res:file:a.txt"));
    dag.add_edge(edge("fs_env", "fs", "writer_b", "res:file:b.txt"));

    let _ = execute_dag(&dag, ExecuteConfig::default()).expect("execution should succeed");
    assert!(
        peak.load(Ordering::SeqCst) >= 2,
        "distinct specific file writes should run in parallel"
    );
}

#[test]
fn test_execute_simple_pipeline() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("hello".to_string())),
    ));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    assert_eq!(log.entries.len(), 1);
    assert_eq!(log.entries[0].node_id, "A");
    match &log.entries[0].outputs.get("out") {
        Some(Value::Str(s)) => assert_eq!(s, "hello"),
        _ => panic!("expected string output"),
    }
}

#[test]
fn test_dry_run_intercepts_transport_executor() {
    // A transport executor node consumes TransportRequest
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(
        Node::opaque(
            "execute_transport",
            // This input marks it as a transport executor - will be intercepted
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            TestOp::produce("response", Value::Str("real-response".to_string())),
        )
        .with_kind(NodeKind::TransportExecute),
    );

    // In dry-run mode, transport executor nodes should be intercepted
    let mut mocks = BoundaryMocks::new();
    mocks.set_value(
        "execute_transport",
        "response",
        Value::Str("mock-response".to_string()),
    );

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(mocks),
            strictness: DryRunStrictness::Lenient,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(log.entries.len(), 1);
    assert!(log.entries[0].was_intercepted);
    match &log.entries[0].outputs.get("response") {
        Some(Value::Str(s)) => assert_eq!(s, "mock-response"),
        _ => panic!("expected mock response"),
    }
}

#[test]
fn test_real_mode_executes_boundary() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "create_gist",
        vec![],
        vec![port("url", "String")],
        TestOp::produce("url", Value::Str("real-url".to_string())),
    ));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    assert_eq!(log.entries.len(), 1);
    assert!(!log.entries[0].was_intercepted);
    match &log.entries[0].outputs.get("url") {
        Some(Value::Str(s)) => assert_eq!(s, "real-url"),
        _ => panic!("expected real url"),
    }
}

#[test]
fn test_pure_node_not_intercepted() {
    // Pure nodes (no TransportRequest input) should never be intercepted
    // Only transport executor nodes should be intercepted
    let mut dag: Dag<Produce> = Dag::new();

    // Pure node - prepares a request but doesn't execute it
    dag.add_node(
        Node::opaque(
            "prepare",
            vec![port("content", "String")],
            vec![port("request", "TransportRequest")],
            TestOp::produce("request", Value::Str("prepared-request".to_string())),
        )
        .with_kind(NodeKind::TransportPrepare),
    );

    // Transport executor - consumes the request (will be intercepted)
    dag.add_node(
        Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")], // This makes it a transport executor
            vec![port("response", "TransportResponse")],
            TestOp::produce("response", Value::Str("real-response".to_string())),
        )
        .with_kind(NodeKind::TransportExecute),
    );
    dag.add_edge(edge("prepare", "request", "execute", "request"));

    let mut mocks = BoundaryMocks::new();
    mocks.set_value("execute", "response", Value::Str("mocked".to_string()));
    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(mocks),
            strictness: DryRunStrictness::Lenient,
            ..Default::default()
        },
    )
    .unwrap();

    // prepare is NOT a transport executor — should execute normally
    let prepare_entry = log.get("prepare").unwrap();
    assert!(!prepare_entry.was_intercepted);

    // execute IS a transport executor — should be intercepted
    let execute_entry = log.get("execute").unwrap();
    assert!(execute_entry.was_intercepted);
}

#[test]
fn test_simulate_basic() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("hello".to_string())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![port("in", "String")],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("world".to_string())),
    ));
    dag.add_edge(edge("A", "out", "B", "in"));

    // Configure simulation with timing
    let config = SimConfig::new()
        .with_timing("A", Duration::from_millis(100))
        .with_timing("B", Duration::from_millis(200))
        .with_mocks(BoundaryMocks::new());

    let result = simulate(&dag, config).unwrap();

    // Check that simulation ran
    assert!(!result.log.entries.is_empty());

    // Check timeline
    assert_eq!(result.timeline.len(), 2);

    // Check total time is sum of node times (sequential execution)
    assert_eq!(result.total_time, Duration::from_millis(300));
}

#[test]
fn test_simulate_with_mocks() {
    // Transport executor node (consumes TransportRequest) should be intercepted in simulation
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(
        Node::opaque(
            "transport_node",
            vec![port("request", "TransportRequest")], // Makes it a transport executor
            vec![port("result", "String")],
            TestOp::produce("result", Value::Str("real-value".to_string())),
        )
        .with_kind(NodeKind::TransportExecute),
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_value(
        "transport_node",
        "result",
        Value::Str("simulated-value".to_string()),
    );

    let config = SimConfig::new().with_mocks(mocks);

    let result = simulate(&dag, config).unwrap();

    // Transport executor should be intercepted with mock value
    let entry = result.log.get("transport_node").unwrap();
    assert!(entry.was_intercepted);
    assert_eq!(
        entry.outputs.get("result"),
        Some(&Value::Str("simulated-value".to_string()))
    );
}

#[test]
fn test_fan_in_to_list_port_collects_values() {
    // Two producers feed into a single fan-in port — values should be collected
    // into a Value::List in canonical edge order.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("alpha".to_string())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("beta".to_string())),
    ));
    dag.add_node(Node::opaque(
        "C",
        vec![list("items", "String")], // fan-in port: multiple edges merge into list
        vec![list("items", "List<String>")], // echo: passes inputs through as outputs
        TestOp::echo(),
    ));
    // Two edges to the same fan-in port, with explicit indices for ordering
    dag.add_edge(Edge::with_index("A", "out", "C", "items", 0));
    dag.add_edge(Edge::with_index("B", "out", "C", "items", 1));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let c_entry = log.get("C").unwrap();
    match c_entry.outputs.get("items") {
        Some(Value::List(items)) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Str("alpha".to_string()));
            assert_eq!(items[1], Value::Str("beta".to_string()));
        }
        other => panic!("expected Value::List, got {:?}", other),
    }
}

#[test]
fn test_coercion_trace_exposes_coerced_input_value() {
    // Fan-in port receives a scalar, wraps into list.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("alpha".to_string())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(Edge::new("A", "out", "B", "items"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();
    let b_entry = log.get("B").unwrap();

    // Fan-in wraps scalar into list
    let received = b_entry.input_value("items").unwrap();
    assert!(
        matches!(received, Value::List(values)
            if values == &vec![Value::Str("alpha".to_string())]),
        "fan-in should wrap scalar as single-element list, got {received:?}"
    );
}

#[test]
fn test_list_output_to_list_input_passes_through() {
    // A list output feeding a list input should not become a list-of-lists.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![list("items", "List<String>")],
        TestOp::produce(
            "items",
            Value::List(vec![
                Value::Str("alpha".to_string()),
                Value::Str("beta".to_string()),
            ]),
        ),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![list("items", "List<String>")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "items", "B", "items"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let b_entry = log.get("B").unwrap();
    match b_entry.outputs.get("items") {
        Some(Value::List(items)) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Str("alpha".to_string()));
            assert_eq!(items[1], Value::Str("beta".to_string()));
        }
        other => panic!("expected Value::List, got {:?}", other),
    }

    assert!(
        b_entry.coercions_applied.is_empty(),
        "list->list flow should not record scalar/list coercions"
    );
}

#[test]
fn test_scalar_port_takes_single_value() {
    // A scalar port with one incoming edge should still work as before.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("hello".to_string())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![port("data", "String")],
        vec![port("data", "String")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "out", "B", "data"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    // B echoes its input — should receive the scalar value from A
    let b_entry = log.get("B").unwrap();
    assert_eq!(
        b_entry.outputs.get("data"),
        Some(&Value::Str("hello".to_string()))
    );
}

#[test]
fn test_scalar_port_fan_in_takes_last_non_skipped() {
    // A scalar port with one Skipped edge and one real edge should take the
    // non-Skipped value (conditional merge: only one branch fires).
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Skipped),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("beta".to_string())),
    ));
    dag.add_node(Node::opaque(
        "C",
        vec![port("data", "String")],
        vec![port("data", "String")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "out", "C", "data"));
    dag.add_edge(edge("B", "out", "C", "data"));

    let log = execute_dag(&dag, ExecuteConfig::default()).expect("scalar fan-in should succeed");
    let c_entry = log.get("C").expect("C should have executed");
    let data = c_entry.outputs.get("data").expect("C should produce data");
    assert_eq!(
        data,
        &Value::Str("beta".to_string()),
        "expected the non-Skipped value 'beta', got {:?}",
        data
    );
}

#[test]
fn test_scalar_port_fan_in_rejects_multiple_non_skipped() {
    // Two non-Skipped values arriving at a scalar port means two conditional
    // branches both fired, which is a structural error.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("alpha".to_string())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("beta".to_string())),
    ));
    dag.add_node(Node::opaque(
        "C",
        vec![port("data", "String")],
        vec![port("data", "String")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "out", "C", "data"));
    dag.add_edge(edge("B", "out", "C", "data"));

    let err = execute_dag(&dag, ExecuteConfig::default())
        .expect_err("should reject multiple non-Skipped values at scalar port");
    let msg = err.to_string();
    assert!(
        msg.contains("conditional merge error"),
        "expected conditional merge error, got: {msg}"
    );
}

#[test]
fn test_list_port_zero_edges_defaults_to_empty_list() {
    // A fan-in port with no incoming edges should default to an empty list.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let a_entry = log.get("A").unwrap();
    match a_entry.outputs.get("items") {
        Some(Value::List(items)) => assert!(items.is_empty()),
        other => panic!("expected empty Value::List, got {:?}", other),
    }
}

#[test]
fn test_optional_to_list_skips_unit() {
    // Optional output (Unit) feeding a fan-in input should preserve Unit as
    // an explicit dependency token.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![optional("item", "Optional<String>")],
        TestOp::produce("item", Value::Unit),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "item", "B", "items"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let b_entry = log.get("B").unwrap();
    match b_entry.outputs.get("items") {
        Some(Value::List(items)) => assert_eq!(items, &vec![Value::Unit]),
        other => panic!("expected Value::List([Unit]), got {:?}", other),
    }
}

#[test]
fn test_optional_to_list_skips_skipped() {
    // Skipped output feeding a fan-in input should not insert Skipped.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![optional("item", "Optional<String>")],
        TestOp::produce("item", Value::Skipped),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "item", "B", "items"));

    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let b_entry = log.get("B").unwrap();
    match b_entry.outputs.get("items") {
        Some(Value::List(items)) => assert!(items.is_empty()),
        other => panic!("expected empty Value::List, got {:?}", other),
    }
}

// =========================================================================
// collect_fan_in unit tests
//
// These test the extracted fan-in function directly, mapping 1:1 to the
// CoercionKind variants in coerce.rs.
// =========================================================================

#[test]
fn fan_in_wraps_scalar() {
    // WrapScalar: scalar [1,1] value → single-element vec
    let val = Value::Str("hello".into());
    let elements = collect_fan_in(&val, Cardinality::ONE).unwrap();
    assert_eq!(elements, vec![Value::Str("hello".into())]);
}

#[test]
fn fan_in_skips_absent_optional() {
    // OptionalToList uses Unit as a dependency token for absent optionals,
    // so it should be retained when flowing into list fan-in ports.
    let elements = collect_fan_in(&Value::Unit, Cardinality::ZERO_OR_ONE).unwrap();
    assert_eq!(elements, vec![Value::Unit]);
}

#[test]
fn fan_in_wraps_present_optional() {
    // OptionalToList (present): real value from [0,1] port → single-element vec
    let val = Value::Str("present".into());
    let elements = collect_fan_in(&val, Cardinality::ZERO_OR_ONE).unwrap();
    assert_eq!(elements, vec![Value::Str("present".into())]);
}

#[test]
fn fan_in_flattens_list() {
    // Widen: list [2,5] value → flattened elements
    let val = Value::List(vec![
        Value::Str("a".into()),
        Value::Str("b".into()),
        Value::Str("c".into()),
    ]);
    let elements = collect_fan_in(&val, Cardinality::new(2, Some(5))).unwrap();
    assert_eq!(
        elements,
        vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]
    );
}

#[test]
fn fan_in_skips_skipped_value() {
    // Skipped sentinel is never collected regardless of cardinality
    assert!(collect_fan_in(&Value::Skipped, Cardinality::ONE).is_none());
    assert!(collect_fan_in(&Value::Skipped, Cardinality::ZERO_OR_ONE).is_none());
    assert!(collect_fan_in(&Value::Skipped, Cardinality::ZERO_OR_MORE).is_none());
}

#[test]
fn fan_in_unit_from_required_port_is_kept() {
    // Unit from a required [1,1] port is NOT skipped — only empty-allowing
    // ports treat Unit as absence.
    let elements = collect_fan_in(&Value::Unit, Cardinality::ONE).unwrap();
    assert_eq!(elements, vec![Value::Unit]);
}

#[test]
fn fan_in_skips_unit_from_empty_list() {
    // Unit from a [0,∞) port is retained as an explicit dependency token.
    let elements = collect_fan_in(&Value::Unit, Cardinality::ZERO_OR_MORE).unwrap();
    assert_eq!(elements, vec![Value::Unit]);
}

#[test]
fn runtime_file_guard_allows_matching_declared_path() {
    let node = Node::opaque(
        "writer",
        vec![resource(
            "file:out.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "response".to_string(),
        file_response("out.txt", FileOp::Write),
    );

    enforce_runtime_file_guard(&node, &outputs, true)
        .expect("matching file declaration should be accepted");
}

#[test]
fn runtime_file_guard_rejects_missing_declaration() {
    let node = Node::opaque(
        "writer",
        vec![],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "response".to_string(),
        file_response("out.txt", FileOp::Write),
    );

    let err = enforce_runtime_file_guard(&node, &outputs, true)
        .expect_err("missing declaration should be rejected");
    let msg = err.to_string();
    assert!(msg.contains("runtime file guard"));
    assert!(msg.contains("out.txt"));
}

#[test]
fn runtime_file_guard_rejects_mismatched_declared_path() {
    let node = Node::opaque(
        "writer",
        vec![resource(
            "file:other.txt",
            "FilesystemHandle",
            AccessMode::Write,
        )],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "response".to_string(),
        file_response("out.txt", FileOp::Write),
    );

    let err = enforce_runtime_file_guard(&node, &outputs, true)
        .expect_err("mismatched declaration should be rejected");
    let msg = err.to_string();
    assert!(msg.contains("out.txt"));
    assert!(msg.contains("other.txt"));
}

#[test]
fn runtime_file_guard_wildcard_normalized_to_coarse_file() {
    // Wildcard `file:*` is normalized to coarse `file` at Port construction
    // time (R2: wildcard resource semantics deferred).
    let wildcard_node = Node::opaque(
        "wildcard_writer",
        vec![resource("file:*", "FilesystemHandle", AccessMode::Write)],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );
    // Verify normalization: the port name should be `res:file`, not `res:file:*`.
    assert_eq!(
        wildcard_node.inputs[0].name.0, "res:file",
        "wildcard file:* must be normalized to coarse res:file at construction"
    );

    let coarse_node = Node::opaque(
        "coarse_writer",
        vec![resource("file", "FilesystemHandle", AccessMode::Write)],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "response".to_string(),
        file_response("nested/out.txt", FileOp::Append),
    );

    enforce_runtime_file_guard(&wildcard_node, &outputs, true)
        .expect("normalized coarse res:file should allow any file write");
    enforce_runtime_file_guard(&coarse_node, &outputs, true)
        .expect("coarse res:file should allow writes");
}

#[test]
fn runtime_file_guard_requires_write_or_exclusive_access_mode() {
    let node = Node::opaque(
        "writer",
        vec![resource(
            "file:out.txt",
            "FilesystemHandle",
            AccessMode::Read,
        )],
        vec![port("response", "TransportResponse")],
        TestOp::echo(),
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "response".to_string(),
        file_response("out.txt", FileOp::Write),
    );

    let err = enforce_runtime_file_guard(&node, &outputs, true)
        .expect_err("read-only declaration should not satisfy write guard");
    assert!(err.to_string().contains("AccessMode::Write/Exclusive"));
}

#[test]
fn test_sim_config_builder() {
    let config = SimConfig::new()
        .with_timing("node1", Duration::from_secs(1))
        .with_timing("node2", Duration::from_secs(2))
        .with_seed(42)
        .with_resources(
            ResourceBudget::unlimited()
                .with_memory(1024 * 1024)
                .with_cpu(5000)
                .with_concurrency(4),
        );

    assert_eq!(
        config.node_duration(&NodeId::from("node1")),
        Duration::from_secs(1)
    );
    assert_eq!(
        config.node_duration(&NodeId::from("node2")),
        Duration::from_secs(2)
    );
    assert_eq!(
        config.node_duration(&NodeId::from("unknown")),
        Duration::ZERO
    );
    assert_eq!(config.random_seed, Some(42));
    assert_eq!(config.resources.max_memory, Some(1024 * 1024));
    assert_eq!(config.resources.max_cpu_ms, Some(5000));
    assert_eq!(config.resources.max_concurrency, Some(4));
}

#[test]
fn test_loop_body_executes_per_element() {
    use gunbc_ir::patterns::{LoopBuilder, PatternOp};

    // Build a body DAG with a single transform node that appends "_processed"
    #[derive(Debug, Clone)]
    enum TestLoopOp {
        Pattern(PatternOp),
        AppendSuffix,
    }

    impl From<PatternOp> for TestLoopOp {
        fn from(op: PatternOp) -> Self {
            TestLoopOp::Pattern(op)
        }
    }

    impl Executable for TestLoopOp {
        fn execute(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            match self {
                TestLoopOp::Pattern(op) => op.execute(inputs),
                TestLoopOp::AppendSuffix => {
                    let element = inputs
                        .get("element")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let mut out = HashMap::new();
                    out.insert(
                        "result".to_string(),
                        Value::Str(format!("{}_processed", element)),
                    );
                    Ok(out)
                }
            }
        }
    }

    // Body DAG: single node that takes "element" and outputs "result"
    let mut body_dag: Dag<TestLoopOp> = Dag::new();
    body_dag.add_node(Node::opaque(
        "transform",
        vec![port("element", "String")],
        vec![port("result", "String")],
        TestLoopOp::AppendSuffix,
    ));

    // Build the loop node
    let loop_node: Node<TestLoopOp> = LoopBuilder::new("test_loop")
        .with_input("items", "String", Cardinality::ZERO_OR_MORE)
        .with_element("element", "String")
        .with_body(body_dag)
        .with_output("results", "String")
        .build();

    // Build a DAG: producer → loop → consumer
    let mut dag: Dag<TestLoopOp> = Dag::new();
    dag.add_node(Node::opaque(
        "source",
        vec![],
        vec![list("items", "List<String>")],
        TestLoopOp::Pattern(PatternOp::LoopUnpack {
            // Repurpose as a producer that outputs a list
            input_port: "unused".to_string(),
            element_port: "items".to_string(),
        }),
    ));

    // Actually, let's use a simpler approach: use input mocks
    let mut dag: Dag<TestLoopOp> = Dag::new();
    dag.add_node(loop_node);

    // Use input mocks to inject the list (set_input for DAG entry injection)
    let mut mocks = BoundaryMocks::new();
    mocks.set_input(
        "test_loop",
        "items",
        Value::List(vec![
            Value::Str("alpha".to_string()),
            Value::Str("beta".to_string()),
            Value::Str("gamma".to_string()),
        ]),
    );

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();

    // Find the pack node's output
    let pack_entry = log
        .entries
        .iter()
        .find(|e| e.node_id.ends_with("/pack"))
        .expect("should have a pack node entry");

    match pack_entry.outputs.get("results") {
        Some(Value::List(items)) => {
            assert_eq!(items.len(), 3, "should have 3 processed items");
            assert_eq!(items[0], Value::Str("alpha_processed".to_string()));
            assert_eq!(items[1], Value::Str("beta_processed".to_string()));
            assert_eq!(items[2], Value::Str("gamma_processed".to_string()));
        }
        other => panic!("expected Value::List, got {:?}", other),
    }

    // Verify iteration count
    match pack_entry.outputs.get("iterations") {
        Some(Value::Int(n)) => assert_eq!(*n, 3),
        other => panic!("expected iterations=3, got {:?}", other),
    }
}

#[test]
fn test_loop_empty_list_produces_empty_output() {
    use gunbc_ir::patterns::{LoopBuilder, PatternOp};

    #[derive(Debug, Clone)]
    enum TestLoopOp {
        Pattern(PatternOp),
        Identity,
    }

    impl From<PatternOp> for TestLoopOp {
        fn from(op: PatternOp) -> Self {
            TestLoopOp::Pattern(op)
        }
    }

    impl Executable for TestLoopOp {
        fn execute(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            match self {
                TestLoopOp::Pattern(op) => op.execute(inputs),
                TestLoopOp::Identity => {
                    let mut out = HashMap::new();
                    if let Some(v) = inputs.get("element") {
                        out.insert("result".to_string(), v.clone());
                    }
                    Ok(out)
                }
            }
        }
    }

    let mut body_dag: Dag<TestLoopOp> = Dag::new();
    body_dag.add_node(Node::opaque(
        "passthrough",
        vec![port("element", "String")],
        vec![port("result", "String")],
        TestLoopOp::Identity,
    ));

    let loop_node: Node<TestLoopOp> = LoopBuilder::new("empty_loop")
        .with_input("items", "String", Cardinality::ZERO_OR_MORE)
        .with_element("element", "String")
        .with_body(body_dag)
        .with_output("results", "String")
        .build();

    let mut dag: Dag<TestLoopOp> = Dag::new();
    dag.add_node(loop_node);

    // Inject empty list (set_input for DAG entry injection)
    let mut mocks = BoundaryMocks::new();
    mocks.set_input("empty_loop", "items", Value::List(vec![]));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();

    let pack_entry = log
        .entries
        .iter()
        .find(|e| e.node_id.ends_with("/pack"))
        .expect("should have a pack node entry");

    match pack_entry.outputs.get("results") {
        Some(Value::List(items)) => assert!(items.is_empty()),
        other => panic!("expected empty Value::List, got {:?}", other),
    }
}

#[test]
fn test_loop_resource_input_flows_to_body_iterations() {
    use gunbc_ir::patterns::{LoopBuilder, PatternOp, ResourceInput};

    #[derive(Debug, Clone)]
    enum TestLoopOp {
        Pattern(PatternOp),
        ConcatToken,
    }

    impl From<PatternOp> for TestLoopOp {
        fn from(op: PatternOp) -> Self {
            TestLoopOp::Pattern(op)
        }
    }

    impl Executable for TestLoopOp {
        fn execute(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            match self {
                TestLoopOp::Pattern(op) => op.execute(inputs),
                TestLoopOp::ConcatToken => {
                    let element = inputs
                        .get("element")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExecError::new("missing element"))?;
                    let token = inputs
                        .get("res:token")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExecError::new("missing res:token"))?;

                    let mut out = HashMap::new();
                    out.insert(
                        "result".to_string(),
                        Value::Str(format!("{}@{}", element, token)),
                    );
                    Ok(out)
                }
            }
        }
    }

    let mut body_dag: Dag<TestLoopOp> = Dag::new();
    body_dag.add_node(Node::opaque(
        "transform",
        vec![port("element", "String"), port("res:token", "String")],
        vec![port("result", "String")],
        TestLoopOp::ConcatToken,
    ));

    let loop_node: Node<TestLoopOp> = LoopBuilder::new("token_loop")
        .with_input("items", "String", Cardinality::ZERO_OR_MORE)
        .with_element("element", "String")
        .with_resource_input(ResourceInput::new("res:token", "String"))
        .with_body(body_dag)
        .with_output("results", "String")
        .build();

    let mut dag: Dag<TestLoopOp> = Dag::new();
    dag.add_node(loop_node);

    let mut mocks = BoundaryMocks::new();
    mocks.set_input(
        "token_loop",
        "items",
        Value::List(vec![
            Value::Str("alpha".to_string()),
            Value::Str("beta".to_string()),
        ]),
    );
    mocks.set_input("token_loop", "res:token", Value::Str("t".to_string()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .expect("loop execution should succeed with resource input");

    let pack_entry = log
        .entries
        .iter()
        .find(|e| e.node_id.ends_with("/pack"))
        .expect("should have a pack node entry");

    match pack_entry.outputs.get("results") {
        Some(Value::List(items)) => {
            assert_eq!(
                items,
                &vec![
                    Value::Str("alpha@t".to_string()),
                    Value::Str("beta@t".to_string()),
                ]
            );
        }
        other => panic!("expected Value::List, got {:?}", other),
    }
}

// =========================================================================
// execute_dag with input_mocks unit tests
// =========================================================================

#[test]
fn test_input_mocks_inject_into_entrypoint() {
    // Node with no upstream edges receives value from input mocks
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "echo",
        vec![port("data", "String")],
        vec![port("data", "String")],
        TestOp::echo(),
    ));

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("echo", "data", Value::Str("injected".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();

    let entry = log.get("echo").unwrap();
    assert_eq!(
        entry.outputs.get("data"),
        Some(&Value::Str("injected".into())),
        "input mock should be injected into entrypoint"
    );
}

#[test]
fn test_input_mocks_with_dry_run_mode() {
    // Combine input mocks with DryRun boundary interception
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(
        Node::opaque(
            "prepare",
            vec![port("arg", "String")],
            vec![port("request", "TransportRequest")],
            TestOp::produce("request", Value::Str("built-request".into())),
        )
        .with_kind(NodeKind::TransportPrepare),
    );
    dag.add_node(
        Node::opaque(
            "execute_http",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            TestOp::produce("response", Value::Str("real-response".into())),
        )
        .with_kind(NodeKind::TransportExecute),
    );
    dag.add_edge(edge("prepare", "request", "execute_http", "request"));

    // DryRun mocks intercept the transport executor
    let mut dry_mocks = BoundaryMocks::new();
    dry_mocks.set_value(
        "execute_http",
        "response",
        Value::Str("mock-response".into()),
    );

    // Input mocks inject the entrypoint arg
    let mut input_mocks = BoundaryMocks::new();
    input_mocks.set_input("prepare", "arg", Value::Str("injected-arg".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(dry_mocks),
            input_mocks: Some(&input_mocks),
            ..Default::default()
        },
    )
    .unwrap();

    // prepare should run normally with the injected input
    let prepare = log.get("prepare").unwrap();
    assert!(!prepare.was_intercepted);

    // execute_http should be intercepted
    let exec = log.get("execute_http").unwrap();
    assert!(exec.was_intercepted);
    assert_eq!(
        exec.outputs.get("response"),
        Some(&Value::Str("mock-response".into()))
    );
}

#[test]
fn test_input_mocks_per_port_on_non_root_node() {
    // Node B has two inputs: x (wired from A) and y (unwired entrypoint).
    // Input mock injects B.y; B.x should come from A's output.
    // This verifies per-port entrypoint injection, not per-node.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("from-A".into())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![port("x", "String"), port("y", "String")],
        vec![port("x", "String"), port("y", "String")],
        TestOp::echo(), // echoes all inputs as outputs
    ));
    dag.add_edge(edge("A", "out", "B", "x"));

    // Provide input mock for the unwired entrypoint port B.y
    let mut input_mocks = BoundaryMocks::new();
    input_mocks.set_input("B", "y", Value::Str("from-mock".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&input_mocks),
            ..Default::default()
        },
    )
    .unwrap();

    let b = log.get("B").unwrap();
    assert_eq!(
        b.outputs.get("x"),
        Some(&Value::Str("from-A".into())),
        "wired port B.x should receive value from upstream A"
    );
    assert_eq!(
        b.outputs.get("y"),
        Some(&Value::Str("from-mock".into())),
        "unwired entrypoint port B.y should receive value from input mock"
    );
}

#[test]
fn test_input_mocks_none_works() {
    // Passing None for input_mocks should work the same as default config
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("hello".into())),
    ));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(log.entries.len(), 1);
    assert_eq!(
        log.entries[0].outputs.get("out"),
        Some(&Value::Str("hello".into()))
    );
}

#[test]
fn test_input_mocks_do_not_intercept_in_real_mode() {
    // Real mode should never allow output interception from input_mocks.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "compute",
        vec![],
        vec![port("out", "String")],
        TestOp::produce("out", Value::Str("real".into())),
    ));

    let mut input_mocks = BoundaryMocks::new();
    input_mocks.set_value("compute", "out", Value::Str("mocked".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&input_mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let entry = log.get("compute").expect("compute entry should exist");
    assert!(!entry.was_intercepted);
    assert_eq!(entry.outputs.get("out"), Some(&Value::Str("real".into())));
}

#[test]
fn test_log_detail_node_override_captures_inputs() {
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(
        Node::opaque(
            "echo",
            vec![port("data", "String")],
            vec![port("data", "String")],
            TestOp::echo(),
        )
        .with_log_detail(LogDetailLevel::IncludeInputs),
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("echo", "data", Value::Str("captured".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let entry = log.get("echo").expect("echo entry must exist");

    let inputs = entry.inputs.as_ref().expect("inputs should be captured");
    assert_eq!(inputs.get("data"), Some(&Value::Str("captured".into())));
}

#[test]
fn test_log_detail_input_port_override_include_only() {
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(
        Node::opaque(
            "echo",
            vec![port("x", "String"), port("y", "String")],
            vec![port("x", "String"), port("y", "String")],
            TestOp::echo(),
        )
        .with_log_detail(LogDetailLevel::Basic)
        .with_input_log_detail("x", LogDetailLevel::IncludeInputs),
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("echo", "x", Value::Str("xv".into()));
    mocks.set_input("echo", "y", Value::Str("yv".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let entry = log.get("echo").expect("echo entry must exist");
    let inputs = entry.inputs.as_ref().expect("x should be captured");
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs.get("x"), Some(&Value::Str("xv".into())));
    assert!(!inputs.contains_key("y"));
}

#[test]
fn test_log_detail_input_port_override_can_suppress_node_default() {
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(
        Node::opaque(
            "echo",
            vec![port("public", "String"), port("secret", "String")],
            vec![port("public", "String"), port("secret", "String")],
            TestOp::echo(),
        )
        .with_log_detail(LogDetailLevel::IncludeInputs)
        .with_input_log_detail("secret", LogDetailLevel::Basic),
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("echo", "public", Value::Str("p".into()));
    mocks.set_input("echo", "secret", Value::Str("s".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let entry = log.get("echo").expect("echo entry must exist");
    let inputs = entry
        .inputs
        .as_ref()
        .expect("public should still be captured");
    assert_eq!(inputs.get("public"), Some(&Value::Str("p".into())));
    assert!(!inputs.contains_key("secret"));
}

#[test]
fn test_log_detail_subdag_override_inherits_to_inner_nodes() {
    let mut inner: Dag<TestOp> = Dag::new();
    inner.add_node(Node::opaque(
        "inner",
        vec![port("data", "String")],
        vec![port("data", "String")],
        TestOp::echo(),
    ));

    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::subdag("wrapper", inner).with_log_detail(LogDetailLevel::IncludeInputs));

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("wrapper", "data", Value::Str("v".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let entry = log
        .get("wrapper/inner")
        .expect("lowered inner node entry must exist");
    let inputs = entry
        .inputs
        .as_ref()
        .expect("subdag log detail should propagate to inner node");
    assert_eq!(inputs.get("data"), Some(&Value::Str("v".into())));
}

#[test]
fn test_remap_input_mocks_preserves_non_subdag() {
    // remap_input_mocks should keep original entries alongside remapped ones.
    // We test this indirectly via execute_dag with input_mocks on a flat DAG.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "a",
        vec![port("x", "String")],
        vec![port("x", "String")],
        TestOp::echo(),
    ));
    dag.add_node(Node::opaque(
        "b",
        vec![port("y", "String")],
        vec![port("y", "String")],
        TestOp::echo(),
    ));

    let mut input = BoundaryMocks::new();
    input.set_input("a", "x", Value::Str("alpha".into()));
    input.set_input("b", "y", Value::Str("beta".into()));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: Some(&input),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        log.get("a").unwrap().outputs.get("x"),
        Some(&Value::Str("alpha".into()))
    );
    assert_eq!(
        log.get("b").unwrap().outputs.get("y"),
        Some(&Value::Str("beta".into()))
    );
}

// =========================================================================
// remap_input_mocks unit tests
// =========================================================================

#[test]
fn test_remap_input_mocks_with_remaps() {
    let mut mocks = BoundaryMocks::new();
    mocks.set_input("subdag", "port_a", Value::Str("value".into()));

    let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    remaps.insert(
        ("subdag".to_string(), "port_a".to_string()),
        vec![("subdag/inner_entry".to_string(), "inner_port".to_string())],
    );

    let result = remap_input_mocks(&mocks, &remaps);

    // Original key should still exist
    assert_eq!(
        result.get_input("subdag", "port_a"),
        Some(&Value::Str("value".into()))
    );
    // Remapped key should also exist
    assert_eq!(
        result.get_input("subdag/inner_entry", "inner_port"),
        Some(&Value::Str("value".into()))
    );
}

#[test]
fn test_remap_input_mocks_empty_remaps() {
    let mut mocks = BoundaryMocks::new();
    mocks.set_input("node", "port", Value::Int(42));

    let remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();

    let result = remap_input_mocks(&mocks, &remaps);
    assert_eq!(
        result.get_input("node", "port"),
        Some(&Value::Int(42)),
        "empty remaps should preserve all inputs"
    );
}

#[test]
fn test_remap_input_mocks_multi_target() {
    let mut mocks = BoundaryMocks::new();
    mocks.set_input("subdag", "data", Value::Str("shared".into()));

    let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    remaps.insert(
        ("subdag".to_string(), "data".to_string()),
        vec![
            ("subdag/inner_a".to_string(), "input_a".to_string()),
            ("subdag/inner_b".to_string(), "input_b".to_string()),
        ],
    );

    let result = remap_input_mocks(&mocks, &remaps);

    // Both targets should receive the value
    assert_eq!(
        result.get_input("subdag/inner_a", "input_a"),
        Some(&Value::Str("shared".into()))
    );
    assert_eq!(
        result.get_input("subdag/inner_b", "input_b"),
        Some(&Value::Str("shared".into()))
    );
}

#[test]
fn test_remap_mode_inputs_dry_run() {
    let mut dry_mocks = BoundaryMocks::new();
    dry_mocks.set_input("subdag", "port", Value::Int(99));

    let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    remaps.insert(
        ("subdag".to_string(), "port".to_string()),
        vec![("subdag/inner".to_string(), "inner_port".to_string())],
    );

    let mode = ExecutionMode::DryRun(dry_mocks);
    let result = remap_mode_inputs(mode, &remaps);

    match result {
        ExecutionMode::DryRun(mocks) => {
            assert_eq!(
                mocks.get_input("subdag/inner", "inner_port"),
                Some(&Value::Int(99)),
                "DryRun mocks should be remapped"
            );
        }
        _ => panic!("expected DryRun mode"),
    }
}

#[test]
fn test_remap_mode_inputs_simulate() {
    let mut sim_mocks = BoundaryMocks::new();
    sim_mocks.set_input("subdag", "port", Value::Int(99));

    let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    remaps.insert(
        ("subdag".to_string(), "port".to_string()),
        vec![("subdag/inner".to_string(), "inner_port".to_string())],
    );

    let mode = ExecutionMode::Simulate(SimConfig::new().with_mocks(sim_mocks));
    let result = remap_mode_inputs(mode, &remaps);

    match result {
        ExecutionMode::Simulate(config) => {
            assert_eq!(
                config.boundary_mocks.get_input("subdag/inner", "inner_port"),
                Some(&Value::Int(99)),
                "Simulate boundary mocks should be remapped"
            );
        }
        _ => panic!("expected Simulate mode"),
    }
}

#[test]
fn test_remap_mode_inputs_real_unchanged() {
    let remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    let mode = remap_mode_inputs(ExecutionMode::Real, &remaps);
    assert!(matches!(mode, ExecutionMode::Real));
}

// =========================================================================
// Coercion tracking in execution trace (CO6)
// =========================================================================

#[test]
fn test_coercion_tracking_wrap_scalar() {
    // A scalar output → fan-in input should collect the value as a single-element list.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "producer",
        vec![],
        vec![scalar("value", "String")],
        TestOp::produce("value", Value::Str("hello".into())),
    ));
    dag.add_node(Node::opaque(
        "consumer",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("producer", "value", "consumer", "items"));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: None,
            log_detail: LogDetailLevel::IncludeInputs,
            ..Default::default()
        },
    )
    .unwrap();

    let consumer_entry = log.get("consumer").unwrap();
    // Fan-in wraps scalar into a list
    match consumer_entry.outputs.get("items") {
        Some(Value::List(items)) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Value::Str("hello".into()));
        }
        other => panic!("expected Value::List, got {:?}", other),
    }
}

#[test]
fn test_coercion_tracking_no_coercion_for_matching_cardinality() {
    // Scalar → scalar should have no coercions recorded.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![scalar("out", "String")],
        TestOp::produce("out", Value::Str("x".into())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![scalar("input", "String")],
        vec![scalar("result", "String")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "out", "B", "input"));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: None,
            log_detail: LogDetailLevel::IncludeInputs,
            ..Default::default()
        },
    )
    .unwrap();

    let b_entry = log.get("B").unwrap();
    assert!(
        b_entry.coercions_applied.is_empty(),
        "no coercion should be recorded for matching cardinalities"
    );
}

#[test]
fn test_coercion_tracking_optional_to_list() {
    // Optional [0,1] → fan-in port should collect the value.
    let mut dag: Dag<TestOp> = Dag::new();
    dag.add_node(Node::opaque(
        "A",
        vec![],
        vec![optional("item", "Optional<String>")],
        TestOp::produce("item", Value::Str("present".into())),
    ));
    dag.add_node(Node::opaque(
        "B",
        vec![list("items", "String")],
        vec![list("items", "List<String>")],
        TestOp::echo(),
    ));
    dag.add_edge(edge("A", "item", "B", "items"));

    let log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::Real,
            input_mocks: None,
            log_detail: LogDetailLevel::IncludeInputs,
            ..Default::default()
        },
    )
    .unwrap();

    let b_entry = log.get("B").unwrap();
    match b_entry.outputs.get("items") {
        Some(Value::List(items)) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Value::Str("present".into()));
        }
        other => panic!("expected Value::List, got {:?}", other),
    }
}

#[test]
fn validate_node_kinds_rejects_kindless_transport_node() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "transport",
        vec![port("req", "TransportRequest")],
        vec![port("resp", "TransportResponse")],
        Produce::produce("resp", Value::Str("ok".into())),
    ));
    let err = validate_node_kinds_for_interception(&dag);
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("transport"),
        "expected transport mention: {msg}"
    );
    assert!(
        msg.contains("kind: Pure"),
        "expected kind: None mention: {msg}"
    );
}

#[test]
fn validate_node_kinds_rejects_kindless_tool_consumer() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "consumer",
        vec![port("tool:clippy", "ToolHandle")],
        vec![port("result", "String")],
        Produce::produce("result", Value::Str("clean".into())),
    ));
    let err = validate_node_kinds_for_interception(&dag);
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("ToolHandle input"));
}

#[test]
fn validate_node_kinds_rejects_kindless_tool_environment() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "env",
        vec![],
        vec![port("tool:clippy", "ToolHandle")],
        Produce::produce("tool:clippy", Value::Str("handle".into())),
    ));
    let err = validate_node_kinds_for_interception(&dag);
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("ToolHandle output"));
}

#[test]
fn validate_node_kinds_rejects_kindless_resource_environment() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("handle", "FilesystemHandle")],
        Produce::produce("handle", Value::Str("fs".into())),
    ));
    let err = validate_node_kinds_for_interception(&dag);
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("resource-environment"));
}

#[test]
fn validate_node_kinds_accepts_pure_kindless_node() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "pure",
        vec![port("input", "String")],
        vec![port("output", "String")],
        Produce::produce("output", Value::Str("ok".into())),
    ));
    let result = validate_node_kinds_for_interception(&dag);
    assert!(result.is_ok());
}

#[test]
fn validate_node_kinds_accepts_classified_transport_node() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(
        Node::opaque(
            "transport",
            vec![port("req", "TransportRequest")],
            vec![port("resp", "TransportResponse")],
            Produce::produce("resp", Value::Str("ok".into())),
        )
        .with_kind(NodeKind::TransportExecute),
    );
    let result = validate_node_kinds_for_interception(&dag);
    assert!(result.is_ok());
}

#[test]
fn dry_run_rejects_kindless_effectful_node() {
    let mut dag: Dag<Produce> = Dag::new();
    dag.add_node(Node::opaque(
        "transport",
        vec![port("req", "TransportRequest")],
        vec![port("resp", "TransportResponse")],
        Produce::produce("resp", Value::Str("ok".into())),
    ));
    let mocks = BoundaryMocks::new();
    let result = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(mocks),
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("kind: Pure"));
}

// =========================================================================
// Tiered test execution (T11 — Phase 4b)
//
// The test suite uses three execution tiers to provide layered confidence:
//
//   Tier 1 — DryRun (structure):
//     All transport/resource/tool nodes are intercepted with explicit mocks.
//     Proves DAG wiring, port cardinality, coercion, conditional skip, and
//     topological ordering. No real I/O occurs. This is the tier used by
//     the majority of existing tests.
//
//   Tier 2 — Selective Real (computation):
//     Pure nodes and safe environment interactions (env var reads, timestamp,
//     filesystem reads in temp dirs, conditionals) execute for real. Only
//     external-facing transport nodes remain mocked. Proves that computation
//     logic within the DAG produces correct values, not just correct shapes.
//
//   Tier 3 — Full Real (integration):
//     All nodes execute for real. Only used in controlled environments with
//     sandboxed credentials (e.g., CI with scoped tokens). Proves end-to-end
//     behavior including HTTP calls and cloud API interactions.
//
// The test below demonstrates Tier 2: a DAG node that reads a real
// environment variable, executed in Real mode with no mocking.
// =========================================================================

/// Tier 2 test: a single-node DAG reads a real environment variable in
/// Real mode and produces a `Value::Secret`. This proves that Real-mode
/// execution works for safe, pure-environment operations — the node is
/// never intercepted.
#[test]
fn env_var_read_real_mode() {
    use gunbc_ir::value::SecretString;

    #[derive(Debug, Clone)]
    struct EnvReadOp {
        var_name: String,
    }

    impl Executable for EnvReadOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let value = std::env::var(&self.var_name)
                .map_err(|_| ExecError::new(format!("env var '{}' not set", self.var_name)))?;
            let mut out = HashMap::new();
            out.insert(
                "credential".to_string(),
                Value::Secret(SecretString::new(value)),
            );
            Ok(out)
        }
    }

    // Serialize env-mutating tests to prevent process-global races.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ENV_LOCK.lock().unwrap();

    let var_name = "GUNBC_TEST_CREDENTIAL_T11";
    let secret_value = "test-secret-value-t11";
    // SAFETY: Serialized by ENV_LOCK above. No other test uses this variable.
    unsafe {
        std::env::set_var(var_name, secret_value);
    }

    let mut dag: Dag<EnvReadOp> = Dag::new();
    dag.add_node(Node::opaque(
        "env_read",
        vec![],
        vec![port("credential", "Secret")],
        EnvReadOp {
            var_name: var_name.to_string(),
        },
    ));

    // Act: execute in Real mode — no DryRun, no mocks
    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    // Assert
    assert_eq!(log.entries.len(), 1);
    let entry = log.get("env_read").expect("env_read node should exist");
    assert!(
        !entry.was_intercepted,
        "Real-mode execution must not intercept pure nodes"
    );

    #[allow(clippy::disallowed_methods)]
    match entry.outputs.get("credential") {
        Some(Value::Secret(s)) => {
            assert_eq!(
                s.expose_plaintext_for_transport(),
                secret_value,
                "Secret value should match the env var we set"
            );
        }
        other => panic!(
            "expected Value::Secret with the test credential, got {:?}",
            other
        ),
    }

    // Verify no interception occurred anywhere
    assert!(
        !log.has_intercepted(),
        "Real-mode execution log should have zero intercepted nodes"
    );

    // Cleanup
    unsafe {
        std::env::remove_var(var_name);
    }
}

/// Tier 2 test: verifies that Real mode actually *executes* a node that
/// DryRun would intercept. A node with `NodeKind::ResourceEnvironment`
/// is intercepted in DryRun but should run its real body in Real mode.
#[test]
fn real_mode_executes_resource_environment_node() {
    use gunbc_ir::value::SecretString;

    #[derive(Debug, Clone)]
    struct CredentialProviderOp {
        var_name: String,
    }

    impl Executable for CredentialProviderOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            let value = std::env::var(&self.var_name)
                .map_err(|_| ExecError::new(format!("env var '{}' not set", self.var_name)))?;
            let mut out = HashMap::new();
            out.insert(
                "credential".to_string(),
                Value::Secret(SecretString::new(value)),
            );
            Ok(out)
        }
    }

    static ENV_LOCK2: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ENV_LOCK2.lock().unwrap();

    let var_name = "GUNBC_TEST_CRED_RESOURCE_T11";
    let secret_value = "real-credential-from-env";
    // SAFETY: Serialized by ENV_LOCK2 above. No other test uses this variable.
    unsafe {
        std::env::set_var(var_name, secret_value);
    }

    let mut dag: Dag<CredentialProviderOp> = Dag::new();
    dag.add_node(
        Node::opaque(
            "cred_env",
            vec![],
            vec![port("credential", "Credential")],
            CredentialProviderOp {
                var_name: var_name.to_string(),
            },
        )
        .with_kind(NodeKind::ResourceEnvironment),
    );

    // In DryRun this node would be intercepted; in Real mode it runs.
    let log = execute_dag(&dag, ExecuteConfig::default()).unwrap();

    let entry = log.get("cred_env").expect("cred_env node should exist");
    assert!(
        !entry.was_intercepted,
        "Real mode must execute ResourceEnvironment nodes, not intercept them"
    );

    #[allow(clippy::disallowed_methods)]
    match entry.outputs.get("credential") {
        Some(Value::Secret(s)) => {
            assert_eq!(s.expose_plaintext_for_transport(), secret_value);
        }
        other => panic!("expected Value::Secret, got {:?}", other),
    }

    // Contrast: the same DAG in DryRun *would* intercept
    let mut mocks = BoundaryMocks::new();
    mocks.set_value(
        "cred_env",
        "credential",
        Value::Secret(SecretString::new("mocked-cred".to_string())),
    );
    let dry_log = execute_dag(
        &dag,
        ExecuteConfig {
            mode: ExecutionMode::DryRun(mocks),
            ..Default::default()
        },
    )
    .unwrap();
    let dry_entry = dry_log.get("cred_env").unwrap();
    assert!(
        dry_entry.was_intercepted,
        "DryRun should intercept ResourceEnvironment nodes"
    );

    unsafe {
        std::env::remove_var(var_name);
    }
}
