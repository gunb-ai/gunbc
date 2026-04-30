# Parallelism lens migration — audit / prep (PROXY → `Lens<C>`)

**Dispatch:** inbox #1130 / #4344943395 (parallelism.dag lens migration audit slice).  
**Status:** PRE-IMPLEMENTATION — no `.dag` instance, no hand-Rust `Lens<C>` scaffolding.  
**Current blocker (confirmed from HEAD 2026-04-30):** a real
`data <lens>: Lens<C> = { ... }` still requires the **full
`Lens<C>` / `DimensionReport<C>` fold** plus a settled workflow-scoped
entry shape for Stage 2e. Function-valued record fields have a narrower
path now, but that does **not** make `parallelism_lens` honest by itself.
Do not fake the instance.

---

## 1. Sources read (authority map)

| Artifact | Role today |
|----------|------------|
| `src/v3/std/lens.dag` | Director-locked **6-field `Lens<C>`** contract: `name`, `read`, `sequential`, `branch`, `iterate`, `validate`. |
| `src/v3/lenses/parallelism.dag` | **STUB:** `analyze_parallelism` always returns `report_parallelism_unsupported(LensSurfacePending, …)`. |
| `src/v3/compiler/src/workflow_parallelism.rs` | **Authoritative Stage 2e analysis** (`analyze_parallelism(d, workflow_root)`). |
| `src/v3/std/effects.dag` | Carriers: `WorkflowParallelismReport`, `ParallelismUnsupportedKind`, `CompositionVerdict`, `WorkflowEffect::ParallelEffect`. |
| `docs/design-lens-framework.md` **§M4** | Acceptance: TestClaim `parallelism_lens_via_framework_correct`; retire placeholder after migration. |
| `src/v3/compiler/src/lens_apply.rs` | `fold_lens_over_reflected_program` exists, but explicitly remains a reflect-then-apply seam; it does **not** perform generic `Lens<C>` / `DimensionReport<C>` aggregation. |

---

## 2. Target carrier naming (`ParallelismVerdict` vs substrate)

Dispatch uses **“ParallelismVerdict”**; the locked substrate type is **`WorkflowParallelismReport`** (`effects.dag`):

- `ParallelCompositionVerdict(CompositionVerdict)` with `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: … }`
- `ParallelismUnsupported(ParallelismUnsupportedDetail)` with explicit `ParallelismUnsupportedKind` (including `LensSurfacePending` for the current stub path).

**Recommendation:** treat **`C = WorkflowParallelismReport`** (or a thin newtype alias in `.dag` if nominal opacity is needed) for the lens instance **only if** the fold can lawfully produce `DimensionReport<WorkflowParallelismReport>` without inventing monoid structure that `WorkflowParallelismReport` does not satisfy (see §5).

---

## 3. Mapping `Lens<C>` fields → intended parallelism semantics

| `Lens<C>` field | Intended role for parallelism | Current Rust analogue |
|-----------------|------------------------------|------------------------|
| **`name`** | `DimensionReport.dimension_name` / human id for the dimension. | Implicit `"lane2_stage2e_parallelism_lens"` in `DOWNSTREAM` / `downstream_stage` strings. |
| **`read`** | Per-**Behavior** evidence: `Witness<C>` (fail-closed, no fabricated `C`). | **Mismatch risk (§6):** Rust walks **`ParallelEffect` branches** at **`workflow_root`**, not arbitrary `Behavior` nodes. Evidence is **cross-branch operation lists** + commutativity / breaking checks. |
| **`sequential`** | `Monoid<C>` for `BindNode` along a **linear** branch. | Linear branch ops are `Vec<OperationEffect>`; idempotent op commute is **pairwise**, not a single monoid fold over `C` as today’s cost monoid unless `C` encodes partial evidence. |
| **`branch`** | `fn(C,C)->C` join over **exclusive** branch arms (max/join semantics). | Cross-branch **pairwise** `operations_commute` across **different** arms — algebraically closer to a **join over per-arm summaries** than to `branch` on two `C` values unless each arm collapses to one `C`. |
| **`iterate`** | `fn(C, LoopBound)->C` for loops. | **Not used** in Stage 2e v1: non-linear parallel interior → `NonLinearParallelBranch` unsupported. |
| **`validate`** | `fn(Dag, C) -> OptionalDiagnostic` aggregate side-conditions. | Partially folded into **unsupported reasons** and `BrokenBy` path; no separate aggregate pass today beyond commutativity / breaker scan. |

---

## 4. Branch-arm coverage / dependency-graph facts

**Within current Rust scope (v1):**

- Requires **`WorkflowEffect::ParallelEffect { branches }`** where **every** branch is **`LinearEffect { ops }`** (`extract_linear_branches`). Any other shape → `NonLinearParallelBranch`.
- **Breaking scan** is **global across flattened branch ops** (`first_breaking_across_branches`) — order is the **branch-order flattening** contract documented in Rust; `WorkflowEffect::operation_at` must resolve `ElementRef` consistently.
- **Commutativity** is **pairwise across distinct branches** (not along a single sequential spine).

**Beyond `Lens<C>.read: (Dag, Behavior) -> Witness<C>` as written:**

- The analysis is **rooted at a workflow projection** (`lane2_workflow_effect_at` / `lane2_workflow` on `Value`/`Bind`), not at “each Behavior in a post-order fold.”
- True **schedule-level dependency graph** (partial order of ops across branches with data deps) is **not** modeled in `workflow_parallelism.rs` today — only **operation-shape commute** on classified `OperationEffect` pairs.

So: **branch-arm coverage is required** for the current algorithm, but **fine-grained data-dependence between branches** is **out of scope** in the existing Rust oracle. If the lens framework migration accidentally forces a **pure per-Behavior read**, we would **lose** the current **explicit parallel-branch structure** unless we redesign how `read` obtains its evidence.

---

## 5. Blockers

### 5.1 Shared prerequisite (blocking everyone, narrowed on HEAD)

- **Function-valued record-field lowering is no longer the deepest
  blocker in the non-generic case.** `lower_structural_field_value`
  accepts value-level Arrow declarations when
  `declaration_ref_types_equivalent` proves the expected Arrow
  signature; `m2_feature_parity_test::test_3a2_lensish_int_carrier_lowers_branch_and_monoid_fn_refs`
  pins the simple `branch` / nested `sequential.op` shape.
- **Generic `Lens<C>` / `Monoid<C>` instance bodies remain blocked.**
  `idempotency_lens_instance_blocker_test::generic_lens_monoid_function_field_refs_are_current_lowerer_gap`
  still records the unresolved generic data-body path.
- **The real fold is still absent.** `fold_lens_over_reflected_program`
  reflects a program and applies a named Arrow lens, but its own API
  comment keeps `Lens<C>` / `DimensionReport` aggregation out of scope.
  No workaround with hand-Rust lens instances or callable-form fakery.

### 5.2 Additional substrate / design gap (beyond 5.1)

**Yes — one substantive gap beyond the shared prerequisite:**

**`Lens<C>.read` input domain vs Stage 2e analysis domain**

- `Lens.read` is defined over **`(Dag, Behavior)`** (per `lens.dag` and M4/M8 narrative in the design doc).
- `analyze_parallelism` is defined over **`(Dag, workflow_root: NodeId)`** and inspects a **`WorkflowEffect`** subtree (`ParallelEffect` → list of **`LinearEffect`** branches).

**Implication:** a naïve “port Rust into `read` bodies” without framework extension risks:

1. **Fabricating** per-Behavior `Witness<C>` values that do not correspond to any real Stage 2e evidence step, or  
2. **Encoding workflow-root** as a magic `Behavior` (bridge / non-structural), or  
3. **Splitting** the analysis into artificial per-Behavior slices that do not match the current **cross-branch** proof obligations.

**Disposition options (pick at lens-framework design time, before M4 implementation PR):**

- **(A) Framework extension:** introduce a **workflow-scoped** analysis hook (analogous to design-doc **L6** rationale — input space not reducible to per-Behavior `read` only), *or* generalize `read` with an explicit **witness context** parameter (larger substrate change — **not** this audit’s decision).
- **(B) Carrier redesign:** make `C` a **per-arm partial summary** so `read`/`branch`/`sequential` genuinely factor the existing proof — requires proving the Rust algorithm is expressible as that fold **without** hidden global state (hard; needs formal sketch).
- **(C) Keep parallelism outside strict `Lens<C>`:** conflicts with **M4** as written; only viable if M4 is reinterpreted (unlikely without Director).

**Explicit statement for acceptance:** there **is** an **extra design blocker** — **read-channel / entry-point mismatch** between **`Lens.read`** and **workflow-rooted `ParallelEffect` analysis** — **in addition to** the shared class-5 + `fold_lens<C>` prerequisite.

### 5.3 Deepest gap after HEAD audit

The deepest current gap is **not** the `.dag` stub body. It is the
missing substrate shape that lets a `Lens<C>` fold consume **workflow-
rooted, cross-branch evidence** without pretending that evidence is a
per-`Behavior` fact.

Current Stage 2e authority has these obligations:

- Entry is `analyze_parallelism(d, workflow_root: NodeId)`, not a
  rootless `fold_lens<C>(lens, d)` and not `read(d, behavior)` for every
  behavior.
- Evidence is a `WorkflowEffect::ParallelEffect` attached via
  `lane2_workflow_at` to a `Value` / `Bind` root.
- Branch evidence is cross-product shaped: every operation in branch A
  must commute with every operation in branch B. A per-arm summary may be
  possible, but it has not been proven as a `Monoid<C>` + `branch(C,C)`
  algebra.
- `NonLinearParallelBranch`, `PairwiseNonCommute`, and `BrokenBy` are
  semantic outcomes, not missing-data defaults.

Therefore the next honest substrate slice is one of these, chosen before
implementation:

1. **Workflow-scoped lens fold:** extend/name the fold target so a lens
   instance can receive the workflow-root authority explicitly and call
   `lane2_workflow_at(d, root)` without encoding the root as a magic
   `Behavior`.
2. **Parallelism summary carrier proof:** define a carrier whose `read`,
   `sequential`, and `branch` operations genuinely factor the current
   pairwise cross-branch algorithm. Acceptance requires showing that the
   carrier reproduces all existing `lane2_stage_2e_parallelism_test`
   cases without hidden global state.

Until one of those lands, the only correct `.dag` behavior is the current
`LensSurfacePending` fail-closed stub.

---

## 6. M4 implementation checklist (post-prerequisites)

Use after class-5 + `fold_lens<C>` land; order is suggestive, not a commitment.

1. **Lock disposition** for §5.2 (A vs B); record in `docs/design-lens-framework.md` or Director thread — **before** authoring `data parallelism_lens: Lens<…>`.
2. **Freeze carrier `C`** — default proposal: `WorkflowParallelismReport` **iff** monoid/branch/iterate story composes; else define a **summary carrier** + final `validate` projection to `WorkflowParallelismReport`.
3. **Author `Lens<C>` in `.dag`** with six fields; **no** Rust literals for the record body beyond what lowering allows.
4. **Port logic** from `workflow_parallelism.rs` into field bodies / `std.effects` helpers per chosen disposition (minimize duplicate authority with Rust — pick **one** execution site until self-host parity).
5. **Replace** `parallelism.dag` stub with real dispatch (mirror `idempotency.dag` pattern: `lane2_workflow_at` + `match` on `WorkflowEffect` where applicable).
6. **Add** TestClaim **`parallelism_lens_via_framework_correct`** per **M4**; extend `lane2_stage_2e_parallelism_test.rs` to compare against **frozen** oracle vectors (same discipline as other lens migrations).
7. **Retire** placeholder `LensSurfacePending` path for the migrated entrypoint once `.dag` path is green.
8. **SG-0 / manifest** if new hand-authored files appear; **regen_bootstrap** if `effects.dag` / `lens.dag` shapes change.

### 6.1 Acceptance criteria for the next substrate slice

- `parallelism.dag` stops returning `LensSurfacePending` only in the same
  PR that adds an executable `.dag` path for the Stage 2e cases.
- The executable path reads `WorkflowEffect` through `lane2_workflow_at`
  or a named workflow-root fold authority. No `SourceSpan.file`, stringly
  root selection, or fabricated per-Behavior witness stands in for root
  identity.
- Existing Rust oracle vectors in
  `lane2_stage_2e_parallelism_test.rs` are mirrored by a
  `parallelism_lens_via_framework_correct` claim/test before
  `workflow_parallelism.rs` can be retired or demoted.
- Upsert x Upsert remains fail-closed until a merge/value witness exists
  on the operation/resource substrate; the lens must not infer
  commutativity from operation-name equality or matching `KeySource`
  alone.
- Nested `BranchEffect` / `LoopEffect` inside a `ParallelEffect` branch
  remains explicit `NonLinearParallelBranch` unless the same PR adds the
  operation-effect/resource-threading carrier needed to analyze it.

---

## 7. Non-goals (this audit)

- No Rust implementation edits in this slice.
- No `Lens<ParallelismVerdict>` naming in substrate until `WorkflowParallelismReport` rename is a Director-owned decision.
