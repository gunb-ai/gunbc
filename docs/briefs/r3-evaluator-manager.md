# R3 Evaluator Manager Brief

**Status:** LIVE — **R3 close / Gap 3** (PR #3013 §1 Gap 3 refined at 97cfb9d4c). **Session:** R3 Evaluator Mgr (neat-heron-793), spawned under Director per operator §4 sub-item 5 ratification (2026-05-13).

**R2-era program archive (design history + PR-A..E tables):** [`r2-evaluator-manager.md`](r2-evaluator-manager.md). **Do not duplicate** that file’s long-form cadence tables here; amend there only when a fact is **R2-historical** or shared across R2/R3 readers.

## Orient before reading

- **Gap 3 authority:** five sub-lanes in [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §Evaluator Manager — `runtime_value_model_structural`, `body_evaluator_structural`, `lens_application_complete_reflection`, `witness_construction_structural`, `cross_target_equivalence_harness_structural`. **HEAD state** is refreshed in that section (2026-05-13); this brief names dispatch priorities, not a second ledger.
- **R3 implementation dispatch:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) — E1–E6 slices, STOP+PING, runner dissolution discipline.
- **Closure residuals (docs-only):** [`r2-evaluator-closure-residuals.md`](r2-evaluator-closure-residuals.md).
- **Verification matrix:** [`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) — widen / forbid surfaces; update row text when stale vs `main`.
- **Phase audits:** [`../audit/r3-evaluator-phase4-audit-handoff.md`](../audit/r3-evaluator-phase4-audit-handoff.md), [`../audit/r3-evaluator-phase5-post-e3-closure-handoff.md`](../audit/r3-evaluator-phase5-post-e3-closure-handoff.md).
- **Q-EVAL ratifications (plan authority):** [`../r3-program-plan.md`](../r3-program-plan.md) §10.3 — G0d-Dispatch, Descent-Termination-Contract, Lens-Fold-First-Slice (G1.a), Q-Reification Option A.

## Owned deliverables (Gap 3 — through R3 close)

| Ledger gate | HEAD summary (2026-05-13) | Next dispatch |
|---|---|---|
| `runtime_value_model_structural` | **green** — `evaluator_runtime_value_model_landed` Pass; `runtime.dag` carriers + PR-A.3 strategy/memo surface landed. | Cadence only: lazy/thunk widening per matrix when scheduled. |
| `body_evaluator_structural` | **in-flight** — `evaluate_body` + major E3/E4/E5/G0d/Descent landings; **missing** named structural `.dag` hook `evaluator_body_evaluator_correctly_executes_std_termination`. | Worker: land termination.dag representative `TestClaim` + suite Pass (or Director-amended equivalent). |
| `lens_application_complete_reflection` | **in-flight** — reflect+apply, G1.a Option 3, `lens_declaration_apply` migration, self-app gates; **missing** `evaluator_lens_application_complete_reflection` fixture + full fold residuals per phase-5 handoff. | Workers per landed slice briefs; wire structural reflection gate when ready. |
| `witness_construction_structural` | **in-flight** — partial #1857 path; full `evaluator_witness_construction_per_lens_correct` not closed. | Worker after lens/body prerequisites per `r2-evaluator-manager.md` Acceptance. |
| `cross_target_equivalence_harness_structural` | **in-flight** — PR-D slices 0–1 Pass; slice 2 / strict L5 deferred per PR-D brief. | Worker when LanguageSpec + grounding deps clear; do not invent `ForAllTargets` semantics early. |

## Cross-lane coordination (standing)

- **F-β.2 (warm-wolf-698):** effect_enumeration atomic-migration — **substrate first**; Evaluator consume-side wiring **after** carrier lands per [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 / §6.2. Coordinate via sibling message; do not pre-empt substrate shape.
- **F-α gate #81:** parallelism walker landed (#2795) — Evaluator Mgr **reviews** consume-side for drift when touching related runner/evaluator paths.
- **Cluster M Phase 3:** coordinate dispatch sequencing with PM (deep-wolf-155) after PR #3013 merge + Phase A close-audit skeleton.

## Reporting + formal signals

- **Planning truth:** this brief + closure-ledger Evaluator rows (refreshed on cadence or after major merges).
- **Formal lane-close / `green` row ack:** **R2 Release Manager** per [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) Signal-receiver protocol — Evaluator Mgr emits cross-manager payloads; Release Manager owns queue ack + canonical row edits **unless** Director explicitly delegates a factual HEAD refresh (as with neat-heron-793 2026-05-13 Evaluator block).

## INVARIANTS

Substrate-fact introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 before any new carrier or `TestPredicate` variant.
