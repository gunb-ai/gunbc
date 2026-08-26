# Wet self-host behavioral receipts: what a re-add would have to be

Status: **PROPOSAL. Nothing here is enrolled and nothing here may be enrolled** until an
operator agreement exists for this specific floor-cut re-add (DESIGN, Building & checks:
"each re-add re-derived from its own first principles under its own operator agreement").
Design and measurement are unblocked; making a required run consume any of this is not.

Subject: the 17 `dag/test/claim/self_host_*_behavioral_witness_test.dag` receipts, whose
doc rows still name the nightly falsifier Wet cadence as their home. `falsifier.yml` and
`gunbc_falsifier_plan` were deleted at `611fd02770` / #8283 on 2026-08-15.

## 1. The population is not one population, and this is the load-bearing finding

The brief reads "17 receipts lost their executor." Measured, that is true of **one** of them.
The 17 split three ways, by different mechanisms, with different asks.

**(a) One receipt genuinely lost a scheduled executor.**
`self_host_logic_behavioral_receipt_holds` is a template row in
`v2.compiler.self_host.wet_receipt_enrollment` `falsifier_self_host_wet_template_entries`
and was scheduled until `falsifier.yml` was deleted. This one, and only this one, is a
floor-cut re-add in the ordinary sense.

**(b) Six are expect_red quarantine rows whose consumer is deleted, and this is the worst
of the three.** `self_host_parse_engine_hooks`, `self_host_use_site_verdict`,
`self_host_discovery_enumeration`, `self_host_target_carriers`, `self_host_body_producer`
and `self_host_03_normalize` are `known_red_probe` rows on
`gunbc.explicit_witness_admission`, `kind: ExecutionWitnessKind`, admitted 2026-07-25
(review #7199) after going red on assemble on cadence run 30126573464. Each row's own reason
says how it is held: *"it runs Wet expect_red on the falsifier quarantine batch; still-red is
agreement."* That batch is `falsifier_self_host_wet_known_red_entries`, and the falsifier
that consumed it is deleted. **An expect_red row is a claim that something is red; with no
executor, nothing establishes the red and nothing would notice it turning green.** The rows
are still counted as admitted debt, so they read as managed. They are unmeasured.

**(c) Ten are per-module receipts that were never enrolled and have never been measured.**
They dispatch through `tools.self_host_module_behavioral_transport_roster`
`compiler_module_behavioral_receipt_for`, keyed by `module_path`, and their enrolment gate is
a `SelfHostWetReceiptBinding` in
`v2.compiler.self_host.seed_emitter_behavioral_wet_module_bindings` — a roster that is
**empty**.

Say what that emptiness does and does not establish, because an earlier revision of this
document got it wrong in the direction that flatters the proposal. The roster's
`falsifier_self_host_wet_module_bindings_revocation_note` records that **"the six SelfEmitted
wet bindings were RED on cadence run 30126573464"** — six, and they are the same six that are
the quarantine rows of (b). The note says nothing about these ten. They have no binding, no
cadence run, and no recorded verdict in either direction. **Their standing is UNKNOWN, not
red.** The earlier revision read the revocation note as covering the whole unenrolled
remainder, which is the authority-substitution failure DESIGN's recurring-failure list names:
a real authority (the revocation) borrowed to answer for a population it does not describe,
with no relation claimed by either carrier. Caught by review 56141 on this PR.

One thing that looks like a defect and is not: `self_host_03_normalize` carries both an
`OfflineLocalRecipe` exclusion row in `gunbc.ci_layer_roots` and a quarantine row here. The
admission row explains it at exact grain — the file's sibling `declared_source_refs_complete_holds`
is offline, so classifying the FILE as expect_red would have orphaned the sibling. Two facts
at two grains, neither restating the other. Left alone.

**What follows for the ask, stated at the grain the evidence actually supports.** Of the 17:
one has a scheduled home and lost it; six carry an executed red from 2026-07-25 and their
dissolution condition states the order explicitly — green locally, *then* restore the binding,
*then* enrol; ten have no measurement at all. **No bound on the number that would pass is
available from any authority in the tree, and this document does not assert one.** The
unmeasured ten are the reason to measure, not evidence for a number.

That is enough to decide the shape without deciding the size. A cadence re-added over "the 17"
would enrol six subjects into a lane guaranteed red on its first run, in the opposite order to
their own stated dissolution, and ten subjects nobody has ever run — which is asking to re-add
an executor in order to find out whether its subjects work. So this proposal asks for a
**measurement route first**, and an enrolment sized by **whatever subset that measurement shows
green**, which may be all of them, none of them, or anything between.

## 2. What this proposal would NOT re-add

Named explicitly, because the sibling lanes are right that "one cron fixes all of them" is
the wrong shape and `gunbc.ci_layer_roots`' own restoration clause argues against it.

- Not the falsifier workflow, the affected-set cadence, or `gunbc_falsifier_plan`. Those were
  cut for their own reasons; nothing here restores an affected-set control.
- Not the ten unmeasured per-module receipts of (c) as a batch. They are measured individually
  and re-admitted one binding at a time; a batch enrolment of subjects with no measurement is
  the same error as a batch enrolment of subjects measured red.
- Not the six quarantine rows of (b) as expect_red. If a measurement shows one green, the
  correct move is the one its dissolution names — delete the admission row and promote the
  witness — not to re-add a batch that inverts a verdict nobody has taken.
- Not the other rosters in the same carrier — `falsifier_native_cache_cold_entries`, the
  emit-on-demand family aggregates, the interpreter dispatch bijection row. Each is a distinct
  class with its own budget and owner, and folding them into one ask is the heterogeneity error.
- Not a hermetic-floor enrolment. These perform live host effects (`extdeps.shell`,
  `extdeps.cargo_build`, `Filesystem.Write`); `run_required_floor`'s hermetic envelope refuses
  them by construction, and mocking that refusal would pass them against a fabricated exit
  status. The floor is the wrong home and there is no version of this that puts them there.

## 3. The measurement route (free to build; asks for nothing)

Before any enrolment question is decidable, one receipt must be runnable by identity and
its outcome must be readable. DESIGN already blesses the shape and the prior art is in tree:
`tools.emission_entry_instrument` `measure_entry_emission`, whose `EmissionMeasurement` has
no spelling in which an unreached stage renders as a stage that ran and found nothing.

Proposed: `tools.wet_receipt_instrument` `measure_receipt`, invoked as an ordinary entry
point (`gunbc run --entry dag/tools/wet_receipt_instrument.dag --function measure
--arg receipt=<identity> --arg report=<path>`), reusing that instrument's stage vocabulary
rather than minting a second one. It runs the existing kernel — `tools.self_host_curated_seed_linked_harness`
`cssl_seed_linked_behavioral_receipt` — and returns a typed measurement, not a `Bool`.

This is a measurement route and **not a gate**: no workflow invokes it, no phase enrols it,
and its exit status reports whether the *instrument* completed, never whether the subject
passed. That distinction is the same one `tools.emission_entry_instrument` carries.

## 4. Execution provenance is the whole design constraint

Requirement, from the sibling lanes and from DESIGN's execution-provenance-loss entry:
**there must be no rendering under which a run that never executed a receipt is
indistinguishable from a run that executed it and found nothing.**

The cautionary case is in the required floor today and is why the current standing is so
easy to misread: the floor's `executed=` counter counts a witness *reaching the fold*, not
its assertion running. Twelve of these are `OfflineLocalRecipe` rows in
`gunbc.ci_layer_roots` and six are admitted expect_red rows on
`gunbc.explicit_witness_admission`; every one is discovered, classified, counted — and none
runs its assertion. A reader counting witnesses named `behavioral_receipt`, or reading the
floor's `known_red_held` figure, concludes the class is held. It is classified, which is a
different fact, and the two render identically today.

The construction, rather than a convention:

- The measurement carrier's stopped states are **variants naming the stage that stopped**
  — emit refused, assembly refused, cargo failed to build, binary not produced, driver ran
  but the pass marker was absent — never a `false`, never an absent count, never a zero.
  `Bool` is the return type that loses this, and it is the return type the receipts have
  today: `compiler_module_behavioral_receipt_for` returns `false` for an unrecognized
  `module_path` (fail-closed, correctly) *and* `false` for a receipt whose comparison
  genuinely diverged. Those two have opposite owners and opposite repairs — a
  not-applicable rendered as a failure, DESIGN's most-repeated state-space conflation.
  Splitting them is a prerequisite of the instrument, not a nicety.
- A pass is only reportable when the driver **executed** and emitted its `pass_marker`.
  Marker absent and marker present are distinct from driver-never-reached.
- If a lane is ever enrolled, its ledger reports per-receipt terminal states, and a lane
  that ran zero receipts must be unrepresentable as a lane that ran N and found none.
  Any aggregate count published without its per-receipt terminal roster reintroduces
  exactly the `executed=` defect one level up.

## 5. Cost, which is why the lane cannot be one batch

Last measured walls, from `falsifier_self_host_wet_cadence_rehome_note` and the #7199
routing: `namespace_import_closure` ~21 min, `self_host_logic` ~34 min, the interpreter
dispatch bijection ~692s against a 600s whole-receipt budget. These are emit + cargo build
+ native run per receipt. A single lane over 17 subjects is hours of wall on one runner,
and the receipts mix already-green ~100s rows with over-budget ones — which is the reason
the 2026-07-25 rehome rejected a single expect_red batch in the first place: the fast rows
would unexpected-green.

So any enrolment ask is sized *after* the instrument reports per-receipt wall, and the unit
of enrolment is a receipt, not the roster.

## 6. What the operator would actually be deciding

Stated as one bounded question rather than a request for a cadence:

> May a scheduled lane exist that runs *the wet self-host behavioral receipts that have an
> executed green measurement*, on a stated cadence, under a stated per-receipt wall budget
> with kill-at-deadline, reporting a per-receipt terminal ledger?

With, attached: the instrument (§3), the per-receipt measurements it produces, and the count
of receipts that are actually green. **That count is unknown today and this proposal offers no
estimate of it** — six receipts have a year-old-by-CI-standards red from one run, ten have
never been measured, and guessing at the total is exactly the fabricated-plausible-output
failure the instrument exists to remove. The decision is meant to be taken against the
measurement, not against a forecast of it.

The proposal is therefore indifferent to which way the count comes out. If enough receipts are
green, a lane is worth asking for and is sized by them. If few or none are, the honest outcome
is **no lane**, and the work converts to root-cause on why they are red — which several sibling
lanes are already positioned to dissolve. A proposal that would accept "no lane" as an answer is
the only kind worth putting in front of an operator.

## 7. Standing correction to the carriers

Independent of any agreement, the doc rows on all 17 receipts and in
`tools.self_host_module_behavioral_transport_roster` still say "Enrolled on nightly
falsifier when frontier row is SelfEmitted." The frontier roster is deleted, the falsifier
is deleted, and the bindings roster is empty, so that sentence names three things that do
not exist. Correcting it is not part of the re-add and needs no agreement; it is the
ordinary stale-citation repair DESIGN §3 already requires, and it is tracked separately
from this proposal so that an unanswered operator question does not hold a correction
hostage.
