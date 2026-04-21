# Lane 1e-3v — Verified Phase 3 dispatch gate (post-#616)

**Purpose:** Replace the speculative `docs/emit-target-spec-gaps.md` (2026-04-20 snapshot) with a **re-verified** map of Lane 1e clusters against **live** `src/v3/spec/*.dag`, `src/v3/std/computation_model.dag`, and emitter sources after **#616** (Path A / Class 5 Gap 1 — Bool → BooleanAlgebra grounding for logical operators).

**Status:** Analysis only — no emitter refactor in this brief.

**Authorities read:** `docs/single-emitter-design.md`, `docs/emit-target-spec-gaps.md`, `docs/emit-bridges.md`, `src/v3/compiler/src/emit.rs`, `src/v3/compiler/src/emit/rust_target.rs`, `src/v3/compiler/src/emit/python_target.rs`, `src/v3/spec/{rust,go,python}.dag`, `src/v3/std/computation_model.dag`.

**Reading order (handoff):** **Executive summary** → **Per-cluster verdict** → § **Phase 3.0** → **Dispatch checklist** — this file is **live dispatch guidance** only.

**PR #621 audit trail (archival):** **[`1e-3v-pr621-review-ingest.md`](1e-3v-pr621-review-ingest.md)** — api-review transcripts (**Reviews A–Y**), **three** blocking inline rebuttals, and **Review Z** (receipt that the chronicle was split out per **`INVARIANTS.md`** “Documentation Describes Live State,” codex `bc6bf2c8`).

---

## Executive summary

- **#616 closes the Cluster F modeling gap** that #608/#610 misclassified: there are **no** `OperatorKind::Logical` bypass branches under `src/v3/compiler/src/emit/`; **all** binary ops (including logical) go through **`render_operator` → `algebra_field_for_operator` → `operator_carrier_realization`**, which **reads carriers from indexed `OperatorRealization` rows** in `spec/*.dag` (e.g. `python_bool_meet` / `python_bool_join`, `rust_bool_*`, `go_bool_*`).
- **Clusters B + C from the old Category 2 list are implemented and consumed:** `TargetExecutionModel`, `SourceFiltering` / `*_source_filtering`, and `ExecutionModelRequirement` data rows exist; emitters filter declarations via `SourceFilteringBinding` (see `emit.rs` Go path, `rust_target.rs`, `python_target.rs`).
- **Cluster A** is **not** a missing `TypeRecursionStrategy` row in practice: Go type names are built from **`TypeRealization` / instantiation carriers** and `type_applications.optional` (see `emit.rs` `go_type_name_for_decl_at_depth`). Remaining cost is **hand-authored recursive walk** in Rust until a walker owns it — same shape as other type-render paths.
- **Cluster D** is still accurate as **code dedup**, not a spec gap: **`port_is_consumed_from`** is **structurally** duplicated between `emit.rs` (Go) and `rust_target.rs` (Rust) — same graph walk, differing only where each calls `go_behavior_result_port` vs `behavior_result_port` on loop bodies until unified; **`behavior_result_port` / `go_behavior_result_port`** themselves are byte-identical **modulo** the function name.
- **True remaining emitter-gap surface** for Lane 1e Phase 2+ is **narrow**: expression-level optional lowering for **Go** (`render_optional_branch` nil / deref strings) is still handwritten; everything else from the old “5 clusters” list is either **already covered** or **residual per-target** (see Category 3 below).

**STOP (director):** If a future audit revives **`LogicalOperatorCarrier`** or **`TypeRecursionStrategy`** without new evidence — those were **paper carriers** over existing authorities (#610, 1e-2b).

---

## Per-cluster verdict (labels)

| Cluster | Old `emit-target-spec-gaps` bucket | Label after live verification | Notes |
|--------|-------------------------------------|------------------------------|-------|
| **A** — Go container / optional type recursion | MISSING_SPEC_ROW | **Already covered (spec + indexes); algorithm still handwritten** | `go_type_name_for_decl_at_depth` uses instantiation carriers + optional template — not a missing row. Walker absorbs the match, not new `.dag` shape (unless open question in gap doc §307 bites). |
| **B** — Execution model branch | MISSING_SPEC_ROW | **Already covered** | `TargetExecutionModel` parsed in each target; `MemoryModel::OwnershipBased` drives Rust ownership path (`is_ownership_based`). Coarser `ExecutionModelRequirement` exists in spec for future walker-only dispatch (`computation_model.dag` notes it). |
| **C** — Bootstrap / stdlib path filtering | MISSING_SPEC_ROW | **Already covered** | `SourceFiltering` + per-target `*_source_filtering` consumed at index build. |
| **D** — Pattern binding liveness | Category 2 → dedup | **Still missing DRY (implementation debt)** | Shared **graph** fact; should be **one** `fn port_is_consumed_from(dag: &Dag, …)` (+ shared `behavior_result_port`). Not a `.dag` extension. |
| **E** — Optional type / expr rendering | MISSING (split in 1e-2b) | **Split** | **Wrapper** path: **covered** (`type_applications.optional`). **Expression** path for Go optional branch: **still handwritten** in `emit.rs` (`render_optional_branch`). |
| **F** — Logical `&&` vs `and` | MISSING_SPEC_ROW | **Already covered (post-#616)** | **Verified:** unified `render_operator` + `operator_carrier_realization` on all targets; Bool rows `*_bool_meet` / `*_bool_join` in `spec/*.dag`; no `OperatorKind::Logical` in `emit/` (**rg**-empty). **Blocking** inline rebuttals (three threads, `render_operator` proof, `rg` table) — [`1e-3v-pr621-review-ingest.md`](1e-3v-pr621-review-ingest.md). |
| **G** — Callable dispositions | Misclassified | **Already covered** | `ParameterDisposition` / shared schema — as in gap doc. |
| **H** — Unused pattern bindings | MISSING_SPEC_ROW | **Already covered** | `PatternBindingRule` / clean emission contract (e.g. Go underscore elision). |
| **I** — Variant payload field access | Misclassified | **Already covered** | `PatternBindingRule` / `clean_emission.dag`. |
| **J** — Variant ctor ordering | Misclassified | **Already covered** | Per-target ctor templates in spec. |

---

## Category 2 — “Real remaining gaps” vs noise

**Real remaining gaps (emitter-spec / walker, not modeling):**

1. **Go optional expression emission** — `emit.rs` `GoCtx::render_optional_branch` encodes nil checks and `*({scrutinee})` style projection as **Rust format strings**. Could become template rows **or** stay a **narrow per-target hook** (Category 3-style) if we decide nil semantics are intrinsic to Go.

2. **Unused coarse row** — `*_execution_requirement` data exists; emitters still branch on full `TargetExecutionModel`. Low priority until walker wants a single enum.

**Not Category 2 (do not spec-row blindly):**

- **Cluster F** — **Closed by #616** (Executive summary bullet; ingest rebuttals in [`1e-3v-pr621-review-ingest.md`](1e-3v-pr621-review-ingest.md)).
- **Cluster A `TypeRecursionStrategy`** — **Superseded** by existing instantiation + optional syntax bindings unless open question §307 proves template abstraction leaks.

---

## Category 3 — Residual per-target (unchanged)

Aligned with `emit-target-spec-gaps.md` §233–244:

| Item | Still accurate? | Blocker |
|------|-----------------|--------|
| Rust ownership pipeline | Yes | LS-4 / Track 2 |
| Go module / import ordering | Yes | Narrow hook |
| Go `Behavior::Loop` unsupported | Yes | Target capability |
| Python indentation | Yes | Intrinsic syntax |

---

## Concrete paths to **delete or collapse** in Phase 3 (single walker)

**Whole-file deletion (end state — Phase 4 in gap doc, not first tranche):**

- `src/v3/compiler/src/emit/rust_target.rs` (~5k+ LOC) — monolithic Rust emitter body.
- `src/v3/compiler/src/emit/python_target.rs` — monolithic Python emitter body.
- Go inline surface inside `src/v3/compiler/src/emit.rs` — `GoCtx` + helpers (`go_type_name_for_decl_at_depth`, `render_optional_branch`, `render_sum_branch`, …).

**First-tranche deletion-oriented targets (small, safe, no semantic change):**

| Location | What disappears | Why |
|----------|-----------------|-----|
| `rust_target.rs` — `fn behavior_result_port` | **Delete** | Duplicate of `emit.rs` `go_behavior_result_port` — unify to **one** `emit::behavior_result_port` (or move to `emit/mod` helper). |
| `emit.rs` — `fn go_behavior_result_port` | **Delete** | Same as above. |
| `rust_target.rs` — `fn port_is_consumed_from` | **Delete** | Replace with shared helper used by Go + Rust paths (same graph walk; only `behavior_result_port` name differed — gone after unification). |
| `emit.rs` — `fn port_is_consumed_from` (Go `impl`) | **Delete** | Same. |

**Net:** ~90–120 lines of **duplicate** graph logic removed, **zero** output change — ideal ratchet before larger walker extraction.

---

## One deletion-oriented Phase 3 brief (first walker tranche)

**Title:** `Phase 3.0 — Deduplicate emit port-liveness + loop-result helpers (Cluster D closure)`

**Scope (strict):**

1. Introduce **one** module-level `fn behavior_result_port(behavior: &Behavior) -> PortId` in the emit module tree (e.g. `emit.rs` or `emit/helpers.rs`), used everywhere.
2. Introduce **one** `fn port_is_consumed_from(dag: &Dag, root: PortId, target: PortId) -> bool` using that helper for `Behavior::Loop` traversal — **identical** to current semantics in both copies today.
3. **Delete** the duplicate `behavior_result_port` / `go_behavior_result_port` and both `port_is_consumed_from` method bodies.
4. **Tests (required):**
   - **Unit-first (`TESTING.md`):** At least one **focused** test that calls the shared helpers on a **minimal constructed `Dag`** (or smallest hermetic fixture) and asserts the graph-walk behaviors that matter (e.g. `behavior_result_port` matches each `Behavior` variant’s result port; `port_is_consumed_from` reaches / does not reach a payload port across a small `Branch` / `Transform` / `Loop` spine). One claim per test where practical.
   - **Regression belt:** Existing emit / determinism / golden coverage stays green — output **byte-identical** (DB-8).

**Non-goals:** No walker architecture, no spec `.dag` edits, no change to optional/Go sum rendering. **Out of scope for Phase 3.0:** `dimension.rs` and `lens_cost_symbolic_generated.rs` — see context in [`1e-3v-pr621-review-ingest.md`](1e-3v-pr621-review-ingest.md) (**Reviews A/P**); absorb in **Phase 3.0b** if a crate-visible helper is introduced. For the **lens** file: change flows from **`src/v3/lenses/cost.dag`** (or the project’s lens regen entrypoint) — **regenerate** output; do **not** hand-edit `lens_cost_symbolic_generated.rs` (**Review P**).

**STOP-AND-ESCALATE:** If unification requires different `Loop` result-port treatment per target — **report** (would imply the two copies were not actually equivalent — today they match structurally).

---

## Relation to `single-emitter-design.md` Phase 3 (TCO / blocks)

That document’s **Phase 3** refers to **v2** `05_emit*.dag` TCO unification — **orthogonal** to Lane 1e’s **v3** walker phases. Do not merge the two roadmaps without an explicit porting brief; this artifact is **v3 Lane 1e** only.

---

## Dispatch checklist (next worker)

- [ ] Do **not** reopen Cluster F or propose `LogicalOperatorCarrier` without a new substrate gap.
- [ ] Treat **B, C, F, G, H, I, J** as **done** from a spec-gap perspective; remaining work is **walker + DRY + Go optional strings**.
- [ ] Start Phase 3 implementation with **Phase 3.0** brief above unless director reprioritizes Go optional templating; Phase 3.0 PR must include **unit-first** helper tests per **Review B** and `TESTING.md`, not integration-only.
- [ ] After Phase 3.0, optionally schedule **Phase 3.0b** — dedupe `behavior_result_port` with `dimension.rs` (and lens codegen path) if a shared `crate::dag`-level or `crate` helper is justified; do not expand Phase 3.0 scope mid-flight; any change to **`lens_cost_symbolic_generated.rs`** must be via **lens/regen**, not a direct edit to generated Rust (**Review P**).
