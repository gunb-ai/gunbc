# Transport-as-DAG Migration

Feasibility analysis and migration plan for modeling transport executor
behavior as DAG nodes, bringing the executor under the testgen umbrella.

---

## 1. The Problem

The transport executor (`lib/transport/src/executor.rs`) is a **testing
blind spot**. It contains 5 monolithic dispatch functions:

| Function         | Lines | Tests | Coverage |
|------------------|-------|-------|----------|
| `execute_rest`   | ~45   | 0     | Via tool integration tests only |
| `execute_http`   | ~55   | 0     | Via REST (rest wraps http) |
| `execute_file`   | ~180  | 30+   | Good: unit tests for each FileOp |
| `execute_tcp`    | ~35   | 0     | **Zero coverage** |
| `execute_shell`  | ~40   | 2     | Basic happy-path only |

The swapped-timeout bug in `execute_tcp` (consolidation.md 17.1) survived
because TCP is the **only transport type with zero test coverage**, and
the executor sits outside the DAG system where testgen operates.

### Why the current architecture misses this

Every tool (deps, gist, clippy, etc.) follows the Prepare -> Execute -> Parse
triplet. Testgen generates tests for the **Prepare** and **Parse** nodes
(pure functions) and DryRun-intercepts the **Execute** node with mocks.
This verifies that:

- Prepare builds the correct `TransportRequest` shape
- Parse handles the expected `TransportResponse` shape
- Skip/failure propagation works

But it **never tests what happens inside `execute_transport()`** -- the
actual I/O boundary. The `TransportBackend` trait allows mock backends in
tests, but nobody writes those tests because the executor is a leaf
function, not a DAG.

### The gap in one sentence

**The DAG system tests everything above the transport boundary; nothing
tests the boundary itself.**

---

## 2. Current Architecture (How Transport Works Today)

```
Tool DAG:
  PrepareXxx (pure)       -- builds TransportRequest from domain inputs
       |
  TransportOps::Execute   -- THE boundary; calls execute_transport()
       |
  ParseXxx (pure)         -- extracts domain outputs from TransportResponse

Executor (monolithic):
  execute_transport(request) -> match request {
      Rest(r)  => execute_rest(r),   // ureq HTTP + JSON parse
      Http(r)  => execute_http(r),   // ureq HTTP raw
      File(r)  => execute_file(r),   // std::fs dispatch on FileOp
      Tcp(r)   => execute_tcp(r),    // TcpStream::connect + read/write
      Shell(r) => execute_shell(r),  // Command::new + piped I/O
  }
```

Key properties:
- 5 transport types, dispatched by enum variant
- Each executor function is monolithic (connect, configure, execute, parse)
- `TransportRequest`/`TransportResponse` carry **all** config as flat fields
- The `TransportBackend` trait exists for test mocking but is under-used
- `execute_request()` is `pub(crate)` -- external code MUST use DAG nodes

---

## 3. Three Approaches

### Approach A: Behavioral Sub-DAGs (Full DAG Modeling)

Model each transport type's **internal execution steps** as a sub-DAG
that replaces the monolithic executor function.

**TCP example:**

```
Current (monolithic):
  execute_tcp(TcpRequest) -> TcpResponse

Proposed (sub-DAG):
  DecomposeTcpRequest       -- extract host, port, timeouts, data
       |
  TcpConnect                -- TcpStream::connect(addr) [I/O]
       |
  ConfigureReadTimeout      -- stream.set_read_timeout(read_timeout_ms) [I/O]
       |
  ConfigureWriteTimeout     -- stream.set_write_timeout(connect_timeout_ms) [I/O]
       |
  TcpWrite                  -- stream.write_all(data) [I/O]
       |
  TcpRead                   -- stream.read_to_string() [I/O]
       |
  AssembleTcpResponse       -- build TcpResponse from results
```

**What this gives you:**
- `read_timeout_ms` connects via a named port to `ConfigureReadTimeout`,
  making the swap bug structurally impossible
- Each step is independently mockable/testable
- Testgen generates windowed tests for every node pair
- The `DecomposeTcpRequest` node is pure and gets full example-based testing

**Problems:**
- OS handles (`TcpStream`) are not `Value` types. The DAG execution model
  passes `HashMap<String, Value>` between nodes. A live `TcpStream` cannot
  be serialized, cloned, or stored in `Value`.
- Sequential I/O on a single connection is fundamentally different from the
  current DAG model (which coordinates independent I/O calls). The stream
  must stay open across multiple "nodes."
- Massive overhead: DAG machinery (topo sort, value routing, generation
  tracking) for what is currently a 35-line function.
- The `Value` enum would need a new variant (`Value::Handle(Box<dyn Any>)`)
  or the stream would need to be threaded through closures.

**Verdict: Not feasible without significant `Value` model changes.**
The handle problem is fundamental. DAG nodes communicate via serializable
values; OS resources (streams, file handles, process pipes) don't fit.

---

### Approach B: Specification DAGs + Code Generation

Model each transport type as a **specification DAG** that describes the
behavioral contract. Use this to **generate** both the executor
implementation and the test suite.

**How it works:**

1. Define a `TransportSpec` for each transport type as a DAG:

```rust
fn tcp_spec() -> TransportSpec {
    TransportSpec::new("tcp")
        .step("connect")
            .input("host", "String")
            .input("port", "u16")
            .output("stream", "TcpStream")    // abstract, not Value
            .error("connection failed: {}")
        .step("configure_read_timeout")
            .input("stream", "TcpStream")
            .input("read_timeout_ms", "Option<u64>")  // <-- named binding
            .output("stream", "TcpStream")
        .step("configure_write_timeout")
            .input("stream", "TcpStream")
            .input("write_timeout_ms", "Option<u64>")  // <-- named binding
            .output("stream", "TcpStream")
        .step("write")
            .input("stream", "TcpStream")
            .input("data", "Option<String>")
            .output("bytes_sent", "usize")
            .output("stream", "TcpStream")
        .step("read")
            .input("stream", "TcpStream")
            .output("data", "Option<String>")
            .output("bytes_received", "usize")
        .assemble("TcpResponse")
            .field("connected", "true")
            .field("data", "read.data")
            .field("bytes_sent", "write.bytes_sent")
            .field("bytes_received", "read.bytes_received")
}
```

2. The spec generates:

   a. **The executor function** (`execute_tcp`) with correct field routing
      (the spec binds `read_timeout_ms` to `configure_read_timeout`, making
      the swap structurally impossible in the generated code)

   b. **Contract tests** that verify:
      - Each step's error handling
      - Field routing correctness (read_timeout -> read_timeout, not write)
      - Timeout behavior with mock servers
      - Edge cases (None timeouts, empty data, connection refused)

   c. **A mock server harness** for integration tests (e.g., a TCP listener
      that accepts but never responds, for timeout testing)

**What this gives you:**
- The spec is the source of truth; the executor is derived
- Field routing is explicit and verified
- Test generation covers each behavioral step
- No handle problem (the spec describes structure; generated code uses real handles)
- The executor remains a normal function (no DAG overhead at runtime)

**Problems:**
- New abstraction layer (`TransportSpec`) to design and maintain
- Code generation for imperative I/O is harder than for DAG structures
- Generated code may be harder to debug than hand-written code
- Significant upfront investment before any payoff

**Verdict: Feasible but heavy. Best for when transport types proliferate.**

---

### Approach C: Decomposed Prepare/Parse + Behavioral Tests (Recommended)

Keep the executor as-is but **decompose the request/response data flow**
into testable pure functions, and add a behavioral test layer using the
existing `TransportBackend` mock system.

**The insight:** The swapped-timeout bug isn't a handle problem -- it's a
**field routing problem** in the prepare-to-execute handoff. The request
struct carries both `read_timeout_ms` and `connect_timeout_ms` as flat
fields, and the executor manually destructures them. If the request struct
were decomposed into typed intermediate representations, the compiler
would catch the swap.

**Three concrete changes:**

#### C.1: Transport-specific Prepare/Parse ops (bring executor internals into the DAG)

Instead of one opaque `TransportOps::Execute` that handles all 5 types,
introduce transport-specific prepare/parse ops that decompose the request
and assemble the response:

```rust
pub enum TransportOps {
    Execute,                  // existing: polymorphic execute

    // NEW: transport-specific decomposition
    PrepareTcp,              // TcpRequest -> (host, port, read_timeout_ms, ...)
    ParseTcpResponse,       // (connected, data, bytes_sent, ...) -> TcpResponse
    PrepareHttp,             // HttpRequest -> (url, method, headers, ...)
    ParseHttpResponse,       // (status, headers, body) -> HttpResponse
    PrepareShell,            // ShellRequest -> (command, args, env, ...)
    ParseShellResponse,      // (exit_code, stdout, stderr) -> ShellResponse
    // ... etc
}
```

Each `PrepareXxx` decomposes the request into individual named ports.
Each `ParseXxxResponse` assembles individual ports into the response.
The actual I/O node receives **individual fields**, not an opaque struct.

**For TCP, the triplet becomes:**

```
PrepareOp (pure, tool-specific)
     |  request: TransportRequest
     v
PrepareTcp (pure)               -- decomposes TcpRequest
     |  host: String
     |  port: u16
     |  read_timeout_ms: Option<u64>     <-- explicit port
     |  write_timeout_ms: Option<u64>    <-- explicit port (renamed!)
     |  data: Option<String>
     v
TcpExecute (boundary)           -- actual I/O, receives individual fields
     |  connected: Bool
     |  data: Option<String>
     |  bytes_sent: usize
     |  bytes_received: usize
     |  error: Option<String>
     v
ParseTcpResponse (pure)         -- assembles TcpResponse
     |  response: TransportResponse
     v
ParseOp (pure, tool-specific)
```

**Why this prevents the swap bug:** The port names are `read_timeout_ms`
and `write_timeout_ms`. The DagBuilder validates edge types. The generated
tests verify that each port is wired correctly. A swap would require
miswiring two explicitly-named ports, which is visible in code review
and caught by testgen's windowed tests.

**Bonus:** The `connect_timeout_ms` field name on `TcpRequest` is
misleading (it's used as write timeout). Introducing the decomposition
layer lets us rename it to `write_timeout_ms` at the port level without
changing the struct.

#### C.2: Behavioral test harness for the executor

Add a test module in `lib/transport/src/executor.rs` that tests each
executor function against controlled scenarios:

```rust
#[cfg(test)]
mod tcp_tests {
    // Slow-reader server: accepts, never sends (tests read timeout)
    // Echo server: sends back what it receives (tests write + read)
    // Refused server: nothing listening (tests connect error)
    // Partial-write server: accepts, reads partial, closes (tests write error)
}
```

These don't require DAG modeling -- they're plain integration tests
with `TcpListener`-based mock servers. But they fill the **zero coverage**
gap immediately.

#### C.3: Transport MockSpec pattern

For each transport type, define a `MockSpec`-like specification that
describes the behavioral contract:

```rust
pub struct TransportBehavior {
    pub name: &'static str,
    pub request_type: &'static str,
    pub response_type: &'static str,
    pub steps: Vec<BehaviorStep>,
    pub error_modes: Vec<ErrorMode>,
}

pub struct BehaviorStep {
    pub name: &'static str,
    pub consumes: Vec<(&'static str, &'static str)>,  // (field, type)
    pub produces: Vec<(&'static str, &'static str)>,
}
```

This feeds into testgen to generate:
- Per-step behavioral tests
- Error-mode coverage (each ErrorMode gets a test scenario)
- Field-routing verification

**Verdict: Recommended. Incremental, fits existing patterns, immediate payoff.**

---

## 4. Recommended Migration Plan

### Phase 1: Fill the Testing Gap (immediate)

**Goal:** Zero-to-reasonable test coverage for all 5 executor functions.

1. Add `tcp_tests` module to `executor.rs`:
   - `test_tcp_connect_success` (echo server)
   - `test_tcp_connect_refused` (no listener)
   - `test_tcp_read_timeout_applied` (slow server, verify timeout < 1s)
   - `test_tcp_write_sends_data` (echo server, verify roundtrip)
   - `test_tcp_no_data_read_only` (server sends, client reads)

2. Add `shell_tests` coverage:
   - `test_shell_nonexistent_command`
   - `test_shell_exit_code_propagation`
   - `test_shell_env_vars_applied`
   - `test_shell_cwd_applied`
   - `test_shell_stdin_piped`

3. Add `http_tests` / `rest_tests` (if feasible without external servers):
   - Use `TransportBackendGuard` or a local HTTP server fixture

**Estimated scope:** ~200 lines of test code. Catches the bug class that
17.1 represents.

### Phase 2: Typed Port Decomposition (Approach C.1)

**Goal:** Make field routing explicit at the DAG level.

1. Define `PrepareTcp`, `ParseTcpResponse`, etc. in `lib/transport/src/ops.rs`
2. Each decomposes the opaque `TransportRequest` into named scalar ports
3. The I/O node receives individual fields (host, port, read_timeout_ms, ...)
4. Add `TransportTriplet` variants that use the decomposed shape
5. Update `add_*_transport_triplet` helpers to optionally use decomposition

**Key design decision:** Whether decomposition is opt-in (new helper
variant) or default (replace existing triplet shape). Recommend opt-in
initially, default after validation.

**Estimated scope:** ~500 lines. Requires updating DAG builders that use
transport triplets, but the triplet helpers absorb most of the change.

### Phase 3: Transport Behavioral Specs

**Goal:** Transport types are specified declaratively, tests generated.

1. Define `TransportBehavior` spec type
2. Write specs for each transport type (TCP, HTTP, REST, File, Shell)
3. Integrate with testgen to generate behavioral tests
4. Optionally generate the executor dispatch code from specs

**Estimated scope:** ~1000 lines. Payoff increases as transport types
grow (WebSocket, gRPC, etc.).

### Phase 4: Full Sub-DAG Modeling (if needed)

Only pursue if Phase 3 proves insufficient or if new transport types
have genuinely complex multi-step behavior (e.g., WebSocket handshake,
OAuth token exchange, streaming responses).

At that point, evaluate adding a `Value::Handle` variant or a separate
`StreamDag` execution model that handles OS resources.

---

## 5. Field Naming Cleanup

The `TcpRequest` struct has a naming problem:

| Current field          | Actual usage                | Better name             |
|------------------------|-----------------------------|-------------------------|
| `connect_timeout_ms`   | `set_write_timeout`         | `write_timeout_ms`      |
| `read_timeout_ms`      | `set_read_timeout`          | `read_timeout_ms` (ok)  |

TCP doesn't have a separate "connect timeout" -- `TcpStream::connect` is
synchronous and blocks until success or OS-level timeout. The
`connect_timeout_ms` field name was misleading from the start, which
contributed to the swap bug.

**Recommend:** Rename `connect_timeout_ms` to `write_timeout_ms` in Phase 2
when introducing the decomposition layer. The port name in the DAG will
be `write_timeout_ms`, making intent unambiguous.

Note: `TcpStream::connect_timeout(addr, duration)` exists as an alternative
to `TcpStream::connect(addr)` for actual connect-with-timeout behavior.
If connect timeout is actually desired, add it as a third field.

---

## 6. What Each Phase Gets "For Free" From DAG Modeling

| Test type                    | Phase 1 | Phase 2 | Phase 3 |
|------------------------------|---------|---------|---------|
| Signature matching           | --      | Yes     | Yes     |
| DryRun completion            | --      | Yes     | Yes     |
| Transport interception       | --      | Yes     | Yes     |
| Per-node I/O examples        | --      | Yes     | Yes     |
| Windowed segment tests       | --      | Yes     | Yes     |
| Skip propagation             | --      | Yes     | Yes     |
| Cardinality boundary tests   | --      | Yes     | Yes     |
| Behavioral step coverage     | Manual  | Manual  | Yes     |
| Error-mode scenarios         | Manual  | Manual  | Yes     |
| Field routing verification   | Manual  | Yes     | Yes     |
| Generated mock servers       | --      | --      | Yes     |

---

## 7. Relation to Existing Architecture

This migration **extends** the current architecture rather than replacing it:

- The Prepare -> Execute -> Parse triplet remains the fundamental pattern
- `TransportOps::Execute` remains the I/O boundary
- The decomposition adds a layer **inside** the triplet, between the
  tool's Prepare node and the actual I/O
- The `TransportBackend` trait continues to work for mock injection
- The clippy `disallowed_methods` enforcement is unchanged
- Existing tool DAGs keep working unmodified (decomposition is opt-in)

The key architectural insight is: **the DAG already models the right
thing (Prepare/Execute/Parse). The gap is that Execute is opaque.
Decomposing it makes the internal field routing visible to testgen.**
