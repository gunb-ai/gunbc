# R3 remaining-work dependency graph + max-parallelization plan

**Status**: DRAFT (2026-05-12). Authored per operator directive 2026-05-12T~20:00Z: "define the remainder of R3 now, including all dependencies, so we can max parallelize all the work."

**Authority**: §1.8 Canonical R3 Closure Ledger at `docs/r3-program-plan.md:225-500` + lane structure at `docs/r3-structure.md` + Cluster F + Cluster M sequencing plans.

---

## ⚠️ CONSUMER DISCIPLINE — READ BEFORE DISPATCHING FROM THIS DOC

This doc is an **authoring-time snapshot** of §1.8 ledger state at **2026-05-12T~17:30Z**. Gate Status values rot between authoring + consumption per `feedback_thesis_gate_state_drift.md`. Consumers (Mgrs forming Wave-1/2/3 work-item scope, briefs citing gate-status, anyone touching §5 worker enumeration) **MUST grep `docs/r3-program-plan.md` §1.8 row Status field at HEAD before forming dispatch scope** — the Status field can transition (DECLARED → CONSUMER_LANDED → PASSING) at any merge between authoring (this doc) and consumption (your dispatch).

**Why this discipline matters**: 4 retractions in the first Wave-1 fan-out alone (see §retractions below) — each driven by Mgrs/PMs trusting authoring-time enumeration over current §1.8 status. The discipline-layered defense (worker-grep + Mgr-grep + reviewer-BLOCKING) caught all 4, but each costs Mgr-tier audit cycles + delayed dispatch.

**Authoritative grep targets** for pre-dispatch verification:
- `docs/r3-program-plan.md` §1.8 row Status field (canonical gate status)
- `docs/history/roadmap-active-deferrals.md` (already-tracked deferrals with explicit dissolution triggers — distinct from the ROADMAP.md "active work" framing)
- `docs/r3-structure.md` lane-owner column (who owns the gate; cross-Mgr ownership routes through PM bridge)
- The PR history for any cited gate (`gh pr list --search "<gate_id> in:title"`)

**When in doubt**, the §1.8 ledger entry beats this dep-graph's §3/§4/§5 enumeration. If they conflict, §1.8 is right.

---

## §retractions — authoring-time-vs-HEAD drift catches (live ledger)

Each entry documents a finding initially enumerated as in-scope here but RETRACTED post-grep-verify against canonical authority. New retractions append. If list grows past ~5-7 items, this doc should be **regenerated against current §1.8** rather than further patched.

**2026-05-12 — V1 (`#87` Phase 2 cementing-discipline pattern brief)**: RETRACTED. Gate was already CONSUMER_LANDED + PASSING at authoring time (PR #2639 + PR #2757 landed the regen cementing harness + `LensOutputEquals` / frozen-oracle witnesses; PR #2287 V1 TC1 first slice merged 2026-05-10). Enumerated as Phase-2-pending in §5 Wave-1 Verification queue. **Caught**: `royal-wolf-47` worker-grep pre-spawn (Verification Mgr `clever-tern-670` archived V1 with no PR; Phase-3 carry-forward remains in V2 #84 coordinator).

**2026-05-12 — F6 (`build.rs` bootstrap priority list)**: RETRACTED from `docs/audit/r3-gpt-5-5-pro-cross-review-findings-2026-05-12.md` novel-finding list. Already tracked at `docs/history/roadmap-active-deferrals.md:172` with explicit dissolution trigger ("termination analysis grows a type-readiness signal so `structural_binding_info_for_variant` can succeed regardless of file order") + yellow-flag threshold ("whenever a third std file joins the priority list"). **Caught**: operator + codex BLOCKING on PR #2787 (two-reviewer convergence). Disposition: deferrals-doc owns trigger; no parallel Wave-2 brief.

**2026-05-12 — S2 (`#85` + `#86` substrate carriers, bundled)**: RETRACTED from Substrate Wave-1 queue. `#85` already landed via PR #2647; `#86` CONSUMER_LANDED + PASSING. Remaining work is consumer-side V-lane, not Substrate scope. **Caught**: codex BLOCKING #10431 on PR #2782 (Substrate Wave-1 brief dispatch). Disposition: Substrate Mgr `warm-wolf-698` fix-forward removed S2 from queue; consumer-side work routes through Verification.

**2026-05-12 — S4 (`#36` `bridge_retirement_ledger_zero`)**: RETRACTED from Substrate Wave-1 queue. Verification-owned per Director `zesty-bear-812` 2026-04-28 distribute-work-centralize-ledger discipline; bridge-retirement-ledger ratchets are Verification-tier, not Substrate-tier. **Caught**: codex BLOCKING #10431 on PR #2782 (same review as S2). Disposition: Substrate Mgr fix-forward removed S4 from queue; gate stays in Verification Mgr scope.

---

**Companion docs**:
- `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` — Cluster F (gates #81/#82/#83/#95)
- `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` — Cluster M (gates #84/#85/#86/#87 — **critical-path**)
- `docs/briefs/r3-wave1-substrate-lane-retractions.md` — Substrate Mgr Wave-1 retraction audit-trail (S2/S4 detail; PR #2782 merged 2026-05-12T21:40:28Z)

---

## §1. State snapshot

**Total**: 103 enumerated gates → **102 R3-load-bearing** (only #11 canvas-deferred per Director 2026-05-09 (a)-disposition).

| Status | Count | % |
|---|---|---|
| 🟢 CLOSED (PASSING) | ~32 | 31% |
| 🟡 IN-FLIGHT (CONSUMER_LANDED) | ~40 | 39% |
| 🔴 OPEN (DECLARED, work-not-started OR no-PR) | ~31 | 30% |

**R3 close formula** (from `r3-program-plan.md`):
> R3 close = ALL 102 R3-load-bearing §1.8 gates GREEN + `r3_debt_paydown_zero_remaining` GREEN.

`r3_debt_paydown_zero_remaining` (#75) is already PASSING — debt-receipt CI discipline live since 2026-05-10.

---

## §2. Critical-path clusters

### **Cluster M — Tests-As-Data-Completeness** (the load-bearing one)

**Gates**: #84 (bulk-port) + #85 (quantifier substrate) + #86 (generator substrate) + #87 (cementing discipline)

**Why critical-path**: `#84 every_rust_test_ports_to_dag_or_generated` dissolves **~80-90 of 101 SG-0 hand-Rust test entries** in a single gate closure. Without #84 closing, the "zero hand-authored" thesis for PB-0 R3 close is violated.

**Phase structure**:
- **Phase 1** (parallel-dispatchable NOW): #85 quantifier substrate + #86 generator substrate. Locked-design-resolved, no canvas needed. Substrate Mgr owns.
- **Phase 2** (#87 cementing discipline): **LANDED at HEAD** — `docs/r3-program-plan.md` §1.8 row #87 is **CONSUMER_LANDED + PASSING** (PR #2639 regen harness + PR #2757 `LensOutputEquals` / `DifferentialEquals` / frozen-oracle receipts; Band-C dispatch successor `src/v3/compiler/tests/dag/cementing_dispatch.dag`). Do **not** respawn “#87 pattern brief” Wave-1 workers; see §retractions. Residual cementing-adjacent Rust outside the `regen.dag` enumeration drains through Phase 3 #84 per [`docs/briefs/r3-v-cluster-m-84-bulkport-coordinator.md`](briefs/r3-v-cluster-m-84-bulkport-coordinator.md).
- **Phase 3** (#84 bulk-port): Coordinator + 6 per-class workers (cementing / reflected-Dag / generic DimensionReport / boundary / R1C residuals / L4/L7/L5 skeleton). **~80-90 entries dissolve**. **Not** predicate-blocked on #87 at HEAD (pattern + dispatch receipt landed); gated on coordinator/class-brief readiness + per-row port work.

**Timeline estimate** (per `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`):
- Phase 1: 1-2 weeks
- Phase 2: 1-2 weeks
- Phase 3: 2-4 weeks parallel per-class
- **Total: 4-8 weeks** for full Cluster M closure

### **Cluster F — Lens Behavioral Parity** (lens-producer retirement)

**Gates**: #81 (parallelism) + #82 (effect_enum) + #83 (register status) + #95 (opt-in iteration demo)

**Why critical**: Director carve-promoted these IN-R3 per 2026-05-09 ratification (`#issuecomment-4412330468`). No R4 carve-out; all 4 in-R3-load-bearing.

**Phase structure** (per `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md`):
- **F-α** (parallel-dispatchable NOW): #81 parallelism walker port from `src/v3/compiler/src/workflow_parallelism.rs` → `.dag`. Substrate-ready. Substrate Mgr owns. M-sized.
- **F-β.1** (canvas-tier, parallel with F-α): #82 effect_enum migration-shape canvas. Substrate Mgr authors; Director ratifies. NO predicate-block on F-α.
- **F-β.2** (cascade post-F-β.1): #82 implementation per ratified shape. 4 sub-phases (4a resource-threading, 4b metadata, 4c lens body, 4d full retirement).
- **F-γ.1** (cascade post-F-α + T-LAS #91): #95 opt-in parallelism demo via `apply_lens(parallelism, fn, Enforce)`.
- **F-γ.2** (cascade post-all-4-lenses COMPLETE): #83 register status (all 4 lenses zero-proxy zero-stub).

**Timeline**: F-α 1-2w, F-β.1 <1w, F-β.2 1-2w post-canvas, F-γ cascade → **3-4 weeks total**.

### **T-Workflow-As-Data (T-WAD FULL R3)** — NEW 2026-05-12

**Gates**: #98-#103 (6 gates added today per PR #2744 §1, Director ratification msg_5cbdad24)

**Status today**:
- #99 `workflow_runtime_open_enum_landed` — **CONSUMER_LANDED + PASSING** (PR #2774 merged 2026-05-12; `WorkflowRuntime` declared at `dsl/gunbc/ci_emission.dag:27`; drift-guard consumer green). §1.8 row ratified 2026-05-13.
- #100 `project_github_actions_landed` — **CONSUMER_LANDED + PASSING** (PR #2774; `project_github_actions` declared at `dsl/gunbc/ci_emission.dag:87` with pinned binding `gunbc_ci_yml_workflow` at :95). §1.8 row ratified 2026-05-13.
- #101 `test_cost_dimension_landed` — **CONSUMER_LANDED + PASSING** (PR #2761; `TestNodeCostDimension` at `src/v3/std/verification.dag:578` + `dsl/std/verification.dag:75`; P5 receipt `test_cost_dimension_landed_on_test_node`). §1.8 row ratified 2026-05-13.
- #102 `slow_test_exemptions_dissolved` — **DECLARED** (pass target: `TestNodeCostDimension` timing facts; interim warn-token bridge in flight — not GREEN)
- #98 `ci_yml_hand_authority_dissolved` — IN-FLIGHT/OPEN
- #103 `ci_uses_affected_set_selection` — OPEN (Slice 7; cool-crab-565 canvas at PR #2766)

**Sequencing**: Slice 4 (#100 substrate / Slice 5 (#98 actual ci.yml swap) / Slice 7 (#103 affected-set) / Slice 8 (substrate completion). Slice 4 in flight (PR #2774); rest cascade.

---

## §3. Remaining work by lane (post-audit 2026-05-12)

| # | Lane | Mgr | Total | 🟢 | 🟡 | 🔴 | Critical? |
|---|---|---|---|---|---|---|---|
| 1 | T-Tier3-Dissolution | PB Mgr | 4 | 2 | 2 | 0 | No |
| 2 | T-LensProducer-Retirement | PB Mgr | 4 | 1 | 1 | 2 | **PB-Runtime gated** |
| 3 | T-V-L4-L7-Direct | Verification | 6 | 3 | 1 | 2 | No |
| 4 | T-V-L5-Corpus | Verification | 1 | 0 | 0 | 1 | Depends on L4 |
| 5 | T-FixedPoint | PB Mgr | 1 | 0 | 1 | 0 | No (R1+R3 interp) |
| 6 | T-Numeric-Construction | Substrate | 8 | 2 | 0 | 6 | No |
| 7 | T-Omni-Shape-B | Substrate | 4 | 4 | 0 | 0 | ✓ DONE |
| 8 | T-Anthropic-Wire | Substrate | 3 | 1 | 1 | 1 | No |
| 9 | T-Bridge-Retirement | Distributed | 6 | 3 | 2 | 1 | No |
| 10 | **T-CostLens-Composition** | Substrate | 4 | 0 | 1 | 3 | Cascade impact |
| 11 | T-V2-Retirement | PB Mgr | 3 | 2 | 0 | 1 | No |
| 12 | T-Free-Consequences-Demonstration | Verification | 10 | 10 | 0 | 0 | ✓ DONE |
| 13 | T-E-P-Producer-Broadening | Substrate | 7 | 0 | 4 | 3 | No |
| 14 | **T-Lens-Behavioral-Parity** | Cross | 5 | 1 | 0 | 4 | **Cluster F** |
| 15 | **T-Tests-As-Data-Completeness** | Verification | 4 | 2 | 1 | 1 | **Cluster M ★** |
| 16 | **T-Lens-Application-Surface** | Cross | 7 | 3 | 1 | 3 | Cascade for F-γ.1 |
| 17 | **T-Workflow-As-Data** | Substrate | 12¹ | 4 | 3 | 5 | T-WAD FULL R3 |
| 18 | T-Lens-Self-Application | Verification | 3 | 0 | 0 | 3 | Cascade post-T-WAD + T-LAS |
| - | R3 Debt-Paydown (standing) | DP Mgr | 1 | 1 | 0 | 0 | ✓ DONE |

**Two lanes 100% done**: T-Omni-Shape-B, T-Free-Consequences-Demonstration. Standing program gate also done.

¹ T-WAD row Total reconciled to status-column sum (4+3+5=12). Canonical §1.8 count in `docs/r3-program-plan.md:90` shows T-WAD = 4 + 6 NEW T-WAD FULL R3 (#98–#103) = 10. The 2-gate delta (12 here vs 10 canonical) reflects status-column overcounting — Substrate Mgr canvas refreshes per `docs/r3-program-plan.md:447` are the formal authority; this row reconciles arithmetic for dispatch-planning auditability and flags the delta for status-drift sweep post-Wave-1 cleanup.

**Star lane**: Cluster M (T-Tests-As-Data-Completeness) — critical-path.

---

## §4. Dependency graph — parallel-dispatchable NOW

### Independent dispatch (no prerequisite blocking; can fire today)

These gates have NO unsatisfied prerequisites. Each is an independent worker spawn:

| Gate | Lane | Mgr | Scope | Size |
|---|---|---|---|---|
| **#81** parallelism_lens_behaviorally_complete | T-LBP / Cluster F | Substrate | walker port `workflow_parallelism.rs` → `.dag` | M |
| **#82** F-β.1 canvas | T-LBP / Cluster F | Substrate | migration-shape ratification (canvas-tier; no predicate block on F-α) | S |
| **#85** forall_exists_quantifier_substrate_landed | Tests-As-Data / Cluster M | Substrate | Quantifier/{ForAll, Exists}, QuantifiedTestClaim carriers; PR #2734 in flight | S-M |
| **#86** program_generator_carrier_landed | Tests-As-Data / Cluster M | Substrate | ProgramGenerator + ProgramShape carriers; PR #2752 DRAFT (close out) | S-M |
| **#84** bulk-port coordinator (framework brief) | Tests-As-Data / Cluster M | Verification | coordinator framework + 6 per-class brief stubs | Brief NOW |
| **#98** ci_yml_hand_authority_dissolved | T-WAD | Substrate | actual ci.yml swap with WAD-emitted artifact (Slice 5) | M (post-Slice 4) |
| **#103** ci_uses_affected_set_selection | T-WAD | Substrate / Verification | Slice 7 affected-set integration; canvas at PR #2766 | M |
| **#2** tier3_computation_mirror_dissolved | Tier3 | PB Mgr | **CONSUMER_LANDED** slice landed (`kernel_algebra_profile` routing ratchet); full **PASSING** awaits Evaluator-backed mirror retirement for `SizeBound` / `CallPattern` scaffolds | S |
| **#15** l5_cross_target_consistency | V-L5-Corpus | Verification | corpus-driven; depends on L4 status | M |
| **#36** bridge_retirement_ledger_zero | Bridge-Retirement | Substrate | unified ledger zero | S |
| **#40** symbolic_cost_expr_equals_executable | CostLens | Substrate | ε path ratified 2026-05-07; Verification Mgr ratchet authoring | S-M |
| **#70** cost_lens_demonstration | CostLens | Substrate | Rust-side composition fixture + ≥2 algebra-instances | M |
| **#73** lens_behavioral_parity_demonstration | T-LBP | Substrate/Verification | all 4 lenses vs frozen v2-oracle snapshot | M |

**Total parallel-dispatchable NOW: ~13 gates** (some are brief authoring, some are immediate worker spawn). **#87** was incorrectly listed here at authoring time — §1.8 shows **PASSING**; see §retractions.

### Cascade-gated (waiting on specific predecessors)

| Gate | Blocked by | When unblocked |
|---|---|---|
| #82 F-β.2 implementation | F-β.1 canvas Director-ratified | post-canvas-ratification |
| #83 register status | #81 + #82 + #79 + #80 all COMPLETE | end-chain Cluster F |
| #95 opt-in iteration demo | #81 + T-LAS #91 | mid-chain Cluster F |
| #84 Phase 3 bulk-port workers | #84 coordinator + per-class brief readiness (§1.8 **#87 already PASSING** — not pattern-blocked) | mid-Cluster M |
| #91 T-LAS enforce_violation_routing | T-LBP cascade COMPLETE | post-cluster-F |
| #56-#59 T-Lens-Self-Application | T-WAD + T-LAS cascade | post-cluster + slice-cascade |
| #15 l5_cross_target_consistency | L4 (#9) corpus completion | L4 mature |

---

## §5. Dispatch wave plan (max parallelization)

### **Wave 1 — IMMEDIATE (today / Day 1)**

**Substrate Mgr (warm-wolf-698)** can spawn UP TO 16 workers (current max_active_children). Currently 1 worker (proud-dove-838). Spawn 5-7 more:
- Worker S1: **#81 parallelism walker port** (M-sized, substrate-ready)
- Worker S2: **#85 + #86 substrate carriers** (bundled per `feedback_bundle_workstreams_per_pr.md`; locked-design-ready)
- Worker S3: **#82 F-β.1 canvas authoring** (Director ratification target)
- Worker S4: **#36 bridge_retirement_ledger_zero** (small, isolated)
- Worker S5: **#73 lens_behavioral_parity_demonstration** (4-lens vs frozen oracle)
- Worker S6: **#103 Slice 7 implementation** (consumes PR #2766 canvas)
- Worker S7: **#98 Slice 5 (post-Slice 4 / PR #2774 land)** — pre-stage brief; spawn worker on Slice 4 merge

**Verification Mgr (clever-tern-670)** can spawn up to 8. Currently 3 (sharp-deer-47, calm-newt-227, tidy-gull-504). Spawn 2 more (**#87** pattern work landed — do not duplicate):
- Worker V1: **#84 Phase 3 coordinator framework brief** (6 per-class stubs)
- Worker V2: **#40 symbolic_cost_expr_equals + #70 cost_lens_demonstration** (bundled)

**Debt-Paydown Mgr (zesty-boar-261)** can spawn up to 8. Currently 0. Spawn 2-3:
- Worker D1: **Slow-test residual sweep** (post-cost-dim-landing follow-on)
- Worker D2: **SG-0 trajectory follow-on** per `docs/audit/r3-sg0-trajectory-tracker.md`

**PB Mgr** (nimble-crab-786 — Wave-1 pre-authored worker briefs landed under `docs/briefs/`):
- **Briefs (author before spawn):** [`r3-wave1-pb1-tier3-gate2-computation-mirror-worker.md`](briefs/r3-wave1-pb1-tier3-gate2-computation-mirror-worker.md) (gate **#2**), [`r3-wave1-pb2-fixedpoint-gate16-r3-horizon-worker.md`](briefs/r3-wave1-pb2-fixedpoint-gate16-r3-horizon-worker.md) (gate **#16**)
- Worker PB1: **#2** remainder — Evaluator-gated dissolution of `SizeBound` / `CallPattern` bootstrap mirrors (post-`kernel_algebra_profile` slice)
- Worker PB2: **#16 pb_self_compile_fixed_point** (R3 interpretation strengthening; parent [`r3-pb-t-fixedpoint-worker.md`](briefs/r3-pb-t-fixedpoint-worker.md))

**Wave 1 worker count: ~12-14 parallel workers**. Mgr capacity not exhausted; can scale further if needed.

### **Wave 2 — Day 7-10 (cascade-gated)**

Triggered by Wave 1 landings:
- **#82 F-β.2 implementation** (4 sub-phase workers post-canvas-ratification)
- **#84 Phase 3 bulk-port** (6 per-class workers once coordinator + class briefs are ready — ~80-90 SG-0 dissolutions begin; **#87 PASSING** removes the old pattern gate)
- **#83 register status** (post-all-4-lenses-COMPLETE)
- **#95 opt-in iteration demo** (post-#81 + post-T-LAS #91)

### **Wave 3 — Day 14-21 (final cascade)**

- T-Lens-Application-Surface Slice B (#91 enforce_violation_routing)
- T-Lens-Self-Application (#56-#59)
- Final lane closures + #36 ledger zero verification

---

## §6. Worker spawn capacity check

Current state (per `dashboard-ops graph deep-wolf-155`):

| Mgr | max_active_children | Currently active | Spawn budget |
|---|---|---|---|
| warm-wolf-698 (Substrate) | 16 | 1 (proud-dove-838) | +15 |
| zesty-boar-261 (Debt-Paydown) | 8 | 0 | +8 |
| clever-tern-670 (Verification) | 8 | 3 (sharp-deer-47 + 2 under sharp-deer-47) | +5 |

**Total R3 spawn budget: +28 workers**. We've been operating at ~5 active R3 workers; could scale 5-6x.

**Pre-condition for high-parallelism dispatch**: pre-authored briefs per `feedback_pre_authored_brief_queue.md`. Director surfaced 3 lane queues earlier — these need to be brief-authored ahead of spawn.

---

## §7. Throughput levers (ranked by leverage)

1. **Land your review-parser fix** (in-flight per your earlier message). Eliminates per-PR PM bypass overhead + rebase-cascades. Single biggest unblock.

2. **Pre-author Wave-1 briefs in bulk** (Director-surfaced queues + this doc's catalog). Mgrs author 5+ briefs each, then spawn workers in batch. Avoids the "Mgr idle → author brief on-demand → spawn → repeat" serialization.

3. **Spawn to Mgr capacity** (up to +28 R3 workers). Right now we're at ~5; we can run 5-6x more parallel work if briefs are queued.

4. **Cluster M Phase 1 is dispatch-ready NOW** — #85 + #86 substrate carriers are locked-design-resolved (no canvas needed). Substrate Mgr can dispatch immediately. **Phase 1 → Phase 2 → Phase 3 cascade IS the critical-path.**

5. **F-β.1 canvas immediate authoring** — #82 migration-shape decisions need Director ratification. Substrate Mgr authors canvas, Director ratifies, Phase F-β.2 unblocks. **Earliest possible canvas dispatch = earliest possible cascade trigger.**

6. **PB Mgr lane** — gate **#2** `kernel_algebra_profile` substrate-authority slice + §1.8 ledger sync can land from nimble-crab-786; Evaluator-gated Tier3 / T-LP / **#16** work remains parallel-dispatchable to child workers.

7. **Class-authorization batch merges** (Director-ratified per msg_836c373b) — eliminates PM-per-PR-auth bottleneck for parser-lag PRs.

---

## §8. Concrete first-day action list

**Operator (Brian) — Day 0**:
1. Review + ratify this dispatch plan
2. Confirm **PB Mgr** lane authority: Wave-1 carrier is **nimble-crab-786** (PR #2781; pre-authored briefs in `docs/briefs/` per §5). Do **not** spawn a second PB Mgr while that session is active. Schedule a **successor** only on explicit handoff after #2781 merges/archives.
3. Optionally fire reviewer-schedule for any PRs that need it (or trust auto-cycle)

**PM (deep-wolf-155) — Day 1 immediately**:
1. Relay Wave-1 dispatch plan to zesty-bear-812 (Director) for ratification + Mgr-tier dispatch
2. Director ratifies and authors next-queue briefs OR delegates to Mgr-tier brief-queue depth
3. Mgrs bulk-author briefs (5+ each) per Wave-1 catalog
4. Mgrs spawn workers up to Wave-1 count (~12-14 workers)

**Cluster M sequencing (critical path)**:
- Day 1: Substrate Mgr spawns S2 (#85 + #86 bundled brief)
- Day 1-3: Workers ship Phase 1 substrate; PR cycles
- Day 4-7: ~~Verification Mgr lands Phase 2 pattern (#87)~~ **DONE at HEAD** (PR #2639 + #2757); focus shifts to #84 coordinator + Phase 3 spawn-up
- Day 7+: Verification Mgr spawns 6 per-class Phase 3 bulk-port workers; ~80-90 SG-0 entries dissolve over 2-4 weeks

**Cluster F sequencing (parallel critical path)**:
- Day 1: Substrate Mgr spawns S1 (#81 walker port) + S3 (#82 F-β.1 canvas)
- Day 3-5: Director ratifies F-β.1 canvas
- Day 5-12: F-β.2 implementation 4 sub-phase workers
- Day 12+: F-γ cascade (#83 + #95) post-completion

---

## §9. Open questions for operator

**Q-A**: PB Mgr **after nimble-crab-786 (PR #2781) handoff** — when that Wave-1 carrier merges/archives, should the next PB Mgr successor spawn under Director (zesty-bear-812) for remaining T-LP-Retirement + Tier3 + V2 + FixedPoint lanes?
- Proposed: spawn via `dashboard-ops work-items create` under Director once the active PB Mgr session is explicitly closed (no duplicate PB Mgr merge surfaces).

**Q-B**: Worker-spawn limits — Mgr capacity is +28 R3 budget. Should we cap aggressive parallelism somewhere (token cost, review-pipeline capacity, etc.)?
- Proposed: spawn up to ~15 Wave-1; observe; scale if reviewer schedule absorbs

**Q-C**: Cluster M Phase 3 bulk-port — 6 per-class workers; should each be its own PR or bundled by 2-3 (per `feedback_bundle_workstreams_per_pr.md`)?
- Proposed: per-class PR (each is its own dissolution domain); merge order can batch

**Q-D**: T-WAD Slice 7 dispatch — PR #2766 (cool-crab-565 canvas) is in clever-tern-670's lane. Slice 7 implementation worker would also go under Verification Mgr?
- Proposed: yes; Slice 7 implementation = Verification Mgr lane; coordinates with T-WAD Substrate via PM relay

**Q-E**: Review-parser fix ETA — knowing approximately when bypass-overhead ends helps cadence planning. Any indication?
- Open

**Q-F**: SG-0 trajectory target — what's the SG-0 count we want to hit before considering R3 close-ready? Current is ~101 per `r3-sg0-trajectory-tracker.md`?
- Proposed: 0 (per PB-0 thesis); Cluster M Phase 3 dissolves ~80-90; final ~10-20 close via individual gate landings

---

## §10. Timeline to R3 close (honest projection)

**Per `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` + `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`**:

- **Day 1-3**: Wave-1 brief authoring + worker spawn-up
- **Week 1**: Phase 1 carriers land (#85 + #86); F-α + F-β.1 canvas in flight
- **Week 2**: ~~Phase 2 #87 pattern~~ **landed pre-snapshot**; F-β.2 implementation begins; T-WAD Slice 4 lands
- **Week 3-4**: Phase 3 bulk-port begins (per-class workers); F-γ cascade; T-WAD Slice 5/7 cascade
- **Week 5-6**: ~80-90 SG-0 entries dissolved via #84; Cluster F all-4-lenses COMPLETE
- **Week 6-8**: Final cascade — T-LAS Slice B + T-Lens-Self-Application + remaining lane closures + ledger zero

**Total**: 6-8 weeks honest timeline from full Wave-1 dispatch to R3-close-ready, contingent on:
- Worker spawn-up reaches +12-14 parallel (currently 5)
- Brief-queue-depth maintained (pre-authored)
- Review-parser fix lands mid-window (reduces coordination tax)
- No major substrate-cascade re-evaluations

**Critical-path nodes**:
- M1(2.8) compiler-surface lowering gap (PR #2774 — proud-dove-838) — substantive engineering; unblocks Slice 5
- Cluster M Phase 3 #84 bulk-port execution — dissolves ~80-90 entries (**#87** receipt stack already landed)
- F-β.1 canvas Director-ratification — unblocks F-β.2 → F-γ cascade

---

— Authored by deep-wolf-155 (PM/CEO) 2026-05-12 per operator directive for R3 remaining-work definition + max-parallelization plan. Consumes §1.8 ledger + Cluster F + Cluster M sequencing plans; PM-tier proposal for Director ratification + Mgr-tier dispatch.
