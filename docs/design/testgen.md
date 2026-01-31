# Testgen: Design & TODO

> **Goal**: Given that nodes are pure and we control I/O at transport boundaries,
> testgen should verify flows and business logic for mocked scenarios —
> mock one side of a pipeline and check that pure nodes produce expected outputs.

---

## Current State

### What exists

The testgen system (`core/testgen`) generates test files from DAG definitions
and `MockSpec` values. There are 7 generated test files across `gunbc-dag` and
`lib/llm-ops`.

### What it generates today

| Test type | What it does | Useful? |
|-----------|-------------|---------|
| Mock spec self-consistency | Asserts `mock_spec().get_boundary_mock(node, port).is_some()` | **No** — circular; tests that the spec defines what it defines |
| Input expectations count | Asserts `spec.input_expectations.len() == N` | **No** — fragile snapshot of a number |
| Resource acquire | Calls `resource.acquire()`, asserts `Acquired` | **Marginal** — trivial happy path on mock framework |
| Lease timeout | Checks `should_timeout()` before/after duration | **Yes** — tests real semantic behavior |

### What it doesn't generate (but could)

**Boundary tests** are disabled everywhere (`boundary_tests: false`) with this
note in `testgen.rs`:

```
// NOTE: boundary_tests disabled until testgen supports entrypoint input mocking
// (dry-run only intercepts transport executors; pure nodes still need inputs)
```

**Flow verification tests** — the big win — don't exist yet.

### Hand-written tests are better

The `graph_mock.rs` files have hand-written tests (e.g., `ci/graph_mock.rs`)
that verify meaningful properties: success vs failure mock variants, contention
handling, report content. These provide more value than all 7 generated files
combined.

---

## Key Insight: DryRun Executes Pure Nodes

The execution model (`core/exec`) has a critical property that testgen
should exploit:

```
DryRun mode:
  - Transport executor nodes (inputs include TransportRequest) → INTERCEPTED, return mock values
  - Pure nodes (no TransportRequest inputs)                    → EXECUTE NORMALLY
```

This means a DryRun with mocked transport responses runs the **entire pure
logic chain**. Mocked `TransportResponse` values flow into parse nodes, which
produce real outputs, which flow into prepare nodes, which produce real outputs,
all the way to the final report/output node.

### Example: CI Graph DryRun

```
prepare_scan ─req─▶ [execute_scan] ─resp─▶ parse_scan ─▶ prepare_build ─req─▶ [execute_build] ─resp─▶ parse_build ─▶ report
     pure              MOCKED              pure            pure                  MOCKED              pure            pure
     runs              returns mock        runs            runs                  returns mock        runs            runs
```

Every `[bracketed]` node is mocked. Every other node executes its real logic.
The `report` node at the end produces a real result derived from mocked I/O.
**This result is verifiable.**

### The Gap: Entrypoint Inputs

Some DAGs have external inputs — ports that receive values from outside the
DAG (not from upstream nodes). The LLM graphs need `provider`, `model`,
`prompt`, `api_key` injected at the entrypoint. The current `BoundaryMocks`
only provides values for intercepted transport executor outputs; there's no
mechanism for injecting external DAG inputs.

DAGs with no external inputs (bootstrap, CI, makegen — their root nodes are
`prepare_*` nodes with no inputs) could run DryRun today.

---

## Design: What Testgen Should Do

### Level 1: Flow Verification (high value)

Mock transport boundaries, execute the full DAG in DryRun, verify that the
pure node chain produces expected outputs at the terminal nodes.

```rust
// Generated test — CI success flow
#[test]
fn test_ci_flow_all_pass() {
    let dag = build_ci_graph().unwrap();
    let mocks = ci_success_mocks();  // mock all transport responses as "pass"
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify the report node (pure) computed the right result
    let report = log.get("report").unwrap();
    assert_eq!(report.outputs["overall_success"], Value::Bool(true));
    assert!(report.outputs["report"].as_str().contains("SUCCESS"));
}

#[test]
fn test_ci_flow_build_fails() {
    let dag = build_ci_graph().unwrap();
    let mocks = ci_build_fail_mocks();  // mock build transport as exit_code: 1
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    let report = log.get("report").unwrap();
    assert_eq!(report.outputs["overall_success"], Value::Bool(false));
    // Test and lint should be skipped when build fails
    assert!(log.get("parse_test").unwrap().outputs.get("skip").is_some()
         || report.outputs["report"].as_str().contains("SKIPPED"));
}
```

This tests the **real business logic**: does the CI pipeline correctly skip
downstream stages when build fails? Does the report correctly aggregate
results? These are the kinds of bugs that actually happen.

**Requirements:**
- `BoundaryMocks` maps (node_id, port) → mock Value for transport executor outputs
- MockSpec variants for each scenario (success, build-fail, test-fail, lint-fail)
- The graph builder functions already exist; the DAGs are available at test time

**Which DAGs work today (no external inputs):**
- `bootstrap` — root is `prepare_scan_workspace` (no inputs)
- `ci` — root is `prepare_deps_exists` (no inputs)
- `makegen` — root is `prepare_scan` (no inputs)

### Level 2: Entrypoint Input Injection (unblocks LLM DAGs)

Add `EntrypointInputs` to `ExecutionMode::DryRun` so that DAGs with external
inputs can also run flow verification.

```rust
pub enum ExecutionMode {
    Real,
    DryRun(BoundaryMocks),
    DryRunWithInputs {          // new variant
        mocks: BoundaryMocks,
        inputs: HashMap<String, Value>,  // external inputs for entrypoint nodes
    },
    Simulate(SimConfig),
}
```

Alternatively, inject entrypoint values as edges from a synthetic "env" node,
keeping the execution model unchanged. This is a design choice — the synthetic
node approach is cleaner because it doesn't add a new ExecutionMode variant.

**Which DAGs need this:**
- All LLM graphs (need `provider`, `model`, `prompt`, `api_key`)

### Level 3: Cross-DAG Chain Validation

`validate_chain` is already implemented — it checks that upstream MockSpec
outputs satisfy downstream InputConstraints. But testgen never calls it.

```rust
// Generated test — verify LLM output feeds into code-review input
#[test]
fn test_chain_llm_to_code_review() {
    let upstream = openai_mock_spec();
    let downstream = code_review_mock_spec();
    let mapping = HashMap::from([
        ("content".into(), "review_input".into()),
    ]);
    let result = validate_chain(&upstream, &downstream, &mapping);
    assert!(result.errors.is_empty(), "Chain errors: {:?}", result.errors);
}
```

This verifies that when two DAGs compose, the upstream's mocked outputs satisfy
the downstream's input constraints. Catches interface mismatches between
independently developed DAGs.

### Level 4: Scenario Matrix

For each DAG, generate tests for every MockSpec variant (success, each failure
mode, contention, timeout). The CI graph already has 6 mock spec variants in
`graph_mock.rs` — testgen should generate a flow test for each.

```rust
// testgen generates one flow test per MockSpec variant
fn test_ci_flow_success()        { run_flow(ci_mock_spec(), expect_success()); }
fn test_ci_flow_test_fails()     { run_flow(ci_mock_spec_test_fails(), expect_failure("Test")); }
fn test_ci_flow_build_fails()    { run_flow(ci_mock_spec_build_fails(), expect_failure("Build")); }
fn test_ci_flow_prep_fails()     { run_flow(ci_mock_spec_prep_fails(), expect_failure("Prep")); }
fn test_ci_flow_lint_fails()     { run_flow(ci_mock_spec_lint_fails(), expect_failure("Lint")); }
fn test_ci_flow_contended()      { run_flow(ci_mock_spec_build_contended(), expect_failure("blocked")); }
```

---

## What To Remove

The current generated tests (mock spec self-consistency, input expectation
counts, trivial resource acquire) should be replaced once flow verification
is working. They test the mock infrastructure, not the DAGs.

The hand-written `graph_mock.rs` tests should stay — they test the mock specs
themselves and are intentionally authored.

---

## TODO

### Phase 1: Flow Verification for Cargo DAGs

These DAGs have no external inputs and can run DryRun today.

- [ ] **Convert MockSpec boundary mocks to BoundaryMocks**
  Add `MockSpec::to_boundary_mocks() -> BoundaryMocks` that translates
  mock spec boundary values into the format `execute_with_mode` expects.
  The key mapping: MockSpec boundaries are keyed by the *boundary node*
  (unconnected outputs), but DryRun mocks are keyed by *transport executor
  nodes*. Need to bridge this gap — either change MockSpec to use transport
  node IDs, or add a mapping step that uses the DAG edges.

- [ ] **Add flow test generation to TestGenerator**
  New method: `generate_flow_tests()`. For each MockSpec variant, generate
  a test that builds the DAG, runs DryRun, and asserts on terminal node
  outputs.

- [ ] **Define expected outputs per scenario in MockSpec**
  Add `expected_outputs: Vec<ExpectedOutput>` to MockSpec:
  ```rust
  pub struct ExpectedOutput {
      pub node: String,
      pub port: String,
      pub expected: Value,
  }
  ```
  This lets mock specs declare: "given these mocked transport responses,
  the terminal node should produce these values."

- [ ] **Enable flow tests for bootstrap, ci, makegen**
  Update `testgen.rs` targets to generate flow tests. Remove the
  `boundary_tests: false` workaround.

- [ ] **Remove circular tests**
  Drop `test_mock_spec_self_consistent` and `test_input_expectations_documented`
  from generated output once flow tests are in place.

### Phase 2: Entrypoint Input Injection

- [ ] **Design input injection mechanism**
  Choose between: (a) new `ExecutionMode` variant with explicit inputs map,
  or (b) synthetic "env" source node prepended to the DAG. Option (b) is
  cleaner for the execution model.

- [ ] **Implement input injection in executor**
  Modify `execute_with_mode` (or add a DAG transform) to support
  externally-provided inputs for entrypoint nodes.

- [ ] **Add MockSpec entrypoint values**
  ```rust
  MockSpec::new("llm")
      .entrypoint("provider", Value::Str("openai".into()))
      .entrypoint("model", Value::Str("gpt-4".into()))
      .entrypoint("prompt", Value::Str("review this code".into()))
      .entrypoint("api_key", Value::Secret(...))
  ```

- [ ] **Enable flow tests for LLM DAGs**
  Update LLM testgen targets to use entrypoint injection + flow tests.

### Phase 3: Cross-DAG Chain Validation

- [ ] **Generate chain validation tests**
  For known DAG composition points (LLM → code-review, etc.), generate
  `validate_chain` tests that verify interface compatibility.

- [ ] **Define composition registry**
  A simple list of (upstream_spec, downstream_spec, port_mapping) tuples
  that testgen iterates to produce chain tests.

### Phase 4: Scenario Matrix

- [ ] **Auto-discover MockSpec variants**
  Convention: `graph_mock.rs` exports functions matching `*_mock_spec*()`.
  Testgen discovers them and generates one flow test per variant.

- [ ] **Add failure-mode assertions**
  MockSpec variants annotated with expected terminal outputs enable
  automatic assertion generation for each scenario.
