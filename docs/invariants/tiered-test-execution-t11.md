## Tiered Test Execution (T11)

DAG execution tests use three tiers, each proving a different layer of
correctness. Every test explicitly chooses its tier via `ExecutionMode`.

### Tier 1 — DryRun (structure)

All transport, resource-environment, and tool nodes are intercepted with
explicit mocks (`ExecutionMode::DryRun(mocks)`). Pure nodes execute
normally. This tier proves DAG wiring, port cardinality, coercion, guard
evaluation, conditional branching, and topological ordering — without
performing any real I/O. The majority of existing tests operate at this
tier.

### Tier 2 — Selective Real (computation)

The DAG executes in `ExecutionMode::Real`, but the operations themselves
are limited to safe, hermetic effects: reading environment variables,
filesystem operations in temporary directories, timestamps, and
conditional logic. No external HTTP calls or cloud API interactions.
This tier proves that computation within the DAG produces correct
*values*, not just correct *shapes*.

Reference tests: `env_var_read_real_mode`,
`real_mode_executes_resource_environment_node` in
`src/v1/09_execute/exec/src/execute/tests.rs`.

### Tier 3 — Full Real (integration)

All nodes execute for real against live services. Only viable in
controlled environments with sandboxed credentials (CI runners with
scoped tokens, disposable cloud resources). Proves end-to-end behavior
including HTTP transport and cloud API interactions. Not yet implemented;
requires credential injection infrastructure.

