# Witness execution closure: 778 identities discovered, declined, and never run

**Found and closed 2026-08-20 (`nimble-wolf-645`).** The required floor discovered 778 witness
identities every run, declined all at file grain, and executed none — and since the 2026-08-15
floor cut left `claim_executor --required-floor` as the tree's only witness-executing consumer,
"declined by the floor" and "not run anywhere" became one fact.

This document is the finding and the receipt; the change is in `v2.workflow.required_floor`,
`v2.workflow.floor_route_gap`, and the seed's site projection.

---

## The measurement

Counted with the floor's former Rust scanner (a column-zero
`test fn `, and a column-zero `data … LiveTreeDisposition … ReadsLiveTree`), on `4cec10f66a3`:

| population | identities | files |
|---|---|---|
| discovered `test fn` sites | 11,115 | — |
| `DeclinedLongModule` (authored `long.` module name) | 538 | 97 |
| `DeclinedLiveTree` | 782 | 112 |
| routed and executed | remainder | — |

The floor's own run receipt agrees and is the citable number: `gunbc#8638`, run `32337765205`
against `32333231183` at the same base, reported `claims=9782 declined_long=538 declined_live=778`
on the base — the brief's 778 exactly. Quoted in
`dag/test/claim/seed_mirror_constant_lens_witness_test.dag` by a session that had just added four
rows to the declined population and measured the delta.

---

## Three defects, and only the third is about cost

### 1. The decline rested on a premise that had stopped being true

`DeclinedLiveTree` excluded any identity whose FILE declared
`data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree`, on the premise that reaching
the live tree implies "cannot run in the hermetic frame this floor folds".

That premise had been false for some time. Hermetic mode carries the **checkout-input carve-out**
(`v1_interpreter`, the readonly `Filesystem.Read|List` arm guarded by
`hermetic_checkout_input_disposition`): a readonly filesystem operation whose path the disposition
*confirms* sits under the checkout root, with no `.git` or `target` component below it,
**dispatches to real input access** — the commit is the run's deterministic input, so this is
input access, not a host effect. Reading committed `.dag` and `.rs` sources is exactly that case,
and is what most of the 778 do.

So the floor ran a **file-grain prediction of an answer the interpreter already decides exactly,
per identity, at the effect boundary** — and predicted it wrong, in the direction that silently
removed coverage.

Independent confirmation: another session reached the opposite conclusion in writing one day
earlier — *"the required floor folds one hermetic prepared subject, so a live-tree reader cannot
participate by construction"* — followed by a §4b rung drop to *mitigatable* whose next-rung
trigger was *"an executing consumer for the declined-live population"*. The trigger needed no new
lane, only the stale prediction deleted.

### 2. One fact, two computations, in one binary

`reads_live_tree` was derived twice, by methods that cannot agree except by coincidence:

- the former `cli_run` floor scanner — a **syntactic text scan** for the declaration line.
- `cli_run` `reads_live_tree_effective` — reads the same declaration, then falls through to
  `effect_reach_derived_reads_live_tree_for_entry`, a **semantic effect-reachability derivation**
  over the entry's import closure.

These disagree as a function of the import graph (DESIGN §3). The resolution is not to pick one:
the two consumers asked **different questions**, and only one was entitled to the name.
Affected-set selection asks *does this entry's result depend on live tree state* —
`reads_live_tree_effective` keeps that question, untouched. The floor asked *can this identity
execute*, which no authored file-level boolean can answer. The floor's copy is deleted; nothing
replaces it, because execution answers it.

### 3. The run's own honesty check could not see what it dropped

`claims_planned` was the **post-decline** number. The terminal invariant
(`ClaimIdentityCountsDisagree`) compares `planned == executed == receipted`, all measured *after*
the projection dropped whatever it dropped: a projection declining a thousand identities and one
declining none produce identical healthy triples.

And the per-identity receipt (`RequiredFloorDispositionRow` TSV) was gated on
`GUNBC_REQUIRED_FLOOR_DISPOSITION`, which `witnesses.yml` did not set — so on CI the whole
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

**Of ~783 identities admitted, 626 pass.** 157 route-gap, naming exactly the operations this
corpus's exclusion notes have named in prose for months — `Mktemp.Dir` 54, `IsExecutable` 26,
`Run` 17, `emit_host_native_cache_evict` 10, `Check` 10, `Write` 8, `git.Inspect.HeadCommit` 5,
then a tail. **Not one is a committed-source read.** The premise was stale for the large majority
of the excluded population, not an interesting minority.

The cost worry did not materialise: 49 of 783 in the cost tail (~6%), 32 minutes wall against a
180-minute timeout.

### The finding nobody was looking for: a third of the expected-red roster was never red

That run carried **308** enrolled expected-red identities and held **207**. The difference is
**101** — and the 101 enrolled identities that route-gapped are *precisely* that set, by
intersection, not by count coincidence.

Those 101 were already executing on main every run and already reaching a hermetic refusal. Since
that refusal arrived as a `TypeError` carrying prose, `ExpectedRedArm` could only read it as an
ordinary failure and **held it as agreement**. The debt ledger recorded "this witness runs and
fails and someone is fixing it" about 101 rows that never reached their subject; nothing was
fixing them because there was nothing to fix: **they needed a route, not a repair.**

This is the state-space conflation §5 names, inside the mechanism whose job is making debt
visible. It also means main's `passed=8702 / known_red_held=306` counted something narrower than
executed coverage, by 101 identities, independently of the 778.

### The cost tail is two mechanisms, not one budget

The CPU histogram over the 49 is bimodal with a **14.4-second empty gap**:

| band | rows |
|---|---|
| 5001–5013 ms | 23 |
| 6682 ms | 1 |
| *(nothing from 6.7s to 21.1s)* | 0 |
| 21109–53301 ms | 25 |

A 12ms spread across 23 rows is **the interrupt firing**, not 23 witnesses each needing five
seconds — and an interrupted claim's cost is a *lower bound*, so those rows' true cost is
**unmeasured**. The low mode cannot argue the budget is too tight.

24 of the high-mode 25 are *also* interrupted, having accrued 21–53s against a 5000ms deadline —
only possible where the stride poll cannot land: `ClaimPreemptionReachability::OpaqueHostCallUnbounded`,
already modeled in `v2.workflow.required_floor`. **Exactly one of the 25 is the known member** of
`opaque_host_call_grandfather_population()`, which declares itself *exhaustive at one member* and
"grows only when a DIFFERENT operation is found to share this shape, which is a new finding, not
a declaration." This is that finding: the other 24 reach host-fed `*_live` builtins in
`enforcement_live`, `cost_coverage`, `grammar_coverage`, `lifecycle_survivor_corpus_census`, and
`realization_vocabulary_containment`. The "bounded population" was bounded by what anyone had
observed on a floor not executing the population where the others live.

## What replaced it

**Delete-first at the root** (DESIGN §3 replacement migration). `DeclinedLiveTree` and the text
scan feeding it are gone; no intermediate representation was built beside them.

- **One execution route per identity.** Every non-long identity is routed and executed. The route
  is `RequiredFloorClaim.execution_mode`, already per-claim — no new vocabulary restates it.
- **The interpreter's refusal is the classifier, and it is typed.**
  `InterpError::HermeticHostEffectRefused { operation, ground }` replaces three
  `TypeError`-with-prose sites and reaches the floor as `ClaimOutcome::HostEffectRefused` — the
  precedent `TimedOut` and `HostToolUnresolved` set against substring-matching prose. A route gap
  is **not a verdict**: the witness never reached its subject, so it is neither pass nor failure,
  and is reported apart from both because its remedy is a route.
- **`HermeticEffectGround` is closed and names the remedy**: `UnpublishedMockCase`,
  `NoMockResponse`, `FilesystemRemoval`. One `String` would have collapsed three fixes into one
  sentence.
- **The offered population is checked, not just reported.** `SitePartitionInexact` refuses
  unless `offered == routed + declined_long`, stated at the loop that could violate it.
- **The residue is identity-grain debt, not a category.** `v2.workflow.floor_route_gap` reuses
  the `floor_expected_red` contract rather than re-coining it: enrolled-and-gapped is agreement,
  unenrolled-and-gapped reds, enrolled-and-did-not-gap reds as stale, enrolled-and-did-not-execute
  reds. Monotone, identity grain, over an independently discovered closed universe — the four
  conditions DESIGN §5's 2026-08-01 oracle ruling requires of a debt contract. No count assertion
  anywhere.
- **The receipts are emitted on CI**, unconditionally: the disposition TSV and the long-home
  storage agreement TSV are named in `gunbc.witness_floor_workflow`.

Nothing gets a wet route — no live shell, write, or network. That is a separate lane with its own
admission question, deliberately not opened here.

---

## What landed here, and what is staged

This change carries the **mechanism** and the **101 reclassification**; it does **not** delete
the decline arm.

The seam is exact, not chosen: the full-closure run surfaced **55 blockers — 6 witnesses that do
not RESOLVE** (`undefined variable`, `no such function`; never caught because nothing evaluated
them) **and 49 in the cost tail — all 55 newly-admitted, none previously routed.** So the
blocker-free part is a complete change that happens to be smaller, not a partial one. The 101
route-gap on main today and need nothing deleted to be reclassified.

Staged behind their owners: the 6 broken artifacts (route to author or delete — enrolling a
non-resolving witness as expected-red would assert it runs and fails, re-minting the conflation
this change removes), and the cost-tail policy call, which the histogram says is two questions.

## What is NOT claimed

The `long.` decline is untouched: an operator-ruled cost quarantine on a different axis, with
538 identities and no executing consumer — the *other* half of the hidden population, not closed
here.

## The adjacent residue, named rather than left to be rediscovered

`run_required_floor` consults `floor_prepared_subject_exclusions()` **and nothing else** — the
`witness_exclusion_frontier` rosters have no consumer on this path, which that function's own
comment records after a session added rows to them and measured `modules_excluded=2` unchanged.

Three of its five entries exist for **exactly the route-gap reason** — an operation with no
`mock_response`, so "the hermetic floor refuses it and one refusing member fails the run." The
same defect this change removes, one layer over: a **file-grain, uncounted exclusion** where an
identity-grain typed disposition belongs. Strictly worse than a route gap: an excluded module
leaves the prepared subject entirely (not even typechecked), whereas a route-gapped identity
stays, executes, and reports.

Converting those entries into `floor_route_gap` rows is the obvious next step and is deliberately
**not** here: it should follow, not precede, the measurement proving the route-gap mechanism
behaves as designed on its own population.

---

## The same failure one level up: the projection's denominator (2026-08-29, `nimble-ibex-902`)

The population above was lost *inside* the projection — discovered, declined, never run. The
sequel is the population lost *before* it: identities that never reached the projection, so no
partition over "offered" could speak for them.

### The two mechanisms that removed them

`assemble_prepared_subject_closure` drops modules two ways, and a witness declared in a dropped
module was neither planned nor declined:

- **the exclusion substrings**, whose entry-grain receipt (`collect_deferred_discovery_rows`)
  existed but was never joined to the disposition population at identity grain; and
- **the gate closure** introduced by the 2026-08-29 gate cut (§4b rung drop *Required gate
  reduced to the compiler floor*). Only modules in the transitive closure of the gate seeds are
  prepared, so `declined_outside_required_gate` — which reads two figures — can only ever speak
  for modules that *reached* the index.

### The subject universe, measured

Counted with the floor's own rule (a column-zero `test fn ` / `test data `) over `dag/` and
`src/v2`: **13,975 distinct declared identities** in 1,759 files, **zero** duplicate identities,
**zero** module-name collisions. So the gap between the declared corpus and the floor's `offered`
was never duplication but two silent removals. The live figures are the run's own:
`required-floor: declared=… offered=… declined_outside_gate_closure=…`.

### What changed

**Hand-Rust receipt.** The receipt is a roster row, not this paragraph:
`gunbc.floor_population_projection_seed_growth`, enrolled in
`gunbc.seed_growth_admission seed_growth_justification_roster` — item delta, admitted
modifications, owning lane (`v1-hand-queue-drain`), dissolution trigger, and boundary chain live
there, where the census reads them. **Full-index retention (review 57430):** the discovery fold
consumes the prepared full-index views by value inside its own phase (the intermediate
`FloorDiscoverySource` vector is deleted), and the phase's completion line prints
`full_inventory_release_rss_kb_before` / `_trim_reclaimed_kb` / `_rss_kb_after` through the
floor's existing statm/malloc_trim instruments, so the run itself states that outside-closure
bytes end with the phase rather than surviving into claim execution.

1. **The universe is the declared population, answered by one authority.** The floor folds
   `v2.workflow.floor_discovery_producer` (`discover_floor_rows_for_source`) over preparation's
   FULL module index rather than over the prepared closure alone, and classifies each returned
   identity against the prepared closure and the exclusion map: an identity whose module
   preparation dropped carries `DeclinedOutsideGateClosure` or
   `DeclinedDiscoveryExcluded { matched_substring }` — two arms on the existing authority, not
   a second status vocabulary. No Rust rescan stands beside the producer; gunbc#9685's
   single-discovery-authority cut holds for the whole corpus, not only the prepared subject. The
   gate-closure population keeps its own arm rather than folding into
   `DeclinedOutsideRequiredGate`: the two are removed by different mechanisms, restored by
   different triggers, differ by two orders of magnitude, and the first is the rung drop's subject.

   **The consequence, stated now:** the producer's per-file refusals — misplaced `test` decl,
   barren sidecar, misplaced wire contract, malformed `live_tree_disposition` row — now stop the
   required floor for ANY module under the source roots, not only one the gate closure admitted.
   A real widening of what this lane refuses over, survivable today because the corpus carries
   none of those violations (measured 2026-08-29: zero `test`-marked decls outside a `*_test.dag`
   sidecar, zero barren sidecars); the first violation authored anywhere in the tree will red
   this lane rather than the one owning the file.
2. **The partition is an identity join, not a count equality.** The old check was
   `offered == routed + declined_long + declined_fixture + declined_outside_gate +
   declined_cost_debt`, which DESIGN §5 names by shape: green over a projection that drops one
   identity and writes another twice, since every count still agrees. It is now the terminal
   ledger's own reconciliation, through the same function (`reconcile_identity_population`),
   refusing as `FloorDispositionJoinInexact` with the offending identities named. Its calibration
   pair is enrolled beside the terminal-ledger one.
3. **The counters are derived from the rows** instead of accumulated beside them, and duplicate
   detection moved from the *planned* subset to the *whole declared* population — a duplicate
   whose first site declined used to pass unnoticed.
4. **The artifact carries two axes, never folded.** The disposition TSV gains an `outcome`
   column, joined from the terminal ledger through `claim_disposition`; an identity that never
   ran reads `not_executed` — a statement, not a blank.

### Two axes deliberately NOT built, and what each needs first

Both were in the commissioning brief and both would have meant *authoring* the authority they
claim to join to (§3), so they are named as triggers rather than improvised:

- **semantic producer / retire-with-producer.** No authority in the tree maps a witness identity
  to the producer whose behavior it witnesses. Trigger: a modeled producer binding on the witness
  carrier itself, at which point the join is a lookup rather than a classification.
- **the Rust `#[ignore]` roster.** A different universe with no roster authority and no identity
  grain shared with `.dag` witnesses. Trigger: an enumerated, typed roster of ignored Rust tests
  with a stated reason per row — the same shape `floor_cost_debt` already has for its population.

### One inert wall found on the way

`gunbc.discovery_census` claims, twice and in prose, that its wildcard-free matches make a new
`RequiredFloorDisposition` arm *fail to compile*. `DeclinedOutsideRequiredGate` had been added
without either match acquiring an arm, and nothing refused — the module's own witness is outside
the gate closure, so no executing path typechecks it. The arms are added here and the claim
restated at its honest rung, with the trigger recorded in the module.

### A false specification selected out of the nonexecuting population (2026-09-02, gunbc#10092)

The changed-witness sublane selected
`test.claim.discovery_census_witness.w_unrostered_sibling_in_the_same_module_is_planned`, an
identity the ordinary required floor did not execute. The claim called
`required_floor_site_disposition` with module path
`dag.test.claim.lifecycle_survivor_corpus_census` and asserted `Planned`, although that path
matches none of `required_gate_prefixes` and the function therefore answers
`DeclinedOutsideRequiredGate`. The specification was false and stayed green because it did not
run — this document's class at one identity, not a new failure class.

Run `33656005986` is the discriminating receipt. Its identity-grain changed-witness ledger reports
the claim as `planned-without-terminal-verdict`, `outcome=failed`; the aggregate reports
`changed_witnesses=12 changed_witness_blocking=1
changed_witness_declined_in_declared_nonexecuting_root=0`, so the row executed rather than being
declined again. The same run measured the containing universe as 15,383 declared identities,
3,499 routed and 11,277 declined outside the gate closure, and ended with exactly one semantic
failure and zero interrupted verdicts. The repair renames the claim to
`w_unrostered_sibling_in_the_same_module_reaches_the_gate_decline` and asserts the disposition its
own input entails; direct claim execution then passes. The reusable fact is not the spelling fix:
selection of a normally nonexecuting identity is an executable falsifier for specifications that
ordinary floor green cannot adjudicate.
