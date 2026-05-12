# C1 — `tier3_mirror_dissolution_perf_within_budget` Readiness Matrix

**Status:** PROPOSAL (audit/planning only). Authored 2026-05-01 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parent brief:** `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (PR #1331, merged).
**Authority basis:** PR #1319 (Director ratification — ≤2× median / ≤5× p99 thresholds); `docs/r3-structure.md` §"T-Tier3-Dissolution"; INVARIANTS §P2 (no parallel implementations).
**Scope:** docs-only **readiness artifact** (this file does not land Rust). It **records** Phase 1 / Phase 2 prerequisite state observed against the tracked tree—including that **`criterion`** is present when true (**R‑1** refreshed **2026‑05‑11**). **No benchmark fixtures authored here, no `PerfWithinBaseline` variant authoring, no fake pass/fail claims.**

This is the smallest useful preparatory artifact requested by the dispatch: it (a) verifies each Phase 1 / Phase 2 prerequisite at HEAD, (b) inventories the four mirror surfaces with current line ranges, (c) records routing for the open Substrate-Mgr decision, and (d) names what must change before worker dispatch is authorized.

---

## 1. Phase / dispatch readiness

| # | Prerequisite | Phase | Recorded state | Owner | Routing |
|---|---|---|---|---|---|
| R-1 | `criterion` dev-dep present in `src/v3/compiler/Cargo.toml` (or workspace) | Phase 1 (deliverable 0a) | **MET** — **`[dev-dependencies]`** declares **`criterion = "0.5"`**; **`[[bench]]`** **`tier3_mirror_perf`** (**`harness = false`**). Verification: **`grep -n criterion src/v3/compiler/Cargo.toml`** and **`grep -n tier3_mirror_perf src/v3/compiler/Cargo.toml`**. | n/a | n/a |
| R-2 | Tier-3 hand-Rust mirror sites still live (Phase 1 measures them) | Phase 1 | **MET** — see §2 surface inventory; all four mirror surfaces present, but line-range citations in the parent brief have drifted (see §2). | n/a | n/a |
| R-3 | Canonical **baseline capture host** designated for **`cargo bench` / `tier3_baseline.json`** capture (deliverable 0c) | Phase 1 | **MET (designation)** — **`ubicloud-standard-2`** ratified in **`docs/audit/c1-r3-canonical-bench-host-decision-matrix.md`**; PR **`#2702`** forwards **`.github/workflows/tier3-baseline-capture.yml`** with **`runs-on: ubicloud-standard-2`**. Merge-blocking **`ci.yml`** **`v3` / compile jobs** intentionally use **`gunbc-ci`** (self-hosted) — orthogonal pool; Phase 1 `cargo bench` capture authority follows R‑3 matrix + capture workflow **not** the default PR compile farm. **`R‑7.canonical coherence`** still awaits a **successful Ubicloud-backed capture artifact** (`captured_on.host_id`) or Director/PB waiver per R‑7 row. | PB Manager | Substrate Mgr if CI-infra cross-cuts. |
| R-4 | Path **(a)** `PerfWithinBaseline` declarative substrate vs path **(b)** `ExecuteCommand` reuse (`TestPredicate`) | Phase 2 | **MET (substrate)** — **`src/v3/std/verification.dag`** declares closed **`PerfBudgetComparisonOp`** (**~L94–97**) and **`TestPredicate`** variant **`PerfWithinBaseline { subject, comparator, baseline_ref }`** (**~L249–253**); **`src/v3/compiler/src/test_runner.rs`** dispatches **`\"PerfWithinBaseline\"` → `eval_perf_within_baseline`** (**~L2496**). Smoke Fixture: **`src/v3/compiler/tests/fixtures/r3_perf_within_baseline_smoke.dag`**. Verification: **`rg -n PerfWithinBaseline src/v3/std/verification.dag src/v3/compiler/src/test_runner.rs`**. Phase 2 is **not** blocked on “no substrate predicate”: it still awaits **dissolution authoring** (**R‑5/R‑6**) and **coherent R‑7** before the mirror perf gate can clear. Director-signoff **path (b)** remains a downgrade option but is no longer forced by substrate absence. | Substrate Manager (shape stewardship); Director if path **(b)** is ratified instead. | Per parent brief §"STOP conditions": path **(a)** authoring **landed**. |
| R-5 | All four T-Tier3-Dissolution mirror dissolution PRs merged on `main` | Phase 2 | **NOT MET** — all four mirror sites still live (see §2). | T-Tier3-Dissolution worker pack (`r2-pb-tier3-mirror-dissolution-workers.md`). | n/a; gates Phase 2 only. |
| R-6 | R2-Evaluator readiness signal (Phase 2 measures `.dag` body via Evaluator) | Phase 2 | **NOT MET** at audit time — Evaluator is R3 in flight per `r3-structure.md`; readiness signal not yet on the closure ledger. | R2-Evaluator Manager (R3 continuation). | **Hard prereq for Phase 2 dispatch.** |
| R-7 | Phase 1 baseline JSON (`tier3_baseline.json`) on `main` **and** **`R‑7.canonical coherence`** (`captured_on.host_id` + timings from **`ubicloud-standard-2`** for dissolution-linked perf parity) | Phase 2 / dissolution prep | **`R‑7.presence`:** **landing** via PR **`#2702`** (bootstrap JSON path). **`R‑7.canonical coherence`:** **BLOCKED until** Operators run `.github/workflows/tier3-baseline-capture.yml` post-merge (**or equivalent Ubicloud-backed capture**) replacing committed baseline rows/metadata with `host_id: ubicloud-standard-2`, **unless** Director/PB publishes waiver receipt. Temporal ordering unchanged: dissolution PRs relying on canonical perf benches still follow baseline **coherence**, not bootstrap presence alone (`docs/audit/c1-tier3-baseline-capture-procedure.md` §1). | C1 Phase 1 worker + PB Manager (cadence/waiver ledger). | `workflow_dispatch` refresh after YAML on `default`. |

**Summary:** §1 **`MET / NOT MET`** rows are authoritative for this **`2026‑05‑11`** audit refresh (**PR `#2702`**) plus **`2026‑05‑12`** **R‑4** substrate reconcile against HEAD — **R‑1** (**`criterion`**) and **R‑3** (**Ubicloud-backed capture designation**) read **MET** here and align with **`c1-r3-canonical-bench-host-decision-matrix.md`** plus **`tier3-baseline-capture.yml`**. **R‑7** splits **`presence` vs canonical `host_id` coherence** per **`c1-tier3-baseline-capture-procedure.md` §1** (**PR `#2702`**); dissolution-linked **`R‑7.canonical coherence`** remains load-bearing separately. **R‑4** (**path‑(a) `PerfWithinBaseline`**) reads **MET** (**`verification.dag` + `test_runner`**). Phase 2 dispatch stays blocked additionally on **R‑5 + R‑6 + coherent R‑7** (**`R‑7.presence`** alone does not unblock perf parity gates; Phase 2 also needs the dissolution lane `.dag` claims that *consumes* the landed predicate against frozen baseline rows).

---

## 2. Tier-3 mirror surface inventory (drift-corrected)

The parent brief cites mirror line ranges as `dag.rs:628-790` (termination), `:839-915` (computation), `:916-980` (induction), plus `dag/effects.rs` and `compose_operation_effects` (105 LOC in `workflow_idempotency.rs`). Those ranges have drifted at HEAD. The verified ranges below are what Phase 1 bench groups must target.

| Brief claim | Mirror | Current layout / anchors (`2026‑05‑11` refresh) | Verification |
|---|---|---|---|
| `dag.rs:628-790` | **termination** mirror — `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `evidence_rank`, `merge_evidence`, `join_evidence`; **`ShrinkFactor` lives under nested `pub mod computation`**, not contiguous with termination lattice enums | **`src/v3/compiler/src/dag.rs`** — **`pub enum DescentEvidence`** **L1165**; **`PositiveDescentAmount`** **L1189**; **`ProportionalDivisor`** **L1198**; **`evidence_rank`** **L1285**; **`merge_evidence`** **L1293**; **`join_evidence`** **L1309**. Nested scaffold: **`ShrinkFactor`** from **~L162** inside **`pub mod computation`** (**~L84**). | `awk 'NR>=80 && NR<=220 && /^(pub )?(fn\|enum\|struct\|impl)/ {print NR}' src/v3/compiler/src/dag.rs`; `awk 'NR>=1140 && NR<=1360 && /^(pub )?(fn\|enum\|struct\|impl)/ {print NR}' src/v3/compiler/src/dag.rs` |
| `dag.rs:839-915` | **computation** mirror — `SizeBound`, `CallPattern`, recursion-shape scaffolding, `ShrinkFactor`, `tree_size_bound`, … | **`src/v3/compiler/src/dag.rs`** nested **`pub mod computation`** — **`pub enum SizeBound`** **L111**; **`tree_size_bound`** **L122**. | `awk 'NR>=80 && NR<=400 && /^(pub )?(fn\|enum\|struct\|impl)/ {print NR}' src/v3/compiler/src/dag.rs` |
| `dag.rs:916-980` | **induction** mirror — `RecursionShape`, `InductiveField`, `SubValueRelation` | **`src/v3/compiler/src/dag.rs`** — **`pub enum RecursionShape`** **L1368**; downstream symbols in-file. | `awk 'NR>=1340 && NR<=1550 && /^(pub )?(fn\|enum\|struct\|impl)/ {print NR}' src/v3/compiler/src/dag.rs` |
| `dag/effects.rs` (216 LOC) + `compose_operation_effects` (105 LOC in `workflow_idempotency.rs`) | **effect-carrier** mirror | `src/v3/compiler/src/dag/effects.rs` (216 LOC, MET); `src/v3/compiler/src/workflow_idempotency.rs` (105 LOC, MET). | `wc -l` on both files. |

**Drift recommendation:** Phase 1 bench groups should not hard-code line ranges; reference symbol names instead (`DescentEvidence::evidence_rank`, `merge_evidence`, `tree_size_bound`, `compose_operation_effects`, etc.). Symbols are stable across the kind of refactors that move line numbers; lines are not. **Routing question §4.1.**

---

## 3. STOP conditions for this audit's escalation

The dispatch authorizes "STOP+PING if the required retired-mirror surfaces are not live enough to attach fixtures." That STOP does not fire — mirrors are live (R-2 met). The relevant blockers are **upstream of fixture authoring**:

- **STOP-A:** ~~Phase 1 worker dispatch waits on internal R‑1 + R‑3 designation~~ (**both MET in §1 on this `2026‑05‑11` refresh**); **routing / cadence / `R‑7` bootstrap vs canonical capture pairing** remains PB/Director authority per parent brief (**recapture sequencing is not silent**).
- **STOP-B:** Phase 2 worker dispatch is not authorized until R-5 + R-6 + R-7 coherent + **dissolution-scope `.dag` gate authoring** that exercises **`PerfWithinBaseline`** against **`tier3_baseline.json`** (parent brief suites). ~~R‑4 substrate absence~~ is **obsolete** at HEAD — path **(a)** variant + runner eval **have landed** (see §1 **R‑4**). Only a **Director-signoff downgrade to path (b)** `ExecuteCommand` would deliberately bypass that substrate.
- **STOP-C:** Per parent brief §"STOP conditions" item 2, if any T-Tier3-Dissolution mirror PR lands before Phase 1 baseline capture, the perf gate becomes **unrecoverable** without a structural reframe of the threshold (relative → absolute). This is the single most fragile sequencing constraint in the lane and should be reflected in the T-Tier3-Dissolution dispatch order — flagged as routing question §4.2.

---

## 4. Routing questions (for PB Manager / Substrate Manager / Director)

These are NOT decisions — they are routing questions surfaced by readiness verification.

1. **Symbol-keyed vs line-keyed mirror identification:** parent brief cites `dag.rs:628-790` etc.; line ranges have drifted. Phase 1 bench groups should identify mirrors by symbol name, not line range. PB Manager confirm.
2. **Phase-1-before-dissolution sequencing enforcement:** the dispatch order between this lane's Phase 1 PR and the T-Tier3-Dissolution mirror PRs is structurally critical (STOP-C); should the C1 Phase 1 PR be a hard prerequisite in the T-Tier3-Dissolution worker dispatch (i.e., dissolution workers cannot dispatch until `tier3_baseline.json` is on `main`)? PB Manager decision; if yes, the T-Tier3-Dissolution worker pack brief (`r2-pb-tier3-mirror-dissolution-workers.md`) needs an explicit prereq row.
3. **Path (a) vs path (b) for `PerfWithinBaseline`:** Substrate Manager owns shape stewardship. Path **(a)** preserves structural acceptance; path **(b)** reuses `ExecuteCommand` but loses precision (only exit code observed). **Ledger reconcile (`2026-05-12`):** HEAD already lands path **(a)** (`verification.dag` declares **`PerfWithinBaseline`**; **`test_runner`** evaluates it — see §1 **R‑4`). STOP+PING applies only if a **Director‑ratified** pivot to **(b)** is chosen after all.
4. **Canonical **`cargo bench`** capture host vs merge-blocking compile fleet:** §1 **R‑3 designation** (**`ubicloud-standard-2`**) reads **MET** here (decision matrix + `tier3-baseline-capture.yml`). PB Manager confirm there is **no accidental substitution** of **`ci.yml`'s `gunbc-ci` PR pool** for authorised Ubicloud **`cargo bench`** capture when interpreting host provenance (**`captured_on.host_id`**).

---

## 5. Acceptance summary

This readiness matrix is intentionally bounded:

- §1 verifies each Phase 1 / Phase 2 prerequisite with explicit MET / NOT MET state and routing.
- §2 inventories the four mirror surfaces with drift-corrected anchors.
- §3 names which STOP conditions fire / do not fire at audit time.
- §4 routes 4 open questions to PB Manager / Substrate Manager / Director.

**No worker dispatch is authorized by this audit alone.** Phase 2 awaits R‑5 + R‑6 + coherent **R‑7** and dissolution perf gate **claim** authoring (**R‑4** substrate **MET**); Phase 1 **internal** rows **R‑1 + R‑3 designation** show **MET** here while **`tier3_baseline.json` coherence / dissolution pairing** stays governed by **`R‑7` split semantics** (**PR `#2702`**, capture procedure §1). PB Manager re-reads at gate-clear per parent brief §"Dispatch preconditions."

**No Rust, benchmark fixtures, or `PerfWithinBaseline` variant authoring in this file — docs-only prerequisite ledger.**
