# T-V-L5-Corpus Readiness Audit (Research)

**Status:** PROPOSAL — research-only readiness artifact. Does **not** dispatch implementation, edit upstream authority files, or introduce substrate/fixtures.

**Parent dispatch:** Director [#828](https://github.com/gunb-ai/gunbc/issues/828) — Lane 2 audit to bridge standby ([`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) + [`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md)) to a **concrete worker brief** when prerequisites land.

**Cross-refs (read-only):** [`docs/r3-structure.md`](../r3-structure.md) (`l5_cross_target_consistency`, Verification critical path T-V-L4-L7-Direct → T-V-L5-Corpus), [`docs/briefs/r2-grounding-manager.md`](r2-grounding-manager.md) (grounding ledger — not edited here).

---

## 1. Shape A grounding state on HEAD (qualitative)

This section summarizes **what is structurally present in-tree today** versus what L5 needs for **cross-target algebraic receipts** (not byte identity — per [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)).

| Axis | Landed enough for L5 *staging* thought experiments? | Gap vs strict L5 dispatch |
|------|-----------------------------------------------------|---------------------------|
| **R2-Grounding-Rust / Python / Go** (primitive declarations) | Partial — per [`r2-grounding-manager.md`](r2-grounding-manager.md) §"Owned deliverables": **T-Ground-Rust/Python/Go** all **PARTIALLY LANDED** (registry / primitives tranches on main); none are stamped “lane closed” for full Shape A. | Full **three-target** primitive/capability parity for programs L5 will run is still cadence-gated (PR-F/G/H/I narrative in that brief). |
| **R2-T-Ground-LanguageSpec** | Phase 1 registry rows exist (Rust / Python / Go counts cited in manager brief); Phase 2 / cost / higher-order rows still partial. | **Comparable observation types** across targets need the LanguageSpec + emit surfaces to agree on how `Int`/`Bool`/records materialize at runtime boundaries. |
| **Emit production (Rust / Python / Go)** | **Strong:** integration determinism tests (`emit_matrix_program_*`, `emit_matrix_module_*` in `src/v3/compiler/tests/determinism_test.rs`) demonstrate **deterministic emission** for program + module shapes across three targets on pinned fixtures. | Deterministic emit ≠ **L5 harness**: no unified runner path yet ties **same `TestClaim.source`** → three emits → three runs → **one algebraic equivalence** judgement (see §2). |

**Bottom line:** Roughly **~40–50% “structural runway”** for L5 *thinking* — emit triple + substrate scaffolding exist; **the missing half** is grounding closure semantics + **runner-mediated cross-target observation**, not more `.dag` prose.

---

## 2. `ForAllTargets` runner-extension state (code audit)

**Substrate:** `TestPredicate::ForAllTargets` remains a **🟡 scaffold** in [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) (fields: `command`, `args`, `expect_exit_code`) with dissolution notes tying it to `ExecuteCommand` capability typing.

**Rust runner:** [`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs) `eval_claim_by_predicate` matches explicit arms for `ExecuteCommand`, `DifferentialEquals`, etc., and has **no `ForAllTargets` arm**. Unknown labels resolve to:

`NotYetImplemented("TestPredicate::<name> is not wired in the Rust runner yet")`.

**Conclusion:** Neither Substrate nor Evaluator has **absorbed** L5 execution — the finding from cool-crab [#1307](https://github.com/gunb-ai/gunbc/pull/1307) / [#828](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4355916053) still holds on HEAD.

**Implementation path (spec-only):** extend the runner with a **single producer** that (for a frozen target set from the grounding ledger) performs **per-target emit → compile/run → parse observation → algebraic compare**, without introducing a new `TestPredicate` variant (per [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) substrate-fact-introduction procedure). The raw command triple in the current `ForAllTargets` scaffold is **insufficient** for strict L5 (exit-code checks ≠ value observations) — aligns with [`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md) §"ForAllTargets Runner Path".

**Coordination:** Same runner-extension class as Lane 1’s `DifferentialEquals` producer maturity; surface shared blockers to Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276) if Lane 1 readiness audit overlaps.

---

## 3. Cross-target seed corpus (`add_then_branch`)

[`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md) §"Seed Corpus Shape" remains valid on current Shape A posture:

- Exercises **fn + match + `Int` bind** without IO, effects, floats, or host-lib portability cliffs.
- **Portability watch:** once three targets are concrete, regressions will cluster around **match lowering**, **call ABI**, and **integer literal boundaries** — not around the seed’s algebraic intent. Fallback `let l5_out: Int = 1 + 2` remains the documented narrow seed if branch lowering diverges per target.

---

## 4. Critical-path consumption from Lane 1 (boundary)

Authoritative split is unchanged ([`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md) **No L5-absorbs-L4** + scaffold notes §"Critical-Path Consumption From Lane 1"):

**Lane 2 imports**

- Corpus **program source** (same text authority as Lane 1 rows).
- **Fixture/module naming** + **stable claim names** (stability invariant in Lane 1/2 worker briefs).
- **Output-bind convention** (e.g. named `let` for observable `Int`).
- **Classification metadata** once Lane 1 attaches it to corpus rows.

**Lane 2 does *not* import**

- `DifferentialEquals` receipts, evaluator outputs, or target-vs-eval pass/fail artifacts.
- L7 algebraic-law witness rows as inputs to L5 comparison logic.

**Structural import path (when Lane 1 lands):** add rows to the proposed Lane 2 fixture [`src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag`](../../src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag) (path matches [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md)) while preserving **P2 single-authority discipline** for program text: prefer, for steady-state dispatch, (**a**) **shared `.dag` corpus module / declaration import** from Lane 1’s artifact (single compiled authority), (**b**) **generated** L5 rows materialized from that source with an explicit equality ratchet in CI, or (**c**) another Director-approved mechanism that cannot silently fork into a second editable copy. Treat **new** Rust `include_str!` corpus lifts as **transitional** only while distribution-map bridge **#4** (`include_str!` side channels) remains **open** on the T-Bridge-Retirement closure ledger — see **Authority note** immediately below — otherwise route through (a) or (b). **Do not** maintain Lane 1 program source as independently hand-edited duplicate prose in the L5 fixture. Lane 2 still does **not** read L4 proof objects.

**Authority note (bridge #4 vs §4):** Canonical map + gate live in [`docs/r3-structure.md`](../r3-structure.md) §**T-Bridge-Retirement** (five named bridges; **#4** = `include_str!` side channels — PB-owned retirement slices; Verification owns unified **`bridge_retirement_ledger_zero`** — [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)). Row-level status: **`bridge_include_str_side_channels_retired`** in [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md). **Enforcement path:** worker obeys live ledger + PB emission-model posture ([`docs/design-emission-model.md`](../design-emission-model.md)); no independent L5 policy fork. **Dissolution trigger:** bridge **#4** structurally retired → §4 **drops** the transitional `include_str!` carve-out for **new** certification corpus lifts (steady state is (a)/(b) only).

---

## 5. Slice-1 dispatch-ready spec (worker handoff)

When gates fire, **Slice 1** should:

1. **Fixture:** `src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag`, suite `r3_verification_l5_corpus_suite`, gate name `l5_cross_target_consistency` per [`docs/r3-structure.md`](../r3-structure.md) and [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md).
2. **Seed row:** one `TestClaim` with `TestPredicate::ForAllTargets` (existing substrate) over **`add_then_branch`** seed; program text must be **provably identical** to the Lane 1 seed via the same single-authority mechanism as §4 (reference or generated copy + ratchet), not a free-standing duplicate string maintained only for L5.
3. **Observation domain:** normalize per-target outputs to **`Int` / `Bool` / simple records** before equivalence (scaffold notes §"Concrete Observation Contract").
4. **Producer shape:** per frozen target — **emit (`Dag` constant)** → **target compile** → **target run** → **capture named bind** → **parse to structural value** → **algebraic equality** across targets.
5. **Failure taxonomy** (non-exhaustive staging list): emit failure → per-target compile failure → run failure → observation parse failure → **cross-target mismatch**. Optional oracle text is **not** L5 authority (would blur into L4).

**Explicit non-claims:** No L6 absorption; no “L5 dissolves L4”; no new predicate variants; no fixture authoring in this audit PR.

---

## 6. Open inputs for parallel Lane 1 readiness audit

- Shared **runner-extension** scope (`ForAllTargets` vs `DifferentialEquals` producers).
- Whether Lane 1’s first corpus artifact exposes **stable module boundaries** so L5 can **reference** program identity (or feed a generator) **without** a second authoritative copy of source text.

Reply coordination: Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
