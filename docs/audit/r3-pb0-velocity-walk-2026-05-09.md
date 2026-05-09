# R3 PB-0 Velocity Walk + SG-0 Census Trajectory — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier audit. Director-greenlit follow-up to PR #2300 cluster analysis (Director acknowledgment relayed at [gunbc#846 #issuecomment-4411924843](https://github.com/gunb-ai/gunbc/issues/846#issuecomment-4411924843), 2026-05-09; subsequent ratification + partner-work delegation at [gunbc#846 #issuecomment-4412008376](https://github.com/gunb-ai/gunbc/issues/846#issuecomment-4412008376)).
**Parent docs**:
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 gates **#8** + **#84** (Pure-Bootstrap-Zero closure gates)
- [`THESIS.md`](../../THESIS.md):298 — "v3's trajectory is the Pure Bootstrap to Zero program (0 hand-maintained)"
- [`ROADMAP.md`](../../ROADMAP.md):53 (T-PB-A non-test → 0) + :88 (T-PB-B test → 0)
- [`docs/audit/r3-cluster-analysis-2026-05-09.md`](r3-cluster-analysis-2026-05-09.md) — prior cluster analysis (under-weighted #8/#84)

---

## §0. TL;DR — load-bearing finding

**The SG-0 census is growing, not shrinking.** Over the last 9 days the ratchet went from 119 entries (2026-04-30) to **149 entries (2026-05-09 audit-time snapshot)** — net **+30 entries**, **+3.3/day** average. R3 close requires this number to reach **0** (gate #8 non-test = 0 + gate #84 test = 0).

> **Snapshot scope (added 2026-05-09 post-codex BLOCKING review)**: numbers below are **audit-time snapshots** at indicated git sha; not refreshed as `origin/main` advances. Live source-of-truth for SG-0 trajectory is [`docs/audit/r3-sg0-trajectory-tracker.md`](r3-sg0-trajectory-tracker.md) — that artifact is updated daily/per-cycle with fragments-inclusive counts. The trajectory finding (growth rate ≥ +3.3/day; gates cannot reach zero at observed velocity) is **structural** and remains valid regardless of point-in-time count drift; specific count cells below should be read as "as of the audit window" not "as of HEAD now."

| Date | non-test (#8 target=0) | test (#84 target=0) | total | Note |
|---|---|---|---|---|
| 2026-04-30 (HEAD~500) | 38 | 81 | 119 | retroactive baseline (excludes 1 fragment entry) |
| 2026-05-02 (HEAD~300) | 40 | 87 | 127 | retroactive |
| 2026-05-06 (HEAD~150) | 46 | 89 | 135 | retroactive |
| 2026-05-07 (HEAD~50) | 47 | 95 | 142 | retroactive |
| **2026-05-09 (audit-time ~`c25b2d8df`)** | **48** | **101** | **149** | **audit-time** snapshot (excluding fragments); 150 fragments-inclusive |

**Numbers in this audit are point-in-time at the audit window**; main has advanced since (see tracker for live numbers). Per [`r3-sg0-trajectory-tracker.md`](r3-sg0-trajectory-tracker.md) §3 history table for live counts.

**At current trajectory the gates never close.** The path to zero is structural — not per-file dissolution at observed velocity. It depends on **single bulk-dissolution events** firing.

---

## §1. Why the census grows despite SG-0 PR-window net-shrink discipline

Per [`ROADMAP.md`](../../ROADMAP.md):177 the SG-0 PR-window discipline allows three pairings for `+N` deltas:
- **(a)** same-PR retirements listing removed paths
- **(b)** Director-budget citation (issue/comment URL)
- **(c)** structural deferral with named follow-up dispatch

**Option (c) is the dominant path.** Every test entry in `EXPECTED_HAND_AUTHORED_TEST` has a header-comment naming a "dissolution trigger" — but the trigger fires **structurally** when a downstream capability lands, not via per-PR retirement. The result: each new gate or feature can land with hand-Rust acceptance + a deferred-dissolution comment, growing the census.

This is by design (the cascade-promotion 2026-04-25 shape). What it requires: **the named dissolution triggers actually fire**.

---

## §2. Root-cause partition — what dissolves each class

Reading the per-entry "Dissolution trigger" / "Dissolves when..." comments in `sg0_census_test.rs`, the 149 entries cluster into a small number of trigger classes:

### §2.1 Test entries (101 entries, gate #84) — dissolution triggers

| Trigger class | Approx count | Dissolution event |
|---|---|---|
| **Testgen covers reflected-Dag structural assertions over std/ types** | ~25-30 | T-Tests-As-Data-Completeness Cluster M lands (#85 quantifier substrate + #86 program-generator carrier) |
| **`.dag` TestClaim runner can execute generic DimensionReport / cementing assertions** | ~20-25 | Gate #84 + #87 (cementing-test discipline complete) — same family as above |
| **PB-Runtime evaluator-as-data lands** | ~5-8 | Cluster F (T-LP-Retirement) — enables Row-4 / R2-Evaluator corpus comparison without host harness |
| **`.dag`-fn-resolution-against-bootstrap as TestClaim** | ~5-8 | T-Tests-As-Data infrastructure (#1966 §3 ratchet predicate scope) |
| **R1 close dissolves wrappers** (R1C-D / R1C-E scaffolds) | ~3 | R1 close (already past — these may be eligible NOW) |
| **L4/L7/L5 skeleton retirement when TestRunner can evaluate directly** | ~5 | Verification Cluster G + M overlap |
| **Specific carrier-test retirements** (each with own structural trigger) | ~10-15 | Various gates |
| **Process-level: m1/m2 boundary tests** | ~10 | Likely T-V2-Retirement + T-Tests-As-Data |

**Net**: ~80-90 of the 101 test entries collapse via **Cluster M (Tests-As-Data-Completeness)** landing. Cluster M is therefore a **bulk-dissolution event** for the test-side ratchet.

### §2.2 Non-test entries (48 entries, gate #8) — dissolution triggers

| Trigger class | Approx count | Dissolution event |
|---|---|---|
| **Bin-shims** (`src/bin/regen_*.rs`, `r1c_e_emit_gates.rs`, `self_host_fixed_point.rs`) | 9 | PB Item 5 (bin-shim emit pattern) — PR #2282 PROPOSAL merged; full retirement pending |
| **Regen-emit support** (`regen_*_emit.rs`, `bootstrap_regen_fresh.rs`, `regen_tokenize.rs`) | 5 | Retires alongside bin-shims |
| **Lens-producer family** (`lens_apply.rs`, `lens_testgen.rs`, `dimension.rs`) | 3 | Cluster F (T-LP-Retirement gates #5/#6/#7) — gated on PB-Runtime interpreter-as-data |
| **emit/codegen** (`emit.rs`, `emit/python_target.rs`, `emit/rust_target.rs`, `emit_rust*.rs`, `emit/collection_ops_method_contract.rs`) | 6 | Self-host via PB-Runtime trampoline (gate #71) + T-V2-Retirement |
| **dag substrate** (`dag.rs`, `dag/builder.rs`, `dag/effects.rs`, `dag/ports.rs`, `dag/cardinality_payload.rs`) | 5 | T-Tier3-Dissolution Cluster K (gates #1-#4) + dag.rs reflection-completeness work |
| **infer/lower/test_runner** | 3 | Self-host via PB-Runtime + Cluster M for test_runner |
| **bootstrap.rs** | 1 | PB-Runtime trampoline (gate #71) |
| **R3 lane carries** (`tier3_mirror_perf.rs`, `workflow_idempotency.rs`, `workflow_parallelism.rs`, `omni_shape_b_openapi.rs`, `pb_method_template_projection.rs` × 2, `r1c_e_gates.rs`, `int_literal_ranges.rs`, `emit_rust_roundtrip_fixtures.rs`, `process_exit.rs`, `self_host_receipt_p0.rs`, `pipeline_authority.rs`, `post_emit_verifier.rs`, `diagnostics.rs`, `lib.rs`, `build.rs`) | 16 | Various lane-specific dissolutions; many tied to T-V2-Retirement + Cluster K + Cluster F |

**Net**: the 48 non-test entries collapse via **PB-Runtime trampoline + T-LP-Retirement + T-V2-Retirement + T-Tier3-Dissolution** firing across Clusters F + K + E. No single event collapses all 48; multiple bulk-dissolution events needed.

---

## §3. Velocity-to-zero math

### §3.1 At current per-file rate
- 9-day window net delta: **+30 entries**
- Current count: 149
- Implied "weeks-to-zero": **never** (negative velocity)

### §3.2 At observed bulk-dissolution event rate

Recent cycle landings since 2026-05-06, partitioned into actual census reductions vs enabling-only landings (no immediate reduction):

**Census-reducing landings** (the only events that count toward bulk-dissolution rate):
- **PR #2279** (Coercion-Fold ScratchIntExamples retired) — reduced ~3 entries (substrate move collapses class)

**Enabling-only landings** (substrate or scaffolding work that opens future bulk-dissolution paths but does NOT reduce census in-PR; some net-add):
- **PR #2281** (G6 emit-shim coherence test) — added 1 entry; enables future v2-retirement bulk drop (gate #97 is the structural enforcement that fails closed when v2 retires without Grounding shim removal)
- **PR #2271** (Substrate T-LBP complexity-lens substrate) — net 0 in census (substrate-shape-only)
- **PR #2200** (T-E-P P1 Slice 6) — added entries (cementing tests under option-(c) deferral)

**Observed bulk-dissolution rate** (using census-reducing landings only): **1 reduction event in 4-day window** ≈ ~0.25/day or ~0.5-1/cycle depending on cycle definition. Even at the upper bound, **not enough to collapse 149 entries inside 8-12 week R3 window via per-PR retirements**. Bulk events (per §3.3) are required.

**Why this matters for §3.3**: rate measurement only counts census-reducing landings; enabling-only landings are tracked separately because they are *prerequisites* for future bulk events (e.g., #2281 enables T-V2-Retirement future drop) but do not themselves reduce the count.

### §3.3 Required bulk-dissolution events for R3 close

**Test side (gate #84 → 0)**:
- **Cluster M COMPLETE** (Tests-As-Data #85/#86/#87 + #84 closure) → bulk-collapses ~80-90 test entries in single event
- **Without Cluster M**: ratchet cannot reach zero (per-file is too slow)

**Non-test side (gate #8 → 0)**:
- **PB-Runtime interpreter-as-data fully landed** (gates #5/#6/#7 + Items 4+5 disposition) → ~14 non-test entries collapse (LP family + bin-shims + regen support + bootstrap)
- **T-Tier3-Dissolution complete** → ~5 entries collapse (dag-substrate family)
- **T-V2-Retirement complete (gate #42)** → ~10 entries collapse (emit/codegen + R3 lane carries)
- **Self-host via PB-Runtime (gate #71)** → ~5 entries collapse (infer/lower/emit-side)
- **Remaining ~14 entries**: each lane-specific; per-file or small-class dissolutions

**At least 4-6 bulk-dissolution events required** for R3 close. Each is a substantial substrate / cluster milestone.

---

## §4. Reclassification: Cluster M is critical-path, not parallel

The PR #2300 cluster analysis classified Cluster M (T-Tests-As-Data-Completeness) as a **parallel cluster** in §2 (worth 2-3 PRs). This audit corrects:

**Cluster M is critical-path-load-bearing for the entire PB-0 closure thesis.** Without Cluster M COMPLETE:
- Gate #84 (test side → 0) cannot close → SG-0 ratchet stays >0 → R3 close blocked
- ~80-90 of 101 test entries cannot dissolve (no testgen capability)
- Per-file retirement cannot make up the gap inside 8-12 week window

**New dependency picture (replaces PR #2300 §2 Cluster M row)**:

The dependency structure has **two distinct edge-classes** that operate on different axes; conflating them was the error in PR #2300 §2 and an early draft of this section (caught by codex review on PR #2358). Both views are below; the second is the "M → B → E sequencing-critical" referenced in §5 Risk 4 and PR #2300 §4 Risk 2.

**View 1 — Substrate-flow critical path** (lane-input dependencies):

```
A T-E-P-Producer-Broadening
   │
   ├─→ B T-Lens-Behavioral-Parity (B consumes A's descent evidence + producer broadening)
   │
   └─→ M T-Tests-As-Data-Completeness (M's substrate carriers #85/#86 consume A; M does NOT depend on B)

A → {B, M} (parallel post-A; B and M have no mutual substrate dep)
```

**View 2 — PB-0 closure flow** (what each gate needs to honestly fire):

```
M T-Tests-As-Data-Completeness COMPLETE  (gates #85/#86/#87 substrate-landed)
   │
   └─→ B's cementing tests can migrate from hand-Rust to `.dag` form
        (current cementing tests live at `tests/integration/cementing/*.rs`)
        │
        └─→ B-cementing-test-capture-in-`.dag`-form COMPLETE (gate #79/#80 PB-0-honest closure)
             │
             └─→ E T-V2-Retirement (gate #42) — v2 oracle frozen via `.dag` cementing receipts
```

**View 2 is the "M → B → E sequencing-critical" referenced in §5 Risk 4 + PR #2300 §4 Risk 2.** It is **not** a substrate-flow contradiction with View 1; it is a *closure-honesty* sequencing — without M, B's cementing tests stay hand-Rust; without B-cementing-in-`.dag`, oracle freeze cannot be honest after v2 retires.

**Net authoritative picture under INVARIANTS P2/P5**:
- **Substrate flow**: A → {B, M} (B and M parallel-from-A; one canonical edge per lane-input dependency)
- **PB-0 closure**: M → (B-PB-0-honest) → E (sequencing-critical for cementing-capture-before-v2-retires; one canonical edge per closure-readiness dependency)
- The two edge-classes describe different relations (substrate-availability vs PB-0-honest-closure-readiness); both are simultaneously true for distinct nodes-pair purposes
- Gate #84 (~80-90 test entries dissolve) requires M COMPLETE per View 1
- Gate #79/#80 PB-0-honest closure requires View 1 (A → B substrate) AND View 2 (M before B-cementing-`.dag`-migration)
- Gate #8 (non-test → 0) dependency: PB-Runtime + T-V2 + T-Tier3 (parallel cluster events; orthogonal to M/B closure flow)
- gates #8 + #84 BOTH GREEN = PB-0 closure = R3 close minimum bar

---

## §5. Honest-close risk update (additions to PR #2300 §4)

**Risk 5 (NEW) — SG-0 census growth trajectory**:
At observed +3.3/day rate, the census is growing 3-4× faster than dissolution events fire. Without bulk-dissolution events firing in the next 4-6 cycles, R3 close becomes impossible inside the 8-12 week window. **Severity**: load-bearing for R3 thesis.

**Risk 6 (NEW) — Cluster M dispatch / authoring status**:
Cluster M's gates (#84/#85/#86/#87) are all DECLARED. No active worker visible at HEAD on any of the four. Verification Mgr (wise-bear-525 #2075) lane scope includes T-Tests-As-Data-Completeness, but recent dispatches focus on Pattern-A executable (Cluster G) and V7 ValueBody. **Cluster M needs explicit dispatch sequencing**; without active authoring, the bulk-dissolution event for ~80% of the test ratchet is not in flight.

**Risk 4 (UPDATE) — #75 pr_anticipation_discipline_ci_active**:
Earlier flagged as "PM/Debt-Paydown standing-program owned." Closer reading of [`ROADMAP.md`](../../ROADMAP.md):177: the script `scripts/check-pr-sg0-net-shrink-discipline.sh` already exists in CI. Gate #75 may be closer to CONSUMER_LANDED than the §1.8 status indicates — verify with Debt-Paydown Mgr.

---

## §6. PM-tier surfaces (recommendations, not authoring)

These are surfaces for Director cycle absorption, not PM-authored directives:

1. **Cluster M sequencing-criticality**: Director-tier sequencing decision on whether Cluster M should ride the M → B → E sequencing-critical chain (currently only B oracle-freeze is documented as sequencing-critical). M → bulk-dissolution → gate #84 = R3 close enablement.

2. **§10 RED elevation candidates** (Director discretion):
   - SG-0 census growth trajectory (Risk 5 above)
   - Cluster M authoring status (Risk 6 above)

3. **Brian-tier framing question**:
   - Is the "Pure-Bootstrap-Zero by R3 close" claim still load-bearing for R3, or has it drifted into a longer-horizon goal? Current trajectory implies it cannot close inside 8-12 weeks unless Cluster M + 3-4 other bulk-dissolution events fire. If the claim is still load-bearing, dispatch needs to prioritize Cluster M. If it's drifted, the R3 plan §1.5/§1.8 framing needs reconciliation.

---

## §7. What's durable in this audit

§2 partition (per-class dissolution triggers) is durable methodology — useful for any future PM walk against SG-0 census. §3 velocity math is point-in-time but the framework (per-file rate vs bulk-dissolution event rate) is reusable. §4 reclassification updates the cluster analysis structure.

The growth-trajectory finding in §0 supersedes any earlier cluster-analysis claim that "structural execution fits in 4-6 days at 10 PRs/day." That was correct at the per-PR level but missed that **PRs are net-adding to the ratchet, not net-shrinking**.

---

## §8. Meta-finding: closure-claims-vs-HEAD drift (added 2026-05-09 per Director audit sweep)

This audit + 2 parallel Director-tier audits (gunbc#828 sweep 2026-05-09 via Director-spawned Explore agents, surfacing 6 additional drift items at gunbc#846 #issuecomment-4412017502) converge on a meta-finding: **R3 program-plan claims drift in the same direction — running ahead of HEAD reality**.

Specific instances surfaced across the 3 audits:

1. **§1.8 status drift** (this audit §1) — 9 gates likely promotable to CONSUMER_LANDED but ledger Status column not refreshed
2. **SG-0 trajectory drift** (this audit §0) — census growing despite "ratchet to zero" framing in T-PB-A/T-PB-B
3. **TC1 #11 plan-language drift** (Director ask 6) — row #11 claimed "flips PASSING on E3.c merge" but ratified disposition is "stays DECLARED through R3 (canvas-tier #1972 deferred post-R3)"
4. **10 demonstration gates runtime-path drift** (Director ask 7) — gates #65-#74 all DECLARED with no runtime path; some need R4-carve OR honest sub-status amendment
5. **Substrate-gap-class #61 enumeration drift** (Director ask 8) — §1.4 conjunctive closure ("gap-test executes AND bridge count=0"); §2.4 only resolved execution path, not enumeration
6. **Gate-count canonicalization drift** (Director ask 9) — `97 enumerated / 94 load-bearing` math has +/- 1 ambiguity in plan text
7. **Gate #95 carve-doc cross-ref drift** (Director ask 10) — `r3-structure.md` cites C1 carve but `r4-carve-out-routing.md` doesn't enumerate #95 explicitly
8. **§10.3 ratification ledger publication drift** (Director ask 11) — single-authority §10.3 cited from ROADMAP / r3-structure without inline disposition; P2 boundary-discipline drift
9. **R4-carve hand-Rust drift** (PM ask 2026-05-09 at #828 #issuecomment-4412052024) — R4-carved C1/C2 lens implementations (`workflow_parallelism.rs` + likely effect-enum walker) stay hand-Rust at R3 close, contradicting THESIS.md:298 "0 hand-maintained" thesis

### Pattern shape

Every instance is the same structural shape: **document text asserts a closure-state that HEAD does not satisfy**. Either:
- Plan asserts gate will close on event X, but X requires post-R3 substrate (drift instances 3, 7)
- Plan asserts ratchet trajectory shape, but HEAD trajectory diverges (instances 1, 2, 9)
- Plan asserts conjunctive closure but only one disjunct shipped (instance 5)
- Plan internal cross-references drift (instances 4, 6, 8)

### Standing recommendation

**Future R3 readiness audits should grep for status-vs-HEAD before trusting plan-text.** Specific tooling proposal:
- Per Mgr cycle: each lane-Mgr cross-checks own §1.8 rows against HEAD evidence; reports drift to PM
- PM-cycle: weekly status-vs-HEAD grep across §1.8 + §10.3 (using §1.8 ledger gate IDs as grep anchors to `dsl/std/`, `src/v3/std/`, and CI consumer paths)
- This artifact (and the SG-0 trajectory tracker `r3-sg0-trajectory-tracker.md`) should be folded into Director-tier progress-bar authoring per operator ask 2026-05-09; honest-trajectory visualization replaces lane-completion-proxy framing

The meta-finding is structural-not-personnel: **plan-text drifts because the HEAD-vs-text reconciliation cadence is not in the standing PM/Director cycle**. Adding it explicitly closes the drift class.

---

**End of audit.**
