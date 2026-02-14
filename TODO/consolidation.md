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

## 5. `type_id == "List"` dual encoding — DONE

**Resolution**: The dual encoding has been eliminated from domain code:

- **Loop pattern**: Uses element type + cardinality (not `"List"` as type_id)
- **CLI**: `cardinality.allows_many()` determines repeatability
- **Testgen**: Defensive guards reject `type_id == "List"` as invalid
- **Type registry**: `"List"` parsing in type DAGs is infrastructure for
  wrapper kinds, not port type_ids — this is correct and intentional

The canonical model (port type = element type + cardinality) is now
enforced throughout the codebase.

---

## 6. Codebase fragility (non-testgen)

### Builder functions referenced as strings — DONE

**Resolution**: The `#[tool_target]` macro validates builder function
references at compile time. String-based `ToolDef::new()` is replaced
by the inventory-based auto-discovery system. Renames now produce
compile errors.

### `buck-out/gen` hardcoded — Partially resolved

**Resolution**: Output directory constants centralized in
`core/ir/src/lib.rs:79-82`. Remaining occurrences are in `Cargo.toml`
`[[bin]]` paths (can't use constants) and test fixtures (acceptable).

### Static CODEGEN_SOURCES path list — DONE

**Resolution**: Removed from Makefile generator. Codegen freshness now
uses `compute_codegen_input_hash()` with `CODEGEN_GLOB_PATTERNS` and
`CODEGEN_EXTRA_FILES` constants in `core/infra/src/codegen_hash.rs`,
which discovers actual inputs rather than relying on a static list.

---

## 7. Test Pattern Retrospective

**885 manually written tests surveyed** across the codebase (plus
2,334 generated tests from testgen).
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

### Pattern 2: Graph Structure Tests (static DAG properties) — DONE

Fragile node-count assertions eliminated from domain code (build/ci/
codegen/gist/deps/review/clippy). Testgen's Bucket A covers boundary
detection and transport interception. Remaining structural tests use
property-based checks rather than hard-coded counts.

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

**Where**: `lib/tools/gist/tests/integration.rs`.

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

### Pattern 6: graph_mock.rs Test Blocks — DONE

All hand-written tests eliminated from graph_mock.rs files. 0 tests
remain across 13 files — all are now data-only (MockSpec definitions +
NodeExample data). Testgen generates all boundary, content validation,
and structural tests.

---

## 8. Integration Test Gap Analysis

**Current state**: 885 hand-written + 2,334 generated in-memory tests.
46 integration tests added for File/Shell/Git/CLI transport execution
(see `lib/transport/src/executor.rs` and `lib/transport/src/cli.rs`).
External (non-hermetic) transport tests remain a gap.

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
| `RestRequest` (GitHub) | `Rest` | Requires `Credential`, hits `api.github.com` |
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

## 9. Executable boilerplate across ops — DONE

All three boilerplate patterns are fully migrated. Helpers live in
`core/exec/src/helpers.rs` and are re-exported from `gunbc_exec`.

### Input extraction — DONE

Helpers: `require_str`, `require_json`, `require_bool`, `require_int`,
`require_str_list`, `require_map_str_str`, `require_value`, `require_request`,
`require_response`, `optional_str`, `optional_json`, `optional_bool`,
`optional_int`, `optional_str_list`, `optional_map_str_str` (plus `_strict`
variants).

Remaining `inputs.get(...)` calls are intentional:
- Semantic pattern matching (Skipped/variant checks in blob, review, llm-ops, ci/ops)
- Optional presence checks with complex logic (transport/ops, pattern_op BranchMerge)
- Code that can't depend on `core/exec` (core/ir/transport/cli.rs)
- Test/mock code, doc comments, generated code strings

### Output map construction — DONE

`OutputMap` builder used by all production `execute()` methods.
Remaining `HashMap::new()` sites: see §15.

### Response type matching — DONE

`require_shell()`, `require_rest()`, `require_file()` on `TransportResponse`.
All production parse ops migrated.

---

## 10. ShellRequest construction — DONE

`ShellRequest::new()` builder with `.arg()`, `.args()`, `.cwd()`,
`.stdin()`, `.env()`, `.into_transport_request()` in `core/ir/src/transport/mod.rs`.
All struct literal sites migrated: blob (2), gist graph (5 production + 4 mock),
deps ops (3 production + 3 mock), primitives/io (3), codegen ops (2),
bootstrap ops (1 production + 1 mock), makegen registry (1), cargo.rs (1),
transport/ops test (1).

---

## 11. MockSpec test boilerplate — DONE

All 13 graph_mock.rs files are now data-only (MockSpec definitions +
NodeExample data). Zero `#[test]` functions remain. The solution evolved
beyond the proposed parameterized helper into a typed mock builder
pattern (`extract_mock_requirements()`) that enforces correctness at
construction time. Testgen generates all boundary and structural tests.

---

## 12. Skipped-value propagation boilerplate — DONE

`propagate_skipped()` helper in `core/exec/src/helpers.rs`, re-exported
from `gunbc_exec`. All call sites migrated across ci/ops, bootstrap/ops,
llm-ops, git-ops, review, gist, deps, markdown, codegen, pattern_op.

---

## 13. Error mapping boilerplate — DONE

`IntoExecResult::exec_context()` and `ResultExt::context()` in
`core/exec/src/error.rs`. All `.map_err(|e| ExecError::new(format!(...)))`
sites migrated. Two remaining `.map_err(|e| ExecError::new(e.to_string()))`
sites (credential.rs, execute.rs) are context-free error conversions — not
worth adding artificial context messages.

---

## 14. ShellResponse construction — DONE

`ShellResponse::ok()` and `ShellResponse::failed()` constructors in
`core/ir/src/transport/mod.rs`. All production code migrated. Remaining
struct literals are in generated test files (regenerated by codegen),
`lib/transport/src/executor.rs` (needs both stdout+stderr from real
process), and `lib/tools/gist/tests/` (test code).

---

## 15. Unmigrated ops still using raw HashMap — DONE

All production `execute()` methods use `OutputMap`. Remaining
`HashMap::new()` sites are in non-migratable locations:
- `core/ir/src/transport/cli.rs` (4) — can't depend on `core/exec`
- `core/exec/src/execute.rs` (test code)
- `core/test/src/mock.rs` (test code)
- `core/exec/src/helpers.rs` (internal impl, doc comment)

These are intentional and not worth migrating.

---

## 16. Extension features (migrated from architecture-debt.md Phase D)

Architecture debt Phases A–C are complete (moved to TODONE). These
remaining items are feature work that builds on the infra extraction.

| Feature | Depends On | Priority |
|---------|-----------|----------|
| Codegen content-hash manifest | Infra extraction | **High** |
| deps.toml tracking | Infra extraction | High |
| Makefile tracking | Infra extraction | Medium |
| .gitignore tracking | Infra extraction | Medium |
| Per-tool test tracking | Performance fixes | Low |
| ToolHandle unification | Design fixes | Low |

**Codegen content-hash manifest**: ~~The current freshness check relies on
glob patterns to discover inputs.~~ **DONE**: `ManifestEntry.input_files`
now records actual file paths hashed during codegen/testgen. The manifest
stores the complete input set for diagnostics and debugging.

**RUSTC_VERSION**: ~~Hash computation includes `RUSTC_VERSION` from env,
defaulting to "unknown".~~ **DONE**: Replaced `InputPattern::Env("RUSTC_VERSION")`
with `InputPattern::CommandOutput("rustc", ["--version"])`. The actual
compiler version is now captured directly, regardless of environment setup.

**Root artifact tracking**: **DONE** for lint freshness tracking.
`list_tracked_files()` now includes `deps.toml`, `Makefile`, and `.gitignore`
so preflight manifest freshness and CI lint-upsert checks invalidate when these
repo-root artifacts change.

---

## Tasks

**Completed tasks moved to**: `TODO/TODONE/consolidation-complete.md`

### Completed (2026-02-07 cleanup)

- [x] Deleted stale `buck-out/` directory (9 generated CLI entrypoints from removed Buck2 build)
- [x] Deleted unused backward-compat shims: `TestgenTarget` type alias, `iter_targets()` fn, `RustRenderer` type alias
- [x] Eliminated `test_ir.rs` re-export shim — `codegen.rs` now imports directly from `gunbc_ir::code_ir`
- [x] Removed fragile node-count assertions from tool graph tests (build/ci/codegen/gist/deps/review/clippy)

### Remaining (blocked on design/dependencies)

- [ ] Consider `ToolGraphOp<D>` generic wrapper (dag-pattern-ux.md Phase 4) — §3
- [ ] Split `MergeOutputs` dedup from cardinality handling (blocked on engine work) — §1
- [ ] Design rendering DAG for Makefile generation (when adding Justfile) — §2
- [ ] Design rendering DAG for CI workflow generation (when adding second provider) — §2
- [ ] Review hand-written tests for redundancy with testgen (Pattern 1, 5) — §7
- [x] Remove fragile node-count assertions from graph structure tests (Pattern 2) — §7
- [x] Eliminate graph_mock.rs hand-written tests (Pattern 6) — §7
- [x] Resolve `"List"` dual encoding — §5
- [x] Builder strings compile-time validation (`#[tool_target]`) — §6.1
- [x] Remove static CODEGEN_SOURCES path list — §6.3
- [ ] Design hermeticity annotation for `Shell` transport (see §8 design problem)
- [ ] Design DAG typing hardening plan (typed node I/O wrappers + input_mock type validation + semantic carrier refinements) — see `TODO/TODO_hacks.md` §10

### Remaining (extension features — from architecture-debt.md §16)

- [x] Codegen content-hash manifest (record actual inputs, not glob approximation) — §16
- [x] deps.toml tracking — §16
- [x] Makefile tracking — §16
- [x] .gitignore tracking — §16
- [x] RUSTC_VERSION as modeled resource — §16

### Remaining (new functionality — integration tests)

- [ ] Add `make test-integration` target (hermetic transport tests) — deferred: tests run as part of normal `cargo test`
- [ ] Add `make test-external` target (non-hermetic transport tests, scheduled CI) — deferred: no external tests yet

**Completed** (moved to `TODONE/consolidation-complete.md`): File/Shell/Git/CLI
integration tests all added.

## Notes

- "Extract when second consumer appears" — don't prematurely abstract.
  The first consumer defines the interface, the second validates it.
- The §9-15 items (input/output/response helpers) had 18-24 consumers
  each. All extracted and migrated (DONE).
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
- Test retrospective: 885 hand-written + 2,334 generated tests.
  Transport executor (`lib/transport/src/executor.rs`) now has
  46 integration tests (File/Shell/Git/CLI).
- Testgen already subsumes most hand-written integration tests
  (Pattern 5). Focus hand-written tests on edge cases only.
- graph_mock.rs files are now data-only (MockSpec + examples).
  0 tests remain across 13 files. All boundary, content, and structural
  tests are generated by testgen.
- Makefile gen and CI gen should read the testgen registry (inventory)
  to auto-generate check targets (TODO/TODONE/testgen-improvements.md TODO 6.3).
  This keeps "add a new tool" to a single edit (`#[testgen_target]`).
- §9-15 (boilerplate consolidation): All DONE. `propagate_skipped()`,
  `OutputMap`, `ResultExt`, `ShellResponse::ok/failed`, input extraction
  helpers — all migrated. Remaining unmigrated sites are intentional.

---

## 17. Inefficient / Hacky Code Patterns (2026-02-12 scan)

Full codebase scan for needless conversions, unnecessary allocations,
suboptimal idioms, and correctness risks. Findings organized by severity.

### MAJOR — Probable Bug

#### 17.1 Swapped timeout fields in TCP executor

**File:** `lib/transport/src/executor.rs` ~lines 343-352

`connect_timeout_ms` is used for `set_read_timeout`, and `read_timeout_ms`
is used for `set_write_timeout`. These are swapped. If `connect_timeout_ms`
is short (5s) and `read_timeout_ms` is long (30s), reads will time out
prematurely and writes will have an unexpectedly long timeout.

**Fix:** Swap the assignments.

### MODERATE — Correctness Risk or Significant Readability

#### 17.2 `.expect()` in production graph-building code (~70 sites)

**Files:**
- `gunbc-dag/src/ci/graph.rs` (~8 `.expect()` calls on builder methods)
- `gunbc-dag/src/workspace/subdags/bootstrap.rs` (~18 `.expect()` calls)
- `gunbc-dag/src/workspace/subdags/deps.rs` (~25 `.expect()` calls)
- `core/exec/src/topo.rs` (3 `.unwrap()` on HashMap lookups in topo sort)

All these functions already return `Result`. If a malformed DAG or
renamed node causes a lookup failure, the process panics instead of
returning a diagnostic error.

**Fix:** Replace `.expect()` / `.unwrap()` with `.ok_or_else(|| ...)? `.

#### 17.3 `.is_none()` then `.unwrap()` instead of `if let` / `ok_or`

**File:** `core/ir/src/builder.rs` lines 494-513

```rust
let from_port = from_node.and_then(...);
if from_port.is_none() { return Err(...); }
let from_port = from_port.unwrap();
```

Repeated for both `from_port` and `to_port`.

**Fix:** `let from_port = from_port.ok_or_else(|| BuilderError::...)?;`

#### 17.4 Stringly-typed `type_id: String` for CLI param types

**File:** `core/cli/src/lib.rs` line 17 and `core/codegen/src/cli_gen.rs`

`type_id` stores `"Bool"`, `"Int"`, `"String"` as bare strings matched
via `match param.type_id.as_str()`. A typo silently falls through to
the `_` arm.

**Fix:** Define `enum ParamType { String, Int, Bool }`.

#### 17.5 O(n*m) set operations using `Vec::contains()`

**Files:**
- `lib/primitives/src/collection.rs` — `SetOp` uses Vec `.contains()`
  for intersection/difference
- `core/ir/src/value.rs` lines 216-221 — symmetric difference with
  linear containment checks

**Fix:** Convert one side to `HashSet` for O(1) lookups.

#### 17.6 `panic!()` in match arms instead of `Err()` return

**File:** `lib/transport/src/cli.rs` lines 144-157

Several match arms in `Executable` impl for `CliToolOp` use `panic!()`
for unexpected variants. The function returns `Result`.

**Fix:** Return `Err(ExecError::new(...))`.

#### 17.7 `&PathBuf` instead of `&Path` in function signatures

**Files:**
- `lib/transport/src/cli.rs` line 472
- `lib/cloud-ops/src/config_resource.rs` line 53

**Fix:** Change to `&Path` / return `&Path`.

### MODERATE — Systematic Performance

#### 17.8 `push_str(&format!(...))` pattern (~100+ sites)

This is the single most pervasive anti-pattern. Each call allocates a
temporary `String` via `format!()`, borrows it, appends to the buffer,
then drops it. Using `write!()` on `String` (which implements
`fmt::Write`) writes directly into the buffer.

**Affected files (non-exhaustive):**
- `core/ir/src/makefile_render.rs`
- `core/ir/src/plain_render.rs`
- `core/ir/src/dag.rs` (Mermaid rendering)
- `core/codegen/src/cli_gen.rs` (~20 sites)
- `core/codegen/src/file_writer.rs`
- `core/codegen/src/testgen/render_rust.rs` (~30 sites)
- `core/ir/src/transport/github_actions.rs`
- `core/ir/src/transport/ci/providers/github.rs`
- `core/ir/src/transport/ci/providers/gitlab.rs`
- `core/ir/src/transport/github/cli.rs`
- `gunbc-dag/src/build/ops.rs`
- `gunbc-dag/src/makegen/render.rs`
- `gunbc-dag/src/docgen/ops.rs`
- `lib/gcp-ops/src/discovery_ops.rs`
- `lib/markdown/src/lib.rs`
- `lib/tools/clippy/src/config.rs`

**Fix:** `use std::fmt::Write; write!(buf, "...", args).unwrap();`

#### 17.9 `Vec<String>` from string literals (~80 allocations)

**Files:**
- `gunbc-dag/src/makegen/gitignore.rs` (~55 `.to_string()` on literals)
- `gunbc-dag/src/makegen/render.rs` (~40 `.to_string()` on literals)
- `gunbc-dag/src/docgen/ops.rs` (~25 `.to_string()` on literals)

All build `Category`, `Target`, or line vectors from `&'static str`.

**Fix:** Change struct fields to `Cow<'static, str>`. All literal
usages become zero-cost `Cow::Borrowed`. This is the highest-impact
single change for allocation reduction.

#### 17.10 `&format!(...)` passed to `fn new(&str)` (double allocation)

**File:** `lib/blob/src/lib.rs`

`BlobHandleError::new(&str)` takes a `&str` and calls `.to_string()`
internally. Callers frequently pass `&format!(...)`, creating a `String`,
borrowing it, then allocating again inside `new()`.

**Fix:** Accept `impl Into<String>` so callers can pass `format!(...)`
directly.

### MINOR — Style Nits

#### 17.11 `.to_string_lossy().to_string()` double allocation

**File:** `gunbc-dag/src/bin/codegen_cli.rs` lines 174-176, 210, 236

**Fix:** Use `.to_string_lossy().into_owned()` (one allocation).

#### 17.12 `sort()` + `dedup()` instead of `BTreeSet`

**Files:**
- `core/ir/src/types.rs` lines 165-166, 197-198
- `core/codegen/src/registry.rs` lines 301-302
- `core/codegen/src/testgen/cardinality.rs` lines 33-34
- `lib/gcp-ops/src/discovery_ops.rs` lines 603-616

Small collections; stylistic improvement only.

#### 17.13 O(n^2) stage dedup in GitLab CI rendering

**File:** `core/ir/src/transport/ci/providers/gitlab.rs` lines 278-289

Uses `Vec::contains()` in a loop to dedup stages.

**Fix:** Use `IndexSet` or parallel `HashSet`.

#### 17.14 Dead statement: result computed and discarded

**File:** `lib/gcp-ops/src/ops.rs` lines 465-469

`let _ = rest.body.get("expireTime")...` computes a value discarded
by `let _`.

**Fix:** Remove the dead statement or wire the value into output.

#### 17.15 Collect into Vec for indexed char access

**File:** `core/ir/src/language/mod.rs` lines 199-200

Collects `.chars()` into `Vec<char>` for `chars[i-1]` lookback.

**Fix:** Use `.char_indices()` with a `prev_char` variable.

#### 17.16 Hardcoded `/root` fallback for `$HOME`

**File:** `lib/gcp-ops/src/ops.rs` line 740

`std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())`.

**Fix:** Use `dirs::home_dir()` or document the assumption.

#### 17.17 Code duplication: `execute_run` / `execute_run_with_path`

**File:** `lib/transport/src/cli.rs` lines 436-501

~30 lines duplicated; only differs in path resolution.

**Fix:** Extract shared logic into a helper.

#### 17.18 Code duplication: `MapToGcpInputs` / `MapToGcpSecretInputs`

**File:** `lib/cloud-ops/src/ops.rs` lines 78-269

Large overlap in field extraction logic.

**Fix:** Extract shared helper, each variant calls it with its extras.

#### 17.19 Lossy placeholder mapping in gist subdag

**File:** `gunbc-dag/src/workspace/subdags/gist.rs` lines 17-37

12 distinct `GistGraphOp` variants mapped to a single placeholder
`WorkspaceOp::Gist(GistOps::ParseGistResponse)`. If any of these
nodes execute in workspace context, they invoke the wrong operation.

**Fix:** Add corresponding variants to `WorkspaceOp` or embed
`GistGraphOp` as a variant.

### Summary Table

| Severity | Count | Key items |
|----------|-------|-----------|
| **Major** | 1 | Swapped TCP timeout fields (§17.1) |
| **Moderate (correctness)** | 6 | `.expect()` in prod (§17.2), `.is_none()`+`.unwrap()` (§17.3), stringly-typed param (§17.4), O(n*m) sets (§17.5), `panic!()` in Result fn (§17.6), `&PathBuf` (§17.7) |
| **Moderate (systematic)** | 3 | `push_str(&format!())` ~100 sites (§17.8), `Cow` for string literals ~80 allocs (§17.9), double-alloc error ctor (§17.10) |
| **Minor** | 9 | Various style nits (§17.11–17.19) |

### Priority Order

1. **§17.1** — Fix swapped timeout fields (bug, 1 line)
2. **§17.6** — Replace `panic!()` with `Err()` in Result-returning fn
3. **§17.2** — Replace `.expect()` with `?` in graph builders (systematic, ~70 sites)
4. **§17.3** — Replace `.is_none()`+`.unwrap()` with `ok_or_else`
5. **§17.4** — Introduce `ParamType` enum
6. **§17.8** — `push_str(&format!())` → `write!()` (mechanical, ~100 sites)
7. **§17.9** — `Cow<'static, str>` for struct fields with literal values
8. **§17.5** — `HashSet` for set operations
9. Rest — minor items, address opportunistically

### 2026-02-14 Modeling Consolidation Backlog

#### A. Probe-observer analysis/lowering single-source bundle

**Where:** `core/codegen/src/testgen/codegen.rs`

Header coverage reporting and probe-observer test section still compute
overlapping lowering/analysis on separate paths.

**Target:** compute once (lowered DAG + probe-observer analysis + report) and
reuse everywhere.

#### B. Seed policy ownership in IR types (not testgen whitelist)

**Where:** `core/codegen/src/testgen/codegen.rs`, `core/ir/src/types.rs`

Seed safety policy is still a testgen-local string whitelist.

**Target:** move policy classification to IR type model and query from testgen.

#### C. Live-secret requirements as generated workflow metadata

**Where:** testgen metadata + CI workflow env wiring

Required-secret declarations can drift between metadata and workflow exports.

**Target:** single required-secret model that generates CI env wiring and
runtime/test gating.

#### D. Execution trace inputs for coercion/assertion observability

**Where:** `core/exec/src/execute.rs` (`LogEntry`)

Logs record outputs but not inputs, limiting coercion/shape assertions.

**Target:** add opt-in input capture mode for test/verify contexts.
