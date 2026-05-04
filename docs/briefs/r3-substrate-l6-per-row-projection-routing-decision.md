# R3 Substrate — L6 Per-Row Projection Routing Decision

**Status:** STOP+PING — decision receipt only. The chosen routing direction is clear; the substrate shape it implies touches axis-type homes that are not yet in `.dag`, so Director sign-off on the open sub-questions before code lands avoids carrier rework.

**Authority:** Verification audit PR #1593 / `docs/briefs/r3-v-l6-per-row-projection-audit.md` (`l6_method_template_per_row_projection`). Dispatch on parent inbox #1130.

**Scope of this receipt:** route the per-row projection problem to a single Substrate option, name the rejected options with reasons, sketch the carrier shape, and surface the sub-questions that need Director sign-off. Does **not** land the carrier, populate rows, or convert `coverage.rs` — those steps follow the sign-off.

## 1. Problem restatement

`src/v3/grounding_cross_target_meta/src/coverage.rs` projects `MethodTemplateContract` row lists into the L6 `(FormAxis × BehaviorAxis × ShapeATarget)` cross-product by asserting that each per-target list is non-empty and bucket-mapping the *whole list* to `Cardinality × Transform × <target>`. That projection is honest only because today's Phase 1 rows are homogeneous in cell. As soon as a non-`Cardinality × Transform` row enters one of the three lists, list-non-empty stops being a faithful per-row projection.

Existing carriers are not sufficient to make per-row projection structural without re-hardcoding method names or inferring from template strings (audit verdict). The L6 axes themselves (`FormAxis` / `BehaviorAxis` / `ShapeATarget`) live only as **hand-Rust enums** in `src/v3/grounding_cross_target_meta/src/cells.rs` today; there is no substrate-side declaration of them.

## 2. Option evaluation

The dispatch named three audit options and three decision criteria:

- **C1 — orthogonality:** avoid coupling unrelated *template-authoring* facts (runtime/emit templates, wraps_result, placeholder_convention) to *L6-projection* facts (cell coordinates).
- **C2 — structural row fact, no string/template inference:** projection must be a row-local typed fact, not derived from a list-name registry or a render-template scan.
- **C3 — preserve Grounding ownership of row population** if the decision lands a new/extended carrier consumed by Grounding.

### Option 1 — Extend `MethodTemplateContract` with cell facts directly

**Rejected.** Adds `connective: FormAxis`, `behavior: BehaviorAxis`, possibly `cells: List<Cell>` directly onto every existing row.

- Fails **C1**: template-authoring facts and L6-projection facts are orthogonal axes. Templates describe *how to render code at this method*; cells describe *which substrate cross-product slot the row inhabits*. The dispatch is explicit that authoring rows should not absorb verification-only projection facts unless the domain says they are the same row authority. They are not.
- Fails the dispatch's "no broad `MethodTemplateContract` row migration unless strictly required" guardrail: every existing row across three target lists would need to gain L6 cell facts, even though the only consumer is the Verification walker.
- Adds a 5th / 6th field to a load-bearing emit-model carrier whose own dissolution scope (LanguageSpec rewrite) is already tracked. Co-locating verification-only fields here also forecloses a clean future split where the emit-model carrier dissolves but the L6 projection survives.

### Option 2 — Sibling projection carrier keyed by `MethodTemplateContractKey` *(RECOMMENDED)*

A new `EmissionPathProjection` carrier consumed by the L6 walker only:

```dag
// Typed row identity for `MethodTemplateContract` rows. The source row
// identity is the pair `(list_target, dag_method)`: the same
// `MethodRef` (e.g. `count_method`) appears once per target list, so
// `MethodRef` alone is not a unique row key. The projection carrier
// names that pair structurally rather than relying on a containing
// list's name to supply the target half.
type MethodTemplateContractKey {
  target: ShapeATarget             // Rust | Python | Go (one half of the join key)
  dag_method: MethodRef             // other half — matches MethodTemplateContract.dag_method
}

type EmissionPathProjection {
  row_identity: MethodTemplateContractKey
  cells: List<EmissionCell>         // multi-cell rows union; single-cell rows carry a one-element list
}

type EmissionCell {
  connective: FormAxis              // mirrors v3_compiler::dag::TypeConnective discriminant
  behavior: BehaviorAxis            // mirrors v3_compiler::dag::Behavior discriminant
}
```

A single combined `emission_path_projections: List<EmissionPathProjection>` declaration; no per-target list partition.

- Honors **C1**: zero changes to `MethodTemplateContract` rows. Authoring carriers stay clean; verification facts live on a verification-owned carrier.
- Honors **C2**: every projection fact (`target`, `dag_method`, `connective`, `behavior`) is a typed row-local field. The L6 walker reads each row directly — no list-name → cell prose, no template-string scan.
- Honors **C3**: row population is Grounding's. The carrier ships empty in this slice; Grounding's CrossTarget-Meta lane owns the population PR and the `coverage.rs` walker conversion.
- `MethodTemplateContractKey`-keyed lookup means Verification can dispatch per-`MethodTemplateContract` row by joining the row's `(list_target, dag_method)` pair against `EmissionPathProjection.row_identity`. The join is 1:1 with the source row identity; no fan-out filtering, no string identity anywhere.

### Option 3 — Grounding row-class refactor with cell-homogeneous lists

**Rejected.** Split `*_method_template_contracts` into one list per cell (`*_method_template_contracts_cardinality_transform`, etc.) so the list name is the cell carrier.

- Fails **C2**: list-name → cell registry is exactly the prose mapping that lives in `coverage.rs::TARGET_LISTS` today, just relocated. A typo in the list name would silently mis-route. It does not eliminate name-based dispatch; it relocates it.
- Combinatorial blow-up: 3 targets × N cells of list declarations as the cross-product diversifies, and every per-target row authority forks accordingly. Today's three list declarations would scale to 3N.
- Doesn't compose with multi-cell rows (a row that legitimately covers two cells has nowhere honest to live).
- Forces target-mirror surface duplication on `MethodTemplateContract` row authoring even when only the cell projection differs.

## 3. Recommended option (Option 2)

Carrier-shape sketch: see §2 Option 2 above.

**Owner handoff if this lands:**
- **Substrate (this slice, post-sign-off):** declares `FormAxis` / `BehaviorAxis` / `ShapeATarget` / `EmissionCell` / `EmissionPathProjection` in `.dag`. Adds an `emission_path_projections` (or per-target sibling) declaration as the row-list authority. Adds a small ratchet that the carrier exists with the expected field shape and that the populated row count matches `MethodTemplateContract` row count (single-cell rows in Phase 1) once Grounding populates.
- **Grounding CrossTarget-Meta (next dispatch):** populates the projection rows for the existing 42 Phase 1 rows (all `Cardinality × Transform × <target>`), then converts `coverage.rs` from list-non-empty projection to per-row union. Keeps `MethodTemplateContract` untouched.
- **Verification:** authors no follow-up beyond confirming the converted `coverage.rs` projects honestly against mixed-cell row sets once Grounding admits a non-`Cardinality × Transform` row.

**Locality:** `MethodTemplateContract` row population stays Grounding's; nothing in this routing forces a `MethodTemplateContract` row migration.

## 4. Open sub-questions for Director sign-off

These are the reasons this receipt STOPs rather than lands code.

### 4.A — Substrate home for `FormAxis` / `BehaviorAxis` / `ShapeATarget`

These axis types currently exist only as hand-Rust enums in `src/v3/grounding_cross_target_meta/src/cells.rs`. Option 2 requires them as `.dag` `Disj` declarations. Open question: where do they live?

- **(a)** In a new dedicated file `src/v3/std/cross_target_coverage.dag` alongside the projection carrier. Keeps the verification-only L6 axes off `emit_model.dag`. Recommended on orthogonality grounds.
- **(b)** In `src/v3/std/emit_model.dag` next to `MethodTemplateContract`. Co-locates with the existing emit carrier the projection joins against. Risks blurring the C1 boundary between authoring and verification.
- **(c)** Generated mirrors of the existing `v3_compiler::dag::TypeConnective` / `v3_compiler::dag::Behavior` Rust enums via a new mirror generator. Architecturally cleanest (single authority — the substrate `Behavior` declaration) but is a larger lift and not in this dispatch's scope.

Recommendation: **(a)** for this slice; flag **(c)** as the correct dissolution target once a substrate-`Disj`-mirroring generator exists, and document it on the new file's header.

### 4.B — Multi-cell row representation

`EmissionPathProjection.cells: List<EmissionCell>` (recommended) vs splitting one row per cell.

- `List<EmissionCell>` keeps the row-identity axis (`MethodRef`) once per row and lets multi-cell rows declare all cells in one place. Consumers union across `cells`.
- One-row-per-cell duplicates the `MethodRef` and the `target` and forces the walker to deduplicate by `MethodRef`. Cleaner for trivially single-cell rows; worse for genuine multi-cell rows.

Recommendation: `List<EmissionCell>` per the sketch. Phase 1 rows declare a 1-element list; multi-cell rows do not need a different shape.

### 4.C — `target` placement: row-identity key vs cell vs per-list partition

The `MethodTemplateContract` source row identity is the pair `(list_target, dag_method)` — the same `MethodRef` appears once per target list. Three shapes are coherent:

- **(i)** `target` lives on `MethodTemplateContractKey` (the row identity tuple) (**recommended**). One global `emission_path_projections: List<EmissionPathProjection>` list. The walker reads `target` from each row's identity; the projection-row ↔ source-row join is 1:1. `EmissionCell` carries only `connective × behavior`.
- **(ii)** `target` lives on each `EmissionCell` instead, with `row_identity: MethodRef` only. One global list, but a projection row aggregates cross-target cells under a single method name. Walker must filter cells by `cell.target == list_target` per source row — fan-out join, weaker structural match between projection rows and source rows.
- **(iii)** Three per-target lists `{rust,python,go}_emission_path_projections: List<EmissionPathProjection>` mirroring today's `MethodTemplateContract` row-list shape, with `target` on the list name. Reintroduces the very list-name dispatch the projection is supposed to eliminate.

Recommendation: **(i)**. The row-identity tuple is the structurally sufficient match for the source row identity (per #1634 review), 1:1 join semantics, and `target` is still a typed row-local fact (just on the key half of the row, not on each cell). The list name carries no axis authority.

### 4.D — Grounding scope on this slice

Does Grounding populate the 42 Phase 1 rows in the substrate slice that lands the carrier, or in a follow-up Grounding PR?

- **(a)** Substrate slice ships the carrier *and* the 42 trivial `Cardinality × Transform × <target>` rows so the new ratchet can assert per-row count parity immediately. Costs Substrate ~42 rows of straightforward population.
- **(b)** Substrate slice ships the carrier empty; ratchet only asserts the carrier exists. Grounding's CrossTarget-Meta lane owns row population in a follow-up.

Recommendation: **(b)** — preserves the dispatch's "Grounding owns row population" criterion (C3) cleanly. Substrate's slice stays minimal; Grounding's slice is the routing's payoff.

## 5. Verdict

**Routing:** Option 2 (sibling projection carrier `EmissionPathProjection` keyed by `MethodRef`). Options 1 and 3 rejected — see §2.

**Code:** stops here pending Director sign-off on §4.A–§4.D. The shape is otherwise clear and ready to land in a small substrate slice plus ratchet.

**Hand-off:** Substrate authors the carrier slice once §4 lands; Grounding owns the row-population PR + `coverage.rs` conversion.

— sent from tidy-tern-769 (inbox #1288); reply at #1130
