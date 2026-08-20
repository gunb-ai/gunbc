# Witness execution closure: 778 identities discovered, declined, and never run

**Found and closed 2026-08-20 (`nimble-wolf-645`).** The required floor discovered 778 witness
identities every run, declined all of them at file grain, and executed none — and since the
2026-08-15 floor cut left `claim_executor --required-floor` as the only witness-executing
consumer in the tree, "declined by the floor" and "not run anywhere" became the same fact.

This document is the finding and the receipt. The change itself is in
`v2.workflow.required_floor`, `v2.workflow.floor_route_gap`, and the seed's site projection.

---

## The measurement

Counted with the floor's own rule (`cli_run` `witness_file_from_source`: a column-zero
`test fn `, and a column-zero `data … LiveTreeDisposition … ReadsLiveTree`), on `4cec10f66a3`:

| population | identities | files |
|---|---|---|
| discovered `test fn` sites | 11,115 | — |
| `DeclinedLongModule` (authored `long.` module name) | 538 | 97 |
| `DeclinedLiveTree` | 782 | 112 |
| routed and executed | remainder | — |

The floor's own run receipt agrees and is the citable number: `gunbc#8638`, run
`32337765205` against `32333231183` at the same base, reported
`claims=9782 declined_long=538 declined_live=778` on the base — the brief's 778 exactly. That
receipt is quoted in `dag/test/claim/seed_mirror_constant_lens_witness_test.dag`, by a session
that had just added four rows to the declined population and measured the delta to prove it.

---

## Three defects, and only the third is about cost

### 1. The decline rested on a premise that had stopped being true

`DeclinedLiveTree` excluded any identity whose FILE declared
`data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree`, on the premise that reaching
the live tree implies "cannot run in the hermetic frame this floor folds".

That premise is false, and had been for some time. Hermetic mode carries the **checkout-read
carve-out** (`v1_interpreter`, the `Filesystem.Read` arm guarded by
`hermetic_checkout_read_disposition`): a read whose path the disposition *confirms* sits under
the checkout root, with no `.git` or `target` component below it, **dispatches to a real read** —
the commit is the run's deterministic input, so it is input access, not a host effect. Reading
committed `.dag` and `.rs` sources is exactly that case, and it is what most of the 778 do.

So the floor was running a **file-grain prediction of an answer the interpreter already decides
exactly, per identity, at the effect boundary** — and predicting it wrong, in the direction that
silently removed coverage.

The independent confirmation is that another session reached the opposite conclusion in writing
one day earlier: *"the required floor folds one hermetic prepared subject, so a live-tree reader
cannot participate by construction"*, followed by a §4b rung drop to *mitigatable* whose
next-rung trigger was *"an executing consumer for the declined-live population"*. The trigger
did not need a new lane. It needed the stale prediction deleted.

### 2. One fact, two computations, in one binary

`reads_live_tree` was derived twice, by methods that cannot agree except by coincidence:

- `cli_run` `witness_file_from_source` — a **syntactic text scan** for the declaration line.
- `cli_run` `reads_live_tree_effective` — reads the same declaration, then falls through to
  `effect_reach_derived_reads_live_tree_for_entry`, a **semantic effect-reachability derivation**
  over the entry's import closure.

These disagree as a function of the import graph (DESIGN §3). The resolution is not to pick one:
the two consumers were asking **different questions**, and only one of them was entitled to the
name. Affected-set selection asks *does this entry's result depend on live tree state* —
`reads_live_tree_effective` keeps that question and is untouched. The floor was asking *can this
identity execute*, which is not a question an authored file-level boolean can answer. The floor's
copy is deleted; nothing replaces it, because execution answers it.

### 3. The run's own honesty check could not see what it dropped

`claims_planned` was the **post-decline** number. The terminal invariant
(`ClaimIdentityCountsDisagree`) compares `planned == executed == receipted` — all three measured
*after* the projection dropped whatever it dropped. A projection that declined a thousand
identities and one that declined none produce identically healthy-looking triples.

And the per-identity receipt (`RequiredFloorDispositionRow` TSV) was gated on
`GUNBC_REQUIRED_FLOOR_DISPOSITION`, which `witnesses.yml` did not set — so on CI the entire
declined population was two integers in a `[floor-phase]` line.

---

## The measurement, and what it settles

The decline arm was deleted on a branch and the whole population executed — floor run
**32345970386**, sha `c812b9fb6d0`, the first run in which every discovered identity was routed:

```
offered=11103 routed=10565 declined_long=538      (partition exact)
planned=10565 executed=10565 terminal=10565 passed=10116
known_red_held=207 failed=36 route_gap=157
interrupted_before_verdict=47 completed_over_cost_requirement=2
```

**Of ~783 identities admitted, 626 pass.** 157 route-gap, and the operations they name are
exactly what this corpus's own exclusion notes have named in prose for months — `Mktemp.Dir` 54,
`IsExecutable` 26, `Run` 17, `emit_host_native_cache_evict` 10, `Check` 10, `Write` 8,
`git.Inspect.HeadCommit` 5, then a tail. **Not one is a committed-source read.** The premise was
stale for the large majority of the population it excluded, not for an interesting minority.

The cost worry did not materialise: 49 of 783 in the cost tail (~6%), 32 minutes wall against a
180-minute timeout.

### The finding nobody was looking for: a third of the expected-red roster was never red

That run carried **308** enrolled expected-red identities and held **207**. The difference is
**101** — and the 101 enrolled identities that route-gapped are *precisely* that set, by
intersection, not by coincidence of counts.

Those 101 were already executing on main every run, already reaching a hermetic refusal. Because
that refusal arrived as a `TypeError` carrying prose, `ExpectedRedArm` could only read it as an
ordinary failure and **held it as agreement**. The debt ledger recorded "this witness runs and
fails and someone is fixing it" about 101 rows that never reached their subject. Nothing was
fixing them, because there was nothing to fix: **they needed a route, not a repair.**

This is the state-space conflation §5 names, hiding inside the mechanism whose entire job is
making debt visible. It also means main's `passed=8702 / known_red_held=306` counted something
narrower than executed coverage, by 101 identities, independently of the 778.

### The cost tail is two mechanisms, not one budget

The CPU histogram over the 49 is bimodal with a **14.4-second empty gap**:

| band | rows |
|---|---|
| 5001–5013 ms | 23 |
| 6682 ms | 1 |
| *(nothing from 6.7s to 21.1s)* | 0 |
| 21109–53301 ms | 25 |

A 12ms spread across 23 rows is **the interrupt firing**, not 23 witnesses that each need five
seconds — and an interrupted claim's cost is a *lower bound*, so those rows' true cost is
**unmeasured**. The low mode cannot be used to argue the budget is too tight.

24 of the high-mode 25 are *also* interrupted, having accrued 21–53s against a 5000ms deadline —
only possible where the stride poll cannot land. That is
`ClaimPreemptionReachability::OpaqueHostCallUnbounded`, already modeled in
`v2.workflow.required_floor`. **Exactly one of the 25 is the known member** of
`opaque_host_call_grandfather_population()`, which declares itself *exhaustive at one member* and
says it "grows only when a DIFFERENT operation is found to share this shape, which is a new
finding, not a declaration." This is that finding: the other 24 reach host-fed `*_live` builtins
in `enforcement_live`, `cost_coverage`, `grammar_coverage`, `lifecycle_survivor_corpus_census`,
and `realization_vocabulary_containment`. The declared "bounded population" was bounded by what
anyone had observed on a floor that was not executing the population where the others live.

## What replaced it

**Delete-first at the root** (DESIGN §3 replacement migration). `DeclinedLiveTree` and the text
scan that fed it are gone; no intermediate representation was built beside them.

- **One execution route per identity.** Every non-long identity is routed and executed. The route
  is `RequiredFloorClaim.execution_mode`, which was already per-claim — no new vocabulary
  restates it.
- **The interpreter's refusal is the classifier, and it is typed.**
  `InterpError::HermeticHostEffectRefused { operation, ground }` replaces three
  `TypeError`-with-prose sites, and reaches the floor as `ClaimOutcome::HostEffectRefused` — the
  precedent `TimedOut` and `HostToolUnresolved` already set against substring-matching prose. A
  route gap is **not a verdict**: the witness never reached its subject, so it is neither a pass
  nor a failure, and it is reported apart from both because its remedy is a route.
- **`HermeticEffectGround` is closed and names the remedy**: `UnpublishedMockCase`,
  `NoMockResponse`, `FilesystemRemoval`. One `String` would have collapsed three different fixes
  into one sentence.
- **The offered population is now checked, not just reported.** `SitePartitionInexact` refuses
  unless `offered == routed + declined_long`, stated at the loop that could violate it.
- **The residue is identity-grain debt, not a category.** `v2.workflow.floor_route_gap` reuses
  the `floor_expected_red` contract rather than re-coining it: enrolled-and-gapped is agreement,
  unenrolled-and-gapped reds, enrolled-and-did-not-gap reds as stale, enrolled-and-did-not-execute
  reds. Monotone, identity grain, over an independently discovered closed universe — the four
  conditions DESIGN §5's 2026-08-01 oracle ruling requires of a debt contract. There is no count
  assertion anywhere in it.
- **The receipts are emitted on CI**, unconditionally: the disposition TSV and the long-home
  storage agreement TSV are now named in `gunbc.witness_floor_workflow`.

Nothing gets a wet route. No live shell, no write, no network — that is a separate lane with its
own admission question, and this change deliberately does not open it.

---

## What landed here, and what is staged

This change carries the **mechanism** and the **101 reclassification**. It does **not** delete
the decline arm.

The seam is exact rather than chosen: the full-closure run surfaced **55 blockers — 6 witnesses
that do not RESOLVE** (`undefined variable`, `no such function`; never caught because nothing
ever evaluated them) **and 49 in the cost tail — and all 55 are newly-admitted, none previously
routed.** So the part that carries no blocker is a complete change that happens to be smaller,
not a partial one. The 101 route-gap on main today and need nothing deleted to be reclassified.

Staged behind their owners: the 6 broken artifacts (route to author or delete — enrolling a
non-resolving witness as expected-red would assert it runs and fails, re-minting the very
conflation this change removes), and the cost-tail policy call, which the histogram above says
is two questions rather than one.

## What is NOT claimed

The `long.` decline is untouched: it is an operator-ruled cost quarantine on a different axis,
and 538 identities remain in it with no executing consumer. That is the *other* half of the
hidden population, and this change does not close it.

## The adjacent residue, named rather than left to be rediscovered

`run_required_floor` consults `floor_prepared_subject_exclusions()` **and nothing else** — the
`witness_exclusion_frontier` rosters have no consumer on this path, which that function's own
comment records after a session added rows to them and measured `modules_excluded=2` unchanged.

Three of its five entries exist for **exactly the route-gap reason** — an operation carrying no
`mock_response`, so "the hermetic floor refuses it and one refusing member fails the run." That
is the same defect this change removes, one layer over: a **file-grain, uncounted exclusion**
where an identity-grain typed disposition belongs. And it is strictly worse than a route gap,
because an excluded module leaves the prepared subject entirely — it is not even typechecked —
whereas a route-gapped identity stays in the subject, executes, and reports.

Converting those entries into `floor_route_gap` rows is the obvious next step and is deliberately
**not** in this change: it should follow the measurement that proves the route-gap mechanism
behaves as designed on the population it was built for, not precede it.
