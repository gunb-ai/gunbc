# CI / Test Consolidation Plan

> **Status**: Draft
>
> **Date**: 2026-03-06
>
> **Direction**: planner-first, no transitional compatibility constraints

## Goal

Make `ci` and `test-all` planner-owned composite workflows built from shared
abstract process definitions, with Make and provider CI as projections only.

The desired end state is:

- one canonical execution model,
- one canonical meaning for `ci`,
- one canonical meaning for `test-all`,
- zero duplicated orchestration logic across Make, GitHub Actions, and planner
  command catalogs.

## Verified Current Facts

This plan preserves the parts of the current system that already look right.

### 1. Test budgeting already has a good abstraction

The repo already uses Fermi-budgeted test execution.

- `dsl/config/test_policy.dag` defines a default CI budget of `Xl` and a default
  local budget of `S`.
- generated tests and runtime guards already carry `FermiCost`
- `GUNBC_TEST_MAX_COST` is already the public execution knob

This means the repo already has the right abstraction for "how much testing do
we want to run?" We should build on that, not replace it.

**Known duality**: `FermiDepth` (DSL, in `std/types.dag`) and `FermiCost` (Rust,
in `core/test/src/fermi.rs`) are parallel definitions of the same 5-variant
enum. Additionally, `test_policy.dag:depth_ordinal()` duplicates the ordinal
mapping already in `fermi.dag:fermi_ordinal()`. Both dualities are drift risks
when budget levels change — see Rule 6 and invariants 7-9 below.

### 2. `test-all` is already intended as the public full-test alias

In `dsl/config/build_targets.dag`, `test-all` is explicitly:

- description: `"Alias for test-xl (full Fermi budget)"`
- body: `@$(MAKE) test-xl`

So the intended public meaning is already clear:

- `test-all` is not a separate bespoke lane
- `test-all` is the human-facing name for the full `XL` test budget lane

That semantic should survive the consolidation.

### 3. Planner foundations already exist — and are more complete than they appear

The repo already has substantial, tested planner infrastructure:

- `core/workflow/` — schema, process registry, admission, planner, keying,
  coordination, executor, global plan, SLO, projection, proof
- `gunbc-app/src/workflow/catalog.rs` — DSL-backed catalog that compiles
  `config/workflow_catalog.dag`, builds `WorkflowSpec` from DSL pipeline
  templates, and derives `ProcessUnitRegistry` from all workflow variants
- `dsl/workflows/ci.dag` and `dsl/workflows/test_all.dag` — planner pipelines
- `dsl/config/workflow_catalog.dag` — canonical variant registry
- Contract tests for schema, admission, keying, execution, global dedup, and SLO

Key capabilities already proven:

| Capability | Module | Evidence |
|-----------|--------|----------|
| Typed process units | `process_registry.rs` | `ProcessUnitRef`, `ProcessUnitSpec`, `UnitClaim` |
| Cross-workflow dedup | `global_plan.rs` | `canonical_work_identity()` strips workflow naming |
| Deterministic keying | `key.rs` | SHA256 materialization keys with typed miss reasons |
| Fail-closed admission | `admission.rs` | Validates claims, detects conflicts, checks contracts |
| SLO instrumentation | `slo.rs` | Warm-noop budgets, total-time budgets, slow-unit ranking |
| Execution + reporting | `executor.rs` | Topological execution, fail-closed, exit code 42 for approvals |
| DSL-backed catalog | `catalog.rs` | Compiles workflow templates from DSL at runtime |

This is not greenfield architecture. The gap is **wiring, not theory** — the
planner can already plan and execute workflows with typed process units,
deterministic keys, and SLO checking. What it cannot do today is serve as the
canonical executor for the public `ci` and `test-all` names.

## Core Decision

Preserve the **testing abstraction** and replace the **composite ownership**.

In other words:

- keep Fermi-budgeted testing
- keep `test-all` as the full validation scope (currently mapped to XL)
- move ownership of composite lanes to the workflow planner
- stop letting Make, `gunbc-ci`, and workflow command catalogs each define
  their own orchestration semantics

## Problem Statement

Today the same public names are spread across multiple non-equivalent surfaces:

1. GitHub Actions runs a bootstrap-safe setup and then `gunbc-ci`
2. `make ci` runs `gunbc-ci`
3. `make test-all` aliases to `make test-xl`
4. `dsl/tools/ci.dag` defines a composite CI tool
5. `dsl/workflows/ci.dag` defines a planner workflow
6. `dsl/workflows/test_all.dag` defines a planner workflow
7. `dsl/config/workflow_commands.dag` defines a second command graph for those
   planner workflows

This is too many orchestration owners for two public names.

The resulting drift is structural:

- `dsl/tools/ci.dag` is 3 stages (build, test, clippy); `dsl/workflows/ci.dag`
  is 11 stages — these are not the same thing under the same name
- `gunbc-ci` binary executes the tool DAG, not the planner workflow
- planner `ci` is not the public `ci`
- planner `test-all` is not the public `test-all`
- `workflow_commands.dag` hand-enumerates 124 lines of shell commands that are
  already derivable from the tool registry and process specs
- `build_targets.dag` hand-enumerates one `MetaTargetDef` per budget level
  (lines 206-278) instead of deriving them from the `FermiDepth` type

## Target Model

### 1. Process definitions are the abstraction boundary

Composite workflows should not be built from shell commands.

They should be built from typed process definitions with stable identity and
declared semantics.

The abstraction boundary should be:

- **leaf process**: a typed execution unit with declared inputs, outputs,
  resource claims, effect class, and semantic version
- **composite workflow**: a DAG of references to those leaf processes plus
  aggregate/report nodes

The planner already has the right shape for this in
`WorkflowOp::InvokeProcessUnit(ProcessUnitRef)`.

### 2. Testing is a parameterized process family

The test surface should be modeled as a shared abstract family, not as ad hoc
command strings. Three distinct concepts must stay separate:

- **leaf process**: a typed execution unit (`TestProcess`), registered in the
  process registry with declared cost, kind, gating mode, and claims
- **selector/set**: a predicate over process metadata that resolves to a set
  of leaf processes (e.g., "all test processes eligible for full validation")
- **workflow/lane**: a public composite name (`test-all`, `ci`) whose test
  stage is defined by a selector, not by enumerating processes

The key indirection is **validation scope**, not budget level:

- `ValidationScope = Default | Full | Exhaustive`

Current policy maps `Full` to `FermiCost <= XL`, but that mapping lives in
`test_policy.dag`, not in the workflow definition. `test-all` means "full
validation scope." Today that means XL. If budget levels change, only the
policy mapping changes — the workflow definition and public semantics stay
stable.

Test processes carry orthogonal metadata (see Section 4 below):

- `estimated_cost: FermiCost` — Fermi-estimated execution cost
- `kind: Unit | Integration | External | E2E` — structural classification
- `gating_mode: Required | Advisory | Quarantined` — merge-blocking behavior
- `requires: List<String>` — secrets, runner capabilities, etc.

**Derivation requirement**: The Make test targets (`test-xs` through `test-xl`)
and the corresponding `MetaTargetDef` entries in `build_targets.dag` must be
**derived from the `FermiDepth` type**, not hand-enumerated. Today there are 5
nearly-identical blocks in `build_targets.dag` (lines 206-278) that differ only
in the budget level string. Adding a new budget level should require editing the
type definition, not copying a block.

### 3. CI rides on top of the abstract test process

CI should not define its own special test step by restating `cargo test`.

Instead, planner-owned `ci` should compose the same shared abstract processes
that local execution uses, for example:

- codegen ensure
- test generation
- build compile
- full test lane (`TestLane { budget: XL }`)
- clippy / guardrails / verification / report

That gives one semantic owner for testing and lets CI "ride on top" of the
test abstraction instead of forking it.

### 4. Future processes join by typed participation, not by hand wiring

We should not require a human to edit three places whenever a new process should
participate in CI or in a full validation lane.

Instead, process definitions should declare structured metadata across
**orthogonal dimensions**, not a single overloaded "profile" axis:

- `role: Test | Verify | Generate | Build | Guardrail | Report`
  — what the process *does* (structural category)
- `validation_scope: Default | Full | Exhaustive`
  — which validation depth includes this process
- `trigger_eligibility: PR | Merge | Nightly | Manual`
  — when the process should run
- `gating_mode: Required | Advisory | Quarantined`
  — whether failure blocks merge
- `estimated_cost: FermiCost`
  — Fermi-estimated execution cost (for budget filtering)
- `requires: List<String>`
  — secrets, runner capabilities, network access, etc.

Then composite workflows can be authored in terms of selectors over those
dimensions instead of command strings:

- `ci` = all processes with `trigger_eligibility includes PR` and
  `gating_mode == Required`
- `test-all` = all test processes with `validation_scope <= Full`
- `nightly` = all processes with `trigger_eligibility includes Nightly`

That is how drift gets prevented by design:

- adding a new qualifying process to the registry automatically changes planner
  resolution,
- Make and CI YAML only project the resolved planner-owned workflow,
- there is no second command catalog to forget to update.

**Why orthogonal dimensions matter**: A single "profile membership" axis (e.g.,
`LocalDefault | FullValidation | CI | Nightly`) conflates execution context,
validation scope, trigger cadence, and workflow membership into one bucket.
Every new dimension pushes toward a bigger mixed taxonomy — exactly how CI
models become brittle. Separate fields age better.

**Scope note**: This is a follow-on design, not a prerequisite for the initial
consolidation (Phases A-G). The current repo has ~12 workflows — not enough to
justify a full selector system yet. Rules 1-4 and 6 in the "Drift Prevention"
section deliver the bulk of the drift prevention value without participation
metadata. See "Follow-on: Selector and Placement Design" below for the full
treatment.

## Recommended Architecture

### Canonical sources of truth

The long-term ownership boundaries should be:

1. **Process registry**
   - source of truth for executable process units
   - owns semantic identity, role, and participation metadata
   - already exists: `ProcessUnitRegistry` in `core/workflow/src/process_registry.rs`
   - already populated from DSL: `build_process_unit_registry()` in `catalog.rs`

2. **Workflow specs**
   - source of truth for composite orchestration
   - owns topology, ordering, and policy-level composition
   - already exists: `WorkflowSpec` built from `dsl/workflows/*.dag`
   - already backed by: `dsl/config/workflow_catalog.dag` variant registry

3. **Make/Just/CI YAML projections**
   - source of truth for nothing
   - only render adapters around workflow/process entrypoints

### What should stop owning composite semantics

The following should not remain semantic owners for `ci` / `test-all`:

- `dsl/tools/ci.dag`
- `dsl/config/build_targets.dag` command bodies for public composite lanes
- `dsl/config/workflow_commands.dag`
- `.github/workflows/ci.yml`

They may remain projections or leaf-tool surfaces, but not independent
definitions of the same lane.

### What should own composite semantics

- `dsl/workflows/ci.dag`
- `dsl/workflows/test_all.dag`
- planner process registry and workflow resolution

## Public Semantics

After cutover, the public meanings should be:

- `test-all` = "run all tests at full validation scope"
- `ci` = "run the canonical repository verification workflow"

`test-all` is defined by validation scope (`Full`), not by literal budget level.
Today `Full` maps to `FermiCost <= XL` via `test_policy.dag`. If budget
granularity changes (e.g., adding `XXL`), the policy mapping updates — the
public name and workflow definition stay stable.

### Recommendation

`ci` should include the same full-validation test abstraction that powers
`test-all`.

Not because the names must collapse, but because the test semantics should not
fork.

That gives:

- `test-all` as the public full-test lane
- `ci` as a larger lane that includes `test-all` plus any additional repo gates

This is the cleanest model:

- one test abstraction (defined by validation scope)
- one full-test public name
- one CI workflow that composes it

## Drift Prevention By Design

The main prevention mechanism is not more tests. It is deleting duplicate
semantic surfaces.

### Rule 1: composite names are planner-only

Public composite names like:

- `ci`
- `test-all`

must not be owned in both `dsl/tools` and `dsl/workflows`.

Leaf tools are allowed in `dsl/tools`.
Composite lanes live only in planner workflows.

### Rule 2: Make targets are projections

`make ci` and `make test-all` should not embed orchestration command chains.

They should become thin projections:

- `make ci` -> `gunbc-workflow execute ci`
- `make test-all` -> `gunbc-workflow execute test-all`

### Rule 3: provider CI is a projection

GitHub Actions should not spell out lane semantics beyond bootstrap-safe
generated-binary setup.

Its responsibility should be:

1. ensure the generated entrypoints exist in a clean runner
2. invoke the canonical public workflow

### Rule 4: workflow execution commands are derived, not authored twice

`dsl/config/workflow_commands.dag` should be **deleted and replaced with
derivation** from the process registry and tool registry.

Today it maps planner node IDs to `cargo run -p gunbc-app --bin gunbc-foo`
invocations (124 lines, 11 workflow command sets). These commands are already
known: the tool registry knows every binary name, and the process registry knows
every unit. The mapping is mechanical — maintaining it by hand is a drift source.

Concrete plan: derive `UnitCommand` entries in `commands.rs` from
`ProcessUnitSpec` + `ToolDef` instead of compiling `workflow_commands.dag`.
Delete the file in Phase F.

### Rule 5: process participation is typed

Any process that should automatically join CI or full validation must declare
that through typed participation metadata, not through naming conventions or
manual inclusion in YAML.

That lets future process families join by category.

Examples:

- a new `Verify` process can join `ci` automatically
- a new `Test` process with `budget <= XL` can join `test-all`
- a new `Guardrail` process can join `ci` if its profile says so

**Scope note**: This rule is aspirational. It describes the end state to design
toward. The immediate consolidation (Phases A-F) delivers value without it.

### Rule 6: budget levels are derived from the type, not enumerated

Make test targets (`test-xs` through `test-xl`) and their `MetaTargetDef`
entries must be generated from the `FermiDepth` sum type variants.

Adding a budget level to `FermiDepth` should automatically produce the
corresponding Make target, Justfile recipe, and `GUNBC_TEST_MAX_COST` prefix.

This also applies to the `FermiCost` Rust enum — either generate it from the
DSL type, or enforce parity with a guardrail test.

## Concrete Long-Term Shape

### Process layer

Extend `ProcessUnitSpec` with orthogonal metadata fields:

- `process_id`, `semantic_version` — identity (already exists)
- `required_claims` — resource claims (already exists)
- `role: Test | Verify | Generate | Build | Guardrail | Report`
- `validation_scope: Default | Full | Exhaustive`
- `trigger_eligibility: Set<PR | Merge | Nightly | Manual>`
- `gating_mode: Required | Advisory | Quarantined`
- `estimated_cost: FermiCost`
- `requires: List<String>` — secrets, capabilities, runner constraints

For testing specifically, `kind` is a refinement of `role == Test`:

- `kind: Unit | Integration | External | E2E`

### Workflow layer

The planner workflow specs compose process references, not shell commands.
Workflow stages reference selectors, not literal process lists:

- `test-all` = all processes where `role == Test` and
  `validation_scope <= Full`
- `ci` = all processes where `trigger_eligibility includes PR` and
  `gating_mode == Required`, plus report aggregation

### Projection layer

Generated projections become trivial:

- Make renders aliases and wrappers
- CI YAML renders provider jobs that call those wrappers
- help text reads from the same workflow catalog

## Migration Plan

The planner infrastructure is more complete than it may appear. Phases A-C are
primarily wiring existing pieces together, not building new abstractions.

### Phase A: Commit to planner ownership

Decide now that composite public lanes are planner-owned.

Immediate implications:

- `dsl/tools/ci.dag` is no longer the long-term owner of `ci`
- public composite names are reserved for planner workflows
- Make/CI projections will eventually delegate to planner execution

### Phase B: Preserve and formalize the current test abstraction

Keep:

- Fermi budgeting
- `GUNBC_TEST_MAX_COST`
- `test-all` = full validation scope (currently mapped to XL)

But model that abstraction explicitly in planner-owned process/workflow terms.

Additionally:

- Eliminate the `depth_ordinal()` duplication between `test_policy.dag` and
  `fermi.dag` (use `fermi.dag:fermi_ordinal` as the single source)
- Add a guardrail test asserting `FermiDepth` DSL variants match `FermiCost`
  Rust variants

Acceptance:

- `test-all` remains the public name for full-budget tests
- no second CI-only test abstraction exists
- one ordinal mapping, not two

### Phase C: Rewrite `gunbc-ci` to use the planner

The `gunbc-ci` binary currently builds the `dsl/tools/ci.dag` func DAG (3
stages: build, test, clippy) and executes it directly. This bypasses the planner
entirely — the 11-stage `dsl/workflows/ci.dag` pipeline is unused by the public
`ci` name.

Rewrite it to:

1. Call `ci_workflow_spec()` (already exists in `spec_builders.rs`)
2. Call `plan_workflow()` (already exists in `planner.rs`)
3. Call `execute_workflow_plan()` (already exists in `executor.rs`)

This is a bounded task — all three functions already exist and are tested. The
change is routing, not architecture.

Acceptance:

- `gunbc-ci` invokes the planner, not the tool DAG
- `gunbc-workflow plan ci` and `execute ci` produce equivalent results
- `gunbc-workflow plan test-all` and `execute test-all` are real

### Phase D: Rebuild `test-all` as a planner-owned workflow over test abstraction

Planner `test-all` should become the canonical full-test lane.

Its semantics should reference the shared test abstraction, not a bespoke
command string.

Acceptance:

- planner `test-all` is the semantic owner of the full validation lane
- `make test-all` becomes a projection over planner `test-all`

### Phase E: Rebuild `ci` as a planner-owned workflow that composes `test-all`

Planner `ci` should include the same abstract full-test process used by
`test-all`, plus any additional repository gates.

Acceptance:

- planner `ci` owns the public CI lane
- the test portion of `ci` is the same shared abstraction used by `test-all`

### Phase F: Delete duplicate orchestration surfaces

After planner cutover:

- delete or demote `dsl/tools/ci.dag` to a leaf tool (not the public `ci`)
- stop authoring public composite lane bodies in `build_targets.dag`
- delete `dsl/config/workflow_commands.dag` (replace with derived commands
  in `commands.rs` from process registry + tool registry)

Acceptance:

- one semantic owner per public lane
- no duplicate command graph remains
- `workflow_commands.dag` does not exist

### Phase G: Make projections mechanical

After cutover:

- `make ci` projects planner `ci`
- `make test-all` projects planner `test-all`
- GitHub Actions projects planner `ci`
- test-budget Make targets derived from `FermiDepth` variants

Acceptance:

- changing planner semantics automatically changes all public adapters
- no projection file can silently redefine the lane
- adding a budget level to `FermiDepth` automatically produces the
  corresponding Make target

## Required Invariants

The consolidated system should enforce these mechanically:

1. No public composite name exists in both `dsl/tools` and `dsl/workflows`.
2. `test-all` resolves to the full validation scope (currently mapped to `XL`
   via `test_policy.dag`).
3. `ci` consumes the same abstract full-test process used by `test-all`.
4. Make and CI YAML invoke planner-owned public lanes, not direct bespoke
   binaries.
5. Every executable leaf in the lowered plan resolves to exactly one registered
   process unit. (Stated this way to allow future selector-based expansion
   without breaking the invariant.)
6. `FermiDepth` DSL variants and `FermiCost` Rust variants are identical
   (enforced by guardrail test).
7. Test-budget Make targets are derived from `FermiDepth` variants, not
   hand-enumerated.
8. `depth_ordinal()` exists in exactly one place (no duplicated ordinal
   mappings).

## Validation Plan

The following tests should exist after cutover:

1. Planner snapshot tests for `plan ci` and `plan test-all`
2. Projection equivalence tests:
   - Make `ci` -> planner `ci`
   - Make `test-all` -> planner `test-all`
   - GitHub Actions `ci.yml` -> planner `ci`
3. Registry coverage tests (follow-on, after selector design):
   - every process with `trigger_eligibility includes PR` and
     `gating_mode == Required` appears in resolved `ci`
   - every test process with `validation_scope <= Full` appears in resolved
     `test-all`
4. Public name exclusivity tests:
   - no composite public name is owned by both tool discovery and workflow
     discovery
5. Budget parity tests:
   - `FermiDepth` DSL variants == `FermiCost` Rust variants
   - every `FermiDepth` variant has a corresponding `test-{name}` Make target
   - `test_policy.dag` does not re-implement `fermi_ordinal()`

## Exit Criteria

This consolidation is complete when all of the following are true:

1. `test-all` means "full validation scope" (currently mapped to XL).
2. `ci` uses the same shared test abstraction as `test-all`.
3. planner workflows own all public composite semantics.
4. Make and provider CI are projections only.
5. there is one authoritative budget taxonomy (`FermiDepth`), and all other
   surfaces (Rust enum, Make targets, Justfile recipes) are derived from it
   or mechanically validated against it.
6. `workflow_commands.dag` does not exist (commands derived from registries).

The following is explicitly **not** an exit criterion for this consolidation:

- automatic process participation via selectors (see follow-on design below).

That is a valuable end state, but it requires the selector/placement model
described in the follow-on section, and the consolidation delivers substantial
value without it.

---

## Follow-on: Selector and Placement Design

This section scopes the work that is **not part of the consolidation** but is
required to reach "change any underlying dimension and CI just adapts
automatically."

### What the consolidation does not solve

The consolidation (Phases A-G) eliminates duplicate semantic owners and makes
budget derivation mechanical. It does **not** provide:

1. **Selector-based process participation** — a new test process still requires
   a human to add it to the relevant workflow DAG.
2. **Provider placement / lowering** — a process that requires a different
   runner class (macOS, GPU, secrets) still requires manual CI YAML edits.
3. **Gating mode enforcement** — advisory vs required vs quarantined tests are
   not yet distinguished by the planner.

### Selector model

The missing abstraction is a **selector**: a predicate over process metadata
that resolves to a set of leaf processes at plan time.

Conceptually:

```
selector full_validation_tests {
  role == Test
  validation_scope <= Full
  trigger_eligibility includes PR
}
```

A workflow stage references a selector instead of enumerating process IDs:

```
pipeline test_all {
  stage test_run [after build_compile] {
    select full_validation_tests
  }
}
```

The planner resolves the selector against the process registry at plan time,
expanding the stage into one executable leaf per matching process. Invariant 5
("every executable leaf resolves to one process unit") still holds.

### Placement / lowering model

Once the planner resolves selectors, it must **lower** the execution plan into
provider-specific jobs grouped by runner constraints:

1. Partition executable leaves by `requires` (secrets, runner OS, network, etc.)
2. Map each partition to a provider job (GitHub Actions runner class, etc.)
3. Project each job into provider YAML.

Without this, adding a runner/permission dimension still forces CI file changes.

The projection story from the consolidation ("call one wrapper") is correct for
a single-runner model. Multi-runner requires an explicit lowering step between
planning and projection.

### Stress test outcomes after both layers

| Change | Consolidation alone | + Selector + Placement |
|--------|--------------------|-----------------------|
| Add budget level (e.g., XXL) | Mechanical (derived targets, parity test) | Same |
| Add new test process to CI | Manual workflow DAG edit | Automatic via selector |
| Add `NightlyOnly` / `Advisory` dimension | Manual, no principled place | Selector field + predicate |
| Add `RequiresSecrets` constraint | Manual CI YAML edit | Placement partitioning |
| Add new runner OS | Manual CI YAML edit | Placement partitioning |

### Recommended sequencing

1. Finish the consolidation (Phases A-G) first. It delivers value immediately.
2. Add orthogonal metadata fields to `ProcessUnitSpec` (cheap, additive).
3. Implement selectors as a planner planning-phase step.
4. Implement placement/lowering as a post-planning projection step.

Each step is independently useful and does not require the next.
