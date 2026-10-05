# Where the route-gap population drops out of the required run — finding, then proposed repair

Lane: calm-koi-257 (claim-execution-route). Status: finding complete; repair BUILT and under review on
PR #13376 (coordinator ack covered all four conditions; reviews 38602/76419/76431 addressed).

## The population and the run

`v2.workflow.floor_route_gap.floor_route_gap_roster` (authority: `src/v2/workflow/floor_route_gap.dag`)
enrolls identities the floor executes and that reach a host effect the hermetic route has no arm for
(`NoMockResponse` 420, `FilesystemRemoval` 10, `UnpublishedMockCase` 8 among the 425 typed rows; 100 more
as bare identity strings in chunks 00–05). 525 distinct identities across 146 modules; every one currently
resolves against the tree (verified per identity: module present, `test fn` tail present — zero stale rows
on current main; an earlier count of 2 stale rows was an artifact of my worktree lagging main and was
withdrawn).

On required run 37236808750 (merge_group, pr-13305, floor job 111537440982):

    [floor-cost-debt]     floor_route_gap: 5   enrolled identity(ies) suppressed (cost-debt roster withholds them)
    [floor-required-gate] floor_route_gap: 536 enrolled identity(ies) suppressed (module outside the required gate, never loaded)
    [floor-route-gap] roster carries 5 enrolled identity(ies)
    [floor-route-gap] 5 enrolled identity(ies) held as route-gapped; 0 unenrolled route gap(s) reported

Arithmetic: 546 decoded = 536 outside-gate + 5 cost-debt-withheld + 5 carried. The source on the run's merge
commit carried 421 typed rows + 120 legacy strings; current main carries 425 + 120.

## The chain, and the three drop points

For an enrolled identity the required run promises (route-gap .dag header): execute → gap → held; or execute
→ pass → stale-row red; or enrolled → did not execute → red. What actually happens for 541 of 546:

1. **Discovery/import** — preparation ranges over the gate closure. The gate (`required_gate_prefixes`, 11
   prefixes + seed modules, cut 2026-08-29, rung drop "Required gate reduced to the compiler floor") admits
   only 5 of the 146 modules carrying enrollments. 536 rows sit in modules never loaded: no plan, no
   execution, no receipt. *Declared* — the bankruptcy names the population per run in
   `required_floor_disposition.tsv` (`declined_outside_gate_closure=26844`, `declined_outside_required_gate=696`
   on this run; summary line `declared=28274 offered=1430 routed=704`).
2. **Selection/admission** — the only mechanism asserting "enrolled ⇒ executed" is the route-gap join
   (`required_floor_runner.rs`). `suppress_withheld` removes the 541 from the roster *before* the join, so the
   join's universe is the 5 gate-inside rows. The suppressed list is produced **per identity with grounds** —
   and then discarded: route-gap publishes only the two aggregate count lines above. Expected-red got the
   per-identity treatment on 2026-09-01 (`v1_compiler_expected_red_roster_join.rs`: "THE DENOMINATOR IS THE
   ENROLLED ROSTER, NOT THE SURVIVORS", suppressed rows recorded with their ground); route-gap never did.
3. **Decision** — the run greens with "5 held; 0 unenrolled gaps". 541 enrollments are in no ledger the run
   publishes: the claim-cost TSV carries only executed identities (704 rows, 5 `host_effect_refused` — the
   carried ones: `test.claim.parse_test.parse_witness_floor_holds`, `...parse_witness_perf_holds`,
   `test.claim.namespace_import_closure_witness.namespace_import_closure_receipt_holds`,
   `test.claim.namespace_structural_root_exposure_generated_witness_test.namespace_structural_root_exposure_generated_witness_holds`,
   `test.claim.v1_dag_parse_witness.v1_dag_parse_witnesses`); the measurement receipt carries only
   `standing` + `blockers`; the disposition/join TSVs are written by the floor job but **not uploaded**.

## The (a)/(b)/(c) split (sampled, identity grain)

- **(a) Deliberately outside the supported guarantee** — the wet/service families: machine intake
  (`test.claim.machine_intake.mtcollins1_kvm_observer_protocol_wet_witness.*`, 44 rows), fabric (31), spark
  serving (74), runner (13), codex supervised turn (21), workspace storage — claims whose execution needs
  service fabric the hermetic required run does not attach by design. Their route-gap rows were enrolled from
  the pre-bankruptcy era; the bankruptcy (declared 2026-08-29) is the standing disposition for the family,
  and `DeclinedNoCiWetLane`/`DeclinedOutsideRequiredGate` type the planning-level fact per identity. For
  these, a per-identity suppressed record on a required run is **measurement, not closure** — closure for
  them is the gate/wet-lane decision already declared in the rung drop.
- **(b) Meant to support, unobserved** — the hermetic-adjacent families whose route gap is exactly the
  missing mock arm the register exists to demand: filesystem/store mock arms (`artifact_store_fs_witness`,
  `durable_exclusive_hold_file_store`, `materialization_store_local`, `effect_plan_bash`: operations Dir 127,
  DirWithTemplate 130, DigestStdin 42, Run 46, Check 26) and the withdrawn `v2.test.execution.emit_on_demand_*`
  family (probed 2026-10-01, amendment names the restoration trigger: "the restoration trigger above stands
  whole"). The register's demand semantics — "a row leaves when a route is supplied" — require observation;
  with no observing run, no arm gets built and the trigger cannot fire. The suppression comment's promise
  ("observable again ... in the whole-corpus receipts run") is **a promise with no executor**: no workflow in
  `.github/workflows/` (8 files, none scheduled for a corpus pass) runs it, and the rung-drop authority
  itself rules the escape hatch out ("a receipt-only run outside the required path does NOT retire the row").
  So for (b) the absence of that run is not merely a gap — it is the defect: the demand register's only
  claimed observation point does not exist.
- **(c) Defect (structural, live today at zero count)** — the class "enrollment the tree no longer declares".
  The mechanism cannot see it: suppression counts it with the (a)/(b) rows, no arm refuses it, and the
  roster header itself warns "an enrollment nothing observes is a row that can never ask to be removed".
  Zero live rows today (audited); the class is one rename away, and nothing would notice. Also (c): the
  route-gap contract's four-arm sentence still claims whole-roster observability and was never amended when
  gate-bounded suppression landed (2026-08-29) — two authorities now disagree silently.

## Proposed repair (one partition in the existing admission path)

In `suppress_withheld` + the route-gap join in `required_floor_runner.rs`, partition the roster at identity
grain — the cost-debt partition precedent (gunbc#9684) and the expected-red join-report precedent
(2026-09-01), no new system, no new CI job:

1. **Declared + gate-inside** → observed today, join as now (held / stale-red).
2. **Declared + outside-gate (or cost-debt withheld)** → typed, located, per-identity suppressed row carried
   into the outcome (like the expected-red join report) and named per identity on the existing stderr
   channel (pattern: the cost-debt "kept as record" line), each with its ground
   (`OutsideRequiredGate` / `CostDebtWithheld`). Label: for (a) this is measurement; for (b) it is the
   explicit disposition that replaces the green absence.
3. **Undeclared** (module absent from the discovery roots / identity tail absent) → **typed refusal**, red,
   naming identity and roster. Fires on zero rows today; keeps it that way.
4. **Text amendments**: the route-gap header's four-arm sentence and the suppression comment's
   "whole-corpus receipts run" promise are amended to state the real standing (dormant until the gate
   widens; no receipts run exists and none retires rows).

Acceptance path: author a fresh claim in an out-of-gate module, enroll it in `floor_route_gap`, and the
required run records it per identity with ground `OutsideRequiredGate` — or, if misnamed, refuses. Heavy
runs remote/CI only.

## Repair as approved and built (coordinator ack, four conditions)

Built in the coordinator's order; the Rust delta mirrors `partition_cost_debt_roster`, which also lives in
seed Rust (`cli_run.rs`) — stated here and in the PR body because the route-gap .dag **cannot** express the
refuse-if-undeclared decision: the roster decodes in a hermetic frame that never loads out-of-gate modules,
so declaredness is knowledge only the discovery walk has. The .dag authority owns what the arms MEAN (its
header contract is amended below); Rust executes them.

1. **The wall (closure)**: `route_gap_suppressed_undeclared` + the refusal at the route-gap suppression
   site — an enrollment the tree does not declare (module or tail absent from the discovery roots; the
   disposition index covers every declared witness identity, gunbc#9684) refuses the run with
   `cause=RouteGapEnrollmentUndeclared`, naming the identities. Fresh-recurrence control: a newly misnamed
   enrollment refuses (`route_gap_admission_partition_tests`, three tests, including the declared-dormancy
   negative control).
2. **Single authority**: the route-gap .dag header contract now carries the gate-bounded amendment (the
   two further arms: suppressed-dormant with ground, undeclared-refuses) beside the original four; the
   `suppress_withheld` stderr line no longer promises a whole-corpus receipts run — it states the true
   standing: no other observation point exists, and consuming rosters record suppressed identities per
   identity, never as a bare count.
3. **Measurement (labeled)**: the route-gap suppression site prints every suppressed identity with its
   ground (`suppression_ground_label`), one line per ground, headed MEASUREMENT — for the (a) families this
   records declared dormancy and closes nothing.
4. **NOT closed here — class (b)**: an out-of-gate enrollment still demands the route it names, and a
   per-identity suppressed row for the `artifact_store_fs` / `effect_plan_bash` / `emit_on_demand` families
   remains a green absence of something meant to be observed. What would actually observe them, with its
   trigger: **(i)** gate admission of those modules — the bankruptcy's own restoration trigger, "a required
   lane resolves every module"; each readmitted module brings its enrollments straight back under the
   four-arm join; or **(ii)** a dispatch instrument row that schedules a receipts run over the named
   families with the same join armed — which would need a required lane to hang it on, because the rung
   drop rules receipt-only runs out as a retirement path. Neither is built in this PR.

**The wall's real standing (corrected per review 76431 — an earlier revision of this paragraph claimed a
shipped fixture; the fixture is gone, per review 38602, and this is what actually holds):**

- **Green by execution on the required path:** the partition IS the run's own admission step — on every
  required run it executes over the real roster, names every suppressed enrollment per identity with its
  ground (the `[floor-route-gap]` suppressed lines), and the reverse join decides over the observable
  remainder. The consumer is the fold itself; nothing about the green arm is test-only. A freshly authored
  claim shaped like the dropped population (declared, out-of-gate, enrolled) gets the same typed, located
  disposition on that run as the 536 do.
- **Discriminating red: unit-authored only, no CI path.** The misnamed-enrollment control lives in
  `route_gap_admission_partition_tests` and runs via `cargo test --release -p v1-compiler --lib`, which
  `docs/onboarding.md` (line 175) says runs on NO required step — a unit-test red does not block a merge.
  The refusal arm's red has no CI author today, and shipping one would mean shipping a misnamed enrollment
  to the production roster — rejected by review 38602, because it would red main forever and fabricate a
  gap the §5 oracle forbids. The named follow-up that could author it on a required lane is the same (b)
  follow-up above: a dispatch instrument row that runs a probe roster with the join armed. Until then, this
  PR does not claim a CI-authored red for the wall; it claims the typed refusal, green by execution, with
  the discriminator at unit grain.
