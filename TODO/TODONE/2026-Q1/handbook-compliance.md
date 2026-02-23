# Handbook Compliance: Unification & Simplification

**Status**: Active (Tier 1 complete, Tier 2 triaged)
**Date**: 2026-02-06

Consolidates two rounds of feedback on the gunbc Handbook. Identifies
missing integrations, redundant patterns, and a prioritized action plan.

The handbook positions itself as the practical "how the codebase works" map,
with two key unification tracks: **unified emission** and **unified registration**.
This doc maps what's done, what's partial, and what's next.

---

## What's already integrated

These are working, enforced, and don't need further action.

| Area | Status | Evidence |
|------|--------|----------|
| Transport boundary | Enforced | `TransportOps::Execute` is the only I/O path; clippy.toml bans `std::fs`/`std::process`; transport internals not exported |
| Testgen registration | Fully automated | `#[testgen_target]` + inventory; 12 targets registered; enforcement test in `mock_spec_registration.rs` |
| BuildConfig as SSOT | Working | `BuildConfig::cargo()` is the single source for all build/test/lint commands; makegen derives from it |
| Pure ops + MockSpec enforcement | Working | Testgen panics if transport nodes lack mocks or pure nodes lack examples (unless skipped) |
| DAG-ify all binaries | Complete (7/7) | codegen, bootstrap, build, makegen, ci, pragma, testgen all use DAG execution |
| Resource acquisition (Phases 1-5) | Complete | Resource trait, env nodes, `res:` port convention, SubDag delegation, resource accounting |
| `#[tool_target]` annotations (R1) | Complete (7/7) | All 7 tool graph builders annotated; `tool_registration.rs` validates bidirectional consistency |
| OutputMap consistency (D2) | Complete | No raw `HashMap<String, Value>` returns found in any op implementation |
| Transport compliance (V3) | Complete | `pragma_lint.rs` test enforces disallowed-methods allowlist; no untracked exceptions possible |
| Transport triplet helper (C1) | Complete | `add_skippable_transport_triplet` / `add_transport_triplet` in `core/ir/src/patterns/transport_triplet.rs`; CI graph reduced from ~1011 to ~830 lines |
| graph_mock Pattern E dedup (D1) | Complete | `assert_typed_builder_rejects_invalid_slot` shared in `core/test/src/lib.rs`; 7 graph_mock.rs files use it |
| ShellResponse constructors (D3) | Complete | All 26 test-site `ShellResponse { exit_code: 0, .. }` migrated to `ShellResponse::ok()` in `integration.rs` |

---

## Track 1: Unified Registration

### Current state

| Registrable unit | Mechanism | Auto-discovered? | Dual-source? |
|-----------------|-----------|-------------------|--------------|
| TestgenTarget | `inventory` + `#[testgen_target]` | Yes | No |
| ToolDef | Manual `all_tools()` vec (7 tools, ~360 lines) | No | **Yes** (boundaries) |
| tool_target | `inventory` + `#[tool_target]` | Yes (7/7 annotated) | No |
| ResourceDef | Manual helper functions (2) | No | No |

### Problem: ToolDef boundaries are dual-sourced

The same boundary mock data lives in two places that must stay in sync:

**Source 1 — `ToolDef.boundary()` in `core/codegen/src/registry.rs`:**
Used by CLI generator to emit DryRun mock setup code.
```rust
.boundary("fs_env", vec![("fs:write", "FilesystemHandle::cross_platform(...).into()")])
```

**Source 2 — `MockSpec.boundary()` in each `graph_mock.rs`:**
Used by testgen for generating tests.
```rust
.boundary("fs_env", "fs:write", mock_fs_handle())
```

Adding/removing a boundary requires updating both places. Mismatches cause
runtime failures with no compile-time detection.

### Action items

**R1. Activate `#[tool_target]` annotations** ~~(medium effort)~~ **DONE**

All 7 tool graph builders have `#[tool_target]` annotations. The validation
test (`gunbc-dag/tests/tool_registration.rs`) verifies bidirectional
consistency and testgen coverage. Next step: derive `all_tools()` from the
registry to eliminate the manual list.

**R2. Eliminate boundary dual-source** ~~(high effort, high value)~~ **DONE (mock_spec_call inlined)**

`mock_spec_call` was dual-sourced: `mock_spec_for()` lookup in `registry.rs`
and `#[tool_target]` annotations. Now each `ToolDef` sets `.mock_spec_call()`
inline — the `mock_spec_for()` function and auto-populate loop are deleted.
The validation test in `tool_registration.rs` still ensures ToolDef and
`#[tool_target]` stay in sync. Full boundary unification (Option C above)
remains a future opportunity but the immediate dual-source is eliminated.

**R3. ResourceDef registration** (low priority)

Only 2 resource defs exist (`codegen_resource_def`, `testgen_resource_def`).
Not worth automating until there are more. Track but defer.

---

## Track 2: Unified Emission

### Current state

| System | Has IR? | Has Renderer trait? | Notes |
|--------|---------|-------------------|-------|
| Testgen | Yes (`test_ir.rs`) | Yes (`TestRenderer`) | Gold standard |
| CI YAML | Yes | Yes (`CiRenderer`) | Working |
| Makegen | **No** | **No** | Direct string building in `render.rs` |
| CLI gen | Yes (`code_ir.rs`) | Yes (`RustCodeRenderer`) | Uses `Item::Use(Import)` + `Item::Fn(FnDef)` + `Expr::RawCode` |
| Terminal | **No** | **No** | Direct string building |

The handbook notes: *"Five rendering systems, four different traits, two with
no IR at all."*

A unified IR + renderer trait hierarchy already exists in
`core/ir/src/render_ir.rs` with `OutputMedium`, `CodeRenderer`,
`MarkupRenderer`, `StructuredRenderer`, `FrameRenderer`, `DocumentRenderer`.
But Makegen and CLI gen don't use it.

### Action items

**E1. Migrate CLI gen to IR + renderer** (medium effort, good proof-of-concept)

CLI gen produces Rust source files (generated `main.rs` entrypoints). It
already uses structured data (`ToolDef`, `CliEntrypoint`) but renders via
string concatenation in `cli_gen.rs`.

Steps:
1. Model CLI output as `test_ir` types (or extend with `CliFile` type) —
   imports, fn signatures, match arms, mock setup blocks
2. Render via `CodeRenderer<PlainText>` (Rust output)
3. Validate output stability via snapshot/golden file tests

**E2. Migrate Makegen to IR + renderer** ~~(medium effort)~~ **DONE**

All targets use `StructuredBlock::Target`. The 4 remaining `Raw` blocks
are correctly non-target content (header comments, variable definitions,
phony declarations). A test validates that all target blocks use the
structured representation.

**E3. Terminal emission** (low priority)

Terminal output uses `TerminalProfile` for progress display. Less urgent
because it's not a generated artifact — it's ephemeral runtime output.
Defer until E1/E2 prove the pattern.

---

## Track 3: CI Graph Simplification

### Problem: Repeated prepare-execute-parse scaffolding

The CI graph (`gunbc-dag/src/ci/graph.rs`, 1011 lines) has 6 transport
triplets that follow the same pattern:

```
prepare_X → execute_X (TransportOps::Execute) → parse_X
```

Each triplet manually:
- Creates 3 nodes with similar port shapes (`request`, `response`, `skip`, `skip_reason`)
- Wires `prepare.request → execute.request`, `execute.response → parse.response`
- Propagates skip/skip_reason through the chain

This pattern also appears in `lib/tools/deps/src/graph.rs` (multiple triplets)
and `gunbc-dag/src/workspace/subdags/bootstrap.rs`.

No helper exists today — each graph builder stamps these out manually.

### Problem: CI lint stage is bespoke (should use Clippy SubDag)

The CI graph has a custom lint stage:
```
prepare_clippy_lint → clippy_lint (CliToolOp) → parse_clippy_lint
```

But a reusable `build_clippy_upsert()` SubDag already exists in
`lib/tools/clippy/src/graph.rs` (delegates to generic `build_cli_upsert`
in `core/ir/src/transport/cli.rs`). The handbook explicitly calls this out
as a consolidation opportunity.

### Action items

**C1. Create transport triplet helper** ~~(small effort, high leverage)~~ **DONE**

Implemented in `core/ir/src/patterns/transport_triplet.rs`:
- `add_skippable_transport_triplet()` — 5 skippable triplets (testgen, build, test, guardrail, verify)
- `add_transport_triplet()` — 1 non-skippable triplet (deps_exists)
- CI graph reduced from ~1011 to ~830 lines
- Generic `DagBuilder<T>` — reusable across all graph builders

**C2. Remove CI EnvOp (self-acquiring lint)** ~~(small effort)~~ **DONE**

EnvOp removed; `CliToolOp` self-acquires via `upsert_tool_with()` before
running. The `runner_env` node, `Env(EnvOp)` variant, and `env.rs` module
are deleted. CI graph has -1 node, -1 edge. The defensive check/install
behavior is preserved — it just happens inside the `CliTool` execution
handler instead of a separate env node.

**C3. LoopBuilder for repeated iteration patterns** (medium effort)

Two places manually iterate where `LoopBuilder` (already in
`core/ir/src/patterns/loop_pattern.rs`, 339 lines) would be appropriate:

- `lib/tools/gist/src/graph.rs` — batch file read via `sh -c` with
  `;`-joined `cat` commands → LoopBuilder over file paths
- `lib/tools/deps/src/graph.rs` — manual `install_plan` iteration →
  LoopBuilder of UpsertBuilder

These reduce custom control flow and increase pattern-level analyzability.

---

## Track 4: Redundancy Cleanup

Small, mechanical items that each reduce noise and improve consistency.

**D1. Make `graph_mock.rs` data-only** (in progress via testgen Phase 8)

Per `TODO/TODONE/testgen-improvements.md`, patterns already safe to delete:
- Pattern A (boundary presence checks) — testgen Bucket A covers this
- Pattern C (self-chain checks) — testgen generates these
- Pattern D (resource presence) — testgen emits lease tests

Remaining: Pattern B (content/URL checks) → migrate to NodeExamples;
Pattern E (signature validation) → **DONE** — shared
`assert_typed_builder_rejects_invalid_slot` in `core/test/src/lib.rs`,
called from 7 `graph_mock.rs` files.

**D2. Migrate remaining ops to `OutputMap`** ~~(small effort)~~ **ALREADY DONE**

All ops use `OutputMap` — no raw `HashMap<String, Value>` returns found.

**D3. Finish `ShellResponse` constructor migration** ~~(small effort)~~ **DONE**

26 test-site constructions migrated to `ShellResponse::ok()` in
`lib/tools/gist/tests/integration.rs`. The 1 production site in
`lib/transport/src/executor.rs` uses all three fields (exit_code,
stdout, stderr from runtime values) — not a candidate for `ok()`/`failed()`.
6 sites in `buck-out/gen/` are generated code — will be fixed when
codegen regenerates.

**D4. Quarantine `"List"` type_id** ~~(medium effort)~~ **DONE**

No production `type_id` usage of `"List"` remains. Only defensive guards
(mock generation hard-fails on `List`/`Set`) and comments exist. The
`StringList`/`OptionalString` type_ids have been eliminated from all
active code paths.

---

## Track 5: CI Verification Gaps

### Current state

`make verify` runs `--mode=verify` on makegen, bootstrap, testgen, and pragma.
`make test` includes `verify` in its dependency chain.

### Gaps

**V1. Clippy config verification** (small effort)

The pragma binary generates `clippy.toml`. `make pragma-check`
(`gunbc-pragma --mode=verify`, deprecated `--check`) exists and is wired
into `make verify`. Confirm this is actually in the CI pipeline (not just
the Makefile).

**V2. Lint-allowances documentation** (low priority)

Generate `lint-allowances.md` from the clippy model so approved exceptions
are always accurate. Nice-to-have, not blocking.

**V3. Transport compliance check** ~~(small effort)~~ **ALREADY DONE**

The `pragma_lint.rs` test enforces the disallowed-methods allowlist.
Any new `#[allow(clippy::disallowed_methods)]` site not in the allowlist
causes a test failure.

---

## Prioritized Action Plan

### Tier 1: Small effort, high leverage — **ALL DONE**

| ID | Item | Status |
|----|------|--------|
| C1 | Transport triplet helper | **DONE** — `add_skippable_transport_triplet` / `add_transport_triplet` in `core/ir/src/patterns/transport_triplet.rs` |
| D1 | graph_mock Pattern E dedup | **DONE** — shared `assert_typed_builder_rejects_invalid_slot` in gunbc-test |
| D3 | ShellResponse constructor migration | **DONE** — 26 sites migrated to `ShellResponse::ok()` |
| V3 | Transport compliance CI check | **ALREADY DONE** — `pragma_lint.rs` enforces allowlist |
| R1 | `#[tool_target]` annotations | **ALREADY DONE** — 7/7 annotated |
| D2 | OutputMap consistency pass | **ALREADY DONE** — no raw HashMap returns found |

### Tier 2: Medium effort, high value

| ID | Item | Effort | Impact | Status |
|----|------|--------|--------|--------|
| C2 | Remove CI EnvOp (self-acquiring lint) | M | Removes CI-only acquisition pattern | **DONE** — EnvOp removed; CliToolOp self-acquires via `upsert_tool_with()`; -1 node, -1 edge |
| E1 | CLI gen → IR + renderer | L | Proves unified emission pattern | **DONE** — `cli_gen.rs` uses `Item::Use(Import)` + `Item::Fn(FnDef)` + `Expr::RawCode`; `CliBoundary` removed (dead code); both standard and step modes use proper IR |
| C3 | LoopBuilder for gist/deps iteration | M | Replaces shell `sh -c` hacks | **FOUNDATION** — `Pattern(PatternOp)` variant + `From<PatternOp>` added to `GistGraphOp`; `LoopBuilder<GistGraphOp>` validated in test; actual graph wiring deferred (requires executor loop iteration) |

### Tier 3: High effort, strategic

| ID | Item | Effort | Impact |
|----|------|--------|--------|
| R2 | Eliminate boundary dual-source | H | **DONE** — `mock_spec_call` inlined on each ToolDef; `mock_spec_for()` deleted |
| E2 | Makegen → IR + renderer | M | **DONE** — All targets use `StructuredBlock::Target`; 4 remaining `Raw` blocks are correctly non-target content |
| D4 | Quarantine "List" type_id | M | **DONE** — No production `type_id` usage remains; only defensive guards and comments |

### Not prioritized (defer)

| ID | Item | Why defer |
|----|------|-----------|
| R3 | ResourceDef registration | Only 2 defs exist |
| E3 | Terminal emission | Ephemeral output, not a generated artifact |
| V2 | Lint-allowances doc | Nice-to-have |
| S1 | Enforce transport skip wiring | Missing `skip` currently defaults to false; testgen doesn't flag missing wiring. Consider `TransportOps::ExecuteSkippable` or a structural lint. |

---

## Cross-references

- `TODO/TODONE/testgen-improvements.md` — Phases 8-9 (graph_mock cleanup, DagSpec)
- `TODO/consolidation.md` — Generic ops, rendering DAGs
- `TODO/TODONE/refactor-pressure.md` — Structural gap prevention
- `TODO/design-codegen-quality.md` — IR completeness
- `TODO_hacks` — Dual encoding, ShellResponse, compound shell commands
- `TODO/TODONE/design-unified-resource-model.md` — Resource phases 6-7

---

## Architectural exceptions (documented, not gaps)

**Bootstrap codegen** intentionally violates the transport pattern (circular
dependency). Uses direct `fs`/`process` by design. Future: could be expressed
as a DAG executed by a minimal bootstrap executor, but not blocking.

**CI context** is external to the DAG (passed as a side-channel to the executor
for output formatting). If a node ever needs to branch on "am I in CI?", this
would need structural modeling. Currently not a problem.
