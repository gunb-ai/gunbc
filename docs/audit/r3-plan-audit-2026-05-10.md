# R3 Plan Audit — does the plan deliver R3? — 2026-05-10

**Author**: deep-wolf-155 (PM)
**Trigger**: Brian directive 2026-05-10 (~01:00Z): *"audit the R3 plan and make sure it holds up - this is your actual job as the PM - to keep the higher level vision and make sure the plans align."*
**Authority scope**: PM-tier audit. Findings = PM observations + recommendations; ratification of any scope/structural changes is operator-tier (Brian) or Director-tier (zesty-bear-812 / gunbc#828).

**Reading order**: §1 (thesis) → §2 (status summary) → §3-§5 (coverage matrices) → §6 (findings) → §7 (critical-path) → §8 (recommendations).

---

## §1. R3 thesis — what we are delivering

Per `docs/r3-program-plan.md` §1 (Brian-ratified at gunbc#846 2026-05-05) + cascade promotions through 2026-05-09:

R3 closes when **ALL** of the following hold:

| # | Predicate | Plain language | §1 ref |
|---|---|---|---|
| 1 | **PB-0** | Compiler self-hosts with **0 hand-Rust + 0 TESTING residual** — every `.rs` in stage0/tests/emitters generated from `.dag` | §1.5 + cascade promotion 2026-04-25 |
| 2 | **v2 retired** | `src/v2/` directory deleted; no `.rs` test consumers reference v2 | §1.2 |
| 3 | **BridgeLedgerZero** | Class A/B/C/F/G named identity bridges all dissolve to **0**; ledger ratchet at zero | §1.3 |
| 4 | **Verification executable** | Pattern-A NYI predicates (TC1/TC2/TC3 + RustDagIso + SymbolicCost) run as real TestClaims, not `NotYetImplemented` shells | §1.1 |
| 5 | **5 substrate-gap classes closed** | Parser handles algebra×machine-constraint composition; function-valued data executes; file ingestion structural; CI workflow-as-data; reflection via PB-Runtime | §1.4 |
| 6 | **Zero debt rows** | ROADMAP open + sweep §1 Class A/B/C/F/G + §10 RED items all retired; `r3_debt_paydown_zero_remaining` GREEN | §1.5 + Director poke-hole 2026-05-06 finding 3.1 |

Plus implicit-but-load-bearing within the 96 gates:
- **Multi-target emission with cross-target consistency** (gates #15, #51, #52 — `l5_cross_target_consistency` and the cross_target_optimization pair)
- **Free-consequences demonstration** (10 gates #43-#52 — the thesis claim that lens composition makes parallelism/memoization fall out structurally)
- **TestGen runner closure** (3 `[ext]` predicates referenced from r3-structure.md; structural location ambiguous — see Finding F1)

**Operationalized as**: 96 §1.8 gates GREEN + `r3_debt_paydown_zero_remaining` GREEN (97 enumerated; #11 TC1 V1 strict-fire canvas-deferred per Director (a)-disposition 2026-05-09).

---

## §2. Status summary — where R3 is at HEAD

**§1.8 gate-status distribution** (counted at this commit):

| Status | Count | % of 96 R3-load-bearing |
|---|---|---|
| PASSING / SATISFIED-BY-CONSTRUCTION | 11 | 11.5% |
| CONSUMER_LANDED (executable consumer; not yet full PASSING) | 15 | 15.6% |
| DECLARED-only (document-level; no consumer infrastructure) | 70 | 72.9% |

**Plain-language read**: ~12% green, ~16% partial, ~73% paper-only.

**Lane status distribution** (from §3 lane status table — 19 status rows but **18 canonical lanes + 1 standing program** per `r3-structure.md` §"Summary" line 23 + `r3-program-plan.md` §1.5 line 90; §3 splits T-V-L4 + T-V-L7 as separate status rows for sub-tracking convenience, but canonical lane name is **T-V-L4-L7-Direct** treating them as one lane — see Finding F10 below):

| Status | Count | Lanes |
|---|---|---|
| GREEN (no open work) | 0 | (none) |
| YELLOW (work in flight) | 14 | T-Tier3-Dissolution, T-LensProducer-Retirement, T-V-L4, T-V-L7, T-FixedPoint, T-Numeric-Construction, T-Anthropic-Wire, T-Bridge-Retirement, T-CostLens-Composition, T-Free-Consequences-Demonstration, T-E-P-Producer-Broadening, T-Tests-As-Data-Completeness, T-Lens-Application-Surface, T-Workflow-As-Data, T-Debt-Paydown |
| RED (blocked on upstream) | 5 | T-V-L5-Corpus, T-Omni-Shape-B, T-V2-Retirement, T-Lens-Behavioral-Parity, T-Lens-Self-Application |

**RED-lane analysis**:
- **T-V-L5-Corpus** — waits on T-V-L4 corpus build-out + Shape A grounding. **Cross-target emission consistency lives here** — load-bearing for emission promise.
- **T-Omni-Shape-B** — waits on R2-Evaluator + Shape A targets. (PR #2251 partially landed but lane-overall is RED.)
- **T-V2-Retirement** — waits on T-FixedPoint + T-LensProducer. **Longest-path lane** for R3 close.
- **T-Lens-Behavioral-Parity** — waits on T-E-P-Producer-Broadening. Critical for lens parity demonstration.
- **T-Lens-Self-Application** — waits on T-Workflow-As-Data + T-Lens-Application-Surface. Final demonstration lane.

**SG-0 census state** (`src/v3/compiler/tests/integration/sg0_census_test.rs` at HEAD):

| Section | Count |
|---|---|
| `EXPECTED_HAND_AUTHORED_NON_TEST` | 53 |
| `EXPECTED_HAND_AUTHORED_TEST` | 107 |
| `EXPECTED_HAND_AUTHORED_FRAGMENTS` | 1 |
| **Total hand-authored** | **161** |

**Trajectory**: per `docs/audit/r3-pb0-velocity-walk-2026-05-09.md`, census grew **+30 entries in 9 days** (+3.3/day) at audit time. Current 161 vs the audit's 150 baseline → +11 entries over ~24h. Direction is **wrong** — target is 0 but trend is positive growth.

**Standing R3 Mgr inventory** (5 active + 1 archived):

| Mgr | Session | Inbox | Status |
|---|---|---|---|
| PB Mgr | warm-dove-618 | gunbc#2074 | active 2026-05-09 22:52Z |
| Substrate Mgr | warm-wolf-698 | gunbc#2068 | working; 12 open assignments |
| Grounding Mgr | sunny-koi-893 | gunbc#2063 | idle with capacity |
| Verification Mgr | wise-bear-525 | gunbc#2075 | (state not surveyed today) |
| Debt-Paydown Mgr | gentle-newt-665 | gunbc#2062 | idle post-Phase-3 Task A |
| ~~Evaluator Mgr~~ | ~~crisp-bat-13~~ | (auto-archived) | **3 stranded workstreams** (TC2/TC3/D4) per Director op-gap flag 2026-05-09 |

---

## §3. Gate-coverage matrix — 96 R3-load-bearing gates × Mgr ownership

Lane-to-Mgr mapping (PM-derived from lane scope + r3-structure.md Mgr columns):

| Lane | Mgr (primary) | Gate count |
|---|---|---|
| T-Tier3-Dissolution | Substrate Mgr | 5 |
| T-LensProducer-Retirement | PB Mgr | 6 |
| T-V-L4-L7-Direct | Verification Mgr | 7 (includes #96) |
| T-V-L5-Corpus | Verification Mgr | 1 (#15) |
| T-FixedPoint | PB Mgr | 1 |
| T-Numeric-Construction | Substrate Mgr | 9 |
| T-Omni-Shape-B | Grounding Mgr | 4 |
| T-Anthropic-Wire | Grounding Mgr | 3 |
| T-Bridge-Retirement | PB Mgr (3) + Substrate Mgr (3) | 7 |
| T-CostLens-Composition | Verification Mgr | 5 |
| T-V2-Retirement | PB Mgr | 4 |
| T-Free-Consequences-Demonstration | Verification Mgr | 10 |
| T-E-P-Producer-Broadening | Substrate Mgr | 5 |
| T-Lens-Behavioral-Parity | Substrate Mgr / Cluster F | 6 |
| T-Tests-As-Data-Completeness | Verification Mgr / Substrate (Phase 1) | 5 |
| T-Lens-Application-Surface | Substrate Mgr | 7 |
| T-Workflow-As-Data | Substrate Mgr | 5 |
| T-Lens-Self-Application | (?) — see Finding F2 | 4 |
| R2-Grounding-Rust (#97 only) | Grounding Mgr | 1 |
| R3 Debt-Paydown (standing #75) | Debt-Paydown Mgr | 1 |

**Mgr-by-Mgr gate load** (rough; lanes split across Mgrs counted by their share):

| Mgr | R3 gates owned (approx) |
|---|---|
| Substrate Mgr (warm-wolf-698) | ~38 (T-Numeric-Construction 9 + T-E-P-Producer-Broadening 5 + T-Lens-Application-Surface 7 + T-Workflow-As-Data 5 + T-Lens-Behavioral-Parity 6 + T-Tier3-Dissolution 5 + T-Bridge 3) |
| Verification Mgr (wise-bear-525) | ~28 (T-V-L4-L7-Direct 7 + T-V-L5-Corpus 1 + T-CostLens-Composition 5 + T-Free-Consequences 10 + T-Tests-As-Data 5) |
| PB Mgr (warm-dove-618) | ~14 (T-LensProducer-Retirement 6 + T-FixedPoint 1 + T-V2-Retirement 4 + T-Bridge 3) |
| Grounding Mgr (sunny-koi-893) | ~8 (T-Omni-Shape-B 4 + T-Anthropic-Wire 3 + #97 1) |
| Debt-Paydown Mgr (gentle-newt-665) | 1 (#75) + standing-debt-ledger |
| Unowned / cross-lane | 4 (T-Lens-Self-Application 4) — see F2 |

**Concentration risk**: Substrate Mgr owns ~40% of all R3 gates. If they stall, R3 stalls.

**Verification Mgr load**: ~30% of R3 gates. State at HEAD not surveyed today; this is itself a finding (F3).

---

## §4. SG-0 census coverage — 161 entries × retirement event

Per the program plan + Cluster sequencing audits, SG-0 entries dissolve via:

| Retirement event | Coverage estimate | Authority doc |
|---|---|---|
| **Cluster M Phase 3 (#84 bulk-port)** | ~80-90 of 107 TEST entries | `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` |
| **Cluster F (F-α through F-γ.2)** | ~14 of 53 NON_TEST entries (lens-producer-retirement) | `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` |
| **Per-gate retirement** (e.g. specific test files closing as their lane's gate flips PASSING) | Remainder (~17 TEST + ~39 NON_TEST + 1 FRAGMENT) | individual §1.8 rows |

**Coverage gap**: of the **~57 entries** outside Cluster M + Cluster F, no comprehensive per-entry retirement plan exists. The 53 NON_TEST entries minus the ~14 lens-producer slice = **~39 NON_TEST entries with no named retirement event**. Same problem for ~17 TEST entries that aren't in Cluster M's bulk-port scope.

This is **F4** below — the SG-0 closure-graph is not actually complete; ~35% of the census has no scheduled retirement.

**Velocity check**: at +3.3/day average (last 9 days), the census trajectory is positive — going UP, not down. Cluster M bulk-event must fire in the 8-12 week R3 window, AND the per-gate retirement track must turn negative-net, OR R3 close becomes structurally infeasible.

---

## §5. Lane-coverage audit — orphans + drift

Cross-checking r3-program-plan.md §3 (19-lane status table) vs r3-structure.md §"Lane structure":

**Lanes IN the plan §3 status table**: 19 (counted) + T-Debt-Paydown standing.

**Lanes IN r3-structure.md §"Lane structure"**: should match exactly (single-authority discipline per codex BLOCKING 2026-05-06 finding 1).

**Suspected orphan: T-TestGen** (Finding F1 — see §6). Listed in r3-structure.md prose as "the gate-enabling lane on two axes" (extends predicate vocabulary + lands runner). Three predicates `testgen_structural_coverage` + `testgen_mock_backed_integration_safe` + `testgen_manual_claim_is_first_class` are all `[ext]` tagged — most other gates depend on these to compile. **But T-TestGen has no row in §3 status table**. Either:
- (a) folded silently into T-Tests-As-Data-Completeness (Cluster M Phase 1 #85/#86 are TestClaim infrastructure — plausible)
- (b) considered closed at R2 (some receipts exist; e.g., `testgen_mock_backed_integration_safe` has fixture)
- (c) genuinely orphaned

PM cannot determine from current docs which is correct. This is itself a finding.

**T-Lens-Self-Application Mgr-ownership**: not clearly mapped to one of the 5 Mgrs in any doc surveyed. Listed in §3 as RED waiting on T-Workflow-As-Data + T-Lens-Application-Surface. Owns 3-4 gates including the #59 "recursive_flex_demonstration_landed" narrative-thesis cash-out. **Finding F2**.

**Verification Mgr (wise-bear-525) state at HEAD**: ~~not surveyed; load-bearing knowledge gap~~ — **AMENDED 2026-05-10**: PM follow-up survey (operator correction) confirmed Verification Mgr active with ~12 worker children. Visible worker sessions tied to verification gates: deep-ibex-520 (#11 TC1), lively-raven-404 (#13 TC3), eager-wren-817 (#14 RustDag), still-ferret-898 (#8 SG-0 non-test), witty-swift-269 (alternate Verification Mgr session). Mgr-tier work flows through worker dispatch → worker PRs, not Mgr-direct PR authoring; PM's first-pass survey using Mgr-direct activity proxies (PR-author / comment-author) was structurally wrong. F3 **RETRACTED**.

---

## §6. Findings — gaps, risks, drift

### F1 — T-TestGen lane orphaned in §3 status table

**Severity**: **HIGH** — most R3 gates depend on T-TestGen schema extensions (`[ext]` tag).

**Evidence**: r3-structure.md §"Lane structure" lists T-TestGen as L-sized with scope "Testgen runner, service simulation, first-class TestClaim, DB-15 follow-up." The 3 testgen predicates are all `[ext]` tagged in §1.6. r3-structure.md prose calls T-TestGen "the gate-enabling lane on two axes." **But r3-program-plan.md §3 lane status table contains no T-TestGen row.**

**Implication**: if T-TestGen is truly without an active dispatch, then any §1.8 gate using `[ext]` predicates cannot ratchet from DECLARED to CONSUMER_LANDED, because the predicate vocabulary they need does not yet compile in the runner. This blocks roughly the entire DECLARED tail (~70 gates).

**Remediation candidates** (operator-tier or Director-tier):
- (a) Confirm T-TestGen scope folded into T-Tests-As-Data-Completeness (Cluster M Phase 1 #85/#86 are quantifier + program-generator carriers — these ARE the schema extensions). If true, amend §3 table to make folding explicit.
- (b) If genuinely orphaned, assign a Mgr (most likely Verification Mgr wise-bear-525) and add §3 row with current dispatch + ETA.
- (c) Audit the 3 testgen predicates for extant receipts and possibly mark each individually as PASSING via existing fixtures (the mock-backed one has a fixture).

### F2 — T-Lens-Self-Application Mgr-ownership unclear

**Severity**: MEDIUM — 4 R3 gates (incl. narrative thesis cash-out #59).

**Evidence**: §3 row says "(waits on Workflow-As-Data + LAS)" — no Mgr-owner column entry. r3-structure.md §"Lane structure" should carry the Mgr authority. PM did not surface in survey.

**Implication**: when T-Workflow-As-Data + T-Lens-Application-Surface dependencies clear, who picks up dispatch? Risk of stranded lane.

**Remediation**: confirm Mgr-owner — likely Substrate Mgr or PB Mgr. Add to §3 lane status row + r3-structure.md §"Lane structure" if missing.

### ~~F3 — Verification Mgr state unsurveyed~~ **RETRACTED 2026-05-10**

**Severity**: ~~HIGH~~ — **N/A**. Finding invalidated by operator correction.

**What was claimed**: that Verification Mgr (wise-bear-525 / gunbc#2075) was dormant 2.5 days with no worker children, owning ~28 R3 gates with no dispatch surface.

**What's true**: Verification Mgr is active with ~12 worker children. Visible verification-lane workers: deep-ibex-520 (#11 TC1), lively-raven-404 (#13 TC3), eager-wren-817 (#14 RustDag), still-ferret-898 (#8 SG-0 non-test), witty-swift-269 (alternate Verification Mgr session). Likely ~7 more not surfaced by PM's title-keyword search.

**Why PM got it wrong**: PM survey used Mgr-direct-activity proxies (PR-author / comment-author / inbox commenter). But Mgr-tier work flows through worker dispatch — workers spawn under Mgr inbox issue references, do their work in worker PRs (which the dashboard tracks but which don't show up under "PRs by wise-bear-525"). Subtree digest from earlier was depth-3 from PM's session (deep-wolf-155); wise-bear-525 is sibling-Mgr under Director, not PM's child, so children weren't visible from PM's vantage.

**Lesson recorded**: PM survey discipline — to assess Mgr-tier health, query worker children + assignment-issue references, not Mgr's direct PR/comment activity. Adjusts methodology for future audits.

**Status**: F3 retracted; no remediation needed. T-Free-Consequences + L4/L5/L7 + T-CostLens have active dispatch via Verification Mgr's worker children.

### F4 — SG-0 closure-graph incomplete; ~57 entries lack named retirement event

**Severity**: HIGH — load-bearing for predicate 1 (PB-0).

**Evidence**: of 161 SG-0 entries, Cluster M Phase 3 covers ~80-90 TEST entries; Cluster F covers ~14 NON_TEST entries. Remaining **~57 entries** (17 TEST + 39 NON_TEST + 1 FRAGMENT) have no per-entry retirement plan in any doc surveyed.

**Implication**: even if Cluster M + Cluster F fire on schedule, SG-0 lands at ~57 not 0. Predicate 1 (PB-0) cannot close.

**Remediation**: PM authors per-entry retirement-event mapping for the ~57 unscheduled entries. Each entry → either a named §1.8 gate's CONSUMER_LANDED retirement OR a new dispatch surface OR explicit Director-budget exception.

### F5 — Cross-target emission consistency RED; emission promise at risk

**Severity**: HIGH — load-bearing for thesis facet 3 ("Multi-target emission. Rust production-grade; Python/Go demonstrably working.").

**Evidence**: gate #15 `l5_cross_target_consistency` lives in T-V-L5-Corpus which is **RED** in §3 status table. Status: "(waits on L4 corpus + Shape A grounding)." Gate is DECLARED-only.

**Cross-target optimization gates #51 + #52** (`cross_target_optimization_constant_fold_consistent` + `cross_target_optimization_cost_structurally_derived`) live in T-Free-Consequences-Demonstration which is YELLOW but waits on R2-Evaluator + T-CostLens-Composition.

**Implication**: the multi-target-emission promise (Rust + Python + Go produce equivalent runtime behavior on certification corpus) has no executable consumer at HEAD. If we close R3 with this gate DECLARED-only, the thesis facet ("Multi-target emission") is paper-only.

**Remediation**: surface to Verification Mgr — what is the path from L4 slice-1 (3 Rust/Int W1 seeds) to L5 corpus PASSING? Is there a brief queued? What unblocks Shape A grounding?

### F6 — TC1 #11 canvas-deferral — narrowest carve, but normalize the pattern

**Severity**: LOW (single gate) — but pattern-establishing for "what does post-R3 canvas-deferral mean for thesis honesty?"

**Evidence**: gate #11 `tc1_eta_equivalence_executable` cannot reach PASSING absent #1972 substrate canvas-tier work which is HELD-CANVAS-DEFERRED past R3 per Director (a)-disposition 2026-05-09. Gate stays DECLARED through R3 close; not load-bearing for "honest-close arithmetic" (97 enumerated − 1 = 96 R3-load-bearing).

**Implication**: this is a **principled** single-gate exception, well-documented. Risk is precedent: when subsequent gates hit hard substrate dependencies, will we accumulate more carve-outs and erode the 96 count?

**Remediation**: explicit policy in r3-structure.md or program plan: "carve-deferrals require Director ratification + canonical post-R3 follow-up surface." Currently #11 has both (Director (a)-disposition + canvas pending), but the policy isn't explicit.

### F7 — DECLARED-only tail (70 gates) has no aggregate dispatch ledger

**Severity**: HIGH — represents 73% of R3 gates with no clear path to CONSUMER_LANDED.

**Evidence**: 70 of 97 gates are DECLARED-only. PM did not find a doc that maps each gate to a current brief / dispatch / blocker / ETA. Lanes have YELLOW/RED status but per-gate dispatch evidence is buried in the §1.8 Notes column or not present.

**Implication**: from operator-tier "are we on track" question, no concrete answer exists. PM today gave Brian a partial answer ("Cluster M for ~80, Cluster F for ~14, per-gate for the rest") but the per-gate slice is unspecified.

**Remediation**: PM authors per-gate dispatch ledger as a second tab of this audit (or sibling doc). 70 rows × {gate ID, lane, owner Mgr, current dispatch evidence (brief / PR / "no plan"), blocker, ETA}. This IS the closure-graph.

### F8 — Mgr concentration risk: Substrate ~40% / Verification ~30%

**Severity**: ~~MEDIUM~~ → **LOW** (downgraded 2026-05-10 — both top-2 Mgrs confirmed healthy with active worker children; concern is now structural-precedent rather than acute-stall risk).

**Evidence**: Substrate Mgr owns ~38 of 96 gates; Verification Mgr owns ~28. Two Mgrs collectively own ~70%.

**Implication (revised)**: if either Mgr ever auto-archives or stalls, ~70% of gates lose dispatch surface. The Evaluator-Mgr-auto-archive pattern (3 stranded workstreams TC2/TC3/D4) demonstrates the failure mode is real — but the V-stack is currently active (per F3 retraction).

**Remediation (advisory, not urgent)**: Director-tier policy — define succession-plan for Mgr-archive events (similar to today's Evaluator Mgr stranded surface). Not load-bearing for current cycle; record as standing-policy gap for next R3 close-program retrospective.

### F10 — Lane-set drift across canonical authority docs (NEW 2026-05-10 per codex BLOCKING review)

**Severity**: MEDIUM — single-authority discipline (INVARIANTS P2) violation in lane representation; cascades into any audit/plan that cites a specific count.

**Evidence** (per codex review on this audit, c#TBD):
- **`r3-structure.md` line 23** (§"Summary"): "**18 lanes + 1 standing program**"
- **`r3-program-plan.md` line 90** (§1.5 composition): "across 18 lanes + 1 standing program" with composition listing 18 lane gates
- **`r3-program-plan.md` §3** (lane status table): **19 status rows + 1 standing** — splits T-V-L4 + T-V-L7 as separate rows for sub-status tracking (different blockers + statuses), but canonical lane name is `T-V-L4-L7-Direct` treating them as one lane

**Implication**: this audit's first draft cited "19 lanes" from §3 status table without footnoting the canonical 18-lane count — codex review correctly flagged this as drift downstream of source-doc inconsistency. The deeper issue is upstream: §3 should either consolidate T-V-L4 + T-V-L7 into a single status row matching canonical, OR explicitly footnote that it splits them as a tracking-convenience subdivision.

**Remediation candidates** (operator/Director-tier ratification needed since lane-set is canonical authority):
- (a) **Consolidate §3 status table**: collapse T-V-L4 + T-V-L7 into single `T-V-L4-L7-Direct` row with sub-status notation in Notes column. Restores lane-count single-authority across all 3 docs.
- (b) **Explicitly footnote §3 as sub-status table**: leave §3 as-is, but add header note "splits T-V-L4-L7-Direct into L4 + L7 sub-rows for blocker-tracking; canonical lane count remains 18 + 1 per `r3-structure.md`."
- (c) **Update r3-structure.md to 19 lanes**: split T-V-L4-L7-Direct into two canonical lanes if the L4 vs L7 distinction is structurally meaningful enough to warrant separate Mgr-ownership lines. (Probably not — they have shared scope per r3-structure.md row.)

**PM recommendation**: option (b) — minimal change; preserves §3's tracking utility while restoring single-authority on lane count. Author the footnote on next program-plan amendment.

### F9 — `r3_debt_paydown_zero_remaining` definitional clarity ✓ (no gap)

**Severity**: NONE — well-defined per Director poke-hole 2026-05-06 finding 3.1.

**Evidence**: §1.5 inclusion list crisp: ROADMAP open `- **` rows + sweep §1 Class A/B/C/F/G entries + §10 RED-flagged escalation items count toward zero. Excluded: PM coordination items, scheduled-deletions-with-named-trigger, retirement-receipts, Class D + Class E bridges (have own gates).

This finding is **non-actionable** — included as positive verification that this predicate is not a closure ambiguity.

---

## §7. Critical-path schedule — estimate against 8-12 week R3 window

**Note**: this is a PM rough-estimate, not a Mgr-cycle plan. Real timing comes from per-Mgr daily snapshots (which itself doesn't yet exist as a regular cadence — see Recommendation R3).

### Critical path

**Longest dependency chain to R3 close** (per §3 lane statuses + Cluster M / Cluster F audits):

```
T-E-P-Producer-Broadening (YELLOW, in flight)
  → T-Lens-Behavioral-Parity (RED, waits T-E-P)
  → all 4 lenses behaviorally complete
  → T-LensProducer-Retirement (YELLOW, gates #5/#6/#7)
  → T-V2-Retirement (RED, longest-path)
  → R3 close
```

Parallel critical paths:

```
Cluster M Phase 1 (#85/#86 substrate carriers — Substrate Mgr canvases)
  → Cluster M Phase 2 (#87 cementing-test discipline — Verification Mgr)
  → Cluster M Phase 3 (#84 bulk-port — Verification coordinator + per-class workers)
  → ~80 SG-0 TEST entries dissolve in single bulk event
```

```
Cluster F Phase F-α (parallelism walker port — Substrate Mgr)
  → Phase F-β.1 (effect-enum migration-shape canvas)
  → Phase F-β.2 (atomic migration)
  → Phase F-γ.1 (parallelism Enforce demo via T-LAS)
  → Phase F-γ.2 (lens_capability_register cascade)
  → ~14 SG-0 NON_TEST entries dissolve + #81/#82/#83/#95 close
```

### Window-feasibility

**Best estimate**: 8-12 week window per `feedback_pb_zero_is_r3_close_target` is tight but not impossible **IF**:

- Cluster M dispatches Phase 1 within 1 week (#85/#86 Substrate canvases ratified + worker dispatched)
- Cluster F Phase F-α completes within 2 weeks
- Verification Mgr surveys operational at HEAD (Finding F3 closes)
- T-V-L5-Corpus unblocks within 4 weeks (L4 corpus build-out finishes)
- DECLARED tail (70 gates) gets per-gate dispatch ledger within 2 weeks (PM-authored, Mgrs maintain)
- ~57 unscheduled SG-0 entries get retirement plan within 2 weeks (PM-authored, Mgrs absorb)

**Risk-adjusted view**: today's velocity (+3.3/day SG-0 growth, 11/96 gates GREEN in ~5 days since plan ratification) suggests the lower bound of the window is unrealistic without bulk events firing. Plausibly we land in the 12-16 week range absent step-changes.

### What would make me confident in 8-12 weeks

1. Cluster M Phase 1 ratified-and-dispatched within **5 days** (today is day 1 post-2026-05-09 ratification)
2. Verification Mgr daily-active for next **2 weeks** (no auto-archive; daily worker dispatch)
3. SG-0 trajectory turns negative (net-shrink not net-grow) within **2 weeks**
4. RED lanes (5) all have unblock plan (concrete brief or PR) within **3 weeks**

If any of those four don't hold, R3 likely slips past 12 weeks.

---

## §8. Recommendations

### R1 — Per-gate dispatch ledger (PM-authored)

**Action**: PM (deep-wolf-155) authors `docs/audit/r3-per-gate-dispatch-ledger-2026-05-10.md` (sibling to this audit). 70 DECLARED-only rows × {gate ID, lane, owner Mgr, current dispatch evidence, blocker, ETA estimate}. Updated weekly.

**Effort**: ~3-4 hours.
**Authority**: PM-tier (compilation; ratification of any "no-plan" → "needs-Mgr-canvas" surfaces is operator/Director).

### R2 — Per-SG-0-entry retirement event mapping (PM-authored)

**Action**: PM authors `docs/audit/r3-sg0-retirement-event-map-2026-05-10.md`. 161 entries × {file path, retirement event (Cluster M / Cluster F / per-gate / NO-PLAN), expected window}. Surface ~57 NO-PLAN entries to Mgrs for absorption.

**Effort**: ~2-3 hours.
**Authority**: PM-tier (compilation).

### R3 — Daily SG-0 trajectory snapshot — establish cadence

**Action**: One Mgr (recommend PB Mgr per scope) commits to daily SG-0 snapshot: net delta + breakdown by retirement-event class + projection to 0. Surface daily to operator inbox via gunbc#828 or PM digest.

**Effort**: ~10 min/day post-setup.
**Authority**: Mgr-tier (PB Mgr scope).

### R4 — Verification Mgr survey + state-recovery (immediate)

**Action**: PM survey wise-bear-525 (gunbc#2075) state today: open assignments, worker children, recent activity, dispatch state. Surface findings to Brian. If Verification Mgr is auto-archive-eligible or stalled, surface as Director-tier op gap (parallel to Evaluator Mgr).

**Effort**: ~30 min.
**Authority**: PM-tier (survey + surface).

### R5 — T-TestGen orphan resolution (Director-tier ratification needed)

**Action**: PM surfaces F1 to Director (zesty-bear-812 / gunbc#828). Recommend explicit answer: folded into T-Tests-As-Data-Completeness, OR closed at R2, OR new Mgr-assignment needed. Outcome amends r3-program-plan.md §3 + r3-structure.md §"Lane structure" if change.

**Effort**: PM ~15 min to draft surface; Director cycle to ratify.
**Authority**: Director-tier ratification (lane scope = Mgr authority).

### R6 — RED-lane unblock plans (per Mgr)

**Action**: each of 5 RED lanes gets explicit unblock plan (brief or PR or "blocked-on-X-pending-Y") within 3 weeks. PM tracks; Mgrs author. RED lanes:
- T-V-L5-Corpus (Verification)
- T-Omni-Shape-B (Grounding)
- T-V2-Retirement (PB)
- T-Lens-Behavioral-Parity (Substrate / Cluster F)
- T-Lens-Self-Application (?)

**Effort**: distributed.
**Authority**: per-Mgr.

### R7 — PB Mgr scope tightening to "rust→0 integrator"

**Action**: as discussed in earlier turn — formalize PB Mgr (warm-dove-618) as rust→0 cross-lane integrator. Daily SG-0 trajectory + weekly Cluster M cross-lane status push. Distinct from Debt-Paydown Mgr (catalogue/classify) which is now active-organize-dispatch.

**Effort**: scope amendment to gunbc#1942 + #2074 lane-tracker issues.
**Authority**: operator-tier (lane scope amendment).

### R8 — Mgr-archive succession policy

**Action**: when a Mgr auto-archives (Evaluator Mgr precedent), explicit succession surface — fresh Mgr session at next operator-trigger OR work redistribution to other Mgrs OR scope amendment. Today's stranded surface (3 Evaluator workstreams TC2/TC3/D4) is a real precedent. Codify pattern.

**Effort**: PM drafts; Director ratifies.
**Authority**: Director-tier policy.

---

## §9. Open questions (for operator + Director)

1. **Is the R3 thesis (§1, 6 predicates) complete?** Or are there commitments not captured here that should be folded in? (E.g., explicit "language is target-agnostic across N targets" gate.)

2. **What's the desired feasibility-target for R3 close?** 8-12 week window per current docs is tight. Is 12-16 weeks acceptable if sub-targets land? Operator may want to ratify a realistic window vs aspirational.

3. **Is the carve-deferral pattern (#11 TC1) extensible?** If subsequent gates hit hard substrate dependencies, do we accept more carve-outs (eroding 96 count) or hold the line at 96?

4. **T-TestGen lane structural status (F1)**: is T-TestGen folded into T-Tests-As-Data-Completeness, closed at R2, or genuinely orphaned? Director-tier answer needed.

5. **Mgr concentration risk (F8)**: at what concentration ratio does operator want to trigger load-balancing?

6. **PB Mgr scope tightening (R7)**: is "rust→0 integrator" a real role the operator wants, or is the existing PB Mgr scope sufficient?

---

## §10. Provenance + audit discipline

- **Trigger**: Brian directive 2026-05-10 ~01:00Z
- **Inputs surveyed**: docs/r3-program-plan.md (§1, §1.5, §1.7, §1.8, §3, §10.3); docs/r3-structure.md (§"Lane structure"); src/v3/compiler/tests/integration/sg0_census_test.rs (counts); ROADMAP.md §:53/§:177; recent merge log (last 30 commits); GH issue list (Mgr inboxes + open assignments); subtree digest 2026-05-09 23:04Z + 2026-05-10 ~00:00Z
- **Audit-doc cross-refs**: `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`; `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md`; `docs/audit/r3-pb0-velocity-walk-2026-05-09.md`; `docs/audit/v3-comprehensive-debt-audit-2026-05-09.md`; `docs/audit/r3-cluster-analysis-2026-05-09.md`; `docs/audit/r3-novel-findings-dispatch-tracker-2026-05-09.md`; `docs/admin/branch-protection-recommendation-2026-05-09.md`
- **PM authority**: this is PM-tier observation + recommendation. Substrate-shape changes / Mgr scope amendments / new ratchet creation ALL require operator-tier or Director-tier ratification per session role doc + project memory `user_session_role_pm_under_director.md`.
- **Drift discipline**: counts at this commit are HEAD-anchored; any stale claim at audit-merge-time is a drift candidate per `feedback_thesis_gate_state_drift`.

---

**End of audit.**
