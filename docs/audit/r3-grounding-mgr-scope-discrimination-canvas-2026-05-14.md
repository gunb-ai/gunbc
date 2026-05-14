# R3 Grounding Manager — scope-discrimination canvas (Gap 13)

**Date:** 2026-05-14  
**Authority:** Operator §4 sub-item 6 ratification (PR #3013 + #3038 + bundled-5-asks Ask 2–3, 2026-05-13); Director audit msg_8ae92369 (d) caveat — **scope discrimination before worker dispatch**.  
**Inputs:** `docs/r2-closure-ledger.md` §Grounding Manager (T-Ground, 11 sub-lanes); `docs/briefs/r2-grounding-manager.md`; `docs/r3-actual-close-plan.md` §Gap 13; `docs/design-emission-model.md` (no-coercion-engine discipline).

## Purpose

Classify each **T-Ground** sub-lane as:

- **(i) Grounding-owned** — discipline-specific work where **Grounding** is the natural program owner: target-side primitive authority, LanguageSpec substrate authoring, lifetime/ownership derivation from program shape, grounding-local diagnostics ordering, cross-target meta **as a grounding completeness obligation**, L4 routing tests, Track-13 dissolution sequencing tied to coercion-as-emission.
- **(ii) Substrate-Mgr-absorbed (organic)** — overlapping **substrate canvas** work the **R3 Substrate** lane has already picked up (carrier introduction, registry rows, emission-path / realization substrate, Coercion-Fold dissolution slices that are structurally **substrate-fact** work).

**Anti-goals (Director-framed):**

1. **Thin wrapper** — Grounding Mgr merely proxies Substrate Mgr execution already in flight.  
2. **Dual authority** — two managers owning conflicting edits to the same carrier rows, scaffold markers, or ratchet tests without an explicit handoff record.

Cross-Mgr coordination is **Gap-tier and lane-named** (R3 Substrate Mgr, R3 Debt-Paydown Mgr, R3 Verification Mgr, R3 Evaluator Mgr), not session-id gossip.

## Classification matrix (11 sub-lanes)

| Sub-lane | Class | Rationale | Coordination / handoff |
|---|---|---|---|
| **T-Ground-Pilot** | **(i)** | End-to-end pilot for inhabitance routing across three targets; program-shaped acceptance gate, not a substrate-carrier introduction lane. | None — closed **green** at ledger; Grounding owns historical receipt. |
| **T-Ground-Rust** | **(i)** | Authoritative **per-target primitive declarations** and Shape-A precondition for L5 corpus; Grounding discipline even when Substrate lands shared parents (Interval, etc.). | Consumes Substrate **cadence / shared-parent** locks (PR-F); Grounding owns **target row population** and structural gate string. |
| **T-Ground-Python** | **(i)** | Same as Rust, Python axis. | Consumes PR-G; Grounding-owned. |
| **T-Ground-Go** | **(i)** | Same as Rust, Go axis. | Consumes PR-H; Grounding-owned. |
| **T-Ground-LanguageSpec** | **Split (i) dominant, (ii) adjacent)** | **Language-agnostic** spec tables + registry-backed rows are the **Grounding** program surface. **Carrier mechanics** (`TypeRealization`, `RealizationCost`, registry density, `cost_target_realization`-class rows) overlap Substrate’s **substrate-fact introduction** procedure (INVARIANTS §P1). | **Handshake:** Substrate Mgr authors **new carrier fields / registry mechanics** under §P1; Grounding Mgr owns **row population toward** `language_spec_language_agnostic_structural` and consumer choreography toward emit-side retirement (per `r2-grounding-manager.md` Phase 1.5 / deferred emit authorities). Document each landing PR as **(substrate shape)** vs **(grounding population)** in closure-ledger Notes to avoid dual edits without acknowledgment. |
| **T-Ground-Coercion-Fold** | **Split (i) intent, (ii) execution risk)** | **Discipline owner** is Grounding per `design-emission-model.md` (coercion = emission; fold over facts). Recent **R3-tier slices** (ScratchIntExamples retirement, SelectedTargetInhabitance-class work) have landed via substrate-heavy paths — structurally easy for Substrate Mgr to **execute** while Grounding retains **gate + dissolution sequencing** authority. | **Handshake:** Grounding Mgr owns `coercion_fold_structural` **acceptance story** and dependency ordering vs LanguageSpec / ValueBody-list+sum; Substrate Mgr may land **fold-shaped substrate + ratchet removals**; either side flags overlapping PRs in **Notes** + cross-lane ping (Gap-tier). |
| **T-Ground-Lifetime-Analyzer** | **(i)** | Structural derivation from program use; not a generic carrier lane. R3 advanced cases may fold through lens-producer retirement, but **ownership stays Grounding** unless Director explicitly re-homes. | Coordinate with **Verification** only where L4/L7 harness consumes derivation facts — facts remain Grounding-authored. |
| **T-Ground-Diagnostic** | **Split (i) consumer, (ii) Layer-1 authority)** | **Q6.5** locks Layer-1 `CompilerDiagnosticKind` as **Substrate-owned** closed sum; Grounding lane owns **cross-target diagnostic ordering** and emission-facing diagnostic **structure** without extending the kind sum. | **Handshake:** no Substrate/Grounding **dual edits** to `CompilerDiagnosticKind`; Grounding consumes + orders; Substrate extends kinds only via §P1. |
| **T-Ground-CrossTarget-Meta** | **Split (i) obligation, (ii) L6 substrate)** | Grounding owns **“meta completeness”** obligation (`cross_target_meta_structural`). **L6 emission-path substrate** (EmissionPathProjection closure, walker keys, ledger ratchet list) is high-touch **substrate canvas** work Substrate Mgr has been landing (e.g. L6 projection closure PRs cited in close plan). | **Handshake:** Substrate lands walker/substrate fixes; Grounding owns **ledger key retirement cadence** vs `check_l6_load_completeness` and the **gate firing** narrative. Refresh closure-ledger L6 block only when walker + row coverage agree (no ledger-only key drops). |
| **T-Ground-Tests** | **(i)** | L4 routing correctness + algebra certification — verification-shaped **claims**, but **grounding program** owns routing substrate prerequisites called out in brief (Q4 + Coercion-Fold body gates). | Coordinate **Verification** for harness/TestClaim wrappers; Grounding owns **routing substrate readiness** gating dispatch. |
| **T-Ground-Dissolve** | **(i)** | Track-13 dissolution is **program sequencing** after Coercion-Fold carries load — not substrate introduction. | Execute only after Coercion-Fold + LanguageSpec obligations are honestly staged; coordinate **PB / Debt-Paydown** on emit.rs ratchet coupling, without taking PB scope. |

## Cross-lane signals (Gap-tier)

| Partner lane | Why | Signal |
|---|---|---|
| **R3 Substrate Mgr** | Organic absorption on Coercion-Fold slices, L6 emission-path, realization registry class landings (#1980, #2103, #2229, #2272, #2279 per close-plan HEAD narrative). | Request explicit **(substrate)/(grounding)** tagging on overlapping PRs; escalate dual-authority edits. |
| **R3 Debt-Paydown Mgr** | `EXPECTED_HAND_AUTHORED_NON_TEST` entries for emit/coercion-class paths may gate on **T-Ground** closures. | Grounding publishes **gate proximity** after each sub-lane refresh; Debt-Paydown sequences retirements without inventing parallel emit authority. |
| **R3 Verification Mgr** | L5 corpus + L4 harness consume Shape-A grounding; Cluster M / tests-as-data touch emit boundaries. | Grounding signals **Shape-A readiness** per target; Verification does not own primitive tables. |
| **R3 Evaluator Mgr** | `.dag`-authored emitter / witness paths interlock with “no hand-Rust coercion engine” end state. | Evaluator owns evaluator gates; Grounding owns **emit.rs retirement sequencing** relative to substrate completeness (close-plan Gap 13). |

## Summary verdict

- **Pure (i):** Pilot (done), Rust, Python, Go, Lifetime-Analyzer, Tests, Dissolve.  
- **Pure (ii):** *None* — even heavily substrate-overlapping lanes retain a **Grounding acceptance / sequencing** slice.  
- **Split (handshake-heavy):** LanguageSpec, Coercion-Fold, Diagnostic (consumer), CrossTarget-Meta (L6 substrate vs meta obligation).

## Next actions (post canvas)

1. **Hold-pattern (landed with PR #3054):** `docs/briefs/r3-grounding-manager.md` + T-Ground closure-ledger `Last signal` / `Notes` refresh — Director/admin-merge when CI cleans.  
2. **Post dashboard `--shape` on `work-items create`:** Dispatch workers only after ledger Notes reflect this canvas (avoid respawn-2 colliding with Substrate flight).  
3. **Do not** run `dashboard-ops replan` on this role-node (Director: auto-archive risk).
