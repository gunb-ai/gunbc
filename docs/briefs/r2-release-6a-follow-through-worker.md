# R2 Release — §6a per-method-metadata follow-through `(M, R2)`

> **R2 Release Manager dispatch.** Per [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md) Goal 5. **Pick is closed** — this brief is post-pick scope only (bulk migration + dissolution-trigger tracking). Reports to R2 Release Manager once R2 spawns; pre-spawn authoring per inbox #828 PM portion.

## Read first

- **[`docs/design-substrate-carrier-port-program.md` §6a:171](../design-substrate-carrier-port-program.md)** — **Decision:** Option 3 unified `MethodContract` carrier. **LOCKED.** Do not re-litigate.
- **[`docs/design-substrate-carrier-port-program.md` §6a:173](../design-substrate-carrier-port-program.md)** — **Live receipt:** `src/v3/std/algebra.dag` declares `MethodContract`; `src/v3/lenses/cost.dag` imports it via `method_contract_cost_shape` minimal demo consumer. The pick + minimal demo landed.
- **[`docs/design-substrate-carrier-port-program.md` §6a:175](../design-substrate-carrier-port-program.md)** — **Dissolution trigger:** `MethodContract` is a transitional carrier, not the endpoint. It dissolves field-by-field as upstream type-system facts land: `size_effect` → cardinality-refined method signatures; `cost_shape` → typed cost surface; `callback_element_position` → typed higher-order callback parameter shape.
- **[`docs/briefs/t-permethodmetadata-pick-worker.md`](t-permethodmetadata-pick-worker.md)** (landed PR #794) — the pick-worker brief that authored the lock + minimal demo. Its scope-closure clause: *"Do not migrate all consumer lenses — one demo migration is sufficient evidence for the chosen shape; **bulk migration is post-pick work**."* This brief owns that post-pick work.
- **[`src/v3/lenses/cost.dag`](../../src/v3/lenses/cost.dag)** + **[`src/v3/lenses/complexity.dag`](../../src/v3/lenses/complexity.dag)** — bulk-migration consumers. Read both to inventory current call-site lookup patterns (lookup-table reads of `size_effect` / `cost_shape` / `callback_element_position` from `*_templates()` results in `dsl/std/algebra.dag`) that `MethodContract` lookup replaces.
- **[`dsl/std/algebra.dag` lines 447-569](../../dsl/std/algebra.dag)** — current v2 metadata authority via `*_templates()` functions. The migration retires lens-side direct reads of these tables in favor of `MethodContract`-keyed lookup at call sites.
- **[`docs/escalation-paths.md`](../escalation-paths.md)** — escalation channel + decision-artifact discipline; Director resolution receives via session-inbox issue comment.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame — post-pick bulk migration + dissolution-trigger tracking

The pick is closed (Option 3 `MethodContract` per §6a:171). The minimal demo landed (`method_contract_cost_shape` consumer in `cost.dag` per §6a:173). **What remains:**

1. **Bulk migration** of all `cost.dag` / `complexity.dag` call sites that read `size_effect` / `cost_shape` / `callback_element_position` directly from `*_templates()` lookup tables. After migration, those reads go through `MethodContract` keyed by `(algebra_id, method_id)` per §6a:171's structural identity discipline.

2. **Dissolution-trigger tracking** for the three carrier fields. `MethodContract` is transitional per §6a:175 — each field has a named upstream-fact landing condition that retires it. This brief tracks those triggers as named ROADMAP debt rows so dissolution is visible (rather than implicit) and so the eventual retirement of `MethodContract` (when all three triggers fire) is a structural event with paired-dispatch discipline rather than ad-hoc cleanup.

The migration is **not** a re-pick. The carrier shape is locked; this is consumer migration of the decided shape.

## Three consumer-side requirements

1. **Inventory current consumption.** Read `cost.dag` + `complexity.dag` end-to-end. Identify every call site that reads `size_effect`, `cost_shape`, or `callback_element_position` from `*_templates()` results in `dsl/std/algebra.dag`. Tabulate sites + which fields are read at each. **Acceptance:** site inventory in PR description; no migration without first knowing the surface.

2. **Bulk-migrate to `MethodContract` lookup.** For each inventoried site, replace the lookup-table read with `MethodContract`-keyed lookup via `(algebra_id, method_id)`. Migration must preserve behavior — gate verification: `cargo test --workspace --exclude v2-compiler-tests` clean; `complexity_merge_sort_is_nlogn` and `complexity_merge_sort_v3_matches_v2_oracle` gates remain green; DB-8 fixed-point converges bit-identically. **Acceptance:** all sites migrated; `*_templates()` reads of these three fields no longer appear in `cost.dag` / `complexity.dag`.

3. **Track field-by-field dissolution triggers.** Add a ROADMAP row (under `## Tracked debts` or analogous) naming each of the three carrier fields + its dissolution trigger from §6a:175:
   - `size_effect` dissolves when collection-cardinality-refined method signatures land.
   - `cost_shape` dissolves when typed cost surface / structural cost derivation lands.
   - `callback_element_position` dissolves when typed higher-order callback parameter shape lands.

   Each trigger references the upstream substrate work that lands the type-system fact (cardinality refinement; typed cost; HOC parameter shape). When all three trigger conditions fire, `MethodContract` retires. **Acceptance:** ROADMAP rows authored; dissolution triggers cite specific upstream lanes / proposals where named (or "post-R2 capability lane TBD" for the un-named).

## Inventory receipt (HEAD `main` post-#1208; post-#1175 template vs `MethodContract` discipline)

**Audit method:** Read `src/v3/lenses/cost.dag` + `src/v3/lenses/complexity.dag` end-to-end; search for `size_effect`, `cost_shape`, `callback_element_position`, `MethodContract`, and `*_templates(` / template-table access patterns named in requirement 1.

| Consumer file | Reads of `size_effect` / `cost_shape` / `callback_element_position` from `*_templates()`-style lookup in `dsl/std/algebra.dag` | Notes |
|---|---|---|
| `src/v3/lenses/complexity.dag` | **None (0 sites).** | Structural integer-depth lens only; no `std.algebra` method-template metadata path. |
| `src/v3/lenses/cost.dag` | **None (0 sites).** | §6a **demo** accessor only: `method_contract_cost_shape(contract: MethodContract) -> CostShape? = contract.cost_shape` — reads `cost_shape` off the **unified carrier** (`src/v3/std/algebra.dag` `MethodContract`), not off `dsl/std/algebra.dag` template tables. |

**Implication for requirement 2:** There is **nothing to mechanically replace** in these two files today: no lens-local `*_templates()` reads of the three fields exist here. **Live call-site** `MethodContract` lookup through `Transform` / call-pattern lowering (when cost analysis needs per-method facts on real call edges) remains **future wiring**, not a table-to-carrier rename inside current `.dag` bodies.

**Implication for requirement 3:** ROADMAP dissolution-trigger rows are still **required** by this brief (PR-3 / follow-up); they are **not** implied complete by an empty migration surface in the two lens files.

**Scope boundary (widen-later, RM review):** Broader §6a migration narratives elsewhere name additional lenses (e.g. `idempotency.dag`, `parallelism.dag`). **This receipt covers requirement 1's two named consumer files only** — it is **not** a repo-wide "no `*_templates()` migration surface" claim. Before widening bulk migration, **extend inventory** to any newly in-scope `.dag` modules and tabulate those sites explicitly.

## Slice — inventory → migrate → track

1. Read `cost.dag` + `complexity.dag` end-to-end; inventory consumption sites for the three carrier fields.
2. PR-1: site inventory + minimal-risk migration (3-5 sites; verify behavior preservation; full gate green).
3. PR-2 (this brief or follow-up worker): remaining bulk migration; full retirement of lens-local lookup-table reads.
4. PR-3 (this brief): ROADMAP `## Tracked debts` row authoring with three named dissolution triggers + upstream-lane citations.

Land as 1-3 PRs at the migrating-worker's discretion based on site count + behavioral risk. Single PR is acceptable if the worker can verify the full surface in one review cycle.

## Acceptance

- [ ] Site inventory captured in PR-1 description (consumer call sites by field).
- [ ] `cost.dag` + `complexity.dag` migrated to `MethodContract`-keyed lookup; lens-local reads of `size_effect` / `cost_shape` / `callback_element_position` from `*_templates()` retired.
- [ ] All R1 gates remain green (`complexity_merge_sort_is_nlogn`, `complexity_merge_sort_v3_matches_v2_oracle`, `lane_e_bundled_witness_host_emit_parity`).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] ROADMAP `## Tracked debts` rows authored for the three field-by-field dissolution triggers, each citing the upstream lane / proposal that lands the trigger condition (or "TBD post-R2" for un-named).

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md) escalation discipline (channel: GitHub session-inbox issue comment; Director's decision artifact format = amendment PR or sibling brief):

- **Bulk migration reveals a `MethodContract`-shape gap** — i.e., the carrier as declared at `src/v3/std/algebra.dag` cannot accommodate a real consumer pattern → STOP. Surface the gap; do not extend the carrier without R2 Release Manager + Substrate Manager design call. The pick may need amending.
- **Bulk migration reveals an algebra-id / method-id pair that has no `MethodContract` row** → STOP. The pick worker authored the minimal demo consumer; absent rows for non-demo algebras are an authority gap that must be surfaced (rows authored where? — depends on whether `MethodContract` rows are declared per-algebra in `algebra.dag` or generated). Surface for design clarification before continuing.
- **Behavioral regression on R1 gates** (`complexity_merge_sort_is_nlogn` / `complexity_merge_sort_v3_matches_v2_oracle` / `lane_e_bundled_witness_host_emit_parity`) → STOP immediately. R1 gate green is non-negotiable; revert + diagnose before re-attempting.
- **DB-8 fixed-point drifts** → STOP immediately. DB-8 gate is concrete (`m2_substrate_inhabitance_test`).
- **Upstream dissolution-trigger lane is mis-cited** — i.e., the brief claims a trigger condition that doesn't match any active lane on main → STOP at the ROADMAP-row authoring step. Verify against current ROADMAP state before authoring; pre-author audit per `feedback_audit_adjacent_authority_first`.

## Cross-refs

- Parent: [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md) (Goal 5 owner).
- Pre-pick authority: [`docs/briefs/t-permethodmetadata-pick-worker.md`](t-permethodmetadata-pick-worker.md) (landed PR #794; pick + lock + minimal demo).
- Design-call authority: [`docs/design-substrate-carrier-port-program.md` §6a](../design-substrate-carrier-port-program.md) (decision locked at `:171`; live receipt at `:173`; dissolution trigger at `:175`).
- Carrier authority: [`src/v3/std/algebra.dag`](../../src/v3/std/algebra.dag) (`MethodContract` declaration).
- Consumer authority: [`src/v3/lenses/cost.dag`](../../src/v3/lenses/cost.dag) + [`src/v3/lenses/complexity.dag`](../../src/v3/lenses/complexity.dag).
- v2 metadata source (current): [`dsl/std/algebra.dag` lines 447-569](../../dsl/std/algebra.dag) — `*_templates()` lookup tables being retired.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
