# Witness execution closure, PR B — scope

PR A (#8646) reclassified 101 identities from verdict debt to route debt and landed the
mechanism that makes the distinction executable. This document scopes what follows it. It is a
scope, not a plan of record: every population below is a MEASUREMENT from run `32358048539`
(head `57a6b67`) unless it says otherwise, and every judgement is marked as judgement.

## What PR A already does — so PR B does not re-do it

The summary line already separates four of the five states parent's proposal names:

```
offered=11103 routed=9783 declined_long=538 declined_live=782
planned=9783 executed=9783 terminal=9783 passed=9475 known_red_held=207
failed=0 route_gap=0 stale_route_gap=0 host_tool_unresolved=0
```

`passed` / `known_red_held` / `route_gap` / `declined_*` are already disjoint and already
printed. What is NOT yet true is the per-ROW label, below.

## 1. The row label is a Bool where the run has five states

**Measured, not argued.** In the PR A fold log, a row the fold deliberately holds prints:

```
//test/claim/exact_witness_admission_witness:both_red_expectations_are_known_red_… FAILED in 2ms
```

while the same run's summary prints `failed=0`. Another session read those lines this cycle and
reported a second breakage on main that does not exist. That is a real cost already paid once.

The defect is a state-space conflation at a named authority — `gunbc.observation_ci_render`
`ci_witness_claim_result_text` takes `passed: Bool` and renders `PASSED`/`FAILED`, so a held
expected-red, a route gap, and a genuine failure are all one token. The seed's
`render_witness_claim_result_text` transports the Bool and its own doc comment already says
every choice about how the line reads lives in the `.dag` authority — so the fix lands there,
widening the parameter to the disposition the fold already computes, and the seed follows.

A held row must print `HELD`, a route gap `NO-ROUTE`, and only an unexpected failure `FAILED`.
This is the cheapest item in PR B and the only one with a measured misread behind it.

## 2. In scope: 56 further route gaps + 29 real defects

- **56 route gaps** beyond PR A's 101, surfaced once the live-tree decline is deleted.
- **29 genuine failures** — witnesses that reach their subject and answer `false`. Each is
  enrolled with a `dissolve_on` trigger.

**Trigger standard (stern-heron-695's, landed 2026-08-19), and it is a gate on this PR:** a
trigger must name a fact FALSE TODAY that becomes true exactly when the fix lands — a new
parameter, a specific new function, a named repro passing. Never a category word, never a
symbol that predates the change. Two rows authored elsewhere tonight had triggers already
satisfied when written, which leaves the row unprotected: it can never fire, so it never
dissolves and never blocks.

**Each of the 29 triggers is run against main before commit. A trigger that passes today is
defective and the row does not land.** That check is part of the diff, not a review request.

## 3. Out of scope, by ruling and now by measurement

- **47 interrupts + 2 over-cost.** Not witness dispositions. The CPU histogram is bimodal with
  an empty 14.4s gap: the low mode is the interrupt FIRING rather than a cost being measured,
  so nobody knows what those 23 identities cost, and 24 of the 25 high-mode rows are opaque
  host calls. An interrupt is a property of the RUN, not of the witness — it does not belong in
  a roster keyed by witness identity, and a fourth roster would assert otherwise.
- **The 6 non-resolving witnesses.** Routed separately — and the routing changed the finding.
  They are NOT broken artifacts. Both missing symbols exist in the tree and neither test file
  references them: `accumulator_copy_findings` is called by
  `v2.lens.complexity_accumulator_copy.roster_gate`, which carries ZERO imports on main while
  using `Finding`, `Optional`, `List`, `dag_language_model`, `port_locus` and more; and
  `AdvisoryLens` is used by `v2.lens.enforcement.lens_module_gate`, whose 12 imports do not
  include `v2.lens.application`. The witnesses are correct artifacts faithfully reporting that
  two production lens modules cannot resolve. Deleting them would delete the only thing
  reporting a live defect.

## 4. The live-tree decline deletion

`DeclinedLiveTree` is retained in PR A and deleted here, with the 782-site cost measurement that
prices it (run `32345970386` measured 626/783 passing under execution). The
`witness_file_from_source` syntactic scan — a parallel authority to the semantic
`reads_live_tree_effective` — is deleted in the same motion, since the split retained it only to
serve the arm.

## 5. The opaque-host-call population — measured, never hand-widened

`ClaimPreemptionReachability::OpaqueHostCallUnbounded` declares
`opaque_host_call_grandfather_population()` exhaustive at ONE member
(`compile_dag_rust_emit_check`). That self-declaration is falsified: 24 further operations share
the shape.

**The instruction is explicit and it is the same rule that repaired the mirror-drift carrier:
do NOT widen the roster to 25 by hand.** Membership is DERIVED from the executing population —
every opaque-host-call identity joined to its exact primitive or operation, whether the external
subject was actually exercised, call count, inclusive and exclusive time, current admission, and
owning realization — and judgement is authored on top of the measurement. A hand-widened roster
would be exactly the census-by-recall that DESIGN §5's oracle ruling rejects: it counts the
members someone remembered.

This is discovery from PR A, not PR B work. It is recorded here so the fast wrong version is not
done later by someone who finds the one-member roster and assumes it needs six more rows.

## 6. Known regression travelling with the mass admission

`v2.test.lens_inert_carrier.inert_carrier_test.inert_carrier_no_unrostered_or_stale` passes in
PR A and fails under the full closure. It is caused by the mass admission, not by the route-gap
mechanism, and it is resolved in this PR rather than carried.
