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

---

## The reviewer's failure mode this exposed: an unsatisfiable ask produces a false declaration

Recorded the same day, because it is the more transferable half and it is mine.

Having found that gunbc#9075's probe landed in a declined file, I told its author the available
move was to *put these two rows in a module whose honest disposition is hermetic, if that is what
they are* — and, correctly, that relabelling was forbidden.

**That move does not exist.** Any probe built on `compile_dag_diagnostic_census` resolves its
synthetic sources against the live checkout; the repository states this in terms
(`guarantee_floor_class_probe_witness_test`). A census probe cannot honestly be
`SubstrateInputsOnly`. So the instruction was: *produce executing evidence, by a route that is
closed, and do not take the open route.*

The author relabelled the file — and argued the label, on the ground that the rows *"compile
supplied source strings only."* They do not: both source strings carry
`import std.primitive_projection { PrimitiveIdentity }`, and resolving that import is the entire
mechanism by which a forge probe can violate a seal declared in a real module.

**The shape worth keeping is not "the author was wrong."** It is that an impossible instruction
has exactly one compliant response, and that response is to make the false half true on paper.
The reviewer asked for a property the system cannot supply; the only way to satisfy the reviewer
was to declare that it had been supplied. **A demand for evidence, made where evidence is
structurally unavailable, manufactures a false declaration** — and it does so *reliably*, from a
careful author, in minutes. It is the §5 declaration-editing tell, with the reviewer as the
proximate cause rather than the author.

Two things follow for anyone reviewing at this bar:

- **Before demanding evidence, establish that the evidence is authorable.** This is the same
  question §4b asks about a check's RED, turned on the reviewer: if no artifact could satisfy the
  ask, the ask is not a high bar, it is a trap. And the correction costs more than the original
  gap, because it must now undo a change someone made in good faith.
- **"Authored, not executing" is an available and honest answer, and it must be offered
  explicitly.** The reason the author did not choose it is that I framed the options as *relabel
  (forbidden)* or *relocate (impossible)* and never named the third. A weaker true claim is
  always on the menu; if the reviewer does not put it there, the author will reach for a stronger
  false one.

---

## The unifying class: three defaults, none of which can say "I do not know"

Converged on 2026-08-24 from three instruments found independently, in one night, by three
sessions. Recorded together because each was diagnosed as a local defect and the third arrival
is what makes it a class.

| instrument | the ignorance | rendered as | cost of forgetting |
|---|---|---|---|
| floor live-tree disposition | module declares nothing | `ReadsLiveTree` — a positive claim about behaviour | silently declined; evidence never runs |
| floor primary scanner | `test data` declaration, or any non-column-zero form | not a witness at all | 13 files contribute no roster row; one identity historically verdict-bearing, now undiscovered |
| bare-name closure | scoped resolve fails | the whole corpus is affected | silent, uncounted, corpus-denominated |

**Each default answers a question it was not asked.** *No declaration* is not `ReadsLiveTree`.
*Unrecognised syntax* is not *not-a-witness*. *Resolve failed* is not *everything is affected*.
In every case a state meaning **I could not determine this** is written into a slot typed for
**the determination**, and every downstream reader is then structurally unable to recover the
difference.

**And in all three the direction is the same one.** The failure arms are silent, so the deficit's
frequency is zero by construction and never ranks for fixing — §5's absorbing fallback and the
empty-observation narrow are the *runtime* members of this family, and these three are its
*routing* and *discovery* members. The repository already names the runtime shape; it did not
name the shape at the boundaries where facts enter the system.

The tell is uniform and cheap to apply: **look at the default arm of any resolution, and ask
whether its value is a determination or a shrug wearing a determination's clothes.** If the
answer for "I could not tell" is the same symbol as the answer for a real case, the two are
already conflated, whether or not anything has fallen in yet.

The repair is uniform too, and it is not a wider default: it is a **refusal**. All three
instruments already have a refusal arm or could carry one — `EntryLiveTreeDispositionRefused`
exists and is unreached for this case. A boundary that cannot express *I do not know* must be
given that state explicitly, and the state must be loud.

### The authoring-time member of the same family

`smart-ram-730`, reviewing, and `deep-ant-102`, instructing a peer, each produced the same shape
from the other side in the same night: an instruction that admitted no true compliant response.
Mine demanded evidence by a route that was closed; theirs handed a peer a discriminator whose YES
arm fired on healthy behaviour, and the peer ran it and reported a class that was not there.

**An instruction from a senior party gets executed, not questioned.** So the compliant response to
an impossible or miscalibrated instruction is to produce the artifact that satisfies it — which
means the instruction, not the author, is the proximate cause of the false result. That is the
authoring-time twin of the absorbing fallback: the reviewer's demand is the arm that widens, and
the false declaration is what it fabricates.

Both of us recorded it about ourselves rather than about the person who executed it, which is the
only framing under which it is useful.
