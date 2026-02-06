# Handbook Compliance: Unification & Simplification

**Status**: Draft
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

---

## Track 1: Unified Registration

### Current state

| Registrable unit | Mechanism | Auto-discovered? | Dual-source? |
|-----------------|-----------|-------------------|--------------|
| TestgenTarget | `inventory` + `#[testgen_target]` | Yes | No |
| ToolDef | Manual `all_tools()` vec (7 tools, ~360 lines) | No | **Yes** (boundaries) |
| tool_target | `inventory` + `#[tool_target]` | Infra exists, **0 in use** | TBD |
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

**R1. Activate `#[tool_target]` annotations** (medium effort)

The `core/tool-registry` + `core/tool-registry-macros` crates already exist
with the right infrastructure. The validation test
(`gunbc-dag/tests/tool_registration.rs`) already checks bidirectional
consistency between `all_tools()` and `#[tool_target]` annotations.

Steps:
1. Add `#[tool_target]` to each of the 7 tool graph builders
2. Verify the validation test passes
3. Once annotations are canonical, `all_tools()` can derive from the registry
   instead of being a manual list

Files:
- `lib/tools/gist/src/graph.rs` (3 variants)
- `gunbc-dag/src/makegen/graph.rs`
- `lib/tools/deps/src/graph.rs`
- `lib/review/src/graph.rs` (if applicable)
- `gunbc-dag/src/bootstrap/graph.rs`

**R2. Eliminate boundary dual-source** (high effort, high value)

The boundary mock data should have a single authoritative source. Options:

*Option A*: MockSpec is the source of truth. CLI generator reads from MockSpec
(or a shared `BoundaryDef` extracted from it) instead of maintaining its own
`.boundary()` calls on `ToolDef`.

*Option B*: `ToolDef` carries the full boundary spec, and MockSpec derives
from it. MockSpec builder gets a `.from_tool_def(&def)` that auto-populates
boundaries.

*Option C*: Both derive from a shared `BoundarySpec` type defined alongside
the graph builder, registered via `#[tool_target]`.

Recommendation: **Option C** — aligns with the unified registration direction.
The `#[tool_target]` macro already has a `builder` attribute; extending it
with boundary metadata keeps everything co-located with the graph definition.

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
| CLI gen | **No** | **No** | Direct string building in `cli_gen.rs` |
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

**E2. Migrate Makegen to IR + renderer** (medium effort)

Makegen produces a Makefile from `ToolRegistry` + `BuildConfig`. Currently
direct string building in `gunbc-dag/src/makegen/render.rs`.

Steps:
1. Define `MakefileIR` types (target, rule, variable, phony declaration)
2. Implement `StructuredRenderer` for Makefile output
3. Validate via `make makegen --check`

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

**C1. Create transport triplet helper** (small effort, high leverage)

Add to `core/ir/src/build/` or `core/ir/src/patterns/`:

```rust
/// Stamp out a prepare → execute → parse triplet with standard wiring.
pub fn add_transport_triplet<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_op: T,
    parse_op: T,
    transport_op: T,  // usually TransportOps::Execute
    extra_inputs: &[Port],   // additional inputs beyond "request"
    extra_outputs: &[Port],  // additional outputs beyond standard set
) -> Result<TransportTriplet, BuilderError>

pub struct TransportTriplet {
    pub prepare: NodeHandle,
    pub execute: NodeHandle,
    pub parse: NodeHandle,
}
```

Standard wiring included: `prepare.request → execute.request`,
`execute.response → parse.response`, plus skip propagation.

This would reduce each CI stage from ~20 lines of node creation + edge
wiring to ~5 lines.

**C2. Replace CI lint with Clippy SubDag** (small effort)

Replace bespoke `prepare_clippy_lint → clippy_lint → parse_clippy_lint`
with the existing `build_clippy_upsert()` SubDag. This removes one full
copy of tool-ensure + run + parse logic and makes lint behave like other
tools: a self-contained SubDag with a clean interface.

Files:
- `gunbc-dag/src/ci/graph.rs` (remove lint triplet, add SubDag node)
- `gunbc-dag/src/ci/ops.rs` (remove `PrepareLint`/`ParseLint` if unused)
- `gunbc-dag/src/ci/graph_mock.rs` (update MockSpec)

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

Per `testgen-improvements.md`, patterns already safe to delete:
- Pattern A (boundary presence checks) — testgen Bucket A covers this
- Pattern C (self-chain checks) — testgen generates these
- Pattern D (resource presence) — testgen emits lease tests

Remaining: Pattern B (content/URL checks) → migrate to NodeExamples;
Pattern E (signature validation) → testgen TODO 8.2 landed.

**D2. Migrate remaining ops to `OutputMap`** (small effort)

Some ops still use raw `HashMap<String, Value>` for output construction
instead of `OutputMap`. Migrate for consistency — `OutputMap` has typed
builder methods (`.int()`, `.str()`, `.bool()`, etc.) that prevent
type mismatches.

**D3. Finish `ShellResponse` constructor migration** (small effort)

~39 direct `ShellResponse { ... }` constructions across 16 files should
use `ShellResponse::ok()` / `ShellResponse::failed()`. Reduces mock
noise and standardizes success/failure patterns.

**D4. Quarantine `"List"` type_id** (medium effort)

Cardinality is the canonical shape layer, but `"List"` is still used as
a `type_id` in some places (dual-encoding shape). Progress has been made
(CLI gen, makegen, loop patterns, deps graphs all fixed). Remaining:
finish removing `StringList`/`OptionalString` type_ids once the type
registry refactor lands. Mock generation already hard-fails on `List`/`Set`.

---

## Track 5: CI Verification Gaps

### Current state

`make verify` runs `--check` on makegen, bootstrap, testgen, and pragma.
`make test` includes `verify` in its dependency chain.

### Gaps

**V1. Clippy config verification** (small effort)

The pragma binary generates `clippy.toml`. `make pragma --check` exists
and is wired into `make verify`. Confirm this is actually in the CI
pipeline (not just the Makefile).

**V2. Lint-allowances documentation** (low priority)

Generate `lint-allowances.md` from the clippy model so approved exceptions
are always accurate. Nice-to-have, not blocking.

**V3. Transport compliance check** (small effort)

Add a CI step (or Makefile target) that verifies no new
`#[allow(clippy::disallowed_methods)]` sites appear outside the documented
exceptions list in `policy/pragma.rs`. Can be a simple grep + diff.

---

## Prioritized Action Plan

### Tier 1: Small effort, high leverage (do first)

| ID | Item | Effort | Impact |
|----|------|--------|--------|
| C1 | Transport triplet helper | S | Reduces CI graph by ~100 lines; reusable across all graph builders |
| C2 | Replace CI lint with Clippy SubDag | S | Removes one full bespoke stage; handbook explicitly calls this out |
| D1 | Delete redundant graph_mock tests (A/C/D) | S | Removes ~30 dead tests across 8 files |
| D3 | ShellResponse constructor migration | S | Consistency across 16 files |
| V3 | Transport compliance CI check | S | Prevents boundary erosion |

### Tier 2: Medium effort, high value

| ID | Item | Effort | Impact |
|----|------|--------|--------|
| R1 | Activate `#[tool_target]` annotations | M | Completes auto-discovery for tools; unblocks R2 |
| E1 | CLI gen → IR + renderer | M | Proves unified emission pattern; removes one "direct string" system |
| C3 | LoopBuilder for gist/deps iteration | M | Replaces shell `sh -c` hacks with structural patterns |
| D2 | OutputMap consistency pass | S-M | Single output-building idiom |

### Tier 3: High effort, strategic

| ID | Item | Effort | Impact |
|----|------|--------|--------|
| R2 | Eliminate boundary dual-source | H | Fixes the drift-prone ToolDef/MockSpec boundary split |
| E2 | Makegen → IR + renderer | M | Second emission system unified |
| D4 | Quarantine "List" type_id | M | Blocked on type registry refactor |

### Not prioritized (defer)

| ID | Item | Why defer |
|----|------|-----------|
| R3 | ResourceDef registration | Only 2 defs exist |
| E3 | Terminal emission | Ephemeral output, not a generated artifact |
| V2 | Lint-allowances doc | Nice-to-have |

---

## Cross-references

- `TODO/testgen-improvements.md` — Phases 8-9 (graph_mock cleanup, DagSpec)
- `TODO/consolidation.md` — Generic ops, rendering DAGs
- `TODO/refactor-pressure.md` — Structural gap prevention
- `TODO/design-codegen-quality.md` — IR completeness
- `TODO_hacks` — Dual encoding, ShellResponse, compound shell commands
- `TODO/design-unified-resource-model.md` — Resource phases 6-7

---

## Architectural exceptions (documented, not gaps)

**Bootstrap codegen** intentionally violates the transport pattern (circular
dependency). Uses direct `fs`/`process` by design. Future: could be expressed
as a DAG executed by a minimal bootstrap executor, but not blocking.

**CI context** is external to the DAG (passed as a side-channel to the executor
for output formatting). If a node ever needs to branch on "am I in CI?", this
would need structural modeling. Currently not a problem.
