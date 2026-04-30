# R3 Verification Manager Brief

**Status:** ACTIVE planning brief. Spawned after R2 closed-with-residuals on 2026-04-30 via Director acceptance #1275 and the R3 Verification Manager dispatch in inbox #1276.

**Authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2. The manager owns `T-Verification-L4-L7-Direct`, `T-Verification-L5-Corpus`, the `bridge_retirement_ledger_zero` audit gate, and the R3-absorbed formal-grounding TestClaim bundle described below.

## Program Scope

R3 Verification owns the runtime-verification consequence surface:

| Lane / gate | Status at spawn | Description |
|---|---|---|
| **T-Verification-L4-L7-Direct** | STANDBY | Evaluator-direct harness for L4 emit/eval match and L7 algebraic-law witnesses. Dispatch gates on R2-Evaluator PR-A.3 carriers plus body evaluator / witness construction readiness. |
| **T-Verification-L5-Corpus** | STANDBY | L5 cross-target equivalence corpus. Sequentially depends on the L4/L7 Direct corpus and Shape A grounding closure for the dispatch-time target set. |
| **T-FormalGrounding-Verification** | AUTHOR-NOW-FIRE-LATER | TC1 / TC2 / TC3 bundle management. TC1 and TC2 fixture hooks exist; TC3 is text-form in the PB fixed-point brief and requires substrate-introduction or an equivalent quantifier-capable proof surface. |
| **`bridge_retirement_ledger_zero`** | LEDGER OPEN | Verification owns the unified audit gate only. Bridge retirement work remains distributed to Substrate / PB per `r3-structure.md` §"Lane structure". |

This manager does **not** own R2-Evaluator implementation, Shape A grounding implementation, `Lens<C>` substrate authoring, PB lens-producer retirement, or bridge-retirement implementation work. It consumes their closure signals and owns the verification/corpus/ledger acceptance surface.

## Owned Briefs

- [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md) — Lane 1 worker brief, standby.
- [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) — Lane 2 worker brief, standby after Lane 1.
- [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) — TC1 / TC2 / TC3 formal-grounding bundle and audit state.

## Current TestClaim Audit

Main currently has the expected author-now-fire-later markers:

- **TC1 eta-equivalence:** `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` declares `tc1_substrate_lens_eta_equivalence_suite` using `SubstrateResearchDeferredClaim`; the runner deliberately scopes this predicate to that fixture path.
- **TC2 Church-Rosser / evaluation-order independence:** `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` declares `tc2_evaluation_order_independence_suite`; `r2-evaluator-manager.md` names it as the slice-0 hook that strengthens after PB-Runtime + `T-Substrate-Lens-Primitive`.
- **TC3 strong normalization:** no `.dag` fixture exists yet. The declarative shape lives in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim"; that section correctly records a substrate gap because current `TestPredicate` variants are per-program and cannot encode a theorem over the whole well-typed fragment.

No drift found in TC1 / TC2 fixture naming. TC3 remains intentionally text-form and blocked on a quantifier-capable structural proof surface.

## Bridge-Retirement Ledger Audit

Verification tracks the five named bridges from `r3-structure.md` and reports whether the unified ledger can close:

| Bridge row | Natural owner | Current audit state |
|---|---|---|
| `bridge_source_span_file_participation_retired` | Substrate | **R3-deferred / open.** `r3-structure.md` records the #1273 deferral receipt: production participation still consults source path / span-file identity in `lens_apply.rs`, `lower.rs`, and `emit.rs`; structural identity carriers are prerequisite. |
| `bridge_mark_bootstrap_secret_nominal_opacity_retired` | Substrate | **Retired.** PR #1272 deleted the `mark_bootstrap_secret_nominal_opacity()` bootstrap bridge; nominal-opacity authority lives in the source declaration. |
| `bridge_canonical_lens_name_dispatch_retired` | PB | **Partial / open.** `r2-pure-bootstrap-manager.md` records retired sentinel/cost-bind surfaces, with remaining canonical lens byte includes and name-keyed lookup arms pinned by the canonical-lens bridge ratchet. |
| `bridge_include_str_side_channels_retired` | PB | **Open.** `pipeline_authority` no longer swaps to runtime file IO, but `compile` remains `ArrowBody::Unparsed`; full retirement awaits derivation or a structural compile-body witness. |
| `bridge_exact_string_patching_residual_retired` | PB | **Lower-helper slice retired.** The `patch_lower_helpers_*` class is at zero and pinned by `bridge_lower_helpers_patch_zero_residual_test`; broader exact-string patching classes remain out of that receipt's scope. |

**Ledger conclusion:** `bridge_retirement_ledger_zero` is open. The net blocking rows are SourceSpan.file participation, canonical lens-name dispatch residuals, and include_str side channels; exact-string patching has a closed lower-helper sub-row but is not a blanket claim over every possible string transform.

## Reporting Cadence

- Lane 1 dispatch readiness → Director + R2/R3 Release ledger after R2-Evaluator prerequisites land.
- Lane 2 dispatch readiness → Director after Lane 1 corpus exists.
- TC bundle drift or substrate-gap escalation → Director and Substrate Manager when it requires substrate introduction.
- Per-bridge retirement signals from PB/Substrate → this manager updates the unified ledger gate; no implementation dispatch from Verification for those rows.

## Acceptance Gates

- `l4_emit_eval_match_holds_per_corpus_program_per_target`
- `l7_algebraic_law_witnesses_evaluate_structurally`
- `l5_cross_target_consistency_holds_per_corpus_program`
- `tc1_eta_equivalent_dag_forms_yield_identical_lens_results` strengthens from deferred claim to executable claim
- `tc2_evaluation_order_independent_lens_results` strengthens from deferred claim to strict strategy-output equality
- `tc3_every_typed_dag_program_terminates_in_bounded_steps` obtains a substrate-backed proof encoding
- `bridge_retirement_ledger_zero`

## Cross-Refs

- R3 lane authority: [`docs/r3-structure.md`](../r3-structure.md)
- No-engine verification framing: [`docs/design-emission-model.md`](../design-emission-model.md)
- Evaluator manager and primitive readiness: [`r2-evaluator-manager.md`](r2-evaluator-manager.md)
- PB bridge distribution / fixed-point TC3 source: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md), [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md)
