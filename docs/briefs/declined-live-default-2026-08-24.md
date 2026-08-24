# The floor's live-tree default renders ignorance as a claim, and the cost is silent non-execution

Found 2026-08-24 while reviewing two unrelated child PRs that had each, independently and
carefully, authored discriminating evidence into a population CI never executes.

## The mechanism

`v2.workflow.floor_discovery_producer` resolves a module's live-tree disposition and ends:

```
match folded.refusal {
  Present { value: reason } => EntryLiveTreeDispositionRefused { reason: reason }
  Absent => match folded.disposition {
    Present { value: disposition } => EntryLiveTreeResolved { disposition: disposition }
    Absent => EntryLiveTreeResolved { disposition: ReadsLiveTree }
  }
}
```

`v2.workflow.required_floor` then declines every `ReadsLiveTree` module into `DeclinedLiveTree`
before the fold sees it. On main's last measured run that arm holds **830** identities.

So a module that declares **nothing** is treated as one that declared `ReadsLiveTree`, and its
witnesses are discovered, counted, and never run.

## Why this is a defect and not a default

*No declaration* is **ignorance**. `ReadsLiveTree` is a **positive claim about behaviour**. The
fold renders the first as the second and attaches a decline to it — so a module whose author
never considered the question is routed identically to one whose author considered it and
answered yes, and **no downstream reader can tell the two apart**. `declined_live` is a single
number over both populations.

That is ⊥-as-ignorance conflated with ⊥-as-answer, at the policy layer rather than in an
observation: the empty-observation narrow the repository already names, applied to a routing
decision. And the direction is the harmful one. Forgetting to declare costs a **silence**, never
a red — the author sees a green PR, the reviewer sees a witness file, the floor summary shows a
count that looks like deliberate quarantine, and nothing anywhere says *this evidence does not
run*.

## Measured, on two PRs in one hour, both by careful authors

- **gunbc#9058** — a sealed brace/equals probe pair, genuinely discriminating (pre-fix exactly
  one of the enrolled rows flips, measured locally by its author). Its module declares **no**
  disposition, so it defaults to declined. The author did not omit it carelessly; the question
  never surfaced, because nothing asks it.
- **gunbc#9075** — a cross-module forge probe keyed on the diagnostic class, with a positive
  control and a `CensusNotRunnable => -1` arm so neither side can pass vacuously. Added to
  `sole_constructor_completeness_audit_probe.dag`, which declares `ReadsLiveTree` honestly. It
  becomes identities 27 and 28 of a file DESIGN §4b already records as never executing.

Two independent lanes, same day, both producing exactly the evidence review asked for, both
landing it where nothing runs it. That is not two authors being careless; it is a default doing
what defaults do.

## What the repair is NOT

**Relabelling.** Several modules carry standing notes forbidding it in terms — `Do NOT dissolve
it by relabelling this file SubstrateInputsOnly` — because editing a declaration to buy admission
is §5's tell of a check satisfied by editing its declaration rather than its subject. The
disposition on those files is a true statement about what they do. The trap here is that the
cheapest fix for each individual file is the forbidden one, which is why the default has to
change rather than the files.

## The repair

The refusal arm **already exists**: `EntryLiveTreeDispositionRefused`. An undeclared disposition
should take it. Then a new witness module refuses at discovery — loudly, located, once — and the
author declares the honest answer in the same minute they wrote the file, instead of discovering
a year later that their wall was never measured. The alternative (default to the executing arm)
is worse: it would silently admit modules that genuinely read the live tree.

This is the §5 construction move at the routing layer: make *undeclared* unrepresentable rather
than quietly meaningful.

## Standing

Reported to both authors with the correction that their evidence is **authored, not executing**,
and that the rung must be cited that way. Not dispatched as a lane: new child work is held while
the merge queue is unserved, and this is a small, decidable change to one arm rather than an
investigation. Recorded here so it is not rediscovered by the third author to hit it.

Related and separately tracked: the deletion of the `DeclinedLiveTree` arm itself is already a
named next-rung trigger in several module notes (`observation_emit_census_witness_test`,
`seed_mirror_constant_lens_witness_test`, `guarantee_floor_class_probe_witness_test`), measured
on a branch as admitting ~783 identities of which 55 are blockers. **This brief is not that
change** — it is the much smaller one that stops *new* evidence entering the population by
accident while that larger deletion is staged.
