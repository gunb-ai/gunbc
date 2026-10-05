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

## Repair as approved and built (coordinator ack, conditions + single-implementation round)

The partition decision is now **modeled on the .dag authority** —
`v2.workflow.floor_route_gap.floor_route_gap_admission_partition` is the single implementation of the
route-gap admission decision; the Rust call site marshals the run's real values (suppressed identities
with their declared ground arms and their per-row membership bit, read O(1) from the keyed disposition
index) into it through `run_in_context_with_args`, and an earlier Rust classifier
(`route_gap_suppressed_undeclared`) was **deleted** rather than kept beside the call — one implementation,
a net reduction of seed decision surface; the seed receipt counts +5 hand items (three for the floor
expectation, two for the modeled partition's shared marshal: the entry name and the suppressed-row
marshal). Membership travels per row as a Boolean so the relation never re-scans a corpus-sized list: an
earlier shape flattened the 28,274-identity declared index for the .dag to linearly re-scan per suppressed
row (541 x 28,274 ≈ 15M interpreted string comparisons per required run) — a cost-shape defect, fixed per
DESIGN §6 rather than retired on "n is small here". What would move even the
marshal into the .dag is a
modeled declared-identity projection the regen lane would maintain; deliberately not built. The
refuse-if-undeclared decision could not have been expressed in .dag on its own because the
declared universe is discovery-walk knowledge (the roster decodes in a hermetic frame whose subject is the
gate closure); modeling the RELATION and marshaling the membership bit per row is the resolution of that.

1. **The wall (closure)**: a refused row names an enrollment the tree does not declare (module or tail
   absent from the discovery roots; the disposition index covers every declared witness identity,
   gunbc#9684) and the run refuses with `cause=RouteGapEnrollmentUndeclared`, naming the identities and
   the modeled entry that decided.
2. **Single authority**: the route-gap .dag header contract states the gate-bounded amendment (the two
   further arms) beside the original four; the `suppress_withheld` stderr line no longer promises a
   whole-corpus receipts run — it states the true standing; and the membership test has exactly one
   implementation, on the authority.
3. **Measurement (labeled)**: every suppressed identity is named with the ground the modeled partition
   returned for it, headed MEASUREMENT — for the (a) families this records declared dormancy and closes
   nothing. The ground itself is a **declared coproduct** (round 3, per review 76626): `ground` carries
   `v2.workflow.floor_route_gap`'s `FloorRouteGapSuppressionGround` (`OutsideRequiredGate |
   WithheldCostDebt | DeclinedNoCiWetLane`, spelled arm-for-arm after the floor's suppression enum as
   the seed realizes it — `v1.expected_red_roster_join`'s suppression ground, generated into the runner
   as a Rust enum) instead of a free label; the marshal decodes the enum into the declared arm and the
   run site refuses an arm the enum does not declare, so a fabricated ground can never pass as a label.
   The arms cannot be shared by import because the eval universe's discovery roots are `dag` + `src/v2`
   (no `src/v1`), so name-alignment is what keeps the two spellings one vocabulary.
4. **Not closed here — class (b)**: unchanged from the section above; the named follow-ups are gate
   admission of those modules, or a dispatch instrument row running a probe roster with the join armed.

**Evidence, pinned on both sides of the boundary:**
- **Floor-side route witness** — `test.claim.route_gap_partition_witness`, gate-admitted at exact module
  grain in `required_gate_authored_modules` (the same mechanism and operator ruling as
  `test.claim.discovery_census_witness` and `test.claim.seed_growth_admission_witness`): drives the
  modeled relation over the shared fixture `src/v2/test/fixture/route_gap_admission_partition.dag` and
  asserts both arms on the floor — the route, executed by a required run. Substrate inputs only
  (constructed lists; no host effect, no service).
- **Seed-side pairing witness** — `route_gap_admission_partition_tests` in `required_floor_runner.rs`
  drives the SAME entry by the SAME constant through `run_in_context_with_args` (the real call path the
  run site uses) and asserts both arms at identity grain; a second test pins the marshal shape (identity
  and declared ground arm, nothing else). Local diligence, not a CI path (`docs/onboarding.md` line 175).
- **One implementation**: with the Rust classifier deleted, the run site's route to the answer is the
  modeled entry by construction; a run-path refusal names the entry that decided it.

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
