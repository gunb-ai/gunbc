# SCM merge: the design ruling, recorded before implementation

This document settles what merge IS in `gunbc.scm` before any merge code exists. It is written
first because the answer to §5 decides the shape of `CorpusManifestObject`, and building that
authority before this ruling would be the §3 replacement-migration trap: a load-bearing authority in
place, every nearby question answered in its vocabulary, and the vocabulary already scheduled for
death.

Nothing here is implemented. Where a question is open, it is marked open rather than resolved by
plausible reasoning.

## 0. The workflow being served, stated exactly

The operator's goal is **not** "resemble git". Git terms are familiar vocabulary; the interface is a
projection over the existing realizations (`gunbc.scm_compatibility.git` / `.mercurial` / `.pijul`).
The workflow is:

- **squash-merge into main**, so an integration is ONE commit;
- **kill dev history after the merge** — the branch line is discarded, deliberately;
- **never rebase**;
- optimize for *simpler and faster merges*.

Every ruling below is derived from that, and several of them differ from what a general two-parent
merge would need. Where a general merge would want a capability this workflow never exercises, the
capability is refused as authored redundancy (§2).

## 1. What already exists, and must be consumed rather than duplicated

**`gunbc.scm.ancestry`** supplies `ancestry_walk(start, history) -> AncestryWalk` with four arms:
`AncestryTraced`, `AncestryBrokenAt`, `AncestryStartNotInHistory`, `AncestryRevisitsACommit`. That
module deleted its own boolean ancestor predicate on the ground that `false` was answering for four
different facts, and it names this document's subject as the intended consumer:

> When a real consumer arrives — merge-base, ahead/behind, a publication decision — it derives its
> answer from `ancestry_walk` and decides for itself what a broken chain means to IT.

**So merge inherits an obligation, not just a function.** It must decide, per arm, what that arm
means to a merge, and it may not collapse them into a Bool or into one refusal. §4 does that.

**`gunbc.scm.supersession`** supplies `HeadSlot { slot: String, generation: ObjectId }`,
`observe_generation`, and `supersede_head_slot` — a compare-and-swap whose evidence is minted from
the slot and cannot be supplied alongside it. Its subject is the **repository document generation**,
not a branch head. It is therefore a *precedent for the shape* of a named, CAS-guarded slot and
**not a reusable authority for commit refs**. Saying otherwise would be a nickname (§3).

**`gunbc.scm.checkout`** supplies `checkout(store, commit) -> CheckoutOutcome`, already modeled with
its refusal population intact.

## 2. RULING: no two-parent ancestry arm. A squash absorption is a receipt, not a parent.

`CommitAncestry = RootCommit | DescendsFrom { parent }` is **already the correct shape** and must not
gain a `MergedFrom { left, right }` arm.

The derivation is mechanical rather than a preference. A squash merge produces one commit whose
parent is the target's tip; the absorbed line is discarded by construction. So no commit this
workflow can produce has two parents, and an arm with no producer is what §4b names *worse than
absent* — it gets cited as a capability the model does not have.

The stronger reason is that a second parent would be **actively wrong**, not merely unused.
`ancestry_walk` follows parents. A `MergedFrom` arm would make the walk traverse into the very line
the workflow exists to discard, so "kill dev history" would be false at exactly the layer that
answers questions about history. The feature would reintroduce what the workflow deletes.

**But discarding the line must not silently discard the FACT.** If the squash commit records nothing
about what it absorbed, then "was this work merged?" is unanswerable, and an unanswerable question
that gets guessed at is the fabricated-plausible-output failure. The resolution is a distinction the
model can hold precisely:

> A **parent** is a structural fact the ancestry walk follows.
> An **absorption** is a historical receipt the walk must NOT follow.

So a merge commit carries a `MergeAbsorption` receipt naming the absorbed tip (and the derived base,
§4) as *data*, while its `CommitAncestry` stays `DescendsFrom { parent: <target tip> }`. History
stays linear and traversal stays honest; the question "what did this absorb" becomes answerable
without reviving the line.

**Open:** whether the absorbed commits' objects are retained in the store or become unreachable. This
is a garbage-collection question with its own ruling, and it is deliberately not decided here. What
IS decided: the receipt names the absorbed tip whether or not that tip's objects survive, because a
receipt that silently becomes dangling is a worse artifact than one that names something collected.

## 3. The minimum ref model, and why it is smaller than branches

To merge, two lines must be designatable at once. Today `RepositoryEnvelope.checked_out` is a single
optional `RepositoryCommitRef`, so checking out the target loses the pointer to the work.

The workflow bounds how much is needed. Because the branch is **discarded at merge**, it does not
need to persist, be pushed, track an upstream, or be renamed. It needs to exist long enough to be
named once and then dropped. That rules out most of what a branch model usually carries: no remote
tracking, no upstream, no reflog, no branch-of-branch, no rename.

**Proposal:** the envelope holds a set of named commit refs, exactly one of which is checked out —
the minimum that lets two lines coexist. Naming is required (not merely a second anonymous slot)
because the merge outcome, the refusal diagnostics and the CLI surface all need to say *which* line,
and an anonymous "other slot" would force position to carry identity, which is the positional-naming
defect §3 rules against.

**Open, and genuinely undecided:** whether merge should take its two refs as *arguments* — leaving
the ref set purely a CLI convenience — or whether merge should read them from the envelope. The
argument form is smaller and testable without a ref model at all; the envelope form is what makes
`merge <name>` expressible. My inclination is arguments for the operation and a ref set for the
surface, so the merge model does not depend on the ref model. Flagged for ruling.

## 4. Base derivation: four walk arms, two sides, and the outcomes they force

Merge derives a base by walking both sides. `ancestry_walk` can answer four ways per side, and this
module owes a decision for each rather than one collapsed refusal.

| what the walk says | what it means to a merge |
|---|---|
| `AncestryStartNotInHistory` | **the caller's fact.** A ref that is not in this repository. Refuse, naming which side. |
| `AncestryBrokenAt` | **damage.** The history is truncated; a base derived across a hole would be a guess. Refuse, naming the commit and its missing parent. |
| `AncestryRevisitsACommit` | **a cycle.** Refuse; the substrate is bounded and forward. |
| `AncestryTraced` (both sides) | a base can be derived from the two chains. |

With both chains traced, the outcomes are:

- **`AlreadyAbsorbed`** — the absorbing tip is already in the target's chain. Nothing to do. This is
  distinct from a conflict-free merge that happens to change nothing, and collapsing them would
  report "merged successfully" for work that was already in.
- **`TargetIsAncestorOfSource`** — the target's tip is in the source's chain. The target has not
  moved since the branch started. No integration is required and the head can simply advance.
  (Whether this is exposed as its own outcome or squashed anyway is **open**: under "always squash"
  there is an argument for treating it identically, and an argument that pretending an integration
  happened is dishonest.)
- **`BaseDerived { base }`** — the lines diverged from a common ancestor. This is the real merge.
- **`NoCommonAncestor`** — two unrelated histories. Refuse; this is not a conflict, it is the absence
  of a base, and reporting it as a conflict would misname it.

A boolean or a single `MergeFailed` here would rebuild exactly the collapse `ancestry.dag` deleted.

## 5. THE QUESTION THAT DECIDES THE MANIFEST: is conflict structural or per-path?

This is why this document precedes `CorpusManifestObject`.

**Per-path (the git shape).** Merge compares `path -> content` maps. Conflict is "the same path
changed on both sides". The manifest IS the merge subject and must carry per-path content identity.

**Structural (native to this substrate).** The store is content-addressed nodes and edges. Merge
compares semantic node graphs. Conflict is "the same semantic node diverged". The manifest is then a
projection used for *checkout and emission* — it says where to write a result — and is not the merge
subject at all.

The shapes are materially different, and the second is where the differentiating claim lives: two
changes to different functions in the same file do not conflict, because they are different nodes.
That is precisely the class §4b describes as beginning *above* the ordinary compiler floor, and it is
also the operator's actual goal — *simpler and faster merges* — rather than a purity argument.

**Recommendation: structural, with the algorithm deliberately unsettled.** What this document rules
is the **subject** of a merge — the semantic node graph, not the path map. What it explicitly does
NOT rule is how a three-way structural merge is computed, which needs its own design and its own
witness floor. Deciding the subject is enough to unblock the manifest, because it settles the
manifest's ROLE: a projection for emission, not the authority a merge reads.

**Risk, stated rather than hidden:** structural merge is harder than per-path merge, and I have not
demonstrated it is tractable here. If the structural algorithm proves out of reach, the fallback is
per-path — and that fallback would change the manifest's shape, which is the one thing this ordering
is meant to prevent. So the honest sequencing is: settle the subject now, and prove the structural
merge is tractable *before* the manifest hardens, not after.

## 5a. Tractability, answered where §5 left it open: the edge label already partitions it

§5 recorded "I have not demonstrated a three-way structural merge is tractable here" as an open
risk. It is now answerable from the model rather than by guess, and the answer is neither yes nor no.

`v2.std.node` gives:

```
Node      { kind, children: List<Edge>, occurrence_id }
Edge      { label: EdgeLabel, target: Node }
EdgeLabel = Named { name: Symbol } | Positional
```

**That coproduct is the partition.** A three-way merge descends from the root comparing digests, and
what it can do at each node depends entirely on which arm its children carry.

**`Named` edges: tractable, and by a known algorithm.** Children align by name, exactly as a tree
merge aligns directory entries by filename. At each aligned child the three digests decide it with no
heuristic: `ours == theirs` take either; `ours == base` take theirs; `theirs == base` take ours; all
three differ, recurse. The recursion terminates because the substrate is bounded and forward. This is
where the differentiating claim of §5 actually lives — two changes to differently-named children do
not conflict, whatever file they were written in.

**`Positional` edges: NOT tractable today, and the reason is structural rather than unfinished.**
Position is the only identity such a child has, so concurrent insertions on both sides cannot be
aligned: there is no fact in the model saying which of *ours[2]* and *theirs[2]* are "the same child
moved". Two escapes exist and both are closed:

- **Occurrence identity is deliberately not available.** `occurrence_id` is outside content identity
  by design — `gunbc.scm.checkout` records that two parses minting different occurrence ids store and
  reconstruct identically, and that asserting on provenance would assert the one property the model
  says must not matter. So it cannot be used to track a moved positional child.
- **A sequence-alignment heuristic is not admissible.** Guessing the correspondence is exactly what
  §4 rules out — in a closed, grounded system a heuristic is never necessary, and reaching for one
  *locates* the anemic modeling rather than solving it. A diff3-style alignment would be a confidence
  threshold selecting an arm, which §5 names as a smuggled heuristic.

**So the ruling is a refusal, not a fallback.** A merge that reaches a node whose positional children
diverged on both sides **refuses**, typed and located, naming the node. It does not align, does not
take a side, and does not degrade to per-path merging for that subtree — a failure arm must refuse,
never widen. `MergeConflict` is therefore not one arm: a *named* conflict is a real content conflict
an author can resolve, while a *positional* one is the model declining to guess, and collapsing them
would report a modeling limit as an authoring problem.

**What this does to §5's risk.** The risk was that structural merge proves intractable and the
fallback is per-path, changing the manifest's shape. That risk is now bounded rather than open: the
named case carries the merge, the positional case refuses, and **neither outcome requires the manifest
to be the merge subject**. §5's ruling therefore stands on a narrower and firmer base than when it was
written, and the manifest may proceed as an emission projection.

**What remains genuinely open, and is now the smaller question:** how much of a real corpus sits under
`Positional` edges. If most of a program's interesting structure is positional, a merge that refuses
on all of it is honest but useless, and the lane would need a *modeled* child identity — a separate
capability with its own ruling, not a heuristic. **That measurement is the next concrete step, and it
is a measurement rather than a design argument:** count `Named` versus `Positional` edges over the
live corpus, at the grain a merge would descend. It is deliberately not estimated here.

## 6. What merge produces, and what it must refuse

A merge is a **candidate then a commit**, not one step: the candidate carries the derived base, the
absorbed tip, and the resolved graph; committing it is a separate act that advances the head. This
mirrors the shape `supersession` already establishes — observe, then compare-and-swap — and keeps a
merge from being authorized by anything other than the observation it rests on.

Merge must refuse, never widen: an underivable base, a damaged history, an unrelated history, an
unresolved conflict. In particular there is **no "take ours" or "take theirs" fallback** available to
the merge itself. That is an escape hatch in §5's sense — a toggle whose only effect is to proceed as
if the refusal had not fired — and a resolution supplied by an author is a different, named input,
not a mode the merge can select on its own.

## 7. The projection obligation

`gunbc.scm_compatibility_shape` already carries what the operator asked for: opaque realization and
target handles, honest capture fidelity, projection plus independent read-back, and — its own words —
*"dispatch is a generic handler value, never `ScmKind` or a vendor switch. A fourth realization
supplies one handler and does not edit this module."*

So merge does not get to invent a second presentation path. Whatever merge exposes must project
through that shape, or it becomes a parallel authority for "how this repository is presented to a
foreign realization" — the §3 fork, in the one place the operator explicitly asked for a projection.

## 8. Author and timestamp: blocked, and merge makes it acute

`ancestry.dag` records why `parent` could land while author and timestamp could not: parent is a
structural fact decidable inside the model; author and timestamp are external observations, and this
repository has no modeled clock and no modeled process identity to observe them WITH.

A merge commit is exactly where that absence bites hardest — an integration with no author and no
time is a weak receipt. This document does **not** resolve it and does not fabricate either field.
It records that the merge lane will want a ruling on acquiring a modeled clock and process identity,
and that the ruling is the operator's, not this lane's.

## 9. The evidence this design owes when it is built

Stated now so the implementation cannot be declared done by typecheck (§5's
specification-without-execution):

- one control per `AncestryWalk` arm, per side, reaching a distinct merge refusal;
- `AlreadyAbsorbed` and `NoCommonAncestor` each discriminated from a real merge;
- a merge whose result is asserted by CONTENT, not by "it succeeded" — the mutation that returns an
  empty or unchanged graph must go red;
- a refusal that stays a refusal: a conflict specimen with no author-supplied resolution must not
  produce a commit;
- the absorption receipt asserted to be present AND the ancestry asserted to remain single-parent,
  because the whole ruling of §2 is that those two facts coexist.

## 10. What this document does not claim

It does not claim a structural merge algorithm is BUILT, and §5a's tractability finding is derived from the node model rather than demonstrated by a running merge. It does not settle the ref model (§3 open), the
fast-forward presentation (§4 open), object retention for absorbed lines (§2 open), or author and
timestamp (§8). It does not authorize `CorpusManifestObject` to be built — it settles what that
object's role must be, which is a precondition for building it and not a substitute.
