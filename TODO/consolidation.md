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

## 5. Test Pattern Retrospective

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

---

## 6. Integration Test Gap Analysis

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

## Tasks

- [ ] Extract `hash_finding_id` to `lib/primitives` as `StableHashOp`
- [ ] Unify blob hash with review hash (both should use SHA256)
- [ ] Extract `FormatDiffArtifact` to `lib/primitives` as `FormatMapOp`
- [ ] Design rendering DAG for Makefile generation (when adding Justfile)
- [ ] Design rendering DAG for CI workflow generation (when adding second provider)
- [ ] Consider `ToolGraphOp<D>` generic wrapper (dag-pattern-ux.md Phase 4)
- [ ] Split `MergeOutputs` dedup from cardinality handling (blocked on engine work)
- [ ] Review hand-written tests for redundancy with testgen (Pattern 1, 5)
- [ ] Remove fragile node-count assertions from graph structure tests (Pattern 2)
- [ ] Design hermeticity annotation for `Shell` transport (see §6 design problem)
- [ ] Add integration tests: `File` transport executor (all 6 FileOp variants)
- [ ] Add integration tests: `Shell` transport executor (stdout, stderr, exit codes)
- [ ] Add integration tests: Git transport (`GitRequest` variants against temp repo)
- [ ] Add integration tests: CLI tool resolution (`resolve_tool_path`, `upsert_tool`)
- [ ] Add `make test-integration` target (hermetic transport tests)
- [ ] Add `make test-external` target (non-hermetic transport tests, scheduled CI)

## Notes

- "Extract when second consumer appears" — don't prematurely abstract.
  The first consumer defines the interface, the second validates it.
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
