# Audit — `idempotency.dag` PROXY/STUB → DB-3 spine + `Lens<C>` (M3 prep)

**Dispatch:** #1139 / inbox prep for migrating idempotency onto the **DB-3 dimension** substrate and, when landed, the **R2 `Lens<C>`** primitive — after shared prerequisites.

**Primary authority (DB-3 + substrate + roadmap):** [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) (DB-3; **Consumers** in that doc: Lane 2 Stage 2f, Lane 4 4b/4c, and Lane 2 Stages **2b/2d/2e** including idempotency as a Dimension-shaped analysis), [`src/v3/std/dimensions.dag`](../src/v3/std/dimensions.dag) (`Witness<Carrier>`, `DimensionReport<Carrier>`, `AnalysisDimension<Carrier>`), [`ROADMAP.md`](../ROADMAP.md) (`DB-3` → [`docs/db-history/db-3.md`](db-history/db-3.md) receipts).

**Shipped idempotency read:** `src/v3/lenses/idempotency.dag`, `src/v3/std/effects.dag` (Stage 2b carriers + `compose_effects` / `lane2_workflow_idempotency_report`), `src/v3/std/substrate.dag` (`lane2_workflow_at` / `ValueNode` / `BindNode`), `src/v3/compiler/src/workflow_idempotency.rs` (Rust oracle), `m2_lens_idempotency_migration_test.rs`.

**Secondary authority (forward R2 lens spec):** [`docs/design-lens-framework.md`](design-lens-framework.md) (`Lens<C>`, Q6.5 Layer-2 diagnostics, abbreviated `fold_lens` sketch). Use only **after** grounding in DB-3 / `dimensions.dag` so the audit does not invert substrate vs proposal (see §6.2.1).

**Constraint (shared):** Honest **`AnalysisDimension<Carrier>`** (DB-3) **data** values require class-5 record bodies plus a lowered **`analyze(d, workflow: NodeId, dim)`**-shaped evaluation path (`design-dimension-abstraction.md` §Dimension evaluation). The **`Lens<C>`** instance + fold consumer (`design-lens-framework.md`; today’s compiler seam: `fold_lens_over_reflected_program` in `src/v3/compiler/src/lens_apply.rs`) **reuses** `Witness` / `DimensionReport` from `dimensions.dag` — it does not replace DB-3 as naming authority. No hand-Rust lens instance, no callable-form fake, no loose per-field decls.

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

## 2. Mapping to forward `Lens<C>` fields (paper exercise; ground in `AnalysisDimension` + DB-3)

`Lens<C>` field names come from **`design-lens-framework.md`** (R2 proposal). **`AnalysisDimension<Carrier>`** in `dimensions.dag` is the landed analysis record (`witness_of` / `compose: Monoid<Carrier>` / `break_diagnostic`; the prior `compose: fn(C,C)->C` + `identity: C` field pair was collapsed into a single `Monoid<Carrier>` field by F2 dispatch / PR #1607, mirroring the `Lens<C>` precedent); DB-3 **`analyze(d, workflow, dim)`** is the evaluation entry. When lowering lands, reconcile names (`read` vs `witness_of`, `sequential` vs `compose`, etc.) against that substrate — do not treat the lens doc alone as the dimension authority.

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

The lens-framework M3 table in `design-lens-framework.md` (sketch) suggests `IdempotencyVerdict = IsIdempotent | IsBreaking(Reason)`. **Authoritative carriers** for dimension-shaped analysis remain `dimensions.dag` + `effects.dag` (below).

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

If M3 adopts full **dimension report** integration (`DimensionReport<Carrier>` from `dimensions.dag`) via the lens fold (`design-lens-framework.md`):

- Need lens-namespace **Layer-2** kinds for unsupported workflow shapes / missing `lane2_workflow` per `design-lens-framework.md` §Layer 2 + `Diagnostic.kind` widening (already flagged in that doc).

If M3 keeps **verdict-only** API (retire only Rust oracle, emit same `WorkflowIdempotencyReport`): **validate** may stay `NoDiagnostic` always, and diagnostics remain out-of-band — partial fit to `Lens<C>` shape.

---

## 6. Blockers

### 6.1 Shared prerequisite (confirmed)

**Lane 2 Stage 2f / DB-3 path:** class-5 lowering for real **`AnalysisDimension<Carrier>`** (or DB-3-equivalent) **data** declarations, plus a generic **dimension evaluation** implementation consistent with **`analyze(d, workflow: NodeId, dim)`** (`design-dimension-abstraction.md`) — **blocking** for honest built-in / user-declared dimension values on the shared spine.

**R2 `Lens<C>` path (consumes the same carriers):** full **`Lens<C>`** **data** instances + evaluator fold remain **downstream** of the above; see `design-lens-framework.md` and T-Substrate-Lens-Primitive in `docs/r2-closure-ledger.md` / `docs/r2-structure.md` — not a substitute authority for DB-3 naming.

### 6.2 Additional substrate / API gaps (beyond 6.1)

1. **Workflow root vs fold entry (spec alignment, not a missing primitive)**  
   `analyze_workflow(d, workflow_root)` matches the **DB-3 evaluation API**: `analyze(d, workflow: NodeId, dim)` in [`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md) §"Dimension evaluation". The dimension record in substrate is **`AnalysisDimension<Carrier>`** (`src/v3/std/dimensions.dag`); DB-3 and older prose use colloquial `Dimension<Carrier>` for that analysis shape — see the **Substrate naming note** in [`docs/design-lens-framework.md`](design-lens-framework.md) §The `Lens<C>` primitive.  
   The one-line diagram `fold_lens<C>: Lens<C> → Dag → DimensionReport<C>` in that lens doc is **abbreviated**; it must **not** be read as superseding DB-3 or inventing a rootless authority. M3 should **name the substrate target** the fold extends (dimension evaluation / `analyze`-shaped driver — today’s seam includes `fold_lens_over_reflected_program` in `src/v3/compiler/src/lens_apply.rs`) and thread **`workflow: NodeId` the same way DB-3 does**, rather than treating “no `NodeId` on the diagram” as a greenfield substrate gap (INVARIANTS.md — *Unnamed substrate target* / design commitments must name the carrier).  
   Remaining work: reconcile the lens-framework diagram (and any lowering plan) with **`analyze(d, workflow, …)`**, plus idempotency-specific bind-tree modeling — **not** “substrate lacks a workflow root hook.”

2. **Report type is not a pure monoid carrier**  
   `WorkflowIdempotencyReport` sums **verdict** + **unsupported**; `Lens<C>` wants one `C` with `Monoid<C>` for sequential. Need a staged model: e.g. extract `WorkflowEffect` + classify shape **before** fold, then run monoid only on linear fragment; **or** widen `C` to an internal tagged type with monoid laws only on a subset (document partial monoid / error algebra).

3. **Witness / read channel**  
   For non-`Value`/`Bind` behaviors (Transform, Branch, Loop), there is no `lane2_workflow` field. `read` / `witness_of` must return **`Violates`** vs **`Inhabits(unit)`** consistently so the fold does not double-count or fabricate carriers (per **`Witness<Carrier>`** in `dimensions.dag`, DB-3).

4. **Optional:** If full `DimensionReport` integration is required, map `Unsupported` / `BrokenBy` to `Diagnostic` + declare kinds — ties to Q6.5 substrate work.

---

## 7. Explicit acceptance line for @briansrls

**There *is* at least one substantive blocker beyond the shared class-5 + DB-3 dimension-evaluation / honest **`AnalysisDimension`** declaration prerequisite:** the **non-monoidal outer report sum** (`WorkflowIdempotencyReport` = composition verdict ∪ unsupported) must be reconciled with a single sequential carrier `Monoid<C>` on the forward **`Lens<C>`** shape (and with `branch` / `iterate` until those algebras exist). **Workflow root scoping is not an extra substrate hole:** DB-3 already locks `analyze(d, workflow: NodeId, …)`; idempotency’s explicit `workflow_root` lines up with that. A rootless whole-`Dag`-only fold would be a **new design commitment** (must name substrate + ratchet), not something the abbreviated `fold_lens` one-liner forces by itself.

If product direction ever locked “idempotency is always rootless whole-program fold,” that would **contradict** DB-3’s named evaluation shape and would need an explicit decision — it does **not** follow from today’s `analyze_workflow` API.

---

## 8. M3 implementation checklist (after prerequisites land)

1. [ ] Lock carrier: `CompositionVerdict` vs full `WorkflowIdempotencyReport` vs new internal `C` with projection to report.
2. [ ] Lock application entry: align emitted/spec fold with DB-3 **`analyze(d, workflow: NodeId, dim)`** (substrate record: **`AnalysisDimension<Carrier>`**); amend `design-lens-framework.md` if the `fold_lens` sketch still omits `workflow` so the named substrate target is explicit end-to-end.
3. [ ] Specify `read(dag, b)` for all five `Behavior` variants + `Witness<C>` for missing / non-holder cases.
4. [ ] Declare `Monoid<C>.op` = first-breaker-wins path equivalent to `compose_effects`; prove associativity on linear fragments.
5. [ ] Declare `branch` / `iterate` stubs or unsupported widening until Branch/Loop/Parallel algebra exists (`effects.dag` already documents graduation).
6. [ ] Decide `validate` + diagnostics: verdict-only vs `OptionalDiagnostic` + Layer-2 kinds.
7. [ ] Author declared instance per landed stack: prefer **`data …: AnalysisDimension<…>`** (DB-3) until class-5 + `analyze` driver is real; then **`data …: Lens<…>`** (`design-lens-framework.md`) when R2 lens primitive is the ratcheted authority — no Rust oracle.
8. [ ] TestClaim `idempotency_lens_via_framework_correct` per M3: emitted fold matches current `lane2_workflow_idempotency_report` / `analyze_workflow` on fixed fixtures (including unsupported + missing-workflow cases).
9. [ ] Retire `workflow_idempotency.rs` oracle only when emitted module is sole authority and migration test is rewired to substrate runner (not rustc harness) if policy requires.

---

## 9. Rust oracle information **not** captured by “per-Behavior read + aggregate validate” alone

- **`ElementRef<OperationEffect>`** in `BrokenBy` ties the verdict to **index position in the op list** — a `read` per `Behavior` does not naturally yield that without either carrying list context in `Witness<C>` or composing at a **list-shaped** substrate node (not one `Behavior` = one op today).
- **`analyze_workflow` root selection** is caller-provided `NodeId` — aggregate `validate(dag, c)` (lens-framework shape) does not take `workflow` as an argument, so a **naïve** “only validate at the end” story can lose the root unless `C` / `DimensionReport` carries it. **Mitigation:** thread `workflow` at the **DB-3 `analyze(d, workflow, dim)`-shaped** fold entry (same authority as dimension evaluation), not only inside `validate`.

So: **yes**, there is oracle information (list-indexed breaker + explicit root) that a naïve “read each Behavior once, validate once” story does not encode without extra fields or a richer `Witness<C>` payload.
