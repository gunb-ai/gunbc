# Lane 1e-3v — Verified Phase 3 dispatch gate (post-#616)

**Purpose:** Replace the speculative `docs/emit-target-spec-gaps.md` (2026-04-20 snapshot) with a **re-verified** map of Lane 1e clusters against **live** `src/v3/spec/*.dag`, `src/v3/std/computation_model.dag`, and emitter sources after **#616** (Path A / Class 5 Gap 1 — Bool → BooleanAlgebra grounding for logical operators).

**Status:** Analysis only — no emitter refactor in this brief.

**Authorities read:** `docs/single-emitter-design.md`, `docs/emit-target-spec-gaps.md`, `docs/emit-bridges.md`, `src/v3/compiler/src/emit.rs`, `src/v3/compiler/src/emit/rust_target.rs`, `src/v3/compiler/src/emit/python_target.rs`, `src/v3/spec/{rust,go,python}.dag`, `src/v3/std/computation_model.dag`.

### PR review ingest (#621, 2026-04-21)

**Review A (claude / claude-opus-4-7, schedule)**

- **Verdict:** APPROVE — api-review spot-checks matched this brief: no `OperatorKind::Logical` under `emit/`; emitter `behavior_result_port` / `go_behavior_result_port` and both `port_is_consumed_from` bodies are byte-identical as stated; duplication is correctly scoped as implementation DRY, not a `.dag` gap.
- **Queued follow-up (optional, not a finding on #621):** Two other `behavior_result_port` definitions exist outside the emit pair:
  - `src/v3/compiler/src/lens_cost_symbolic_generated.rs` — **generated** from the cost lens (`src/v3/lenses/cost.dag`); treat as codegen output; any shared helper likely flows from the lens pipeline, not from a one-off edit to the `.rs` file.
  - `src/v3/compiler/src/dimension.rs` — **hand-authored** third copy (same match on `Behavior` variants). **Phase 3.0** below stays **emit-only** (unify the two emitter copies). A later refactor could move `behavior_result_port` to a small `crate` or `dag` helper and wire **emit + dimension** (and regenerate/consolidate the lens copy per policy) so a “one shared helper” pass does not stop at `rust_target.rs` / `emit.rs` while leaving `dimension.rs` behind.

**Review B (codex / gpt-5.4, schedule)**

- **Verdict:** APPROVE_WITH_COMMENTS — brief is narrowly scoped; no modeling-discipline issue in the doc itself.
- **Comment ingested:** Phase 3.0’s test plan must not rely **only** on broad emit / determinism / golden suites. Per **`TESTING.md`** (unit-first, behavior-driven, minimal-constructed inputs), the implementation PR for Phase 3.0 should add **at least one focused regression test** that exercises the shared `behavior_result_port` and/or `port_is_consumed_from` directly against a **minimal `Dag` shape** (or other hermetic fixture) that pins the structural graph-walk contract. Keep existing integration/golden runs as a **belt** (DB-8), not the sole proof for a small helper refactor.

**Review C (human / director, #621 thread)**

- **Verdict:** Converged — brief acts as a **real dispatch gate**, not another speculative audit: it reclassifies old paper gaps, narrows the true remaining emitter-gap surface, and names a **concrete deletion-oriented** Phase 3.0 tranche (`behavior_result_port` / `port_is_consumed_from`) instead of only gesturing at a future walker.
- **Direction locked:** Phase 3.0 is a **small graph-walk dedup** refactor; **unit-first** coverage (focused regression on shared helper behavior or minimal `Dag`) is the right bar — same requirement as **Review B** and §**Tests (required)** below.

**Review D (claude / claude-opus-4-7, schedule, commit `73e3b628`)**

- **Verdict:** APPROVE — documentation-only PR; dispatch-gate brief is appropriate scope.
- **Spot-checks (worktree at review time; line numbers drift):** No `OperatorKind::Logical` under `src/v3/compiler/src/emit/`; `behavior_result_port` / `go_behavior_result_port` in `rust_target.rs` / `emit.rs` byte-identical modulo name; `port_is_consumed_from` duplicated in those two files; third hand-authored copy in `dimension.rs` called out with Phase 3.0b follow-up — matches **Review A** ingest.
- **Discipline:** Phase 3.0 implementation must satisfy **`TESTING.md` unit-first** bar via §**Tests (required)** (not belt-only integration); `dimension.rs` tracked debt is documented, bounded, and has a named dissolution trigger — consistent with tracked-debt pattern.
- **No violations** of INVARIANTS / CODING / TESTING in the doc; Cluster D classified as implementation DRY (not substrate / `.dag` gap); STOP on reviving `LogicalOperatorCarrier` / `TypeRecursionStrategy` as paper carriers — affirmed.

---

## Executive summary

- **#616 closes the Cluster F modeling gap** that #608/#610 misclassified: there are **no** `OperatorKind::Logical` bypass branches left under `src/v3/compiler/src/emit/`; logical ops route through `algebra_field_for_operator` + `operator_carrier_realization` like arithmetic ops, with per-target `OperatorRealization` rows (e.g. `python_bool_meet` / `python_bool_join`).
- **Clusters B + C from the old Category 2 list are implemented and consumed:** `TargetExecutionModel`, `SourceFiltering` / `*_source_filtering`, and `ExecutionModelRequirement` data rows exist; emitters filter declarations via `SourceFilteringBinding` (see `emit.rs` Go path, `rust_target.rs`, `python_target.rs`).
- **Cluster A** is **not** a missing `TypeRecursionStrategy` row in practice: Go type names are built from **`TypeRealization` / instantiation carriers** and `type_applications.optional` (see `emit.rs` `go_type_name_for_decl_at_depth`). Remaining cost is **hand-authored recursive walk** in Rust until a walker owns it — same shape as other type-render paths.
- **Cluster D** is still accurate as **code dedup**, not a spec gap: **`port_is_consumed_from`** is duplicated between `emit.rs` (Go) and `rust_target.rs` (Rust); **`behavior_result_port` vs `go_behavior_result_port`** are byte-identical duplicates.
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
| **F** — Logical `&&` vs `and` | MISSING_SPEC_ROW | **Already covered (post-#616)** | No logical-op special case in emitters; inference + `OperatorRealization` rows. |
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

- **Cluster F** — **Closed by #616** (see above).
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

**Non-goals:** No walker architecture, no spec `.dag` edits, no change to optional/Go sum rendering. **Out of scope for Phase 3.0:** `dimension.rs` and `lens_cost_symbolic_generated.rs` — see **PR review ingest** above; absorb in a follow-on if a crate-visible helper is introduced.

**STOP-AND-ESCALATE:** If unification requires different `Loop` result-port treatment per target — **report** (would imply the two copies were not actually equivalent — today they match structurally).

---

## Relation to `single-emitter-design.md` Phase 3 (TCO / blocks)

That document’s **Phase 3** refers to **v2** `05_emit*.dag` TCO unification — **orthogonal** to Lane 1e’s **v3** walker phases. Do not merge the two roadmaps without an explicit porting brief; this artifact is **v3 Lane 1e** only.

---

## Dispatch checklist (next worker)

- [ ] Do **not** reopen Cluster F or propose `LogicalOperatorCarrier` without a new substrate gap.
- [ ] Treat **B, C, F, G, H, I, J** as **done** from a spec-gap perspective; remaining work is **walker + DRY + Go optional strings**.
- [ ] Start Phase 3 implementation with **Phase 3.0** brief above unless director reprioritizes Go optional templating; Phase 3.0 PR must include **unit-first** helper tests per **Review B** and `TESTING.md`, not integration-only.
- [ ] After Phase 3.0, optionally schedule **Phase 3.0b** — dedupe `behavior_result_port` with `dimension.rs` (and lens codegen path) if a shared `crate::dag`-level or `crate` helper is justified; do not expand Phase 3.0 scope mid-flight.
