---
status: "Director-ratified Mgr canvas (gate #63 §1.8 CANVAS_RATIFIED 2026-05-13 per PR #2831 + Director msg_804cdc93; Q1=Candidate A). Originated from snappy-bear-502 audit msg_140d9bc7."
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #63 `substrate_gap_workflow_scheduling_closed`
authority docs:
  - `docs/r3-program-plan.md:291` — gate #63 row (**CANVAS_RATIFIED** 2026-05-13 via PR #2831; was DECLARED 2026-05-06)
  - `docs/r3-program-plan.md:77` + §4.4 — Class 4 (workflow/scheduling) criterion: "CI workflow modeled as .dag data executes through evaluator + produces DimensionReport<TimingMeasurement>"
  - `docs/r3-structure.md` — T-Workflow-As-Data + T-Lens-Self-Application lane assignment
snappy-bear-502 audit anchor: msg_140d9bc7-6417-45bf-b541-41cba1ea98cd
---

# Gate #63 — workflow_scheduling substrate-shape canvas

## §0. Status

**Single visible surface:** §1.8 row **#63** is **CANVAS_RATIFIED** (2026-05-13, PR **#2831** squash `89df284e3`; Director **msg_804cdc93** — **Q1=Candidate A** administrative closure path). **CONSUMER_LANDED** and **PASSING** are **not** claimed at ratification — program-plan **Notes** + worker brief `docs/briefs/r3-substrate-gate-63-workflow-scheduling-worker.md` own executable closure.

**Provenance:** Row **#63** was **DECLARED** (NEW 2026-05-06); T-WAD Wave-1 substrate landed (PR #2774 YamlStatic, PR #2808 BinaryShim, PR #2798 affected-set, PR #2823 gate #62 FileAttachment). snappy-bear-502 audit **msg_140d9bc7** forced STOP-condition inventory; gate-criterion test passes in isolation (**msg_cef1340b**). §§2–§7 below retain the **pre-ratification** engineering analysis that supported Director disposition; **ledger truth** is always `docs/r3-program-plan.md` §1.8 row **#63** at HEAD.

## §1. Source authority (verbatim)

### Class 4 criterion (`docs/r3-program-plan.md:77`)

> CI workflow modeled as `.dag` data executes through evaluator

### Gate #63 row (`docs/r3-program-plan.md` §1.8)

Authoritative **Status** / **Notes** cells: **§1.8** table row **#63** at HEAD (**CANVAS_RATIFIED** — do not use stale DECLARED-era snapshots). Line numbers in this file’s YAML `authority_docs` cite may drift; grep the row by gate id.

### Class 4 §4.4 (full closure criterion per snappy-bear-502 grep)

> "CI workflow modeled as .dag data executes through evaluator + produces DimensionReport<TimingMeasurement>"

The closure has **two conjuncts**: (a) execution-through-evaluator + (b) DimensionReport<TimingMeasurement> production.

## §2. State at HEAD — partial-landing inventory (snappy-bear-502 audit)

### What exists

- **CIWorkflowDag carrier**: `dsl/gunbc/ci.dag:120-125` with canonical `ci_workflow_dag` at `:191-200`; includes pipeline/edges + `github_actions_workflow` pinned transport
- **WorkflowRuntime 3-arm**: `dsl/gunbc/ci_emission.dag:27` (per ratified Slice 4 canvas) + `project_github_actions` at `:87-92`
  - YamlStatic arm: returns `dag.github_actions_workflow` (PR #2774 landed)
  - BinaryShim arm: concrete thin-shim Workflow body (PR #2808 landed)
  - PythonShim arm: placeholder data
- **Demo evaluator entrypoint**: `src/v3/std/t_ci_workflow_as_data_demo.dag:206-210` returns `DimensionReport<TimingMeasurement>` — the criterion (b) shape exists
- **Integration evaluator receipt**: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:582` exists

### What's the actual state (corrected per snappy-bear-502 msg_cef1340b 2026-05-13)

**The gate-criterion test PASSES in isolation:**
- Test: `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator`
- Command: `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator -- --ignored --exact --nocapture`
- Result: **1 passed in 5.26s** (BuildBuddy `9f22cbce-66ff-41e0-a172-c2e62a69dee0`)

**But sibling tests in the broader `--include-ignored` run fail:**
- BuildBuddy invocation `2e1d435a-a6fe-437e-9e47-aca68c1ed5a7`: 3 passed, 5 failed
- Failure modes (NOT the gate-criterion test itself):
  - `dsl/gunbc/ci_emission.dag` unresolved `CIWorkflowDag` / unknown type
  - `gunbc_ci_emission_binary_shim_workflow` opaque body
  - PythonShim placeholder opaque body
  - `dsl/gunbc/ci_github_actions_workflow.dag` opaque body + `concurrency` field type mismatch
  - `ci_workflow_as_data_demo_pins_*` topology/command tests fail
  - `gunbc_ci_emission_substrate_compiles` fails
  - `gunbc_ci_github_actions_workflow_authority_compiles` fails

**Reading**: the substrate-shape is **closer-to-closed than first thought**. The gate-criterion test itself passes; the `#[ignore]` masks a passing receipt, not a failing one. The 6 sibling failures are **separately-scoped substrate-debt** in adjacent tests/compiles, not the gate-criterion path.

**This materially shifts the candidate evaluation** (see §3 updates below).

## §3. Closure-scope question — three candidates

### Candidate A — minimal repair (un-ignore the passing test) + §1.4 conjunctive predicate (a)+(b) receipts + §4.4 substrate-prereq reconciliation

**Scope shifted per snappy-bear-502 msg_cef1340b correction**:

- Remove `#[ignore]` from `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` at `t_ci_workflow_as_data_demo_test.rs:581` (test ALREADY PASSES in isolation per BuildBuddy `9f22cbce-66ff-...`) — **§1.4 predicate (a) receipt**
- Worker Phase B.1 systematic Class 4 bridge inventory (two-pass — worker brief §4.1 is authoritative): **Pass 1** authority-surface enumeration (closed file list across src/v3/ + dsl/ + .github/workflows/) + all-lines / YAML-structural classification of every workflow/scheduling fact; **Pass 2** content-keyword grep cross-check (never sole receipt). Each fact classified pass-through / allocated-survivor (cite §1.8 row #) / unallocated-survivor; receipt asserts **unallocated-survivor count = 0** — **§1.4 predicate (b) receipt**
- Worker Phase B.2 sibling-debt audit doc: 5 snappy-bear-502 failures enumerated as Director-allocated to rows #99/#100 per Director msg_804cdc93 (the STRUCTURAL exception citation per §7.2 + debt-sweep §3.A); these are Class 4 bridge survivors with explicit Director allocation, not unallocated
- Close gate #63 on **conjunctive (a) + (b) receipts** + §4.4 substrate-prereq reconciliation

**§1.4 conjunctive predicate receipts** (per operator BLOCKING worker:81 + codex BLOCKING 36bb8237): Class 4 closure requires both **(a)** representative gap-test pass AND **(b)** systematic bridge inventory count=0 OR explicit Director allocation per §7.2 STRUCTURAL exception. Phase A satisfies (a); Phase B.1 + B.2 satisfy (b); sample-of-class disqualified.

**§4.4 substrate-prereq reconciliation** (per codex BLOCKING PR #2831 review 67dfd2d4 + operator BLOCKING worker:62): the representative gap-test text is the necessary-but-not-sufficient receipt; §4.4 also enumerates required substrate carriers. Worker brief Phase C performs the explicit reconciliation atomic with the §1.8 row #63 status flip:

| §4.4 prereq | HEAD location |
|---|---|
| `Workflow<Trigger, Steps, Resources>` | `dsl/extdeps/github/actions.dag:29` |
| `WorkflowTrigger` sum | `:51` |
| `WorkflowStep` | `:222` (`type Step`) |
| `WorkflowMatrix<Axes>` | `:300` (`MatrixStrategy`) |
| `WorkflowSecret<Name>` | `:114` |
| `RunnerResource<C>` | `:205` (`RunnerSpec`) + `:211` (`RunnerLabel`) |
| `CIWorkflowDag` (workflow-as-dag canonical) | `dsl/gunbc/ci.dag:120-125,191-200` |

All 7 mapping-table rows (6 §4.4 substrate carriers + CIWorkflowDag canonical composing carrier) EXIST under different paths/names than §4.4 originally sketched. Phase C requires worker to footnote §4.4 with the path-mapping above, citing the closure PR — this dissolves the dual-closure-authority concern (INVARIANTS P2/P5) by making the §4.4 enumeration + §1.8 row #63 closure receipt explicitly consistent. Single closure authority, multi-receipt evidence.

Pros:
- **Smallest possible blast-radius** — single `#[ignore]` removal + Phase B.1/B.2 grep-and-doc receipts + §4.4 footnote + test promotion
- Gate-criterion test is **already passing**; gate closure is administrative
- Sibling-test substrate-debt preserved as separately-scoped (rows #99/#100 carry it)
- No new substrate authoring needed (carriers already landed under different paths)

Cons:
- §4.4 had to be reconciled in-PR rather than canonically aligned earlier; cleanup-of-canonical-§4.4-text is a separate doc-drift sweep

### Candidate B — full CIWorkflowDag/projection path execution

Scope:
- Wire the full repo `.github/workflows/ci.yml`-equivalent CIWorkflowDag through evaluator (not just the synthetic demo)
- All 3 WorkflowRuntime arms (YamlStatic + BinaryShim + PythonShim) must execute end-to-end
- DimensionReport<TimingMeasurement> production for at least one real workflow execution
- Fix all 5 failures + remove `#[ignore]`

Pros:
- "Executes through evaluator" reads as production-grade, not demo-grade
- Closes Python placeholder + opaque body shapes structurally
- Forward-compatible with downstream T-Lens-Self-Application

Cons:
- Significantly larger scope (3-5x Candidate A)
- Risk of substrate-shape questions surfacing during implementation (cascades)
- May need cross-lane coordination with T-Lens-Self-Application Mgr
- Cost-of-change-1 satisfied only at the per-workflow level

### Candidate C — wait for T-Lens-Self-Application

Scope:
- Gate #63 closure waits on T-Lens-Self-Application landing first
- T-Lens-Self-Application provides the generic `DimensionReport` producer route that gate #63's evaluator path consumes
- Once T-Lens-Self-Application gates close, return to gate #63 with substrate-prereqs satisfied

Pros:
- Avoids re-doing work if T-Lens-Self-Application changes the DimensionReport producer shape
- Acknowledges the gate row's dual-lane assignment (T-WAD + T-Lens-Self-Application)
- Preserves the partial-landing as substrate-progress receipt

Cons:
- T-Lens-Self-Application lane status unknown at this canvas authoring; may be R4-bound
- Operator may have intended Class 4 to close IN-R3 (cf. operator framing from msg_4fd650b7 on gate #105: "we need to land this all in R3 please")
- Defers known-broken state (`#[ignore]`-masked 5 failures) instead of repairing

## §4. Key load-bearing finding — `#[ignore]` masking

The `#[ignore]` at `t_ci_workflow_as_data_demo_test.rs:581` is the substrate-closure-state load-bearing fact:
- If it was added when the demo was authored (pre-Wave-1 substrate landing), it represents **deferred fail-closed receipt** awaiting the substrate landing
- If it was added after a regression, it represents **silently-masked broken state**
- Worker audit at HEAD shows the substrate IS landed; therefore the `#[ignore]` should be removable IF Candidate A repair is in scope

**Per `feedback_load_bearing_ratchet_preservation` discipline**, `#[ignore]` masking on gate-criterion tests is a fail-closed-discipline anti-pattern; any closure-scope choice should plan for `#[ignore]` removal.

## §5. Cross-lane coupling — T-Lens-Self-Application

Gate row says lane = `T-Workflow-As-Data + T-Lens-Self-Application`. Two lane interpretations:

1. **AND semantics**: gate #63 requires both lanes' substrate to land before closure → Candidate C
2. **OR / contributory semantics**: either lane's substrate contributes to closure; closure proceeds when sufficient substrate is present → Candidate A or B

Worker audit didn't probe T-Lens-Self-Application substrate state. Canvas authoring at HEAD doesn't have visibility into that lane's progress. Director-tier disposition needed.

## §6. Practice 4 (coproduct dissolution) check

No new sum-type proposed in this canvas. All three candidates work with existing carriers (CIWorkflowDag + WorkflowRuntime + DimensionReport). Practice 4 GREEN/YELLOW/RED not applicable.

## §7. Cost-of-change accounting (REVISED per snappy-bear-502 msg_cef1340b)

Cost-of-change is per the **revised Candidate A scope** (administrative un-ignore; 6 sibling failures excluded as separately-scoped substrate-debt). See §2 + §3 (Candidate A) + §10 for the revised framing.

| Candidate | Files edited to close gate #63 |
|---|---|
| A (administrative un-ignore + ledger; revised) | ~1-3 (single `#[ignore]` removal + ledger row + sibling-debt audit doc) |
| B (full path) | ~10-20 (close 3 WorkflowRuntime arms + repo workflow path + cross-cutting infra) |
| C (defer) | 0 (now); unknowable later |

## §8. Anti-patterns (Mgr-derived for reviewer enforcement)

1. **Closure declared without `#[ignore]` removal** — fail-closed-discipline (§4); any closure must un-ignore the gate-criterion test
2. **Silent-mask preservation** — adding more `#[ignore]` to mask new failures (violates `INVARIANTS.md` P5 "Progress Is Dissolution"; atomic-migration discipline per `feedback_load_bearing_ratchet_preservation`)
3. **Parallel-authority on CIWorkflowDag execution path** — if T-Lens-Self-Application has a competing DimensionReport producer shape, canvas should surface, not duplicate
4. **Demo-bound closure pretending to be production** — if Candidate A repairs only demo evaluator but the criterion text intended broader scope, that's a P2 boundary / duplicate-fact issue (demo passes; production doesn't — two sources of truth). The Q1=A narrowing IS Director-ratified per msg_804cdc93; the anti-pattern fires only on implicit "demo IS production" claims, not the explicit Director-allocated demo-as-receipt + rows #99/#100-as-residual structure. Worker brief §7 #4 carries the verbatim ratification cite.
5. **Scope-broadening a gate closure to absorb sibling-test substrate-debt with own §1.8 ledger rows** (Director-ratified anti-pattern per msg_804cdc93) — gate #63 closure MUST NOT pull the 6 sibling failures (snappy-bear-502 audit) into its scope; those are Director-allocated to rows #99 (workflow_runtime_open_enum_landed) + #100 (project_github_actions_landed) per Phase B.2 STRUCTURAL exception (§7.2 + debt-sweep §3.A). Worker brief §7 #5 carries the verbatim cite.

## §9. Open questions (RATIFIED 2026-05-13 per Director msg_804cdc93; preserved for audit trail)

At authoring time, the following questions were routed to Director:

- **Q1 — Closure-scope candidate**: A (demo repair) / B (full path) / C (defer to T-Lens-Self-Application) → **RATIFIED Q1=A** (Director msg_804cdc93, supersedes msg_4b13e93f)
- **Q2 — Lane semantics**: AND vs OR-semantics → **RATIFIED OR-semantics**
- **Q3 — `#[ignore]` history**: planned-deferral vs regression-mask → **RESOLVED planned-deferral** (anchor commit `73969f4a9` Director-verified)
- **Q4 — Cross-lane coordination**: substrate-lane-owned vs cross-Mgr → **RATIFIED Substrate-lane-owned** with informational cross-Mgr notice (no AND-gate)

Worker brief authored on this ratified shape: `docs/briefs/r3-substrate-gate-63-workflow-scheduling-worker.md`.

## §10. Mgr recommendation (REVISED per snappy-bear-502 msg_cef1340b)

**Initial recommendation** (pre-correction): Candidate B with PythonShim carve-out.

**Revised recommendation** (post-correction): **Candidate A with sibling-debt receipt** — given that the gate-criterion test passes in isolation, gate #63 closure is administrative (un-ignore + ledger). Candidate B would re-author already-working substrate; that's `feedback_no_short_term_solutions` inverted (don't redo passing work).

Recommended **revised scope**:
- Q1=A: Un-ignore `ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` (already passing per `9f22cbce-66ff-...`)
- Document the 6 sibling-test failures as **separately-scoped substrate-debt**; surface to Mgr for follow-on triage but NOT block gate #63 closure
- Close gate #63 on the unblocked passing receipt + ledger update
- Optional follow-on canvas: if criterion text "executes through evaluator" requires more than demo-grade evidence, surface a Tier-2 expansion canvas after gate closure

**Q2: OR-semantics** — both lanes contribute; gate closes when sufficient substrate is present.

**Q3: RESOLVED planned-deferral** (see §9; anchor commit `73969f4a9` Director-verified) — `#[ignore]` was added pre-substrate-landing as planned-deferral, so Candidate A is the correct receipt. (Initial pre-ratification framing "defer to Director" is superseded; preserved in §9 audit trail only.)

**Q4: Substrate-lane-owned** with informational cross-Mgr notification when authoring.

**Sibling-debt note**: the 5 failures from `--include-ignored` are separately worth tracking. Worker brief Phase B.2 surfaces the sibling-debt audit document with the 5 specific failure modes as Director-allocated to rows #99/#100 per msg_804cdc93 (the STRUCTURAL exception receipt) — not a gate #63 closure-blocker.

## §11. Reference

- snappy-bear-502 audit: msg_140d9bc7-6417-45bf-b541-41cba1ea98cd
- BuildBuddy diagnostic invocation: `2e1d435a-a6fe-437e-9e47-aca68c1ed5a7`
- Gate #63 row: `docs/r3-program-plan.md:291`
- Class 4 framing: `docs/r3-program-plan.md:77` + §4.4
- CIWorkflowDag: `dsl/gunbc/ci.dag:120-125`
- WorkflowRuntime: `dsl/gunbc/ci_emission.dag:27`
- Demo entrypoint: `src/v3/std/t_ci_workflow_as_data_demo.dag:206-210`
- `#[ignore]`-masked test: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581-582`

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
