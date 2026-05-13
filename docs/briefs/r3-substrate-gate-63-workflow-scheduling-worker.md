---
status: dispatchable (worker brief; ratified Q1=A per Director msg_804cdc93 relayed via PM msg_1e52a61b 2026-05-13 — SUPERSEDES prior B-shape relay msg_dbc2e5e0)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #63 `substrate_gap_workflow_scheduling_closed`
parent canvas: PR #2831 / `docs/briefs/r3-substrate-gate-63-workflow-scheduling-canvas.md` — Q1=A RATIFIED (revised)
ratification anchor: PM msg_1e52a61b relaying Director msg_804cdc93
---

# Gate #63 — workflow_scheduling closure worker brief (Q1=A administrative)

## §0. Status — DISPATCH-READY (canvas-merge-gated)

Director ratified Candidate A (revised, supersedes prior B-disposition msg_dbc2e5e0). Worker dispatch gates on **PR #2831 (canvas) merging**.

Key structural framing (Director verbatim per msg_804cdc93):

> The sibling failures are NOT 'directly load-bearing for modeled as .dag data criterion' — they're load-bearing for rows #99 + #100 substrate-shape closure paths, which have their own §1.8 ledger entries + closure scopes. Gate #63 closing via Candidate A does NOT hide substrate-debt because rows #99 + #100 carry that debt explicitly with their own DECLARED → CONSUMER_LANDED arc.

Gate #63 closure is **administrative** (un-ignore the already-passing test + ledger flip). The 5 sibling failures from snappy-bear-502's `--include-ignored` run are owned by rows #99/#100, NOT gate #63.

## §1. Ratified scope (Q1=A administrative)

1. Remove `#[ignore]` at `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581`
2. §1.8 row #63 status flip DECLARED → **CONSUMER_LANDED + PASSING** (Director specifies BOTH; `#[ignore]`-removal IS the closure receipt per Q3 planned-deferral anchor commit `73969f4a9`)
3. Audit doc enumerating 5 sibling failures as **"scoped under rows #99 + #100"** — separate Mgr-tier follow-on, NOT R3-close-blocking for #63

## §2. The 5 sibling failures — ledger-mapping (Director-verified)

| BuildBuddy failure (msg_140d9bc7) | Owning §1.8 row |
|---|---|
| `dsl/gunbc/ci_emission.dag` unresolved `CIWorkflowDag` | #100 `project_github_actions_landed` |
| `gunbc_ci_emission_binary_shim_workflow` opaque body | #99 `workflow_runtime_open_enum_landed` (BinaryShim arm) |
| PythonShim placeholder opaque body | #99 (PythonShim arm) |
| `dsl/gunbc/ci_github_actions_workflow.dag` opaque body + `concurrency` type mismatch | #100 |
| `gunbc_ci_emission_substrate_compiles` + `..._authority_compiles` | #99 + #100 substrate-shape close path |

These are **not gate #63 scope**. Audit doc records the mapping for downstream rows #99/#100 brief authoring.

## §3. Phase A — `#[ignore]` removal

Remove `#[ignore]` directive at `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581`. The test below it (`ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator`) passes in isolation per BuildBuddy `9f22cbce-66ff-...`; un-ignored, it now runs under default `cargo test --workspace` invocation as well.

Verification: `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` (no `--ignored` flag needed) passes.

**STOP if test does NOT pass without `--ignored`** — that would indicate the isolation-pass status has regressed since snappy-bear-502 audit; surface to Mgr.

## §4. Phase B — Class 4 systematic bridge inventory + sibling-debt audit document

**§1.4 / §4 conjunctive predicate** requires gate #63 to receipt both **(a)** representative gap-test pass (Phase A) **AND (b)** systematic enumeration of Class 4 class-bridges with count = 0 OR explicit Director allocation (per `docs/r3-program-plan.md` §4 + §7.2 STRUCTURAL exception). Phase A alone is sample-of-class, NOT closure-of-class.

Author `docs/audit/r3-gate-63-sibling-debt-mapping.md` with **two** receipts:

### §4.1 Phase B.1 — Systematic Class 4 bridge inventory (predicate (b))

Two-pass audit (codex BLOCKING e9143f67 — content-keyword grep alone is insufficient because YAML workflow files contain workflow/scheduling FACTS that may not contain any of the keyword tokens; the inventory must be **authority-surface enumeration + all-lines / YAML-structural classification**, with the keyword grep used only as a cross-check).

**Pass 1 — Class 4 authority-surface enumeration (closed file list)**: worker confirms each surface exists at HEAD; any moved/renamed surface is a STOP-and-surface to Mgr.

| Surface | Path | Audit mode |
|---|---|---|
| extdeps GitHub Actions carriers | `dsl/extdeps/github/actions.dag` | every type/sum/record line classified |
| gunbc CI workflow-as-dag substrate | `dsl/gunbc/ci.dag` | every type/sum/record line classified |
| gunbc CI emission DSL | `dsl/gunbc/ci_emission.dag` | every type/sum/record line classified (incl. `type WorkflowRuntime`) |
| gunbc CI GitHub Actions workflow producer | `dsl/gunbc/ci_github_actions_workflow.dag` | every producer node classified |
| gunbc CI demo / evaluator entry | `src/v3/std/t_ci_workflow_as_data_demo.dag` | every node classified |
| Compiler test integration | `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | every `#[test]` fn classified |
| Repo CI workflow YAML | `.github/workflows/*.yml` + `.github/workflows/*.yaml` (ALL FILES) | **all-lines / YAML-structural** — every top-level key (`name`, `on`, `jobs`, `concurrency`, `permissions`, `env`, `defaults`) AND every job-level key (`runs-on`, `steps`, `strategy`, `if`, `needs`, `outputs`, `env`, `concurrency`, `permissions`, `services`, `container`) is a workflow/scheduling fact requiring classification — NOT keyword-filtered |
| Workflow-runtime sibling tests | `src/v3/compiler/tests/integration/` (any file mentioning workflow/ci_workflow/WorkflowRuntime/project_github_actions) | every `#[test]` fn classified |

For YAML surfaces, the audit doc enumerates each file's structural facts (name + trigger shape + per-job `runs-on`/`strategy`?/`concurrency`?/`if`?/`needs`? table) — every YAML fact is a Class 4 fact, and every fact gets a row in the audit doc.

**Pass 2 — Content-keyword cross-check (never the sole receipt)**:

```
git grep -nE "ci_workflow|ci_emission|ci_github_actions|github_actions_workflow|project_github_actions|workflow_runtime|WorkflowRuntime|workflow_as_data|workflow_scheduling|CIWorkflowDag|WorkflowTrigger|WorkflowStep|WorkflowSecret|MatrixStrategy|RunnerSpec|RunnerLabel|concurrency" \
  src/v3/ dsl/ .github/workflows/ \
  | grep -v "^Binary file"
```

Pass 2 regex covers (a) §4.4-sketch carrier names (`WorkflowTrigger`/`WorkflowStep`/`WorkflowSecret`/`CIWorkflowDag`), (b) HEAD-canonical types where they differ from §4.4 sketch (`MatrixStrategy` ≡ §4.4 `WorkflowMatrix`; `RunnerSpec`/`RunnerLabel` ≡ §4.4 `RunnerResource` per §5 mapping table), (c) §1.8 ledger-row identifiers (`project_github_actions` for row #100; `workflow_scheduling` for row #63; `workflow_runtime` for row #99), (d) file-name stems (`ci_emission`/`ci_github_actions`/`github_actions_workflow`), (e) the GitHub Actions scheduling primitive `concurrency`. **Keyword hits OUTSIDE Pass-1 surfaces are themselves discoveries** — potential survivors missed at scoping time; worker MUST add such hits to the surface list and re-classify before the receipt closes (INVARIANTS P2 / modeling-discipline "Facts flow forward"; codex BLOCKING 10885 + e9143f67 + worker:59 + worker:112).

For EACH fact (Pass 1) and each keyword hit (Pass 2), classify as one of:
- **Pass-through**: site executes through v3 cleanly (counts toward predicate (b) GREEN; cite line)
- **Allocated survivor**: site is a known Class 4 bridge that doesn't currently execute through v3 but is **Director-allocated to another §1.8 row** (cite row number, e.g., #99/#100); count separately
- **Unallocated survivor**: NOT allocated to any §1.8 row — **STOP and surface to Mgr**; this is the §1.4 (b) failure mode

Receipt asserts: **unallocated-survivor count = 0**. Allocated survivors are enumerated with row attributions.

### §4.2 Phase B.2 — Sibling-debt audit document (allocation receipts)

Within the same audit doc, enumerate:

- The 5 sibling failures from snappy-bear-502 audit msg_cef1340b with concrete file:line + BuildBuddy invocation `2e1d435a-a6fe-...` cite
- Mapping table per §2 above (failure → owning §1.8 row #99 or #100)
- Director's structural disambiguation quote (msg_804cdc93 verbatim — establishes the Director allocation mechanism)
- Explicit framing: "these failures are Class 4 bridge survivors with explicit Director allocation per msg_804cdc93; they will close as rows #99 + #100 progress through their own DECLARED → CONSUMER_LANDED arcs"
- Note: out-of-scope for this PR; reference for downstream rows #99/#100 brief authoring (Mgr-tier follow-on)

### §4.3 STRUCTURAL exception clause

Per `docs/r3-program-plan.md` §7.2 + `docs/audit/r3-debt-sweep-2026-05-06.md` §3.A: STRUCTURAL exception requires explicit Director allocation citing program shape; Mgr cannot self-classify. Director msg_804cdc93 IS that allocation citation — sibling failures are STRUCTURAL exceptions allocated to rows #99/#100 by Director ratification, not Mgr discretion.

**STOP if Phase B.1 surfaces an unallocated survivor** — gate #63 cannot close on predicate (b) absent a count=0 OR full-allocation receipt; surface to Mgr for Director re-allocation routing.

Cost-of-change: 1 new audit-doc file, no existing-file edits beyond the `#[ignore]` removal in Phase A.

## §5. Phase C — §1.8 row #63 ledger update + §4.4 substrate-prereq reconciliation

Update `docs/r3-program-plan.md` §1.8 row #63 from DECLARED (or CANVAS_RATIFIED if PM ledger-maintenance landed first) → **CONSUMER_LANDED + PASSING**:

- Cite this PR + canvas PR #2831 + Director msg_804cdc93 (the revised ratification, NOT msg_4b13e93f which it supersedes)
- Cite anchor commit `73969f4a9` for `#[ignore]`-planned-deferral receipt
- Cite Phase B audit doc for sibling-debt mapping
- Reference rows #99 + #100 as the rows that carry the sibling-debt explicitly

**§4.4 substrate-prereq reconciliation** (per operator BLOCKING PR #2831 worker.md:62 — INVARIANTS P2/P5 single-closure-authority discipline): `docs/r3-program-plan.md` §4.4 enumerates required substrate carriers (`WorkflowTrigger`, `WorkflowStep`, `WorkflowMatrix`, `WorkflowSecret`, `RunnerResource`, `Workflow<Trigger, Steps, Resources>`). At HEAD these carriers exist under different paths/names:

| §4.4 prereq | Actual HEAD location |
|---|---|
| `Workflow<Trigger, Steps, Resources>` composing carrier | `dsl/extdeps/github/actions.dag:29` `type Workflow` |
| `WorkflowTrigger` sum | `dsl/extdeps/github/actions.dag:51` |
| `WorkflowStep` | `dsl/extdeps/github/actions.dag:222` `type Step` |
| `WorkflowMatrix<Axes>` | `dsl/extdeps/github/actions.dag:300` `type MatrixStrategy` |
| `WorkflowSecret<Name>` | `dsl/extdeps/github/actions.dag:114` |
| `RunnerResource<C>` | `dsl/extdeps/github/actions.dag:205` `type RunnerSpec` + `:211` `type RunnerLabel` |
| `CIWorkflowDag` (canonical workflow-as-dag carrier) | `dsl/gunbc/ci.dag:120-125,191-200` |

Worker brief Phase C MUST update §4.4 carrier-enumeration with a "substrate-prereqs satisfied at HEAD; landing paths differ from original §4.4 sketch — see mapping above" footnote, citing this PR. This dissolves the dual-closure-authority concern: the substrate-prereq enumeration is satisfied + the test-pass receipt is the §1.8 closure receipt. Single closure authority, multi-receipt evidence.

(Note: Director-ratified Q1=A disposition assumed the substrate was landed; operator BLOCKING surfaced that §4.4 needed explicit reconciliation in the worker brief. Phase C now does this reconciliation atomically with the §1.8 row #63 status flip — preserves Director ratification + addresses operator concern.)

## §6. STOP conditions

1. **Gate-criterion test FAILS without `--ignored`** at HEAD — isolation-pass regressed since snappy-bear-502 audit (msg_cef1340b 2026-05-13T04:21Z); surface to Mgr immediately
2. **Director-anchored `#[ignore]` commit at `73969f4a9` has been amended/replaced** — git log audit at HEAD before authoring; if commit history changed, surface and re-verify
3. **`feedback_load_bearing_ratchet_preservation` violation tempted** — if Phase A authoring tempts adding `#[ignore]` to mask any other failure, **STOP** — anti-pattern §7.2 fires
4. **Scope-creep tempted** — if Phase A diagnosis tempts fixing any of the 5 sibling failures, **STOP** — anti-pattern §7.5 fires (those are rows #99/#100 scope, NOT gate #63)
5. **Row #99 or #100 progress has changed status since 2026-05-13** in `docs/r3-program-plan.md` §1.8 — re-verify mapping table; surface to Mgr if rows are no longer DECLARED
6. **Phase B.1 surfaces an unallocated Class 4 bridge survivor** (i.e., a site that doesn't execute through v3 AND has no §1.8 row attribution) — STOP and surface to Mgr; gate #63 cannot close on §1.4 predicate (b) absent count=0 OR full Director-allocated survivors

## §7. Anti-patterns (4 Director-ratified + 1 new Mgr-derived ratified)

PR body MUST cite verbatim + assert receipt-of-compliance:

1. **Closure declared without `#[ignore]` removal** — fail-closed-discipline; un-ignore IS the closure receipt (Phase A satisfies)
2. **Silent-mask preservation** — NO new `#[ignore]` added (atomic-migration; Phase A only removes)
3. **Parallel-authority on CIWorkflowDag execution path** — N/A under Q1=A (no substrate authoring)
4. **Demo-bound closure pretending to be production** — the criterion text "CI workflow modeled as `.dag` data executes through evaluator + produces `DimensionReport<TimingMeasurement>`" is **acknowledged strictly broader than a synthetic demo**. The Q1=A narrowing (demo + gate-criterion test pass = §1.8 closure receipt, with broader production-CI-workflow-as-data scope explicitly deferred) is **Director-ratified scope-narrowing** per msg_804cdc93 (revised disposition; supersedes msg_4b13e93f) — NOT Mgr self-classification. The demo is the receipt instrument Director allocated to gate #63; the residual broader production-CI-workflow scope is Director-allocated to rows #99 (workflow_runtime_open_enum_landed) + #100 (project_github_actions_landed) per Phase B.2 STRUCTURAL allocation doc + §7.2 / debt-sweep §3.A. PR body MUST cite msg_804cdc93 as the explicit narrowing-ratification anchor (codex BLOCKING worker:132); no implicit "demo IS production" claim — the narrowing is an explicit Director scope decision.
5. **NEW — scope-broadening a gate closure to absorb sibling-test substrate-debt with own ledger rows** (Director-ratified per msg_804cdc93 + msg_1e52a61b) — Phase A explicitly does NOT fix any of the 5 sibling failures; they're owned by rows #99/#100

## §8. Verification

- `cargo test --workspace` green (must include the un-ignored gate-criterion test under default invocation) — predicate (a) receipt
- `cargo test -p v3-compiler --test integration t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` green WITHOUT `--ignored` flag (Phase A check)
- Phase B.1 bridge-inventory grep receipt: unallocated-survivor count = 0 — predicate (b) receipt (§1.4 conjunctive)
- Phase B.2 audit doc lands at expected path with 5-survivor allocation table + Director msg_804cdc93 quote (STRUCTURAL exception receipt per §7.2)
- §1.8 row #63 status updated to CONSUMER_LANDED + PASSING with all required cites
- PR body cites:
  - Gate #63 closure (Phase C ledger update)
  - Canvas PR #2831 + Director disposition (PM msg_1e52a61b relaying msg_804cdc93) — the REVISED ratification; explicitly note supersession of prior msg_dbc2e5e0
  - §1.4 conjunctive predicate (a)+(b) receipts (Phase A pass + Phase B.1 unallocated-survivor-count=0 + Phase B.2 STRUCTURAL allocation per §7.2)
  - 5 anti-patterns receipt-of-compliance (§7)
  - Phase B audit doc path
  - `#[ignore]`-anchor commit `73969f4a9` cite
  - 5-row sibling-debt mapping table (§2)

## §9. Out of scope

- **Fixing any of the 5 sibling failures** — owned by rows #99 + #100; explicitly NOT this PR
- **PythonShim arm closure** — owned by row #99 (PythonShim sub-arm); not gate #63
- **Real CI workflow evaluator receipt beyond demo** — the prior Q1=B framing required this; under Q1=A the demo passing receipt is sufficient
- **T-Lens-Self-Application coordination** — Q4 informational-only; cross-Mgr notice already sent by Mgr (msg_a35ec43c to swift-deer-459)
- **Doc-drift sweep on §1.7 vs §1.8** — separate Wave-2 batch

## §10. Reference

- Canvas: PR #2831 / `docs/briefs/r3-substrate-gate-63-workflow-scheduling-canvas.md`
- Director ratification (REVISED, supersedes prior): PM msg_1e52a61b (relaying Director msg_804cdc93)
- Prior superseded ratification: PM msg_dbc2e5e0 (relaying Director msg_4b13e93f) — note explicit supersession in PR body
- snappy-bear-502 audit anchor: msg_140d9bc7 + correction msg_cef1340b
- BuildBuddy invocations: `2e1d435a-a6fe-...` (5 sibling failures), `9f22cbce-66ff-...` (isolation pass)
- `#[ignore]`-anchor commit: `73969f4a9` (2026-05-12T22:23Z)
- Gate #63 row: `docs/r3-program-plan.md:291`
- Sibling-debt ledger rows: #99 `workflow_runtime_open_enum_landed`, #100 `project_github_actions_landed`, #53 `workflow_substrate_carriers_landed` (partial)
- Gate-criterion test: `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs:581-582`
- `feedback_canvas_recommendations_are_preliminary` (Director-cited as Mgr discipline lesson)
- `feedback_grep_substrate_before_naming_ratification` (Director-cited as own discipline lesson)

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Dispatch gate**: PR #2831 (canvas) merged.
**Note**: prior B-shape worker brief at commit `cf163e240d` (this same file) is **SUPERSEDED** by Director's revised Q1=A disposition; this rewrite captures the administrative-closure shape.
