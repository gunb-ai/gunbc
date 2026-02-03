# Consolidation: Generic Ops and Rendering DAGs

**Status**: Ongoing
**Date**: 2026-02-03

Tracking for misplaced generic ops, duplicated patterns, and rendering
workflows that should become DAGs. Add items as they're discovered.

---

## 1. Generic ops living in domain crates

Ops that aren't domain-specific but currently live in a specific tool.
Extract to `lib/primitives` or `core/ir` when a second consumer appears.

### HashOp — stable ID generation

**Where it lives**: `lib/review/src/lib.rs` (`hash_finding_id()`)
and `lib/blob/src/lib.rs` (`BlobMeta::compute_hash()`)

**Problem**: Two different hash implementations for the same purpose
(stable content-addressed IDs). Review uses SHA256, blob uses
`DefaultHasher`. Any tool producing findings, diagnostics, or
content-addressed artifacts needs this.

**Proposed**: Generic `StableHashOp` in `lib/primitives`:

```rust
/// Compute a stable content hash from N input strings.
/// Produces a hex-encoded truncated SHA256.
pub struct StableHashOp {
    /// Number of bytes to keep (default 16 → 32 hex chars).
    pub truncate_bytes: usize,
}
```

Inputs: `parts` (list of strings to hash with `:` separator)
Output: `hash` (hex string)

**Consumers**: review findings, blob metadata, any dedup scenario.

### MergeOutputs — cardinality-aware collection

**Where it lives**: `lib/review/src/lib.rs` (`MergeOutputs`)

**Problem**: The null→0, object→1, array→N cardinality handling is
universal. The dedup-by-id logic is domain-specific. These are mixed
in one op.

**Proposed**: Split into two concerns:

1. **Engine-level** (cardinality design doc): The execution engine
   should normalize cardinality before ops see it. Once that lands,
   the defensive deserialization in MergeOutputs disappears.
2. **Generic DeduplicateOp** in `lib/primitives`: Dedup a JSON array
   by a configurable ID field. Review's `MergeOutputs` becomes a
   thin wrapper that calls this.

**Blocked on**: Cardinality-transparent execution (separate design doc).

### FormatDiffArtifact — structured-to-text rendering

**Where it lives**: `lib/review/src/lib.rs` (`FormatDiffArtifact`)

**Problem**: Joining a `MapStrStr` of filenames→chunks into a formatted
string is not review-specific. Gist does similar rendering. Any tool
that processes per-file data and needs a combined view would use this.

**Proposed**: Move to `lib/primitives` or `lib/markdown` as
`FormatMapOp`:

```rust
/// Join a MapStrStr into a formatted string.
/// Each entry rendered as "--- {key}\n{value}".
pub struct FormatMapOp {
    pub separator: String,     // default "\n\n"
    pub entry_format: String,  // default "--- {key}\n{value}"
    pub empty_text: String,    // default "(empty)"
}
```

---

## 2. Rendering workflows that should become DAGs

Currently, rendering is done in plain functions. These are pure
transforms (structured data → text) which is exactly what DAG ops
are for. Moving them into DAGs would make them testable, composable,
and interceptable.

### Makefile rendering

**Where it lives**: `gunbc-dag/src/makegen/render.rs` (479 lines)

**Current**: `MakefileRenderer` struct with `render_makefile()` function
and many `render_*` helpers. Implements `Renderable` trait.

**Why DAG**: Makefile rendering is a pipeline:
1. Scan workspace → crate list
2. Load tool registry → target definitions
3. Render targets → target blocks
4. Render help → help text
5. Assemble → final Makefile with header

Each step is a pure transform. As a DAG, individual render steps
could be tested, mocked, or swapped (e.g., render to Justfile
instead of Makefile).

**Complexity**: Medium. The renderer has ~15 helper functions.
Converting to a DAG means each becomes a node. Worth doing when
adding a second output format (Justfile, Taskfile, etc.).

### CI workflow rendering

**Where it lives**: `core/ir/src/transport/ci/render.rs`
and provider-specific renderers (GitHub Actions YAML).

**Current**: `CiRenderer` trait with `render()` method producing YAML.

**Why DAG**: CI rendering is already structured as:
1. Collect jobs from DAG transport nodes
2. Map to provider-specific YAML structure
3. Render YAML text

A DAG version would make provider-switching composable and testable.

**Complexity**: Low-medium. The trait is clean.

### Code generation (graph.rs, cli)

**Where it lives**: `core/codegen/src/dag_gen.rs`, `cli_gen.rs`

**Current**: Functions that generate Rust source code for DAG binaries.

**Why DAG**: Codegen is a multi-step pipeline:
1. Read tool registry → tool definitions
2. Generate GraphOp enum → Rust source
3. Generate CLI parser → Rust source
4. Generate main.rs → Rust source
5. Write files

Each step is pure. Template rendering already uses a generic
`Template` struct. Making this a DAG would let us generate for
other languages/frameworks.

**Complexity**: High. Codegen is meta-circular (the DAG generates
code that builds DAGs). Worth doing only if we need codegen for
non-Rust targets.

---

## 3. Duplicated patterns

### GraphOp wrapper enums

Every tool defines a union enum:

```
ReviewGraphOp  { Blob, Git, Review, Llm, Transport }
GistGraphOp    { Git, Gist, ..., Transport }
CIGraphOp      { CI, Prepare, Transport, CliTool }
MakegenGraphOp { Makegen, Primitive, Transport }
DepsGraphOp    { Deps, Transport }
```

**dag-pattern-ux.md Phase 4** proposes `ToolGraphOp<DomainOp>`:

```rust
pub enum ToolGraphOp<D> {
    Domain(D),
    Primitive(PrimitiveOp),
    Transport(TransportOps),
}
```

This eliminates 5+ enum definitions. Lower priority — the current
enums work, they're just boilerplate.

### Config node pattern

Review uses `LoadPipelineConfig(ReviewPipelineConfig)`. CI uses
`EnvOp::Ci`. Both are zero-input nodes that emit build-time constants.

Not worth abstracting yet — the pattern is simple enough that each
tool can implement it. Extract if a third tool needs it.

---

## 4. Rendering infrastructure that's well-placed

These are already generic and in the right location:

| What | Where | Status |
|------|-------|--------|
| `Renderable` trait | `core/ir/src/render.rs` | Generic, good |
| `Template` struct | `core/codegen/src/template.rs` | Generic, good |
| `FormatOp` / `ConcatOp` | `lib/primitives/src/data.rs` | Generic, good |
| `CollectionOp` (Map/Filter/Fold) | `lib/primitives/src/collection.rs` | Generic, good |
| `ParseOp` (JSON/TOML) | `lib/primitives/src/data.rs` | Generic, good |
| `MarkdownOp` | `lib/markdown/src/lib.rs` | Domain-specific, correct |
| `build::port/edge/optional` helpers | `core/ir/src/dag.rs` | Generic, correct |

---

## 5. `type_id == "List"` dual encoding

Cardinality is intended to be the canonical shape layer (element type +
cardinality interval), but `"List"` is embedded as a `type_id` string
in multiple code paths. This creates dual encoding: multiplicity is
expressed both through cardinality AND through the type name.

**The canonical model**: Port type = element type + cardinality.
- `"String"` + `ZERO_OR_MORE` = tape of strings
- `"String"` + `ONE` = exactly one string
- `"String"` + `ZERO_OR_ONE` = optional string

Under this model, no port should have `type_id == "List"`. List-ness
*is* cardinality.

**Where `"List"` appears as type_id:**

| Location | File | What it does |
|----------|------|-------------|
| `repeatable` detection | `gunbc-dag/src/makegen/registry.rs:419` | `ep.type_id == "List"` to detect CLI repeatables |
| Loop pattern input | `core/ir/src/patterns/loop_pattern.rs:64` | Hardcodes `input_port_type: "List"` |
| Loop pattern output | `core/ir/src/patterns/loop_pattern.rs:69` | Hardcodes `output_port_type: "List"` |
| CLI generation | `core/codegen/src/registry.rs` | Registry port defs with `"List"` type |

**Already fixed**: `tool_names` port in makegen was
`PortDef::list_nonempty("tool_names", "List")` (list-of-lists),
corrected to `"String"` (list-of-strings).

**Migration strategy** (incremental, not big-bang):
1. Stop introducing new `"List"` uses
2. Migrate semantically critical paths: CLI parsing (repeatable =
   `cardinality.max > 1`), loop patterns (element type from port)
3. Keep compatibility shims until registry + runtime agree
4. `TypeContract::from_type_dag` already extracts cardinality from
   type DAGs — this can become the canonical port type representation

---

## 6. Codebase fragility (non-testgen)

### Builder functions referenced as strings

**Where**: `core/codegen/src/registry.rs` `ToolDef::new()` —
`graph_builder` parameter is `&str`.

The registry stores builder function names as strings (e.g.,
`"build_gist_graph"`). Testgen and codegen look up these strings
to generate code. If a builder function is renamed, no compile
error is produced — it fails at runtime or generates wrong code.

**Fix**: Store an enum key (e.g., `GraphBuilderId`) instead of a
string. Map enum → function in one place. Renames then produce
compile errors.

### `buck-out/gen` hardcoded in 16 locations

**Where**: 5 source files across `core/codegen/src/main.rs`,
`gunbc-dag/src/makegen/render.rs`, `gunbc-dag/src/ci/ops.rs`,
`gunbc-dag/src/bin/ci.rs`, `gunbc-dag/src/ci/graph_mock.rs`.

The output directory path is scattered as string literals. If
it ever changes, these won't update together.

**Fix**: Single constant in `core/ir` or `core/codegen`, referenced
everywhere. Already noted informally but not tracked.

### Static CODEGEN_SOURCES path list

**Where**: `Makefile:16` and `gunbc-dag/src/makegen/render.rs:156`

```makefile
CODEGEN_SOURCES := $(shell find core/codegen/src core/ir/src -name '*.rs')
```

The directory list is hardcoded in both the Makefile and the Makefile
generator. If codegen gains a new source dependency (e.g., a new
crate in `core/`), the staleness stamp won't track it. Generated
artifacts will appear up-to-date when they're not.

**Fix**: Either derive the list from `Cargo.toml` dependencies, or
at minimum maintain a single source-of-truth list that both the
Makefile and the renderer reference.

---

## 7. Test Pattern Retrospective

**885 manually written tests surveyed** across the codebase.
All are purely in-memory — zero real I/O in any test today.

### Pattern 1: Function Unit Tests (HashMap → execute → assert)

The most common pattern. Build a `HashMap<String, Value>` of inputs,
call the op's `execute()`, assert specific output ports.

**Where**: `gunbc-dag/src/ci/ops.rs`, `gunbc-dag/src/bootstrap/ops.rs`,
`lib/llm-ops/src/ops.rs`, every `ops.rs` file.

```rust
let mut inputs = HashMap::new();
inputs.insert("response".into(), Value::Str(json_string));
let outputs = op.execute(&inputs)?;
assert_eq!(outputs.get("build_success"), Some(&Value::Bool(true)));
```

**Consolidation opportunity**: This is exactly what `NodeExample` now
automates via testgen. Once Tier 1 infra (`execute_single_node`)
is stable, many of these hand-written tests become redundant with
their generated equivalents. Keep hand-written tests only for
edge cases not expressible as `NodeExample`.

### Pattern 2: Graph Structure Tests (static DAG properties)

Tests that verify node counts, boundary lists, entrypoints,
transport ports, and edge connectivity — without executing the DAG.

**Where**: `gunbc-dag/src/ci/graph.rs`, `gunbc-dag/src/makegen/graph.rs`,
`lib/tools/gist/src/graph.rs`.

```rust
let dag = build_ci_graph()?;
assert_eq!(dag.nodes.len(), 15);
assert!(dag.has_node("prepare_build"));
let boundaries = detect_boundaries(&dag);
assert!(boundaries.transport_nodes.contains(&"execute_transport"));
```

**Consolidation opportunity**: Testgen's Bucket A already covers
boundary detection and transport interception. Remaining structural
tests (node counts, specific node existence) are fragile — they
break whenever the graph changes. Consider replacing with
property-based checks (e.g., "all pure nodes have examples")
rather than hard-coded counts.

### Pattern 3: Signature Validation (validate + infer)

Tests that verify type signature consistency at the port level:
validate a node's signature, then infer types from connected edges.

**Where**: `gunbc-dag/src/makegen/graph.rs`, `core/ir/src/dag.rs`.

```rust
let sig = node.signature();
assert!(sig.validate().is_ok());
let inferred = sig.infer_from(&connected_edges);
assert!(inferred.is_compatible_with(&sig));
```

**Consolidation opportunity**: Testgen proves type compatibility by
construction (see header comment in generated files). These tests
are mostly redundant once testgen covers the DAG. Keep only for
testing the signature validation API itself (in `core/ir`).

### Pattern 4: Mode-Based Testing (enum variant parameterization)

Tests that exercise different modes/configurations of the same graph,
verifying structural differences.

**Where**: `lib/tools/gist/src/graph.rs` (Snapshot vs Diff mode).

```rust
let snapshot_dag = build_gist_graph(GistMode::Snapshot)?;
let diff_dag = build_gist_graph(GistMode::Diff)?;
assert!(diff_dag.has_node("git_diff"));
assert!(!snapshot_dag.has_node("git_diff"));
```

**Consolidation opportunity**: Testgen currently generates one test
suite per MockSpec/DAG pair. Mode-parameterized DAGs need one
MockSpec per mode. This is already handled (gist has separate
MockSpecs) but could be formalized as a pattern in the testgen
framework.

### Pattern 5: Execution Mode Testing (DryRun with BoundaryMocks)

Integration-style tests that execute the full DAG in DryRun mode
with mocked transport responses, verifying execution flow.

**Where**: `lib/tools/gist/tests/integration.rs`,
`lib/tools/buck2/tests/integration.rs`.

```rust
let dag = build_gist_graph(GistMode::Snapshot)?;
let mocks = gist_mock_spec().to_boundary_mocks();
let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))?;
assert!(log.get("create_gist").unwrap().was_intercepted);
```

**Consolidation opportunity**: This is exactly testgen Bucket A + C.
These hand-written integration tests are now fully subsumed by
generated tests. Once testgen covers all DAGs, these files can be
deleted or reduced to edge-case-only suites.

### Pattern 6: graph_mock.rs Test Blocks

49 hand-written tests across 8 `graph_mock.rs` files. These test
MockSpec properties (boundary presence, mock value content, chain
validation, resources). All are mechanically generatable.

**See**: `testgen-improvements.md` Phase 8 for the full extraction
plan. Patterns A (boundary presence) and C (`validate_chain`)
are safe to delete now — testgen already generates equivalent tests.
Pattern E (signature validation) needs TODO 8.2 first.

---

## 8. Integration Test Gap Analysis

**Current state**: All 885 tests are in-memory. Zero tests exercise
real transport execution. The transport abstraction layer ensures
correctness of DAG logic, but `lib/transport/src/executor.rs`
is untested against real systems.

### Design Problem: `TransportRequest` doesn't encode hermeticity

We want test categories derived from the transport type system:
**integration** (hermetic, local-only) vs **external** (non-hermetic,
network/auth). But `TransportRequest` variant alone doesn't determine
this.

**The problem is `Shell`.** Higher-level domain types know whether
they're hermetic, but that information is erased when they convert
to `TransportRequest::Shell`:

```
GitRequest::LsFiles.to_shell_request()    → Shell { command: "git", ... }   // hermetic
GistRequest::new().to_shell_request()     → Shell { command: "gh", ... }    // non-hermetic
CargoCommand::Build.to_shell_request()    → ShellRequest { command: "cargo" } // hermetic
```

After conversion, these are indistinguishable at the transport layer.
`ShellRequest` has no field indicating hermeticity. The executor
sees `Shell(ShellRequest { command, args, ... })` and has no way
to know whether it hits the network.

**Where hermeticity actually lives:**

| Producer type | File | Hermetic? |
|---------------|------|-----------|
| `GitRequest` | `core/ir/src/transport/git.rs` | Yes — local repo only |
| `CargoCommand` | `core/ir/src/cargo.rs` | Yes — local build system |
| `CliToolOp::Check/Install` | `core/ir/src/transport/cli.rs` | Yes — local PATH |
| `GistRequest` (shell) | `core/ir/src/transport/gist.rs` | **No** — `gh gist create` hits GitHub |
| `GistRequest` (REST) | `core/ir/src/transport/gist.rs` | **No** — `api.github.com` |
| `RestRequest` (LLM) | `core/ir/src/transport/rest.rs` | **No** — OpenAI/Anthropic APIs |

Hermeticity is a property of the **producer**, not the **transport
variant**. `File` is always hermetic. `Rest`/`Http`/`Tcp` are always
non-hermetic. `Shell` is mixed — depends on what produced it.

**Options to fix (not blocking, but worth designing):**

1. **Add `hermetic: bool` to `ShellRequest`** — Simple, set by
   producers. Executor can assert/filter on it. Downside: ad-hoc
   boolean, easy to get wrong.

2. **Split `Shell` variant** — `TransportRequest::LocalShell` vs
   `TransportRequest::NetworkShell`. Type-safe but changes the enum
   everywhere.

3. **Annotate at the DAG node level** — Add hermeticity metadata
   to the node that wraps `TransportOps::Execute`, not to the
   request itself. The node knows its producer. This aligns with
   how testgen already classifies nodes (boundary detection).

4. **Derive from producer before conversion** — Test categorization
   happens at the domain type level (`GitRequest`, `GistRequest`),
   not at the `TransportRequest` level. Tests import domain types
   directly and never go through the executor's dispatch.

**Current workaround**: Test categories use a manually maintained
mapping. The tables below classify by **producer type**, not by
`TransportRequest` variant, because the variant is insufficient.

### Test Categories

```
make test              # In-memory: unit + generated (DryRun, mocked boundaries)
make test-integration  # Hermetic transport producers (File, local Shell)
make test-external     # Non-hermetic transport producers (Rest, Http, Tcp, network Shell)
```

#### `test-integration` — Hermetic transport producers

Tests that execute real transport but require only the local machine.
No network, no auth tokens, no external services. Safe to run on
every CI commit.

Classified by **producer type** (not `TransportRequest` variant):

| Producer | Transport variant | What executes | Fixtures needed |
|----------|-------------------|---------------|-----------------|
| `FileRequest` | `File` | `std::fs::*` via `execute_file()` | `tempdir` |
| (raw shell) | `Shell` | `Command::new()` via `execute_shell()` | System PATH |
| `GitRequest` | `Shell` | `git` binary with deterministic flags | `tempdir` + `git init` |
| `CliToolOp` | `Shell` | `which`, tool version checks | System PATH |
| `CargoCommand` | `Shell` | `cargo build/test/clippy` | `cargo`, slow (XL) |

Concrete test suites:

| Suite | Producer | What It Covers |
|-------|----------|----------------|
| **File executor** | `FileRequest` | All 6 `FileOp` variants against temp dirs |
| **Shell executor** | raw `ShellRequest` | stdout, stderr, exit codes, env vars, working dir |
| **Git transport** | `GitRequest` | All variants against temp repo; deterministic flags produce parseable output |
| **CLI tool resolution** | `CliToolOp` | `resolve_tool_path()`, `upsert_tool()`, version checks |
| **Cargo workflows** | `CargoCommand` | Build/test/clippy via `CliTool` abstraction. Slow — gate behind `--features slow-integration` |

#### `test-external` — Non-hermetic transport producers

Tests that require network access, auth tokens, or create real
external resources. Run on schedule or manual trigger only.

| Producer | Transport variant | Why non-hermetic |
|----------|-------------------|------------------|
| `RestRequest` (GitHub) | `Rest` | Requires `AuthMethod` credentials, hits `api.github.com` |
| `RestRequest` (LLM) | `Rest` | Requires API keys, hits OpenAI/Anthropic endpoints |
| `GistRequest` (shell) | `Shell` | `gh gist create` requires `gh auth`, creates real resources |
| `HttpRequest` | `Http` | Raw HTTP to remote hosts (currently stubbed to localhost-only) |
| `TcpRequest` | `Tcp` | Raw TCP, requires network connectivity |

Concrete test suites:

| Suite | Producer | What It Covers |
|-------|----------|----------------|
| **GitHub gist (REST)** | `GistRequest` | POST to `api.github.com/gists` |
| **GitHub gist (gh CLI)** | `GistRequest` | `gh gist create` via shell |
| **LLM API calls** | `RestRequest` | OpenAI/Anthropic endpoints |
| **HTTP transport** | `HttpRequest` | Raw HTTP (could become hermetic with fixture server) |

### Boundary Summary

| Boundary | Producer | Variant | Category | Coverage | Gap |
|----------|----------|---------|----------|----------|-----|
| Filesystem | `FileRequest` | `File` | integration | None | High |
| Shell execution | raw `ShellRequest` | `Shell` | integration | None | High |
| Git CLI | `GitRequest` | `Shell` | integration | None | Medium |
| CLI tool resolution | `CliToolOp` | `Shell` | integration | None | Medium |
| Cargo/Clippy/Rustfmt | `CargoCommand` | `Shell` | integration | None | Low |
| GitHub API | `RestRequest` | `Rest` | external | None | Medium |
| GitHub CLI (gh) | `GistRequest` | `Shell` | external | None | Medium |
| LLM APIs | `RestRequest` | `Rest` | external | None | Low |
| Raw HTTP | `HttpRequest` | `Http` | external | None | Low |
| Raw TCP | `TcpRequest` | `Tcp` | external | None | Low |

### Priority

1. **File executor** (integration) — Highest value, lowest cost.
   6 `FileOp` variants, temp dirs, instant.

2. **Shell executor** (integration) — Second highest. Verify
   `execute_shell()` handles stdout/stderr/exit codes correctly.

3. **Git transport** (integration) — Temp repo, exercise all
   `GitRequest` variants, verify deterministic flag output.

4. **CLI tool resolution** (integration) — `which`-based path
   resolution, version checks, upsert pattern.

5. **GitHub REST** (external) — When auth infrastructure exists.

6. **Cargo/Clippy** (integration, gated) — Behind feature flag
   due to compilation time.

---

## 9. Executable boilerplate across ops

Patterns repeated in every `Executable::execute` implementation.
These are the highest-impact consolidation targets by occurrence count.

### Input extraction — 80 occurrences across 18 files

**Pattern**: Every op manually extracts inputs with identical
error-handling boilerplate:

```rust
let x = inputs.get("x")
    .and_then(|v| v.as_str())
    .ok_or_else(|| ExecError::new("missing or invalid 'x' input"))?;
```

70 "missing or invalid" error messages, 80 `.ok_or_else(|| ExecError::new`
calls across the codebase. The heaviest files:
- `lib/tools/deps/src/ops.rs` (9)
- `lib/llm-ops/src/lib.rs` (10)
- `lib/primitives/src/data.rs` (7)
- `lib/review/src/lib.rs` (7)
- `lib/primitives/src/collection.rs` (6)
- `core/exec/src/pattern_op.rs` (6)

**Proposed**: Helper functions on the input map:

```rust
// In core/exec or core/ir
pub fn require_str(inputs: &HashMap<String, Value>, key: &str) -> Result<&str, ExecError>;
pub fn require_json(inputs: &HashMap<String, Value>, key: &str) -> Result<&Value, ExecError>;
pub fn optional_str(inputs: &HashMap<String, Value>, key: &str) -> Option<&str>;
```

These compose: `let x = require_str(inputs, "x")?;` replaces
3 chained method calls. Error messages become consistent automatically.

### Output map construction — 164 occurrences across 24 files

**Pattern**: Every op builds its output HashMap manually:

```rust
let mut out = HashMap::new();
out.insert("key".to_string(), Value::Str(content.to_string()));
out.insert("other".to_string(), Value::Bool(true));
Ok(out)
```

164 `let mut out = HashMap::new()` across 24 files. The heaviest:
- `gunbc-dag/src/ci/ops.rs` (29)
- `lib/tools/deps/src/ops.rs` (22)
- `lib/tools/gist/src/graph.rs` (20)
- `core/exec/src/pattern_op.rs` (9)

**Proposed**: Builder or helper macro:

```rust
// Option A: builder
OutputMap::new()
    .str("key", content)
    .bool("other", true)
    .build()

// Option B: macro
outputs! {
    "key" => Value::Str(content.to_string()),
    "other" => Value::Bool(true),
}
```

### Response type matching — 51 occurrences across 17 files

**Pattern**: Parse ops extract the response variant with identical
match + error fallback:

```rust
match response {
    TransportResponse::Shell(shell) => { /* use shell.stdout */ }
    _ => return Err(ExecError::new("unexpected response type")),
}
```

51 `TransportResponse::Shell(` matches, 11 "unexpected response type"
errors. No convenience methods exist on `TransportResponse`.

**Proposed**: Add typed extraction methods:

```rust
impl TransportResponse {
    pub fn require_shell(&self) -> Result<&ShellResponse, ExecError>;
    pub fn require_rest(&self) -> Result<&RestResponse, ExecError>;
    pub fn require_file(&self) -> Result<&FileResponse, ExecError>;
}
```

Each returns a consistent error. Callers become:
`let shell = response.require_shell()?;`

---

## 10. ShellRequest construction — 20 occurrences across 6 files

**Where**: `lib/blob/`, `lib/tools/deps/`, `lib/tools/gist/`,
`lib/primitives/src/io.rs`, `gunbc-dag/src/bootstrap/`

**Problem**: Raw struct construction with repeated field patterns:

```rust
TransportRequest::Shell(ShellRequest {
    command: "git".to_string(),
    args: vec!["diff".to_string(), ...],
    cwd: Some(repo_path.to_string()),
    env: HashMap::new(),
    stdin: None,
})
```

Some transports have builders (`GitRequest::to_shell_request`,
`CargoCommand::to_shell_request`) but most callers construct raw.

**Proposed**: Builder on `ShellRequest`:

```rust
ShellRequest::new("git")
    .args(["diff", &base_ref])
    .cwd(repo_path)
    .into_transport_request()
```

Consumers: blob fetch, deps ops, gist ops, bootstrap ops, io primitives.

---

## 11. MockSpec test boilerplate — 8 graph_mock.rs files

**Where**: Every crate with a DAG defines a `graph_mock.rs` with
near-identical MockSpec construction and boundary assertion tests.

**Files**: `lib/review/`, `lib/llm-ops/`, `lib/tools/gist/`,
`lib/tools/buck2/`, `lib/tools/deps/`, `gunbc-dag/src/*/`

**Pattern**: Each file has:
1. A `xxx_mock_spec()` function building MockSpec chains
2. Tests asserting boundaries exist: `assert!(spec.get_boundary_mock("node", "port").is_some())`
3. Tests asserting mock specs are complete

The boundary assertion tests are nearly copy-paste across files.

**Proposed**: Parameterized test helper:

```rust
/// Assert all expected boundaries exist in a MockSpec.
pub fn assert_boundaries(spec: &MockSpec, expected: &[(&str, &str)]) {
    for (node, port) in expected {
        assert!(spec.get_boundary_mock(node, port).is_some(),
            "missing boundary mock for {}.{}", node, port);
    }
}
```

This could live in `core/test/` alongside existing test infrastructure.

---

## 12. Skipped-value propagation boilerplate — 8 occurrences across 3 files

**Where**: `gunbc-dag/src/ci/ops.rs` (5), `lib/llm-ops/src/lib.rs` (2),
`gunbc-dag/src/bootstrap/ops.rs` (1)

**Problem**: Every parse op that receives a transport response must
check if the upstream was skipped and propagate `Value::Skipped` to
all outputs. This produces 5–8 lines of identical boilerplate per op:

```rust
if matches!(inputs.get("response"), Some(Value::Skipped)) {
    return OutputMap::new()
        .value("field_a", Value::Skipped)
        .value("field_b", Value::Skipped)
        .value("field_c", Value::Skipped)
        .ok();
}
```

The only thing that varies is the list of output field names.

**Proposed**: Helper function in `core/exec/src/helpers.rs`:

```rust
/// If the given input key is `Value::Skipped`, return all output
/// fields as `Value::Skipped`. Otherwise return `None`.
pub fn propagate_skipped(
    inputs: &HashMap<String, Value>,
    input_key: &str,
    output_keys: &[&str],
) -> Option<Result<HashMap<String, Value>, ExecError>> {
    if matches!(inputs.get(input_key), Some(Value::Skipped)) {
        let mut map = OutputMap::new();
        for key in output_keys {
            map = map.value(key, Value::Skipped);
        }
        Some(map.ok())
    } else {
        None
    }
}
```

Callers become:

```rust
if let Some(result) = propagate_skipped(&inputs, "response",
    &["field_a", "field_b", "field_c"]) {
    return result;
}
```

---

## 13. Error mapping boilerplate — 23 `.map_err` + 4 `.map_err(ExecError::new)` across 10 files

**Where**: `lib/review/src/lib.rs` (5), `lib/llm-ops/src/lib.rs` (4),
`core/exec/src/execute.rs` (4), `lib/primitives/src/data.rs` (3),
`gunbc-dag/src/ci/graph.rs` (3), `lib/blob/src/lib.rs` (2),
`lib/tools/deps/src/ops.rs` (2), `lib/transport/src/ops.rs` (1),
`gunbc-dag/src/ci/env.rs` (1), `gunbc-dag/src/workspace/ops.rs` (1),
`lib/tools/cargo/src/ops.rs` (1)

**Pattern**: Repeated `.map_err(|e| ExecError::new(format!("context: {}", e)))?`
chains for wrapping parse/serialize errors:

```rust
serde_json::from_str(&text)
    .map_err(|e| ExecError::new(format!("JSON parse error: {}", e)))?;
```

**Proposed**: Context-wrapping method on `ExecError`:

```rust
impl ExecError {
    /// Wrap any Display error with a context message.
    pub fn context<E: std::fmt::Display>(msg: &str, err: E) -> Self {
        ExecError::new(format!("{}: {}", msg, err))
    }
}

// Plus a Result extension trait:
pub trait ResultExt<T> {
    fn exec_context(self, msg: &str) -> Result<T, ExecError>;
}
impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn exec_context(self, msg: &str) -> Result<T, ExecError> {
        self.map_err(|e| ExecError::context(msg, e))
    }
}
```

Callers become:

```rust
serde_json::from_str(&text).exec_context("JSON parse error")?;
```

---

## 14. ShellResponse construction — 39 direct constructions across 16 files

**Where**: `core/codegen/src/registry.rs` (6),
`lib/tools/gist/tests/integration.rs` (6),
`lib/tools/gist/tests/generated_tests.rs` (6),
`gunbc-dag/src/bin/ci.rs` (3), `gunbc-dag/src/bootstrap/graph_mock.rs` (3),
`lib/git-ops/src/lib.rs` (2), `gunbc-dag/src/ci/graph_mock.rs` (2),
`lib/review/src/graph_mock.rs` (2), and 8 more files with 1 each.

**Problem**: `ShellResponse` has no convenience constructors, unlike
`FileResponse` which has `written()`, `read_ok()`, `error()`.
Every construction site writes the full struct literal:

```rust
ShellResponse {
    exit_code: 0,
    stdout: output.to_string(),
    stderr: String::new(),
}
```

Most (>80%) are success cases with empty stderr.

**Proposed**: Convenience constructors on `ShellResponse`:

```rust
impl ShellResponse {
    /// Command succeeded (exit_code 0) with given stdout.
    pub fn ok(stdout: impl Into<String>) -> Self {
        Self { exit_code: 0, stdout: stdout.into(), stderr: String::new() }
    }

    /// Command failed with given exit code and stderr.
    pub fn failed(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self { exit_code, stdout: String::new(), stderr: stderr.into() }
    }
}
```

---

## 15. Unmigrated ops still using raw HashMap — 13 sites across 7 files

**Where**: `lib/tools/cargo/src/ops.rs` (3 in `mock_outputs`),
`lib/tools/deps/src/graph.rs` (2 in mock closures),
`gunbc-dag/src/ci/graph.rs` (1), `core/exec/src/execute.rs` (1),
`core/test/src/mock.rs` (1), `core/ir/src/transport/cli.rs` (4),
`core/exec/src/helpers.rs` (1 — internal, fine)

These files still use `let mut out = HashMap::new()` instead of
`OutputMap`. Most are in secondary locations (mock impls, graph
dispatch, CLI tool infra) rather than primary `execute()` methods.

**Proposed**: Migrate to `OutputMap` for consistency.
`core/ir/src/transport/cli.rs` is special — it's in `core/ir`
which doesn't depend on `core/exec`, so it can't use `OutputMap`.
This is fine; it's infrastructure code, not op code.

---

## Tasks

### High priority (widespread, low effort) — DONE
- [x] Add `require_str`, `require_json`, `optional_str` input helpers (80 call sites) — `core/exec/src/helpers.rs`
- [x] Add `OutputMap` builder (164 call sites) — `core/exec/src/helpers.rs`
- [x] Add `TransportResponseExt::require_shell/rest/file` methods (51 call sites) — `core/exec/src/helpers.rs`
- [x] Add `ShellRequest::into_transport_request()` — `core/ir/src/transport/mod.rs`

### High priority (new — widespread, low effort)
- [ ] Add `propagate_skipped` helper (8 call sites across 3 files) — §12
- [ ] Add `ExecError::context()` + `ResultExt` trait (27 call sites across 10 files) — §13
- [ ] Add `ShellResponse::ok()` / `ShellResponse::failed()` constructors (39 sites, 16 files) — §14

### Medium priority
- [ ] Migrate remaining raw `HashMap::new()` output construction to `OutputMap` (9 sites, 5 files) — §15
- [ ] Add `assert_boundaries` test helper to `core/test` (8 files) — §11
- [ ] Extract `hash_finding_id` to `lib/primitives` as `StableHashOp` — §1
- [ ] Unify blob hash with review hash (both should use SHA256) — §1
- [ ] Extract `FormatDiffArtifact` to `lib/primitives` as `FormatMapOp` — §1

### Lower priority / blocked
- [ ] Consider `ToolGraphOp<D>` generic wrapper (dag-pattern-ux.md Phase 4) — §3
- [ ] Split `MergeOutputs` dedup from cardinality handling (blocked on engine work) — §1
- [ ] Design rendering DAG for Makefile generation (when adding Justfile) — §2
- [ ] Design rendering DAG for CI workflow generation (when adding second provider) — §2
- [ ] Review hand-written tests for redundancy with testgen (Pattern 1, 5) — §7
- [ ] Remove fragile node-count assertions from graph structure tests (Pattern 2) — §7
- [ ] Eliminate `type_id == "List"` dual encoding (see §5, incremental migration)
- [ ] Replace string-based builder references with enum keys (see §6)
- [ ] Extract `buck-out/gen` to a single constant (see §6, 16 occurrences)
- [ ] Fix CODEGEN_SOURCES hardcoded path list (see §6)
- [ ] Design hermeticity annotation for `Shell` transport (see §8 design problem)
- [ ] Add integration tests: `File` transport executor (all 6 FileOp variants) — §8
- [ ] Add integration tests: `Shell` transport executor (stdout, stderr, exit codes) — §8
- [ ] Add integration tests: Git transport (`GitRequest` variants against temp repo) — §8
- [ ] Add integration tests: CLI tool resolution (`resolve_tool_path`, `upsert_tool`) — §8
- [ ] Add `make test-integration` target (hermetic transport tests) — §8
- [ ] Add `make test-external` target (non-hermetic transport tests, scheduled CI) — §8

## Notes

- "Extract when second consumer appears" — don't prematurely abstract.
  The first consumer defines the interface, the second validates it.
- The section 5 items (input/output/response helpers) are different:
  they already have 18-24 consumers. These are safe to extract now.
- Rendering DAGs are high value but not urgent. The current functions
  are pure and testable. DAGs add composability and interceptability.
- The cardinality design doc subsumes the MergeOutputs generalization.
  Once the engine handles cardinality, all merge-like ops simplify.
- `Renderable` trait is a good foundation. Rendering DAGs would use
  ops that implement `Executable`, with `Renderable` for the final
  output formatting step.
- Hermeticity is a producer-level property, not a transport-level
  property. `Shell` is overloaded: `GitRequest` → hermetic,
  `GistRequest` → non-hermetic, both produce identical `ShellRequest`.
  Test categories must be derived from the producer type, not the
  `TransportRequest` variant. This is a design gap in the type system.
- Test retrospective: 885 tests, all in-memory. Transport executor
  (`lib/transport/src/executor.rs`) is the untested boundary.
  `File` and `Shell` integration tests are the highest-value additions.
- Testgen already subsumes most hand-written integration tests
  (Pattern 5). Focus hand-written tests on edge cases only.
- graph_mock.rs files should become data-only (MockSpec + examples).
  49 tests across 8 files are deletable once testgen Phase 8 lands.
  Watch: some library targets call `.no_boundary_tests()` — verify
  generated suite still covers those invariants before deleting.
- Makefile gen and CI gen should read `all_testgen_targets()` to
  auto-generate check targets (testgen-improvements.md TODO 6.3).
  This makes "add a new tool" a single edit instead of 3+.
- The skipped-propagation pattern (§12) is mechanical — it affects the
  parse ops that sit after transport boundaries. This will grow as
  more DAGs are added, so worth fixing now.
- The error-mapping pattern (§13) is less urgent since the current code
  works, but the `ResultExt` trait eliminates a lot of visual noise.
- The ShellResponse constructors (§14) are a quick win — `FileResponse`
  already has the same pattern (`written()`, `read_ok()`, `error()`),
  so this is consistent API completion.
- The unmigrated HashMap sites (§15) are mostly in mock/infra code,
  not primary execute paths. Lower urgency but worth cleaning up.
