# Audit — `idempotency.dag` PROXY/STUB → `Lens<C>` (M3 prep)

**Dispatch:** #1139 / inbox prep for `Lens<IdempotencyVerdict>` migration after substrate prerequisites.  
**Authority read:** `src/v3/lenses/idempotency.dag`, `src/v3/std/effects.dag` (Stage 2b carriers + `compose_effects` / `lane2_workflow_idempotency_report`), `src/v3/std/substrate.dag` (`lane2_workflow_at` / `ValueNode` / `BindNode`), `src/v3/compiler/src/workflow_idempotency.rs` (Rust oracle), `docs/design-lens-framework.md` §M3 + `Lens<C>` primitive, `m2_lens_idempotency_migration_test.rs`.

**Constraint (shared):** Real `data <lens>: Lens<C> = { … }` instances require class-5 data-body / function-value lowering plus generic `fold_lens<C>`. No hand-Rust lens instance, no callable-form fake, no loose per-field decls.

---

## 1. Current behavior (as shipped)

| Concern | Where | Behavior |
|--------|--------|----------|
| Entry | `idempotency.dag` | `analyze_workflow(d, workflow_root: NodeId) -> WorkflowIdempotencyReport` |
| Read path | `lane2_workflow_at(d, workflow_root)` → `host lane2_workflow_effect_at` | `None` → `report_unsupported_workflow_variant("Lane2WorkflowRoot", …)` |
| Linear case | `std.effects::lane2_workflow_idempotency_report` | `LinearEffect { ops }` → `WorkflowCompositionVerdict(compose_effects(ops))` |
| Composition | `effects.dag` `compose_effects` | `first_breaker_ref` → **first breaker wins** (`BrokenBy { first_breaker }`) or `IdempotentComposition` |
| Non-linear | `BranchEffect` / `LoopEffect` / `ParallelEffect` | Explicit `IdempotencyUnsupported` with fixed reason strings (no branch/loop algebra yet) |
| Oracle | `workflow_idempotency.rs` | Mirrors `.dag` for rustc round-trip (`m2_lens_idempotency_migration_test`) |

**Important:** Analysis is **workflow-root scoped** (one `NodeId`), not “every `Behavior` in the Dag.” `lane2_workflow` lives only on **Value** / **Bind** nodes (`substrate.dag`).

---

## 2. Mapping to target `Lens<C>` fields (M3 paper exercise)

| `Lens<C>` field | Idempotency analogue today | Gap / note |
|-----------------|----------------------------|------------|
| `name` | e.g. `"lane2_stage2b_idempotency"` (already in unsupported reasons) | trivial |
| `read: (Dag, Behavior) → Witness<C>` | Today: **no** per-Behavior read; one shot on `(Dag, NodeId)` at root | **Major:** must define witness per `Behavior` (likely `Violates` on non-holders; `Inhabits(unit)` or fragment only if we invent a monoid with identity on “no workflow”). |
| `sequential: Monoid<C>` | `compose_effects` order fold = monoid op with **first-breaker-wins** | Matches **only** for the **linear op list** inside one `LinearEffect`. |
| `branch: (C, C) → C` | Not implemented for idempotency (unsupported path) | Until branch algebra exists: fixed `Unsupported` **or** stub `branch` that widens carrier (design choice). |
| `iterate: (C, LoopBound) → C` | Not implemented (unsupported) | Same as branch. |
| `validate: (Dag, C) → OptionalDiagnostic` | Today verdict is **data** (`WorkflowIdempotencyReport`), not `DimensionReport` + diagnostics | Need policy: map `IdempotencyUnsupported` / `BrokenBy` to `SomeDiagnostic` **or** keep verdict-only path outside dimension framework. |

---

## 3. Target carrier `C` (“IdempotencyVerdict”) — reconcile with code

`design-lens-framework.md` M3 table suggests `IdempotencyVerdict = IsIdempotent | IsBreaking(Reason)`.

**Actual** Stage 2b types (`effects.dag` / `dag/effects.rs`):

- `CompositionVerdict` = `IdempotentComposition | BrokenBy { first_breaker: ElementRef<OperationEffect> }`
- `WorkflowIdempotencyReport` = `WorkflowCompositionVerdict(CompositionVerdict) | IdempotencyUnsupported(IdempotencyUnsupportedDetail)`

So:

- **Do not** rename to a toy binary verdict without carrying **ElementRef** evidence and **unsupported** three-field payload unless consumers are migrated.
- **Recommended `C` for the monoid core:** `CompositionVerdict` (matches `sequential.op` story for linear ops).
- **Report shell:** `WorkflowIdempotencyReport` stays the **public** result of `analyze_workflow`; folding may produce `CompositionVerdict` internally then inject `Unsupported` at workflow-shape match — that outer sum is **not** itself a monoid (no meaningful `branch`/`iterate` identity without extra modeling).

---

## 4. First-breaker-wins

Already authoritative in `compose_effects` / `compose_operation_effects` (Rust). Monoid op is “leftmost breaking if any, else idempotent.”

---

## 5. Lens-local diagnostic kinds (Q6.5)

Today: strings inside `IdempotencyUnsupportedDetail` + enum payloads — **no** `Diagnostic` / `OptionalDiagnostic` on the idempotency path.

If M3 adopts full `fold_lens<C> → DimensionReport<C>`:

- Need lens-namespace **Layer-2** kinds for unsupported workflow shapes / missing `lane2_workflow` per `design-lens-framework.md` §Layer 2 + `Diagnostic.kind` widening (already flagged in that doc).

If M3 keeps **verdict-only** API (retire only Rust oracle, emit same `WorkflowIdempotencyReport`): **validate** may stay `NoDiagnostic` always, and diagnostics remain out-of-band — partial fit to `Lens<C>` shape.

---

## 6. Blockers

### 6.1 Shared prerequisite (confirmed)

Class-5 lowering for real `Lens<C>` **data** instances + `fold_lens<C>` in substrate — **blocking** for any honest instance declaration.

### 6.2 Additional substrate / API gaps (beyond 6.1)

1. **Root-scoped analysis vs generic fold**  
   `analyze_workflow(d, workflow_root)` takes an explicit **root** `NodeId`. Spec `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>` has **no** root parameter. Unless every program has a single distinguished root discoverable from `Dag` alone, M3 needs one of:  
   - a **root-parameterized** fold / lens application primitive, or  
   - a **redesign** so idempotency is expressed as per-Behavior reads that still compose to the same verdict only when fold order matches bind-tree discipline (non-trivial), or  
   - acceptance that idempotency dimension runs only on a **filtered** node set (new substrate hook).

2. **Report type is not a pure monoid carrier**  
   `WorkflowIdempotencyReport` sums **verdict** + **unsupported**; `Lens<C>` wants one `C` with `Monoid<C>` for sequential. Need a staged model: e.g. extract `WorkflowEffect` + classify shape **before** fold, then run monoid only on linear fragment; **or** widen `C` to an internal tagged type with monoid laws only on a subset (document partial monoid / error algebra).

3. **Witness / read channel**  
   For non-`Value`/`Bind` behaviors (Transform, Branch, Loop), there is no `lane2_workflow` field. `read` must return **`Violates`** vs **`Inhabits(unit)`** consistently so the fold does not double-count or fabricate carriers (per `Witness<C>` discipline in design doc).

4. **Optional:** If full `DimensionReport` integration is required, map `Unsupported` / `BrokenBy` to `Diagnostic` + declare kinds — ties to Q6.5 substrate work.

---

## 7. Explicit acceptance line for @briansrls

**There *is* at least one substantive blocker beyond the shared class-5 + `fold_lens<C>` prerequisite:** the **root-scoped `NodeId` parameter** and the **non-monoidal outer report sum** (`WorkflowIdempotencyReport`) must be reconciled with the generic per-`Behavior` fold + single-carrier `Monoid<C>` story **before** M3 implementation can be honest without scaffolding.

If product direction locks “idempotency is always whole-program fold with identity off roots,” that is a **modeling decision + possible new primitive**, not just lowering debt.

---

## 8. M3 implementation checklist (after prerequisites land)

1. [ ] Lock carrier: `CompositionVerdict` vs full `WorkflowIdempotencyReport` vs new internal `C` with projection to report.
2. [ ] Lock application model: root-parameterized fold vs whole-Dag fold vs filtered visit strategy; update `design-lens-framework.md` or `dimensions.dag` if new primitive.
3. [ ] Specify `read(dag, b)` for all five `Behavior` variants + `Witness<C>` for missing / non-holder cases.
4. [ ] Declare `Monoid<C>.op` = first-breaker-wins path equivalent to `compose_effects`; prove associativity on linear fragments.
5. [ ] Declare `branch` / `iterate` stubs or unsupported widening until Branch/Loop/Parallel algebra exists (`effects.dag` already documents graduation).
6. [ ] Decide `validate` + diagnostics: verdict-only vs `OptionalDiagnostic` + Layer-2 kinds.
7. [ ] Author `data idempotency_lens: Lens<…> = { … }` instance (no Rust oracle).
8. [ ] TestClaim `idempotency_lens_via_framework_correct` per M3: emitted fold matches current `lane2_workflow_idempotency_report` / `analyze_workflow` on fixed fixtures (including unsupported + missing-workflow cases).
9. [ ] Retire `workflow_idempotency.rs` oracle only when emitted module is sole authority and migration test is rewired to substrate runner (not rustc harness) if policy requires.

---

## 9. Rust oracle information **not** captured by “per-Behavior read + aggregate validate” alone

- **`ElementRef<OperationEffect>`** in `BrokenBy` ties the verdict to **index position in the op list** — a `read` per `Behavior` does not naturally yield that without either carrying list context in `Witness<C>` or composing at a **list-shaped** substrate node (not one `Behavior` = one op today).
- **`analyze_workflow` root selection** is caller-provided `NodeId` — aggregate `validate` sees `Dag` + composed `C` but **not** the root unless `C` or `DimensionReport` carries it.

So: **yes**, there is oracle information (list-indexed breaker + explicit root) that a naïve “read each Behavior once, validate once” story does not encode without extra fields or a richer `Witness<C>` payload.
