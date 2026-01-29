# Executor Model

**Status**: Design  
**Last updated**: January 2026

---

## Overview

The gunbc executor runs lowered (flat) DAGs. This document describes the
work-queue execution model that replaces wave-based batching.

---

## Design Principles

### 1. No Artificial Barriers

Wave-based execution groups nodes into batches:
```
Wave 1: [A, B, C]  →  wait all  →  Wave 2: [D, E]
```

This creates artificial barriers — A may finish quickly but waits for B and C.

Work-queue execution has no barriers:
```
A done → check what's unblocked → run it immediately
```

### 2. Parallelism Falls Out of Structure

If the DAG correctly models dependencies, parallelization is automatic.
No explicit parallel annotations needed. The executor runs as many nodes
as are ready, limited only by resource constraints.

### 3. The DAG Is the Contract

Edges encode dependencies. No implicit ordering. If it's not an edge,
there's no dependency. The executor trusts the DAG structure.

---

## Executor State

```rust
struct Executor {
    /// Nodes not yet runnable (waiting on dependencies)
    pending: HashSet<NodeId>,
    
    /// Nodes with all dependencies satisfied, ready to run
    ready: VecDeque<NodeId>,
    
    /// Currently executing
    running: HashSet<NodeId>,
    
    /// Completed with their results
    completed: HashMap<NodeId, NodeResult>,
    
    /// Dependency graph (node → nodes that depend on it)
    dependents: HashMap<NodeId, Vec<NodeId>>,
    
    /// Reverse dependency graph (node → nodes it depends on)
    dependencies: HashMap<NodeId, Vec<NodeId>>,
}
```

---

## Execution Algorithm

```rust
impl Executor {
    async fn execute(&mut self, dag: &FlatDag) -> Result<ExecutionLog> {
        // Initialize: nodes with no dependencies start ready
        for node in &dag.nodes {
            let deps = self.dependencies.get(&node.id);
            if deps.is_none() || deps.unwrap().is_empty() {
                self.ready.push_back(node.id.clone());
            } else {
                self.pending.insert(node.id.clone());
            }
        }
        
        // Main loop: run until nothing left
        while !self.ready.is_empty() || !self.running.is_empty() {
            // Spawn all ready nodes (up to resource limit)
            while let Some(node_id) = self.ready.pop_front() {
                if self.running.len() >= self.max_concurrent {
                    self.ready.push_front(node_id);
                    break;
                }
                self.spawn_node(node_id).await;
            }
            
            // Wait for at least one completion
            let completed = self.wait_any_completion().await;
            
            // Process completion: update ready set
            for (node_id, result) in completed {
                self.on_node_completed(node_id, result);
            }
        }
        
        self.build_log()
    }
    
    fn on_node_completed(&mut self, node_id: NodeId, result: NodeResult) {
        self.running.remove(&node_id);
        self.completed.insert(node_id.clone(), result);
        
        // Check what's now unblocked
        if let Some(dependents) = self.dependents.get(&node_id) {
            for dependent in dependents {
                if self.all_deps_satisfied(dependent) {
                    self.pending.remove(dependent);
                    self.ready.push_back(dependent.clone());
                }
            }
        }
    }
    
    fn all_deps_satisfied(&self, node_id: &NodeId) -> bool {
        self.dependencies
            .get(node_id)
            .map(|deps| deps.iter().all(|d| self.completed.contains_key(d)))
            .unwrap_or(true)
    }
}
```

---

## Node Execution

Each node execution:

1. **Collect inputs**: Gather values from completed dependency outputs
2. **Evaluate guard**: If guard fails, produce `Skipped` and complete
3. **Execute body**: Run the opaque operation
4. **Store outputs**: Make outputs available for dependents

```rust
async fn execute_node(&self, node: &Node, inputs: NodeInputs) -> NodeResult {
    // Check guard
    if let Some(guard) = &node.guard {
        if !guard.evaluate(&inputs) {
            return NodeResult::Skipped;
        }
    }
    
    // Execute
    match &node.body {
        NodeBody::Opaque(exec) => {
            exec.execute(inputs).await
        }
        NodeBody::SubDag(_) => {
            panic!("SubDag should be lowered before execution")
        }
    }
}
```

---

## Resource Management

### Concurrency Limits

```rust
struct ExecutorConfig {
    /// Maximum concurrent node executions
    max_concurrent: usize,
    
    /// Resource pools for specific resource types
    resource_pools: HashMap<ResourceId, ResourcePool>,
}

struct ResourcePool {
    /// Available capacity (0 = unlimited)
    capacity: usize,
    /// Currently held
    held: usize,
}
```

### Resource Claims

Nodes may declare resource claims:
```rust
struct ResourceClaim {
    resource: ResourceId,
    amount: usize,
}
```

A node only becomes ready when:
1. All dependencies are satisfied
2. All resource claims can be fulfilled

---

## Handling Skipped Nodes

When a node is skipped (guard evaluated false):

1. Output ports produce `Value::Skipped`
2. Dependent nodes receive `Skipped` on their inputs
3. Dependents with guards on that input may also skip
4. Chain continues until a node handles `Skipped`

This implements conditional execution without explicit branching constructs.

---

## Error Handling

### Node Failure

When a node fails:

```rust
enum FailurePolicy {
    /// Fail the entire DAG immediately
    FailFast,
    
    /// Skip dependent nodes, continue independent ones
    SkipDependents,
    
    /// Retry with backoff (requires idempotency)
    Retry { max_attempts: usize, backoff: Duration },
}
```

### Failure Propagation

With `SkipDependents`:
1. Mark failed node as `Failed`
2. All transitive dependents become `Cancelled`
3. Independent branches continue execution

---

## Progress Events

The executor emits events for observability:

```rust
enum ProgressEvent {
    RunStarted { dag_id: DagId, total_nodes: usize },
    NodeQueued { node: NodeId },
    NodeRunning { node: NodeId },
    NodeCompleted { node: NodeId, duration_ms: u64 },
    NodeSkipped { node: NodeId, reason: SkipReason },
    NodeFailed { node: NodeId, error: String },
    NodeCancelled { node: NodeId },
    RunCompleted { duration_ms: u64, succeeded: usize, failed: usize },
}
```

Events are emitted to a sink (channel, file, callback).

---

## Comparison: Waves vs Work-Queue

| Aspect | Wave Model | Work-Queue Model |
|--------|------------|------------------|
| Barrier | Wait for entire wave | No barriers |
| Latency | Bounded by slowest in wave | Continuous progress |
| Complexity | Precompute waves | Dynamic ready set |
| Memory | Store wave groupings | Store dependency graph |
| Parallelism | Wave size | Ready set size |

Work-queue is strictly better for throughput. Wave model is simpler to
reason about but wastes time at barriers.

---

## Integration with Dry-Run

Dry-run execution uses the same model but intercepts boundary nodes:

```rust
enum ExecutionMode {
    Real,
    DryRun(BoundaryMocks),
}

impl Executor {
    async fn execute_with_mode(
        &mut self,
        dag: &FlatDag,
        mode: ExecutionMode,
    ) -> Result<ExecutionLog> {
        // Same algorithm, but execute_node checks mode
        // and uses mocks for boundary nodes in DryRun
    }
}
```

---

## Future: JIT Considerations

When JIT is added, the executor will need:

1. **Dynamic node insertion**: Add nodes to running DAG
2. **Dependency resolution**: New nodes may depend on completed nodes
3. **Partial re-execution**: Run only new/changed subgraph

The work-queue model supports this naturally — new nodes just go into
`pending` or `ready` based on their dependencies.
