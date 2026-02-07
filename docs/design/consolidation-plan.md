# Consolidation Plan

> **Goal**: Close the gap between the handbook's intended patterns and the
> codebase's actual state. Every item here was surfaced by cross-reading the
> handbook against the code (February 2026 reconciliation).

Companion docs:
- `docs/design/unified-registration.md` — registration unification (detailed)
- `docs/design/unified-emission.md` — emission/rendering unification (detailed)
- `docs/design/overview.md` — core invariants and philosophy
- `docs/design/testgen.md` — test generation from proof obligations

---

## Reconciliation Summary

The review identified 16 claims. Of those:

| Status | Count | Items |
|--------|-------|-------|
| **Accurate, work needed** | 6 | Registration unification, emission unification, stringly-typed dispatch, doc consistency, lint-allowances generation, `node.requires()` gap |
| **Accurate, already addressed** | 6 | graph_mock.rs cleanup, OutputMap migration, ShellResponse constructors, resource phases 4-5, `make verify`, "List" dual-encoding |
| **Wrong or overstated** | 4 | Dual-source boundary mocks (not actually dual-sourced), Makefile/CI "not wired" (wired through all_tools), CliToolDef/ToolDef duplication (distinct purposes), resource phases incomplete (all 5 done) |

This plan covers the 6 items that need work plus related cleanup.

---

## Work Streams

### Stream 1: Registration Unification

**Problem**: `inventory` auto-discovery exists only for testgen targets. Tools,
graph builders, and resource defs use manual lists/enums that silently lose
registrations.

**Current state** (from codebase audit):

| Registry | Mechanism | Drift risk |
|----------|-----------|------------|
| Testgen targets | `inventory` + `#[testgen_target]` | None (gold standard) |
| Tool definitions | `all_tools()` hardcoded vec (~360 lines) | High — silent omission |
| Graph builders | `GraphBuilderId` enum + `as_str()` string coupling | High — runtime break |
| Resource defs | Hardcoded glob constants in `defs.rs` | Medium — stale patterns |
| Makefile targets | Auto-derived from `all_tools()` | Low — fragile root |
| CI targets | Auto-derived from `all_tools()` | Low — fragile root |

**Plan**: Implement `unified-registration.md` phases 1-5. The design is
complete; this is pure execution.

#### Phase R1: Tool Registry crate (non-breaking) — DONE

**Created** `core/tool-registry/` and `core/tool-registry-macros/`:
- `ToolRegistration` struct (mirrors `TestgenTarget` design)
- `#[tool_target]` proc macro
- `inventory::collect!(ToolRegistration)` + `iter_tool_targets()`

**Files**:
- `core/tool-registry/Cargo.toml` (new)
- `core/tool-registry/src/lib.rs` (new)
- `core/tool-registry-macros/Cargo.toml` (new)
- `core/tool-registry-macros/src/lib.rs` (new)
- Root `Cargo.toml` workspace members

**Acceptance**: crate compiles, `iter_tool_targets()` returns empty iterator,
proc macro validates required fields at compile time.

#### Phase R2: Annotate existing tools — DONE (partial)

Completed in two sub-phases:
- **R2a**: Replaced `GraphBuilderId` enum with plain strings in `ToolMeta`.
- **R2b**: Added 7 `#[tool_target]` annotations + validation test.

**Added** `#[tool_target(...)]` to each tool crate (7 annotations):
- `lib/tools/gist/src/lib.rs` (3 modes: snapshot, diff, recent)
- `lib/tools/deps/src/lib.rs`
- `lib/review/src/lib.rs`
- `gunbc-dag/src/makegen/mod.rs` (makegen tool)
- `gunbc-dag/src/bootstrap/mod.rs` (bootstrap tool)

**Not annotated** (intentionally excluded):
- `gunbc-dag/src/ci/` — CI has a handwritten main.rs and is not in `all_tools()`

**Validation test**: `gunbc-dag/tests/tool_registration.rs` checks bidirectional
consistency between `all_tools()` and `iter_tool_targets()`.

**Not yet done** (deferred to R3+):
- `all_tools()` still hardcoded (cannot delegate to `iter_tool_targets()` due to
  circular dep: `gunbc-codegen` cannot depend on tool crates)
- `GraphBuilderId` already deleted (R2a)

**Files**:
- `core/codegen/src/registry.rs` — `all_tools()` still hardcoded (R3 will unify)
- `core/codegen/src/cli_gen.rs` — `GraphBuilderId` deleted (R2a)
- All tool crates above + `gunbc-dag/tests/tool_registration.rs`

**Acceptance**: annotations match `all_tools()` metadata (enforced by test).
`GraphBuilderId` grep returns 0 matches.

#### Phase R3: Boundary unification

**Add** `mock_spec` field to `ToolRegistration` linking to the tool's MockSpec
function. CLI generator reads boundary information from MockSpec instead of
ToolDef's `.boundary()` calls.

**Remove** `.boundary()` calls from tool registrations — MockSpec is the single
source of truth for boundary mock values.

**Files**:
- `core/codegen/src/cli_gen.rs` — reads boundaries from MockSpec
- `core/codegen/src/registry.rs` — `.boundary()` method removed

**Acceptance**: generated CLIs have identical boundary handling. No `.boundary()`
calls in registration code. Single definition site for each boundary.

#### Phase R4: Resource input discovery (evaluate)

Replace hardcoded glob patterns in `core/ir/src/resource/defs.rs` and
`gunbc-dag/src/resources.rs` with crate-dependency-derived patterns.

**Option A**: `#[resource_def]` macro resolving crate names to directories via
`cargo metadata`.

**Option B**: derive input patterns from the workspace dependency graph at
build time (resource defs declare input crates, not glob strings).

**Decision needed**: evaluate if the number of resource defs (currently 2:
codegen and testgen) justifies the macro infrastructure. If only 2, a simpler
approach (derive from Cargo.toml deps) may suffice.

**Files**:
- `core/ir/src/resource/defs.rs`
- `gunbc-dag/src/resources.rs`

**Acceptance**: resource input patterns auto-update when crate directory changes.
No hardcoded glob strings.

#### Phase R5: Validation tests — DONE

Validation test `gunbc-dag/tests/tool_registration.rs`:
- [x] Every `all_tools()` entry has a matching `#[tool_target]` (and vice versa)
- [x] Metadata fields (crate_name, graph_builder_call, returns_result) match
- [x] Every `#[tool_target]` builder has `#[testgen_target]` coverage in the same crate

The "every `build_*_graph` has `#[tool_target]`" check is intentionally not
implemented — many `build_*_graph` functions are library graphs (llm-ops),
internal helpers (review_phase), or intentionally excluded tools (ci, codegen,
pragma, testgen, build). The bidirectional `all_tools() ↔ iter_tool_targets()`
check already catches the real drift case (tool added to registry without
annotation).

**Files**: `gunbc-dag/tests/tool_registration.rs`

**Acceptance**: test catches unregistered tool crates and missing testgen
coverage.

---

### Stream 2: Emission Unification

**Problem**: 13 rendering systems, 5 different traits, 8 with no trait at all.
Only testgen (TestFile + TestRenderer) and CI YAML (SharedStep + CiRenderer)
have proper IR + trait separation.

**Current state** (from codebase audit):

| System | Has IR? | Has Trait? | Location |
|--------|---------|-----------|----------|
| Testgen | TestFile | TestRenderer | `core/codegen/src/testgen/` |
| CI YAML | SharedStep | CiRenderer | `core/ir/src/transport/ci/` |
| Makegen | No | Renderable (header only) | `gunbc-dag/src/makegen/` |
| CLI gen | No | No | `core/codegen/src/cli_gen.rs` |
| DAG gen | No | No | `core/codegen/src/dag_gen.rs` |
| Terminal | No | No | `core/exec/src/render.rs` |
| Clippy config | No | Renderable | `lib/tools/clippy/src/config.rs` |
| Pragma text | No | No | `gunbc-dag/src/policy/pragma.rs` |
| CI report | No | No | `gunbc-dag/src/ci/ops.rs` |
| Markdown | No | No | `lib/markdown/src/lib.rs` |
| LLM prompts | No | No | `lib/review/src/lib.rs` |
| CI commands | No | CiProvider | `core/ir/src/transport/ci/providers/` |
| WorkflowConfig | No | Renderable | `core/ir/src/transport/github_actions.rs` |

**Plan**: Implement `unified-emission.md` phases 1-5. The design is complete;
this plan provides execution sequencing.

Phases E1-E5 are fully specified in `unified-emission.md`. Key dependencies:

```
E1 (OutputMedium + content IR) ← no deps, non-breaking
E2 (Code layer) ← E1
E3 (Structured layer) ← E1
E4 (Frame layer) ← E1
E5 (Emit pattern + CI + markup) ← E2, E3, E4
```

E2, E3, E4 are independent of each other and can proceed in parallel.

**Critical design decision for E3** (from emission doc): Makegen must separate
model/policy/presentation. Today `render_meta_target()` hardcodes PrepLevel →
deps mapping (policy) and tool targets blanket-depend on `ensure-codegen`
(policy). After E3:
- Model: typed `TargetRef` / dependency graph
- Policy: derived from per-tool declarations
- Presentation: `MakefileRenderer<M>` receives a resolved graph

**Key crate boundary constraint for E2**: Code IR types (`TestFile`, `Stmt`,
`Expr`, `Assert`) must move from `core/codegen` to `core/ir` so that
`cli_gen.rs` and `dag_gen.rs` can use them without depending on `core/codegen`.
This follows the same pattern as `ResourceId` → `core/infra`.

**Acceptance per phase**: see `unified-emission.md` Definition of Done
(16 criteria, each mechanically verifiable).

---

### Stream 3: String-Coupled Dispatch

**Problem**: `GraphBuilderId::as_str()` maps enum variants to function name
strings. If a tool renames its builder function, the string silently emits the
wrong name in generated code. The generated CLI compiles (it's a string
template) but fails at runtime.

**This is absorbed by Stream 1 Phase R2**: when `#[tool_target]` replaces
`GraphBuilderId`, the builder expression is validated at macro expansion time
(same mechanism as testgen's `builder = "..."` field). No separate work item
needed — just ensure R2 deletes `GraphBuilderId` entirely.

**Additionally**: Makefile meta-target deps are hardcoded strings
(`"testgen-check"`, `"fmt-fix"`). This is absorbed by Stream 2 Phase E3:
typed `TargetRef` replaces raw string deps.

**Acceptance**:
```
grep -r "GraphBuilderId" --include='*.rs' → 0 matches
grep -r '"testgen-check"\|"fmt-fix"\|"pragma-check"' gunbc-dag/src/makegen/ → 0 matches (in dep lists)
```

---

### Stream 4: Documentation Consistency

**Problem**: The handbook has at least two statements that don't reconcile:
- "Structural I/O Enforcement... completed. All tools migrated, escape hatch closed."
- Separate notes showing gist/deps/buck2/bootstrap still have hidden I/O in opaque nodes.

The overview doc (`docs/design/overview.md` line 923) says "Completed. All
tools migrated, escape hatch closed" — but the "Current State" table at
line 976 shows gist, deps, buck2, bootstrap, and lib/fs as NOT migrated.

**Additionally**: `node.requires(&cli::CLIPPY)` is documented in
`core/ir/src/transport/tool.rs` comments (lines 16-22) as the intended
API, but **no `.requires()` method exists on Node**. The actual pattern uses
environment nodes with `res:*` ports instead.

#### Phase D1: Reconcile transport migration status

**Choose one normative statement**. Options:

**Option A** (recommended): The overview doc's "Current State" table is correct
(gist/deps/buck2/bootstrap still have opaque I/O). Update the "Status:
Completed" line to accurately reflect this. Add a date and description of what
IS completed (escape hatch removed from `lib/transport`, clippy enforcement
active, CI migrated) vs what remains (tool-level migration).

**Option B**: If all tools have been migrated since the table was written,
update the table and add a completion date.

**Files**:
- `docs/design/overview.md` — lines 923-993

**Acceptance**: no contradictory status statements. A single "Transport
Migration" section with clear done/not-done items.

#### Phase D2: Resolve `node.requires()` documentation

The comment in `tool.rs` describes an API that doesn't exist. Either:

**Option A**: Remove the comment. The environment node + `res:*` port pattern
is the canonical approach and `.requires()` isn't needed.

**Option B**: Implement `.requires()` as sugar that adds a `res:tool:{name}`
input port. This would be a convenience wrapper, not a new mechanism.

**Files**:
- `core/ir/src/transport/tool.rs` — lines 16-22

**Acceptance**: documented API matches actual API. No aspirational comments
without `TODO` markers.

---

### Stream 5: CI Verification Gaps

**Problem**: The handbook emphasizes determinism and structural correctness,
but two verification gaps remain.

#### Phase V1: Confirm `make verify` runs in CI

`make verify` exists and runs all generators in `--mode=verify` mode:
```makefile
verify: ensure-codegen
    cargo run -p gunbc-dag --bin gunbc-makegen --release -- --mode=verify
    cargo run -p gunbc-dag --bin gunbc-bootstrap --release -- --mode=verify
    cargo run -p gunbc-dag --bin gunbc-testgen --release -- --mode=verify
    cargo run -p gunbc-dag --bin gunbc-pragma --release -- --mode=verify
```

**Verify**: this target is actually invoked in CI. If not, add it to the CI
workflow. This is the "generated artifacts are provably fresh" guarantee the
handbook promises.

**Files**: CI workflow YAML (generated), possibly `gunbc-dag/src/ci/`

**Acceptance**: CI fails when a generated file differs from its generator
output.

#### Phase V2: Clippy config CI verification

The clippy model generates `clippy.toml` (verified via `gunbc-pragma --mode=verify`
in `make verify`). This is already covered IF `make verify` runs in CI
(Phase V1).

**Missing**: `lint-allowances.md` generation from the clippy model. This
was listed as a TODO in the clippy model notes.

**Evaluate**: Is `lint-allowances.md` actually needed? The approved exceptions
are already documented in `clippy.toml` (with `reason` fields) and in
`tools/disallowed-methods-allowlist.txt`. A separate markdown file may be
redundant.

**Decision**: Skip `lint-allowances.md` unless the handbook explicitly
references it as a deliverable. The existing `clippy.toml` `reason` fields
and the allowlist file already serve this purpose.

**Acceptance**: `make verify` in CI catches clippy.toml drift (via Phase V1).

#### Phase V3: Transport compliance check

Add a CI step (or Makefile target) that verifies no `allow(clippy::disallowed_methods)`
sites exist outside the approved allowlist.

**Current enforcement**: `lib/transport/src/pragma_lint.rs` validates this
(lines 108-187). The allowlist is at `tools/disallowed-methods-allowlist.txt`.

**Verify**: this validation is invoked during `cargo test --workspace` or
`make verify`. If not, wire it in.

**Files**:
- `lib/transport/src/pragma_lint.rs` — existing validation
- `tools/disallowed-methods-allowlist.txt` — existing allowlist

**Acceptance**: adding an `allow(clippy::disallowed_methods)` outside the
allowlist fails CI.

---

### Stream 6: CliToolDef / ToolDef Alignment (Low Priority)

**Problem**: Two tool types exist with some field overlap:
- `ToolDef` (`core/ir/src/transport/tool.rs`) — platform satisfiability
- `CliToolDef` (`core/ir/src/transport/cli.rs`) — runtime acquisition

Shared fields: `id`, install-related fields.

**Assessment**: These serve genuinely different purposes. The `tool.rs` docs
explicitly document the separation. Field overlap is minor (~2 fields).
Unifying them would couple planning-time concerns (platform satisfiability)
with runtime concerns (check/install/run), which violates the transport
layer's separation of concerns.

**Decision**: No action. Document this as an intentional design choice, not
technical debt. If field drift becomes a problem (someone updates `id` in one
place but not the other), address with a shared `ToolIdentity` type extracted
from both.

**Files**: None (documentation only if needed)

---

## Execution Order

Streams are partially independent. Recommended sequencing:

```
Week 1-2: Stream 4 (doc consistency) — low effort, high clarity
          Stream 5 Phase V1 (CI verification) — low effort, high value

Week 2-4: Stream 1 Phases R1-R2 (tool registry + annotation)
          Stream 2 Phase E1 (OutputMedium + content IR) — non-breaking

Week 4-6: Stream 1 Phase R3 (boundary unification)
          Stream 2 Phases E2-E4 (in parallel: code, structured, frame layers)

Week 6-8: Stream 1 Phases R4-R5 (resource discovery + validation)
          Stream 2 Phase E5 (emit pattern + remaining emitters)

Week 8+:  Stream 5 Phase V3 (transport compliance in CI)
```

**Critical path**: Stream 1 R1-R2 unblocks R3-R5. Stream 2 E1 unblocks
E2-E4. These are the two most important early deliverables.

**Parallelism**: Stream 2 E2/E3/E4 are fully independent. Stream 1 and
Stream 2 are independent except where emission registration (E5) consumes
tool registration (R2).

---

## Dependency Graph

```
Stream 4 D1, D2 ──────────────────────────────────── (independent)
Stream 5 V1 ──────────────────────────────────────── (independent)

Stream 1:
  R1 (tool-registry crate)
   └──▶ R2 (annotate tools, delete GraphBuilderId)
         ├──▶ R3 (boundary unification)
         │     └──▶ R4 (resource discovery)
         │           └──▶ R5 (validation tests)
         └──▶ Stream 2 E5 (emission registry consumes tool registry)

Stream 2:
  E1 (OutputMedium + content IR)
   ├──▶ E2 (code layer: testgen + dag_gen + cli_gen)
   ├──▶ E3 (structured layer: makegen + clippy + pragma + CI report)
   └──▶ E4 (frame layer: terminal rendering)
         └──▶ (all three) ──▶ E5 (emit pattern + CI + markup + remaining)
```

---

## Verification

After each phase, run:

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
make verify   # generated artifacts unchanged
```

After Stream 1 R2: verify generated CLI `main.rs` files are byte-identical.
After Stream 2 E2-E5: verify all 13 artifact categories are byte-identical
(see `unified-emission.md` Definition of Done criterion 2).

---

## What This Does NOT Cover

These items were evaluated and excluded:

| Item | Reason |
|------|--------|
| Resource phases 4-5 | Already complete (SubDag delegation, resource accounting) |
| graph_mock.rs cleanup | Already complete (all data-only, no `#[cfg(test)]` blocks) |
| OutputMap migration | Already complete (no raw HashMap for outputs found) |
| ShellResponse constructors | Already complete (~50+ `ok()`, ~10+ `failed()` calls) |
| Dual-source boundary mocks | Not actually dual-sourced (makegen derives from codegen) |
| CliToolDef/ToolDef unification | Intentional separation (Stream 6: no action) |
| "List" type_id cleanup | Cardinality handles multiplicity; "List" only in codegen strings |
| `make validate` target | `make verify` already exists and serves this purpose |
| Bootstrapper transport bypass | Intentional, documented, narrowly scoped |

---

## Metrics

Track progress with these counts (automate in CI if possible):

| Metric | Current | Target | Tracks |
|--------|---------|--------|--------|
| Manual tool registrations in `all_tools()` | ~7 | 0 | Stream 1 |
| `GraphBuilderId` references | ~10 | 0 | Stream 1 / 3 |
| Rendering systems without IR | 11 | 0 | Stream 2 |
| Rendering systems without trait | 8 | 0 | Stream 2 |
| Distinct rendering traits | 5 | 1 root + 5 domain | Stream 2 |
| `format!()` constructing source code | ~50 sites | 0 | Stream 2 E2 |
| Manual "Generated by" headers | ~3 | 0 | Stream 2 E3 |
| Hardcoded glob patterns for resources | 2 defs | 0 | Stream 1 R4 |
| Doc contradictions (transport status) | 2 | 0 | Stream 4 |
