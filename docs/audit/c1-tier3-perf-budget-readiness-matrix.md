# C1 — `tier3_mirror_dissolution_perf_within_budget` Readiness Matrix

**Status:** PROPOSAL (audit/planning only). Authored 2026-05-01 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parent brief:** `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (PR #1331, merged).
**Authority basis:** PR #1319 (Director ratification — ≤2× median / ≤5× p99 thresholds); `docs/r3-structure.md` §"T-Tier3-Dissolution"; INVARIANTS §P2 (no parallel implementations).
**Scope:** docs-only readiness verification ahead of any Phase 1 dispatch. **No code, no `criterion` dependency, no benchmark fixtures, no `PerfWithinBaseline` variant authoring, no fake pass/fail claims.**

This is the smallest useful preparatory artifact requested by the dispatch: it (a) verifies each Phase 1 / Phase 2 prerequisite at HEAD, (b) inventories the four mirror surfaces with current line ranges, (c) records routing for the open Substrate-Mgr decision, and (d) names what must change before worker dispatch is authorized.

---

## 1. Phase / dispatch readiness at HEAD `5d03c86b0`

| # | Prerequisite | Phase | State at HEAD | Owner | Routing |
|---|---|---|---|---|---|
| R-1 | `criterion` dev-dep present in `src/v3/compiler/Cargo.toml` (or workspace) | Phase 1 (deliverable 0a) | **NOT MET** — `grep -n criterion Cargo.toml src/v3/compiler/Cargo.toml` returns zero matches at HEAD. | C1 Phase 1 worker (when dispatched). | Internal to lane; no escalation needed. |
| R-2 | Tier-3 hand-Rust mirror sites still live (Phase 1 measures them) | Phase 1 | **MET** — see §2 surface inventory; all four mirror surfaces present, but line-range citations in the parent brief have drifted (see §2). | n/a | n/a |
| R-3 | Canonical CI machine designated for baseline capture (deliverable 0c) | Phase 1 | **NOT MET** — no signal in `docs/r3-structure.md`, `r3-pb-tier3-perf-budget-worker.md`, or recent CI changes naming the canonical bench host. Per brief §"Discipline / baseline noise concerns": Phase 1 capture and Phase 2 measurement must run on the same hardware. | PB Manager. | Substrate Mgr if CI-infra cross-cuts. **Hard prereq for Phase 1 dispatch.** |
| R-4 | Substrate-Mgr decision on `PerfWithinBaseline` `TestPredicate` variant (path (a)) vs `ExecuteCommand` reuse (path (b)) | Phase 2 | **NOT MET** — `src/v3/std/verification.dag:109-160` declares `TestPredicate` variants `Compiles \| FailsWithDiagnostic \| OutputEquals \| PortHasState \| DeclarationHasRefinement \| CostBounded \| BehavioralObservation \| MockBackedInvariant \| ExecuteCommand \| ForAllTargets \| ...`; no `PerfWithinBaseline`. Path (a) requires Substrate Mgr to author the variant before Phase 2 has any `.dag` predicate to reach. | Substrate Manager. | **Hard prereq for Phase 2 dispatch.** Per parent brief §"STOP conditions" item 1. |
| R-5 | All four T-Tier3-Dissolution mirror dissolution PRs merged on `main` | Phase 2 | **NOT MET** — all four mirror sites still live (see §2). | T-Tier3-Dissolution worker pack (`r2-pb-tier3-mirror-dissolution-workers.md`). | n/a; gates Phase 2 only. |
| R-6 | R2-Evaluator readiness signal (Phase 2 measures `.dag` body via Evaluator) | Phase 2 | **NOT MET** at audit time — Evaluator is R3 in flight per `r3-structure.md`; readiness signal not yet on the closure ledger. | R2-Evaluator Manager (R3 continuation). | **Hard prereq for Phase 2 dispatch.** |
| R-7 | Phase 1 baseline JSON (`tier3_baseline.json`) merged on `main` BEFORE any mirror dissolution PR | Phase 2 | **NOT MET** — `find . -name 'tier3_baseline*'` returns no matches. Strict temporal ordering: brief §"STOP conditions" item 2 — reverse order is **unrecoverable**. | C1 Phase 1 (worker, when dispatched). | n/a; structural constraint enforced by sequencing. |

**Summary:** Phase 1 dispatch is blocked on **R-1** (trivial, internal) + **R-3** (PB-Manager confirmation of canonical CI machine — non-trivial, requires PB-Mgr decision). Phase 2 dispatch is blocked additionally on **R-4** + **R-5** + **R-6** + **R-7**.

---

## 2. Tier-3 mirror surface inventory (drift-corrected)

The parent brief cites mirror line ranges as `dag.rs:628-790` (termination), `:839-915` (computation), `:916-980` (induction), plus `dag/effects.rs` and `workflow_idempotency.rs`. Those ranges have drifted at HEAD. The verified ranges below are what Phase 1 bench groups must target.

| Brief claim | Mirror | Current location at HEAD `5d03c86b0` | Verification |
|---|---|---|---|
| `dag.rs:628-790` | **termination** mirror — `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence` | `src/v3/compiler/src/dag.rs:819-1010` (approx). Anchors: `pub enum DescentEvidence` at L819; `pub enum PositiveDescentAmount` at L843; `pub enum ProportionalDivisor` at L852; `pub fn evidence_rank` at L939; `pub fn merge_evidence` at L947; `pub fn join_evidence` at L963; `pub enum ShrinkFactor` at L1084. | `awk 'NR>=800 && NR<=1100 && /^(pub )?(fn\|enum\|struct\|impl)/ {print NR}' src/v3/compiler/src/dag.rs` |
| `dag.rs:839-915` | **computation** mirror — `SizeBound` and recursion-shape derivation | `src/v3/compiler/src/dag.rs` — anchors `pub enum SizeBound` at L1027; `pub fn tree_size_bound` at L1038. (Original brief range `839-915` overlaps with the computation/induction zone but does not match current symbol layout.) | same as above |
| `dag.rs:916-980` | **induction** mirror — `RecursionShape`, `InductiveField`, `SubValueRelation` | `src/v3/compiler/src/dag.rs` — `pub enum RecursionShape` at L1097; further symbols downstream. | same as above |
| `dag/effects.rs` (216 LOC) + `compose_operation_effects` (105 LOC in `workflow_idempotency.rs`) | **effect-carrier** mirror | `src/v3/compiler/src/dag/effects.rs` (216 LOC, MET); `src/v3/compiler/src/workflow_idempotency.rs` (105 LOC, MET). | `wc -l` on both files. |

**Drift recommendation:** Phase 1 bench groups should not hard-code line ranges; reference symbol names instead (`DescentEvidence::evidence_rank`, `merge_evidence`, `tree_size_bound`, `compose_operation_effects`, etc.). Symbols are stable across the kind of refactors that move line numbers; lines are not. **Routing question §4.1.**

---

## 3. STOP conditions for this audit's escalation

The dispatch authorizes "STOP+PING if the required retired-mirror surfaces are not live enough to attach fixtures." That STOP does not fire — mirrors are live (R-2 met). The relevant blockers are **upstream of fixture authoring**:

- **STOP-A:** Phase 1 worker dispatch is not authorized until R-1 (internal) + R-3 (PB-Mgr canonical CI host) are resolved.
- **STOP-B:** Phase 2 worker dispatch is not authorized until R-4 (Substrate-Mgr `PerfWithinBaseline` decision) + R-5 + R-6 + R-7. R-4 is the load-bearing one — without the predicate variant or an explicit Director-signoff downshift to path (b) `ExecuteCommand`, Phase 2 has no `.dag` predicate to reach.
- **STOP-C:** Per parent brief §"STOP conditions" item 2, if any T-Tier3-Dissolution mirror PR lands before Phase 1 baseline capture, the perf gate becomes **unrecoverable** without a structural reframe of the threshold (relative → absolute). This is the single most fragile sequencing constraint in the lane and should be reflected in the T-Tier3-Dissolution dispatch order — flagged as routing question §4.2.

---

## 4. Routing questions (for PB Manager / Substrate Manager / Director)

These are NOT decisions — they are routing questions surfaced by readiness verification.

1. **Symbol-keyed vs line-keyed mirror identification:** parent brief cites `dag.rs:628-790` etc.; line ranges have drifted. Phase 1 bench groups should identify mirrors by symbol name, not line range. PB Manager confirm.
2. **Phase-1-before-dissolution sequencing enforcement:** the dispatch order between this lane's Phase 1 PR and the T-Tier3-Dissolution mirror PRs is structurally critical (STOP-C); should the C1 Phase 1 PR be a hard prerequisite in the T-Tier3-Dissolution worker dispatch (i.e., dissolution workers cannot dispatch until `tier3_baseline.json` is on `main`)? PB Manager decision; if yes, the T-Tier3-Dissolution worker pack brief (`r2-pb-tier3-mirror-dissolution-workers.md`) needs an explicit prereq row.
3. **Path (a) vs path (b) for `PerfWithinBaseline`:** Substrate Manager owns the call. Path (a) preserves structural-acceptance precision; path (b) reuses `ExecuteCommand` but loses precision (only exit code observed). Parent brief §"Acceptance gate" assumes path (a) at finalization; STOP+PING expected if Substrate Mgr chooses (b).
4. **Canonical CI machine for baseline capture:** required for Phase 1 dispatch (R-3). PB Manager designate; if no canonical host exists, escalate to Substrate Mgr (CI-infra cross-cut) before Phase 1 dispatches.

---

## 5. Acceptance summary

This readiness matrix is intentionally bounded:

- §1 verifies each Phase 1 / Phase 2 prerequisite at HEAD with explicit MET / NOT MET state and routing.
- §2 inventories the four mirror surfaces with drift-corrected anchors.
- §3 names which STOP conditions fire / do not fire at audit time.
- §4 routes 4 open questions to PB Manager / Substrate Manager / Director.

**No worker dispatch is authorized by this audit.** Phase 1 awaits R-1 + R-3; Phase 2 awaits R-4 + R-5 + R-6 + R-7. PB Manager re-reads at gate-clear per parent brief §"Dispatch preconditions."

**No code, no `criterion`, no benchmark fixtures, no `PerfWithinBaseline` variant, no fake pass/fail.** Strictly the smallest useful preparatory artifact.
