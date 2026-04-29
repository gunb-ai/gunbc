# R2 PR-E — Lens application as a fold over reflected program DAG

> **Worker slice (Evaluator Manager).** Opened post-**#1170** merge: structural
> reflection for `reflect_program_dag_nodes_in_file` is **complete** and live on
> `main` (`src/v3/compiler/src/lens_apply.rs`). This brief scopes the **next**
> Evaluator slice — applying a `Lens<C>` (lens-instance body) as a **fold /
> aggregate** over that reflected program spine (`FieldValue`), per
> [`docs/design-lens-framework.md`](../design-lens-framework.md) (`Lens<C>` six
> fields, `DimensionReport<C>` accumulation) and
> [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) Lens
> application row.

## Read first

- [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — Lens
  application row; acceptance gate name `evaluator_lens_application_complete_reflection`.
- [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md)
  — complete reflection contract (now implemented in Rust).
- [`docs/design-lens-framework.md`](../design-lens-framework.md) — `Lens<C>`
  shape, aggregate/fold semantics, diagnostic layering (Q6.5).
- [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
  §2–§3 — PB-Runtime / Evaluator convergence; **Value** vocabulary constraints
  (#1176). **Do not** introduce new observable `Value` inhabitants or
  closed-over-environment carriers here — that is **Worker A** (`nimble-tern-266`
  / runtime-value substrate). PR-E cites those docs and leaves explicit
  dependencies.
- [`src/v3/compiler/src/lens_apply.rs`](../../src/v3/compiler/src/lens_apply.rs)
  — `reflect_program_dag_nodes_in_file`, `apply_lens_declaration`,
  `fold_lens_over_reflected_program` (reflect + apply slice).
- [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag),
  [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) — reflected
  carriers and `DimensionReport` / `Witness` authority.

## Scope (this PR / slice)

1. **Plumbing API:** `fold_lens_over_reflected_program` in `lens_apply.rs` is the single seam for
   “lens over reflected program DAG”. **Slice 1 (landed):** it runs
   `reflect_program_dag_nodes_in_file` then [`apply_lens_declaration`], passing the reflected
   carrier as the lens’s **first** argument (additional `inputs` follow). Arity must match
   `1 + inputs.len()` lens formals. Fail-closed errors come from reflection / `apply_lens_declaration`
   (no fabricated `DimensionReport`). [`LensApplyError::UnimplementedLensFold`] remains reserved
   for future fold-driver paths not delegated here.
2. **Documentation:** this brief + `r2-evaluator-manager.md` cross-refs so
   dispatch (#1131) and reviewers share one target.
3. **No** new hand-authored `src/v3/compiler/src/*.rs` files (SG-0); extend
   `lens_apply.rs` only unless a path is producer-owned/generated.

## Out of scope (explicit)

- Choosing or implementing the runtime **`Value`** coproduct or
  **environment** representation (Worker A + `design-pb-runtime-interpreter.md`).
- Full PB-Runtime `.dag` lens-body interpreter (dissolution per
  `lens_apply_dot_rs_retired` in `design-pb-runtime-interpreter.md`).
- `DimensionReport` population beyond fail-closed errors for unsupported paths.

## Contract sketch (implementation follow-ups)

**Inputs (conceptual):** compiled **program** `Dag`, `source_file` filter (same
as reflection), **`id_space`** `Dag` for `List` / `Behavior` constructor ids
(INVARIANTS P2), **lens** `Dag` + lens root `DeclarationId`, lens **inputs**
`&[FieldValue]`.

**Pipeline:**

1. `reflect_program_dag_nodes_in_file(program, source_file, id_space)?` →
   substrate-shaped `FieldValue` (today: `Record { nodes: List<Behavior> }`).
2. Interpret / walk the lens arrow body over that carrier via
   [`apply_lens_declaration`] (slice 1: reflected value is the first argument; same bounded
   interpreter as manual `reflect` → `apply` tests). Later slices may introduce a dedicated fold
   driver / `DimensionReport` path without changing the reflection contract.
3. Emit `FieldValue` / structured diagnostics per `Lens<C>` / framework rules (today: whatever
   `apply_lens_declaration` returns).

## Acceptance hook — `evaluator_lens_application_complete_reflection`

**Gate name** (from `r2-evaluator-manager.md`): `evaluator_lens_application_complete_reflection`.

**Nearest fixture home (not authored in this slice):** add a deferred
`TestClaim` (same pattern as `r1_release_acceptance.dag`) to a **new** template
once R2 evaluator gate plumbing exists:

- **Target path (authoritative name for reviewers):**
  `src/v3/compiler/tests/fixtures/r2_evaluator_lens_application.template.dag`
- **Claim data name:** `evaluator_lens_application_complete_reflection`
- **Predicate:** structural acceptance that the fold over
  `reflect_program_dag_nodes_in_file` output succeeds for a **minimal** lens +
  program pair (exact predicate TBD with testgen / R1C-D patterns).

This PR only **names** that path so the gate is not orphaned in prose.

## Coordination

- **Worker A:** any need for runtime `Value` or environment inside the fold
  **blocks** on their substrate + `design-pb-runtime-interpreter.md` alignment.
- **PB Manager / Substrate:** `Lens<C>` substrate primitive + bin-shim evolution
  per manager brief “Consumes” rows.

## Dissolution / debt

When `fold_lens_over_reflected_program` carries a full `Lens<C>` / `DimensionReport` driver and
the `.dag` claim is green, remove or narrow [`LensApplyError::UnimplementedLensFold`] to truly
unreachable internal states; update this brief status to **LANDED** and wire the gate into the
closure ledger.
