# DSL Codegen Roadmap: From .dag to Working Binaries

**Status**: Working Draft — February 2026
**Companion**: [`dsl-design.md`](./dsl-design.md), [`dsl-roadmap.md`](./dsl-roadmap.md)

**Definition of Done**: `daglang compile foo.dag --target {rust,go,c,mips}` produces
a compilable/assembable artifact that runs identically to the hand-built binary.
Multiple language flavors prove the emit layer understands the computation, not just
the target syntax.

---

## Current State

```
Parse → Resolve → Typecheck → Lower → Derive → Emit
  ✓        ✓          ✓          ✓        ✓       STUB
```

The compiler pipeline is complete through derive. The emit phase (`daglang-emit`) is
scaffold-only — `RustBackend` has 6/7 trait methods returning TODO comments.

Two execution paths exist today:

| Path | Status | How it works |
|------|--------|-------------|
| **Hand-built** | Production (10 binaries) | Rust `Dag<ConcreteOp>` + `gunbc-exec` engine |
| **Exec-bridge** | Makegen only | `Dag<LoweredOp>` → `Dag<ResolvedOp>` runtime dispatch |

Neither path generates source code. The exec-bridge proves DSL → execution works at
runtime for makegen. The codegen path must produce static source files that compile to
equivalent binaries.

### Operation catalog (from codebase survey)

~95 distinct `Executable` implementations across the codebase:
- **~90% pure computation** (string transforms, list ops, JSON extraction, template rendering)
- **1 transport boundary** (`TransportOps::Execute` — the only node that performs world I/O)
- **3 resource acquisition** nodes (`FsEnv`, `ClockEnv`, `NetEnv`)
- **20+ prepare→execute→parse triplets** (universal I/O pattern)

Every operation follows the same contract:
```
fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>
```

---

## Architecture: Three Language Flavors

The backends are chosen to stress-test different dimensions of the emit layer:

| Target | Why | Forces |
|--------|-----|--------|
| **Rust** | Closest to source, reuse gunbc-exec runtime | Correct trait impls, dependency wiring |
| **Go** | GC'd, simple types, different concurrency model | No ownership, different error handling, goroutines |
| **C** | No runtime, manual memory, stepping stone to asm | Explicit allocation, no generics, function pointers |
| **MIPS** | No abstractions at all | Register allocation, memory layout, proves emit understands the actual computation |

Rust starts with Layer 1 (exec-runtime) then moves to Layer 2 (native). Go/C/MIPS
are Layer 2 from the start — they can't use gunbc-exec.

### Layer 1 vs Layer 2

**Layer 1**: Generated code builds a `Dag<T>` and calls `gunbc-exec` to run it.
The binary depends on gunbc-ir + gunbc-exec as libraries. Fast to implement.

**Layer 2**: Generated code IS the computation. Topo-sorted function calls with
data passing between them. No DAG runtime. The binary is standalone.

```
Layer 1 (Rust only):
  Generated main.rs → build_graph() → execute_and_display()
  Dependencies: gunbc-ir, gunbc-exec, gunbc-lib-transport

Layer 2 (all targets):
  Generated code → topo-ordered function calls → direct transport calls
  Dependencies: only libc (C/MIPS) or stdlib (Go) or minimal runtime crate (Rust)
```

---

## Track A: Emit Infrastructure (shared across all backends)

### A1. Redesign CodegenBackend trait
**Files**: `core/daglang/daglang-emit/src/lib.rs`

Replace the current string-in/string-out trait with structured IR input:

```rust
pub trait CodegenBackend {
    fn emit_crate(
        &self,
        dag: &Dag<LoweredOp>,
        artifacts: &DerivedArtifacts,
        config: &EmitConfig,
    ) -> Result<EmissionBundle, EmitError>;
}
```

The backend receives the full lowered DAG with all port types, cardinalities, service
metadata, obligations, and derived artifacts. It returns a bundle of files.

Tasks:
- [ ] A1.1 — Define `EmitConfig` (module name, output dir, target-specific options)
- [ ] A1.2 — Replace `CodegenBackend` trait with `emit_crate` method
- [ ] A1.3 — Update `emit_rust_bundle()` to use new trait (keep scaffold behavior for now)
- [ ] A1.4 — Update `daglang-driver` to pass full config through pipeline

### A2. Computation model for emit
**Files**: new `core/daglang/daglang-emit/src/computation.rs`

Before emitting any language, formalize what each node DOES as a
target-independent computation description:

```rust
enum Computation {
    /// Pure function: read named inputs, apply transform, produce named outputs
    Pure {
        inputs: Vec<(String, ValueType)>,
        outputs: Vec<(String, ValueType)>,
        body: PureBody,
    },
    /// Transport boundary: build request, execute I/O, parse response
    Transport {
        request_builder: RequestSpec,
        response_parser: ResponseSpec,
    },
    /// Resource acquisition: produce a handle value
    ResourceAcquire { handle_type: String, handle_value: String },
    /// Collection operation: apply function to list elements
    Collection { kind: CollectionOpKind, element_computation: Box<Computation> },
}

enum PureBody {
    /// Hardcoded JSON value (LoadRegistry, FsEnv)
    Literal(serde_json::Value),
    /// Template interpolation (Format, RenderMakefile)
    Template { template: String, variables: Vec<String> },
    /// String operation (Concat, Split, Filter, Map)
    StringOp(StringOpKind),
    /// JSON extraction (Extract, ParseJson)
    JsonOp(JsonOpKind),
    /// Comparison (CompareContent)
    Compare { left: String, right: String, output: String },
    /// Conditional passthrough (Guard, Branch)
    Conditional { condition: String, if_true: String, if_false: Option<String> },
    /// Delegated to service-specific handler
    ServiceCall(ServiceCallMetadata),
}
```

This is the intermediate representation between `LoweredOp` and target-specific code.
Every backend consumes `Computation`, not `LoweredOp` directly.

Tasks:
- [ ] A2.1 — Define `Computation` enum and `PureBody` variants
- [ ] A2.2 — Define `ValueType` enum (Str, Bool, Int, Json, List, Map, TransportRequest, TransportResponse, Skipped)
- [ ] A2.3 — Implement `lower_to_computation(node: &Node<LoweredOp>) -> Computation`
- [ ] A2.4 — Write tests: every makegen node → expected Computation variant

### A3. Topo-order emission plan
**Files**: new `core/daglang/daglang-emit/src/plan.rs`

Compute the emit plan: ordered list of computations with data flow between them.
This is what every backend walks to generate code.

```rust
struct EmitPlan {
    /// Nodes in topological execution order
    steps: Vec<EmitStep>,
    /// Entrypoint ports (become CLI args or function params)
    entrypoints: Vec<EntrypointPort>,
    /// Transport node IDs (become intercept points in dry-run)
    transport_nodes: Vec<String>,
}

struct EmitStep {
    node_id: String,
    computation: Computation,
    /// Where this step's inputs come from
    input_sources: Vec<InputSource>,
    /// Where this step's outputs flow to
    output_sinks: Vec<OutputSink>,
}

enum InputSource {
    /// From a previous step's output
    Edge { from_step: usize, from_port: String },
    /// From CLI arg / entrypoint
    Entrypoint { port_name: String },
}
```

Tasks:
- [ ] A3.1 — Define `EmitPlan`, `EmitStep`, `InputSource`, `OutputSink`
- [ ] A3.2 — Implement `build_emit_plan(dag, artifacts) -> EmitPlan`
- [ ] A3.3 — Write tests: makegen DAG → expected plan with 10 steps in topo order
- [ ] A3.4 — Write tests: pragma DAG → expected plan with 3 parallel chains

### A4. `daglang compile` CLI command
**Files**: `core/daglang/daglang-cli/src/main.rs`

Add the compile subcommand that drives the full pipeline:

```
daglang compile <input.dag> --target {rust|go|c|mips} [--out <dir>] [--layer {1|2}]
```

Tasks:
- [ ] A4.1 — Add `compile` subcommand to CLI arg parser
- [ ] A4.2 — Wire compile command through driver → emit pipeline
- [ ] A4.3 — Write output files to `--out` directory
- [ ] A4.4 — Add `--target` flag with backend selection

### A5. Cross-language parity test harness
**Files**: new `core/daglang/daglang-cli/tests/codegen_parity.rs`

For each target, the test harness:
1. Compiles .dag → target source
2. Builds target (cargo build / go build / gcc / mips-gcc)
3. Runs generated binary with test fixtures
4. Captures output
5. Asserts identical output across all targets

Tasks:
- [ ] A5.1 — Define parity test trait/interface
- [ ] A5.2 — Implement file-content comparison (ignore timestamps/paths)
- [ ] A5.3 — Implement cross-target parity assertion
- [ ] A5.4 — Makegen parity fixture: known tool registry → expected Makefile content

---

## Track B: Rust Backend

### B1. Layer 1 — Exec-runtime Rust codegen (fast path)
**Files**: `core/daglang/daglang-emit/src/rust_backend.rs` (new, extracted from lib.rs)

Generate Rust code that uses gunbc-exec as runtime. This is the fastest path to a
working generated binary.

#### B1.1 — Op enum generation
Emit a `#[derive(Debug, Clone)]` enum with one variant per DAG node:
```rust
// Generated: target/generated/makegen/src/ops.rs
#[derive(Debug, Clone)]
pub enum Op {
    LoadRegistry,
    FsEnv,
    RenderMakefile,
    PrepareReadMakegen,
    ExecuteReadMakegen,
    CompareContent,
    PrepareWriteMakegen,
    ExecuteTransport,
    MakegenEntrypoint,
}
```

Tasks:
- [ ] B1.1.a — Walk DAG nodes, collect unique op names
- [ ] B1.1.b — Emit enum definition with derive macros
- [ ] B1.1.c — Emit `impl std::fmt::Display for Op`

#### B1.2 — Executable impl generation
Emit `impl Executable for Op` with a match arm per variant. Each arm delegates to
an executor function (either inline or from runtime library).

For makegen, the 10 executor bodies are already implemented in `daglang-exec-bridge`.
The codegen emits equivalent source code:

```rust
// Generated
impl Executable for Op {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Op::LoadRegistry => {
                // Emit literal JSON value
                OutputMap::new().json("registry", json!({...})).ok()
            }
            Op::RenderMakefile => {
                // Emit template rendering logic
                let registry = require_json(&inputs, "registry")?;
                // ...string building...
                OutputMap::new().str("return", &makefile_content).ok()
            }
            // ...
        }
    }
}
```

Tasks:
- [ ] B1.2.a — Emit executor for `Computation::Pure(Literal)` — hardcoded values
- [ ] B1.2.b — Emit executor for `Computation::Pure(Template)` — string interpolation
- [ ] B1.2.c — Emit executor for `Computation::Transport` — TransportRequest build + execute
- [ ] B1.2.d — Emit executor for `Computation::Pure(Compare)` — content freshness
- [ ] B1.2.e — Emit executor for `Computation::Pure(Conditional)` — skip/guard logic
- [ ] B1.2.f — Emit the full `impl Executable for Op` match block

#### B1.3 — Graph construction generation
Emit `fn build_graph() -> Dag<Op>` that reconstructs the DAG from IR:

Tasks:
- [ ] B1.3.a — Emit node construction (id, input ports, output ports, op variant)
- [ ] B1.3.b — Emit edge construction (from_node, from_port, to_node, to_port)
- [ ] B1.3.c — Port cardinality and type_id emission

#### B1.4 — CLI main() generation
Emit `fn main()` with:
- CLI arg parsing (from entrypoint ports)
- BoundaryMocks wiring
- ExecutionMode setup (Real / DryRun with transport mocks)
- execute_and_display() call

Tasks:
- [ ] B1.4.a — Emit CLI arg parsing from entrypoint port names/types
- [ ] B1.4.b — Emit BoundaryMocks wiring (entrypoint ports → input mocks)
- [ ] B1.4.c — Emit DryRun transport mock setup (from transport node IDs)
- [ ] B1.4.d — Emit execute_and_display() call with mode
- [ ] B1.4.e — Emit Cargo.toml with workspace-relative path deps

#### B1.5 — Makegen end-to-end
Wire everything together and verify:

Tasks:
- [ ] B1.5.a — Generate full makegen crate from `dsl/tools/makegen.dag`
- [ ] B1.5.b — `cargo build` the generated crate (must compile)
- [ ] B1.5.c — Run generated binary → verify Makefile output matches hand-built
- [ ] B1.5.d — Run generated binary in --dry-run → verify no file writes
- [ ] B1.5.e — Structural parity: generated DAG topology == hand-built topology
- [ ] B1.5.f — Add to CI as regression test

### B2. Layer 2 — Native Rust codegen (no DAG runtime)
**Files**: `core/daglang/daglang-emit/src/rust_native.rs`

Generate standalone Rust code that IS the computation — no gunbc-exec dependency.
The EmitPlan drives this: each step becomes a function call in topo order.

```rust
// Generated (Layer 2) — no Dag<T>, no execute(), no gunbc-exec
fn main() {
    let args = parse_cli_args();

    // Step 1: load_registry (pure)
    let registry = json!({ "tools": [{ "name": "makegen", "command": "..." }] });

    // Step 2: render_makefile (pure)
    let makefile_content = render_makefile(&registry);

    // Step 3: prepare_read (transport prep)
    let read_request = TransportRequest::File(FileRequest::read(&args.path));

    // Step 4: execute_read (transport boundary)
    let read_response = execute_file_transport(&read_request);

    // Step 5: compare_content (pure)
    let fresh = read_response.content() == &makefile_content;

    // Step 6-8: conditional write
    if !fresh {
        let write_request = TransportRequest::File(FileRequest::write(&args.path, &makefile_content));
        execute_file_transport(&write_request);
    }

    // Step 9: report
    println!("written: {}", !fresh);
}
```

Tasks:
- [ ] B2.1 — Emit topo-ordered function calls from EmitPlan
- [ ] B2.2 — Emit variable declarations with proper types
- [ ] B2.3 — Emit data flow: step outputs → next step inputs via local variables
- [ ] B2.4 — Emit transport calls via minimal runtime crate (just file I/O + shell exec)
- [ ] B2.5 — Emit conditional execution (skip semantics)
- [ ] B2.6 — Emit CLI arg parsing (standalone, no gunbc-cli dep)
- [ ] B2.7 — Create `daglang-runtime-rs` minimal crate (transport + value types only)
- [ ] B2.8 — Makegen Layer 2 end-to-end parity test

### B3. Expand to more workflows (Rust)

Tasks:
- [ ] B3.1 — Pragma: 3 parallel content upsert chains (tests parallel codegen)
- [ ] B3.2 — Codegen: conditional execution with `when` guards
- [ ] B3.3 — Build: parallel test + clippy after build, aggregate results
- [ ] B3.4 — Bootstrap: workspace scan → generate files
- [ ] B3.5 — Parity tests for each (generated output == hand-built output)

---

## Track C: Go Backend

### C1. Go emission framework
**Files**: new `core/daglang/daglang-emit/src/go_backend.rs`

Go is Layer 2 from the start — no gunbc-exec equivalent exists.

#### C1.1 — Go type emission
Map DSL value types to Go:
```
String → string           List<String> → []string
Bool → bool               Map<K,V> → map[K]V
Int → int64               Json → interface{}
TransportRequest → struct TransportResponse → struct
```

Tasks:
- [ ] C1.1.a — Define Go type mapping table
- [ ] C1.1.b — Emit Go struct definitions from port type_ids
- [ ] C1.1.c — Emit TransportRequest/TransportResponse Go structs

#### C1.2 — Go function emission
Each EmitStep becomes a Go function:
```go
func loadRegistry() map[string]interface{} {
    return map[string]interface{}{
        "tools": []interface{}{
            map[string]interface{}{"name": "makegen", "command": "..."},
        },
    }
}

func renderMakefile(registry map[string]interface{}) string {
    // template rendering
}
```

Tasks:
- [ ] C1.2.a — Emit Go functions from `Computation::Pure(Literal)`
- [ ] C1.2.b — Emit Go functions from `Computation::Pure(Template)`
- [ ] C1.2.c — Emit Go functions from `Computation::Transport`
- [ ] C1.2.d — Emit Go functions from `Computation::Pure(Compare)`
- [ ] C1.2.e — Emit Go functions from `Computation::Pure(Conditional)`

#### C1.3 — Go main() and transport runtime
```go
func main() {
    path := flag.String("path", "Makefile", "output path")
    flag.Parse()
    // topo-ordered calls...
}
```

Tasks:
- [ ] C1.3.a — Emit `package main` with flag parsing from entrypoints
- [ ] C1.3.b — Emit topo-ordered function calls in main()
- [ ] C1.3.c — Emit Go file I/O runtime (os.ReadFile, os.WriteFile)
- [ ] C1.3.d — Emit Go shell exec runtime (os/exec.Command)
- [ ] C1.3.e — Emit `go.mod`

#### C1.4 — Go makegen end-to-end

Tasks:
- [ ] C1.4.a — Generate full Go module from `dsl/tools/makegen.dag`
- [ ] C1.4.b — `go build` succeeds
- [ ] C1.4.c — Output parity: Go binary produces identical Makefile to Rust
- [ ] C1.4.d — Cross-language parity test in CI

#### C1.5 — Expand Go to more workflows

Tasks:
- [ ] C1.5.a — Pragma in Go
- [ ] C1.5.b — Build in Go (tests goroutine-based parallelism in generated code)

---

## Track D: C Backend

### D1. C emission framework
**Files**: new `core/daglang/daglang-emit/src/c_backend.rs`

C forces explicit memory management, no generics, function pointers for dispatch.
This is the stepping stone to MIPS — if C works, the assembly mapping is mechanical.

#### D1.1 — C type emission
```c
// Value type — tagged union
typedef enum { VAL_STR, VAL_BOOL, VAL_INT, VAL_JSON, VAL_LIST, VAL_SKIP } ValueTag;
typedef struct {
    ValueTag tag;
    union {
        char *str;
        int boolean;
        int64_t integer;
        char *json;  // serialized JSON string
        struct { char **items; size_t count; } list;
    };
} Value;
```

Tasks:
- [ ] D1.1.a — Define C Value tagged union
- [ ] D1.1.b — Define C TransportRequest/TransportResponse structs
- [ ] D1.1.c — Emit helper functions: `value_str()`, `value_bool()`, `value_list_get()`

#### D1.2 — C function emission
```c
Value load_registry(void) {
    // Return literal JSON string
    return value_str("{\"tools\":[{\"name\":\"makegen\",\"command\":\"...\"}]}");
}

Value render_makefile(Value registry) {
    // String building
}
```

Tasks:
- [ ] D1.2.a — Emit C functions from `Computation::Pure(Literal)`
- [ ] D1.2.b — Emit C functions from `Computation::Pure(Template)` (snprintf-based)
- [ ] D1.2.c — Emit C functions from `Computation::Transport` (system() or popen())
- [ ] D1.2.d — Emit C string operations (strcmp for Compare, strcat for Template)
- [ ] D1.2.e — Emit C conditional execution

#### D1.3 — C main() and runtime

Tasks:
- [ ] D1.3.a — Emit `main()` with getopt-style arg parsing
- [ ] D1.3.b — Emit topo-ordered function calls
- [ ] D1.3.c — Emit C file I/O (fopen/fread/fwrite/fclose)
- [ ] D1.3.d — Emit C shell exec (popen/pclose)
- [ ] D1.3.e — Emit Makefile or build script for compilation
- [ ] D1.3.f — Memory management: arena allocator or explicit free chains

#### D1.4 — C makegen end-to-end

Tasks:
- [ ] D1.4.a — Generate C source from `dsl/tools/makegen.dag`
- [ ] D1.4.b — `gcc -o makegen makegen.c` succeeds
- [ ] D1.4.c — Output parity: C binary produces identical Makefile
- [ ] D1.4.d — Cross-language parity test in CI
- [ ] D1.4.e — Valgrind clean (no memory leaks)

---

## Track E: MIPS Backend

### E1. MIPS emission framework
**Files**: new `core/daglang/daglang-emit/src/mips_backend.rs`

MIPS forces the emit layer to reason about:
- Register allocation (which values live in $t0-$t9, $s0-$s7)
- Stack frame layout (function arguments, local variables, return values)
- String operations at the byte level (no stdlib beyond syscalls)
- System calls for I/O (read/write/open/close via $v0 syscall numbers)

If we can emit correct MIPS for makegen, the codegen truly understands the computation.

#### E1.1 — MIPS value representation
```asm
# Value = 8-byte tagged pointer
# [tag:4][payload:4]
# tag: 0=str(ptr), 1=bool(0/1), 2=int(i32), 3=list(ptr,len)
# Strings: null-terminated, heap-allocated
# Lists: contiguous array of Value pointers
```

Tasks:
- [ ] E1.1.a — Define MIPS value layout (tagged pointer, 8 bytes)
- [ ] E1.1.b — Emit string literal data section (.data segment)
- [ ] E1.1.c — Emit heap allocator (sbrk-based bump allocator)
- [ ] E1.1.d — Emit value constructors (value_str, value_bool, value_int)

#### E1.2 — MIPS function emission
Each computation step becomes a labeled MIPS procedure:

```asm
load_registry:
    # Return pointer to embedded JSON string literal
    la   $v0, _str_registry_json
    jr   $ra

render_makefile:
    # $a0 = registry JSON pointer
    # Build Makefile content string on heap
    # ... string concatenation via byte copies ...
    # $v0 = pointer to result string
    jr   $ra
```

Tasks:
- [ ] E1.2.a — Emit MIPS functions from `Computation::Pure(Literal)` (la + jr)
- [ ] E1.2.b — Emit MIPS string concatenation (byte-copy loop)
- [ ] E1.2.c — Emit MIPS string comparison (byte-compare loop for freshness check)
- [ ] E1.2.d — Emit MIPS conditional branches (beq/bne for skip logic)
- [ ] E1.2.e — Emit MIPS function calling convention ($a0-$a3 args, $v0 return, $ra save)

#### E1.3 — MIPS I/O and syscalls

```asm
# File read via Linux syscalls
#   li $v0, 5         # sys_open
#   la $a0, filepath
#   li $a1, 0         # O_RDONLY
#   syscall
#   move $s0, $v0     # fd in $s0
#
#   li $v0, 3         # sys_read
#   move $a0, $s0     # fd
#   la $a1, buffer
#   li $a2, 4096      # count
#   syscall
```

Tasks:
- [ ] E1.3.a — Emit MIPS file open/read/write/close syscall sequences
- [ ] E1.3.b — Emit MIPS main entry point with stack setup
- [ ] E1.3.c — Emit MIPS CLI arg parsing (walk argv on stack)
- [ ] E1.3.d — Emit topo-ordered jal (jump-and-link) sequence in main
- [ ] E1.3.e — Register allocation strategy (simple: $s0-$s7 for step outputs, spill to stack)

#### E1.4 — MIPS makegen end-to-end

Tasks:
- [ ] E1.4.a — Generate MIPS assembly from `dsl/tools/makegen.dag`
- [ ] E1.4.b — Assemble with `mips-linux-gnu-as` (or SPIM/MARS for testing)
- [ ] E1.4.c — Link with `mips-linux-gnu-ld`
- [ ] E1.4.d — Run under QEMU user-mode emulation
- [ ] E1.4.e — Output parity: MIPS binary produces identical Makefile
- [ ] E1.4.f — Cross-language parity test: Rust == Go == C == MIPS output

---

## Track F: Test Generation

### F1. Obligation-driven test emission

The derive phase already computes `TestObligations` per workflow. Emit test suites
that satisfy these obligations in each target language.

Tasks:
- [ ] F1.1 — Emit dry-run completion test (execute full DAG with transport mocks)
- [ ] F1.2 — Emit per-transport-node mock test (one test per transport node)
- [ ] F1.3 — Emit pure-node snapshot test (from `NodeIoExample` on nodes)
- [ ] F1.4 — Emit integration test (compile → build → run → verify)
- [ ] F1.5 — Rust: `#[test]` functions in generated crate
- [ ] F1.6 — Go: `func Test*` in generated module
- [ ] F1.7 — C: test runner main with assert macros
- [ ] F1.8 — CI integration: run generated tests alongside hand-built tests

---

## Execution Order & Dependencies

```
Track A (infra):
  A1 (trait) ──→ A2 (computation model) ──→ A3 (emit plan) ──→ A4 (CLI)
                                                    │
                                                    ▼
Track B (Rust):                          ┌── B1 (Layer 1: exec-runtime)
                                         │          │
                                         │          ▼
                                         │   B1.5 (makegen e2e)
                                         │          │
                                         ├── B2 (Layer 2: native) ◄─ can start after A3
                                         │          │
                                         └── B3 (more workflows)
                                                    │
Track C (Go):                            ┌── C1 (emission) ◄─── can start after A3
                                         │          │
                                         └── C1.4 (makegen e2e)
                                                    │
Track D (C):                             ┌── D1 (emission) ◄─── can start after A3
                                         │          │
                                         └── D1.4 (makegen e2e)
                                                    │
Track E (MIPS):                          ┌── E1 (emission) ◄─── can start after A3 + D1.1
                                         │          │
                                         └── E1.4 (makegen e2e)
                                                    │
                                                    ▼
Track A continued:                           A5 (cross-language parity)
                                                    │
Track F (tests):                             F1 (test generation)
```

### Recommended work order

**Week 1**: A1-A3 (emit infrastructure) + B1.1-B1.4 (Rust Layer 1 codegen)
**Week 2**: B1.5 (Rust makegen e2e) + A4 (CLI) + start C1.1-C1.2 (Go types/functions)
**Week 3**: B2 (Rust Layer 2 native) + C1.3-C1.4 (Go runtime + e2e) + start D1.1 (C types)
**Week 4**: D1.2-D1.4 (C functions + e2e) + E1.1-E1.2 (MIPS values + functions)
**Week 5**: E1.3-E1.4 (MIPS I/O + e2e) + A5 (cross-language parity) + B3 (more workflows)
**Week 6**: F1 (test generation) + polish + CI integration

### Parallelism opportunities

Once A3 (emit plan) is done, all four backends (B/C/D/E) can proceed in parallel.
Within each backend, type emission → function emission → main/runtime → e2e is sequential.
Cross-language parity (A5) requires at least 2 backends working.

---

## Gap Analysis

| Component | Current | Needed | Effort |
|-----------|---------|--------|--------|
| **Emit infrastructure** | | | |
| `CodegenBackend` trait | String-in/string-out | Structured IR input | S |
| `Computation` IR | None | Target-independent computation model | M |
| `EmitPlan` | None | Topo-ordered step list with data flow | M |
| `daglang compile` CLI | None | Driver integration | S |
| Cross-language parity harness | None | Build + run + compare framework | M |
| **Rust backend** | | | |
| Layer 1 (exec-runtime) | Stub | Full graph + ops + main codegen | L |
| Layer 2 (native) | None | Standalone topo-ordered codegen | L |
| `daglang-runtime-rs` | None | Minimal transport + value crate | M |
| **Go backend** | | | |
| Type emission | None | struct + interface codegen | M |
| Function emission | None | func codegen from Computation | L |
| Go runtime (transport) | None | os.ReadFile + exec.Command | M |
| **C backend** | | | |
| Tagged union Value type | None | C struct with enum tag | M |
| Function emission | None | C functions from Computation | L |
| Memory management | None | Arena or explicit free | M |
| **MIPS backend** | | | |
| Value representation | None | Tagged pointer layout | M |
| Function emission | None | MIPS procedures from Computation | XL |
| Syscall I/O | None | open/read/write/close sequences | L |
| Register allocation | None | Linear scan or simple strategy | L |

**S** = small (< 1 day), **M** = medium (1-3 days), **L** = large (3-5 days), **XL** = extra-large (5-8 days)

---

## Success Criteria

The project is **done** when:

1. `daglang compile dsl/tools/makegen.dag --target rust` → cargo build → produces correct Makefile
2. `daglang compile dsl/tools/makegen.dag --target go` → go build → produces identical Makefile
3. `daglang compile dsl/tools/makegen.dag --target c` → gcc → produces identical Makefile
4. `daglang compile dsl/tools/makegen.dag --target mips` → mips-as + qemu → produces identical Makefile
5. At least 1 additional workflow (pragma) compiles to Rust and Go with output parity
6. Generated test suites pass in at least Rust and Go
7. Cross-language parity test runs in CI

---

## Relationship to Existing Documents

| Document | Scope | Overlap |
|----------|-------|---------|
| `dsl-roadmap.md` Part 1 | DSL Build (compiler pipeline) | **Complete** — this roadmap starts where Part 1 ends |
| `dsl-roadmap.md` Part 2 | Migration plan | This roadmap is the **prerequisite** — codegen must work before migration |
| `dsl-design.md` §Emit | Emission target spec (13 targets) | This roadmap implements targets 1-4 (Rust, Go, C, MIPS) |
| `TODO_URGENT_dsl_migration.md` | Migration checklist | The "Ready Now" items become Track B3 targets |
