# SCM merge: the design ruling, recorded before implementation

This document settles what merge IS in `gunbc.scm` before any merge code exists. It is written
first because a merge and publication consumer, once cut, fixes the vocabulary every nearby question is
then answered in — and a vocabulary cut against an unproven correspondence contract is the §3
replacement-migration trap.

**An earlier revision of this document opened by saying §5 decides the shape of
`CorpusManifestObject`. That was false and is corrected in §5:** that object already exists in
`gunbc.scm.object_store`, its shape is already ruled there, and it is not gated by this document. The
pending boundary is a consumer / commit-root cut, which is materially smaller.

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

### D1. The receipt is only real if a QUERY consumes it

A receipt nobody reads is data, not evidence. This document therefore fixes the consumer before the
producer, because the producer's shape is determined by what the query must answer.

**The query.** Given a source tip S and a target ref T: *does T's history contain a commit whose
absorption receipt names S?* It is scoped two ways and both are load-bearing:

- **exact source.** The receipt names a specific absorbed tip. "Some commit from that line" is a
  different, weaker question, and answering it while claiming the strong one is fabrication.
- **target lineage.** The answer is relative to T. A receipt sitting in a line T cannot reach does
  not mean the work is in T. So the query walks T's ancestry (§4's arms apply, with the same per-arm
  refusals) and inspects receipts along that chain — it does not scan the store.

**Three facts stay distinct, and collapsing any two is the defect this section exists to prevent:**

1. **Ancestry membership** — S is literally an ancestor of T. Under squash this is normally FALSE for
   absorbed work, which is exactly why (2) exists.
2. **A committed absorption receipt** — some commit reachable from T recorded absorbing S. This is a
   historical claim about an act, and it is the only one the receipt itself supports.
3. **Newly derived content comparison** — the content of S is present in T's current tree. This is a
   fresh derivation over current state, not a receipt read.

**(2) does not imply (3).** Work absorbed and later reverted still carries its receipt. So the query
answers *"was this integrated?"* and never *"is this effective now?"*; a consumer needing the latter
must derive it, and the outcome vocabulary must name which question it answered. Historical
integration and current effect are separately reported, never one boolean.

**Survival of collection.** The query must remain answerable after the absorbed objects are collected
(the open question below). That is a constraint on the receipt's contents: it holds the absorbed tip's
identity and the derived base's identity as VALUES, so answering never requires dereferencing the
absorbed line. A query that needs the collected objects would make garbage collection silently
destroy history's answers.

**Refusals.** Not-found is `NoAbsorptionRecorded`, distinct from `TargetHistoryUnwalkable { cause }`
carrying the §4 walk arm, distinct from `SourceNotInRepository`. "No" and "cannot tell" are different
answers.

**Open:** whether the absorbed commits' objects are retained in the store or become unreachable. This
is a garbage-collection question with its own ruling, and it is deliberately not decided here. What
IS decided: the receipt names the absorbed tip whether or not that tip's objects survive, because a
receipt that silently becomes dangling is a worse artifact than one that names something collected.

## 3. The minimum ref model, and why it is smaller than branches

To merge, two lines must be designatable at once. Today `RepositoryEnvelope.checked_out` is a single
optional `RepositoryCommitRef`, so checking out the target loses the pointer to the work.

The workflow bounds how much is needed, but two of the bounds this section first drew were deductions
the workflow does not license, and they are withdrawn rather than left to decide the model quietly:

- **"Discarded at merge" does not mean "does not need to persist."** Retirement on success says nothing
  about lifetime before success. A multi-command or resumable merge has a lifetime to preserve exactly
  while it is unfinished, which is when the ref matters.
- **"Exactly one checked out" erases the empty and unborn repository.** `checked_out` is
  `RepositoryCommitRef?` today for a reason, and a ref model that cannot express a repository with no
  commits has dropped a state the current model holds.

What the workflow does bound: no remote tracking, no upstream, no reflog, no branch-of-branch, no
rename.

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

**The absorption query runs FIRST, and it does not walk the source.** An earlier revision of this
section put base derivation at the front, which left the §D1 counterexample alive at the consumer even
though the query beside it was correct. After M is published with parent T and a receipt naming S, S is
not on M's parent chain — so a re-request for S fell straight through to base derivation, and once S's
objects were collected the mandatory source walk refused before the surviving receipt was ever read.
**A correct query repairs nothing until the decision consumes it.**

The ordering is therefore fixed:

1. **Establish the target lineage and consult its committed exact-source absorption evidence, without
   requiring the source objects.** This step needs the target's chain and the identity value carried in
   each receipt; it dereferences nothing on the source side.
2. **Only if fresh reconciliation is actually needed** does the operation require the live source and
   derive a base — and only then do the walk arms below apply to the source side.

Two refusals get their place from that ordering. `SourceNotInRepository` refuses a NEW operation that
needs an unavailable source; it may never veto a historical receipt lookup merely because the source
was collected. And `NoAbsorptionRecorded` is supported only by a COMPLETE target walk that found no
matching receipt — an unwalkable target yields `TargetHistoryUnwalkable`, which is a different answer.

**Identity continuity is a requirement, not a consequence of the bytes surviving.** Carrying the source
identity as a value fixes dereferencing; it does not by itself establish that the value still denotes
the same source. The receipt's subject is a source identity that is never reused for different content
within the scope the query answers over. Physical collection scheduling may stay open; identity
continuity may not be left to it.

Step (2) is what the rest of this section specifies. `ancestry_walk` can answer four ways per side, and
this module owes a decision for each rather than one collapsed refusal.

| what the walk says | what it means to a merge |
|---|---|
| `AncestryStartNotInHistory` | **the caller's fact.** A ref that is not in this repository. Refuse, naming which side. |
| `AncestryBrokenAt` | **damage.** The history is truncated; a base derived across a hole would be a guess. Refuse, naming the commit and its missing parent. |
| `AncestryRevisitsACommit` | **a cycle.** Refuse; the substrate is bounded and forward. |
| `AncestryTraced` (both sides) | a base can be derived from the two chains. |

With both chains traced, the outcomes are:

- **`SourceOnTargetParentChain`** — S is literally an ancestor of T. This is **ancestry membership,
  and it is not receipt-backed absorption**; it is reported as its own fact. (`AlreadyAbsorbed` was
  the earlier name and it was doing the work of two facts. Receipt-backed historical integration is
  answered in step (1) above and never reaches this partition, so the four-outcome count here must
  not be preserved by folding the receipt answer back in.) Both are also distinct from a
  reconciliation that happens to compute an unchanged result, which is a **content** fact about a
  fresh derivation and is evidence of no earlier integration whatever.
  Neither answer silently suppresses an explicitly requested reapplication: a caller who asks to
  reapply S after a revert is asking for step (2), and the historical receipt is not a refusal of it.
- **`TargetIsAncestorOfSource`** — the target's tip is in the source's chain. The target has not
  moved since the branch started, so no content reconciliation is required. **This is now RULED
  rather than open, because leaving it open left a loophole that defeats §2.** If publication is
  allowed to advance T onto S's tip, then S's development commits become part of T's history — the
  dev history the workflow exists to kill — and, worse, no absorption receipt is written, so §D1's
  query returns `NoAbsorptionRecorded` for work that WAS integrated. Cheap reconciliation was
  silently converted into a different publication.
  The ruling separates the two: **reconciliation may be trivial; publication is uniform.** Merge
  reports that no reconciliation was needed, and publication still produces a new commit M whose
  `CommitAncestry` is `DescendsFrom { parent: A }` — the exact observed target tip — carrying an
  absorption receipt naming S. M's root may be identical in content to S's root; that is a content
  fact and does not license an ancestry shortcut. Fast-forward as a HISTORY operation is therefore
  not available in this workflow, and the outcome name says "no reconciliation needed", not
  "fast-forward".
- **`BaseDerived { base }`** — the lines diverged from a common ancestor. This is the real merge.
- **`NoCommonAncestor`** — two unrelated histories. Refuse; this is not a conflict, it is the absence
  of a base, and reporting it as a conflict would misname it.

A boolean or a single `MergeFailed` here would rebuild exactly the collapse `ancestry.dag` deleted.

**One consequence for `gunbc.scm.ancestry`.** That module currently names its next-rung trigger as the
arrival of a consumer that merges two commits, on the expectation that such a consumer would force a
several-parents arm. §2 rules that expectation false: the consumer arrived and it does NOT want the
arm. That trigger is therefore superseded rather than satisfied, and it must be replaced rather than
deleted — a trigger removed with nothing in its place is an untracked stall (§4b(2)). The replacement
trigger this document proposes is the arrival of a workflow that must PRESERVE both integrated lines
as traversable history; until then `DescendsFrom` is the ceiling, not a stall.

## 5. CORRECTED: authored source stays authoritative. Structural interpretation is a derived view.

**The first cut of this section was wrong, and it was wrong in the way this document exists to
prevent.** It ruled that the merge subject is the semantic node graph and demoted
`CorpusManifestObject` to "a projection for emission, not the authority a merge reads". That
conclusion forked an authority this repository already holds, and it did so in a document whose
stated purpose is avoiding exactly that. The correction is recorded in place rather than quietly
rewritten, because a design note that hides its own reversal teaches the next reader nothing.

**Two factual corrections come first, because the original section was reasoning from a false census.**

1. `CorpusManifestObject` and `CorpusManifestEntry { path: NonEmptyStr, source: AuthoredSourceTarget }`
   **already exist** in `gunbc.scm.object_store`, and `gunbc.scm.object_table_json` already encodes and
   decodes them. This document's earlier framing of the manifest as unbuilt was wrong.
2. What has NOT changed is `RepositoryCommit.root`, which is still `SemanticNodeObjectRef`. **So the
   pending boundary is a consumer / commit-root cut, not the creation of a missing object kind** —
   materially smaller than "build the keystone", and it means `add`/`commit` are nearer than this lane
   previously reported.

**And the model had already ruled on the question §5 tried to settle.** `object_store.dag` states that
a manifest entry is path-to-authored-source *and deliberately nothing else*, because a semantic root is
a DERIVED INGESTION RESULT whose value depends on the source object, the corpus and its imports, the
ingestion rule, and language and rule versions. Carrying it in the entry would fuse an input with a
realization result and would make a manifest **unauthorable for malformed or not-yet-ingested source
— "precisely the corpora a source-control system must still be able to freeze."** That module names
the same comment-only-edit case used against §5 below.

### The counterexample is information loss, not algorithmic difficulty

Two authored snapshots differ only in a comment. §4c erases annotations from the semantic projection,
so **their semantic graphs are identical**. A merge handed only those graphs cannot determine which
authored change occurred. No choice of three-way graph algorithm recovers the distinction, because the
distinction was destroyed before the algorithm was reached.

Authored placement adds a second obligation the graph cannot answer: where does a moved declaration
belong in the output, and what happens when independent additions select the same destination. Source
that cannot be ingested at all adds a third — it still needs an honest source-control disposition
rather than vanishing because the structural view is unavailable.

### The separation this document now rules

> **The authored snapshot is authoritative for what was authored. Structural interpretation and
> correspondence are DERIVED VIEWS used to reconcile changes. A merge produces a new authored
> snapshot, and must not silently discard distinctions its semantic view cannot represent.**

That is not two authorities for one fact. *What was authored* and *an interpretation of what was
authored* are different questions, and §3's replacement-migration doctrine does not reach them: it
applies when X and Y answer the SAME semantic question and X is intended to disappear. Neither holds
here, and applying it anyway is what produced the error.

Two consequences follow, and they are why the original binary was false:

- **How a complete source snapshot is represented and frozen**, and **at what granularity changes are
  compared and reconciled**, are INDEPENDENT choices. A path-to-source manifest can support structural
  reconciliation; adopting a more conservative merge algorithm later does not inherently change the
  manifest's shape.
- **So the manifest is not blocked behind merge.** The ordering constraint is narrower than this
  document first claimed: *do not make the permanent merge and publication consumer depend on an
  unproven correspondence or source-reconstruction contract.* Preserve the complete authored subject,
  establish a bounded structural capability over it, and cut the consumer against the demonstrated
  contract.

If a lossless authored representation derived from a semantic graph is ever proposed, it is a
**materially different subject with a replacement proof to supply** — not something established by the
phrase "semantic node graph".

### One correction to this document's own git comparison

The original section claimed per-path merging means "the same path changed on both sides". That is not
git's general conflict predicate — git performs three-way merging of file *contents* and admits custom
merge drivers. So "two edits to different functions in one file" is not by itself a differentiating
demonstration, and it is withdrawn as one. The differentiator, if it exists, must be stated in terms of
correspondence guarantees (§5a), not in terms of a strawman.

## 5a. The tractability boundary is CORRESPONDENCE and COMPOSITION

An earlier revision of this section argued that `EdgeLabel = Named { name } | Positional` partitions
the merge problem, with named children alignable and positional children refused. That partition is
real and is retained below, **but it is not sufficient, and presenting it as the tractability answer
overstated it.** Alignment by label is one form of correspondence EVIDENCE; it is not correspondence.

**`ObjectId` recognizes unchanged content. It does not identify one logical declaration across an
edit.** Three counterexamples bound the claim:

- **Changed ancestors.** One side edits `f`, the other edits `g`. Both change their function's digest
  and every enclosing digest up to the root. At root grain *both sides diverged*, so digest comparison
  alone reports a conflict for two edits that never touched each other. Descent requires a justified
  correspondence between components first.
- **Content identity is not occurrence identity.** Two logical occurrences may reference the same
  stored subtree. Editing one must not edit the other, and a replacement keyed only on the old
  subtree's content identity would hit both.
- **Renames and moves.** A name is evidence and a position is evidence, but neither is a stable
  identity across precisely the change that alters it.

And structural disjointness does not establish independence: one side may delete a declaration while
the other adds a caller, and two independently added declarations may collide on one name. The combined
graph still needs binding and admission judgments. **Behavioral compatibility is an additional claim,
never a corollary of touching different nodes.**

### What the first capability must promise

Deliberately narrower than arbitrary structural merge. **Once base-relative correspondence is
established**, the ordinary cases are straightforward: equal target and source agree; a side unchanged
from base yields to the changed side; otherwise descend only into components with an established
composition rule, or return an explicit contention or unsupported-correspondence outcome. **Absence
participates explicitly**, so deletion-versus-modification and competing additions cannot disappear
into a default.

Three interface obligations, which this document fixes and the algorithm design later discharges:

1. **What evidence establishes that these are the corresponding logical components?**
2. **What does the automatic arm guarantee about the combination it produces?**
3. **What does it return when either answer is unavailable?**

The answer to (3) is a **named inability to decide** — never a guessed rename, never an automatic side
selection, and never a silent fallback to path-based merging. The `Positional` case above is one
instance of (3), not a separate mechanism: position is the only identity such a child has, occurrence
identity is deliberately outside content identity, and a sequence-alignment heuristic is what §4 rules
out in a closed system.

### The performance claim is withdrawn until it has a denominator

This document earlier implied structural merge serves the "simpler and faster merges" goal. That is
not established. The denominator is the whole operation — interpretation, correspondence and indexing,
the merge, source reconstruction, and admission — and `object_store` records its own list-backed
lookup and traversal as quadratic. **Content addressing does not by itself establish faster merging**,
and no speed claim should be made until measured against that full path.

Also recorded: reducing the number of reported conflicts is not evidence of improvement, because a more
aggressive reconciliation can miss real conflicts. Any future claim here needs a discriminating
oracle, not a lower count.

## 6. What merge produces, and what it must refuse

A merge is a **candidate then a commit**, not one step: the candidate carries the derived base, the
absorbed tip, and — per §5's corrected output authority — the resulting authored snapshot, of which a
resolved graph is a derived view rather than the product. D5 below states the full binding. Committing
it is a separate act that advances the head. This
mirrors the shape `supersession` already establishes — observe, then compare-and-swap — and keeps a
merge from being authorized by anything other than the observation it rests on.

Merge must refuse, never widen: an underivable base, a damaged history, an unrelated history, an
unresolved conflict. In particular there is **no "take ours" or "take theirs" fallback** available to
the merge itself. That is an escape hatch in §5's sense — a toggle whose only effect is to proceed as
if the refusal had not fired — and a resolution supplied by an author is a different, named input,
not a mode the merge can select on its own.

### D5. The candidate is bound to the generation it observed

"Observe, then compare-and-swap" is not enough on its own, because it does not say WHAT the swap
compares. A candidate is derived against a specific target tip; if the target moves between derivation
and publication, publishing the candidate silently discards whatever moved it — a lost update wearing
the shape of a successful merge.

**An earlier revision routed publication through `observe_generation` and `supersede_head_slot`. That
was wrong at the level of the contract, not the field list, and it contradicted this document's own §1.**
`gunbc.scm.supersession` is a **malformed-document replacement policy**: its `standing_permission_refusal`
answers `none` for `LoadDocumentMalformed` **alone**, and returns `StandingDoesNotPermit` for every other
standing including `LoadComplete`. So an ordinary merge into a healthy repository would have reached
`StandingDoesNotPermit { standing: LoadComplete }` on that route — refused precisely in the normal case,
regardless of whether the generation matched. `SupersessionApplied` also yields a modeled slot and a
warrant; it does not install a commit or a receipt.

**The policy must not be weakened to admit merge.** Its narrowness is the point. What `supersession`
supplies here is the SHAPE — observe, bind the observation, conditionally write — which §1 already
records as precedent, and merge publication owes its own admission contract over its own subject. Nearby
primitives exist and none is promoted by name alone: `std.durable_compare_and_set` declares its own
observation-to-slot binding boundary, and `save_repository` does checked encoding followed by an
**unconditional** write. Citing either does not establish this contract; this document fixes the
contract and does not implement the actuator.

**The candidate binds the whole derivation, not a generation value.** A generation alone is not the
relation. The candidate carries: the repository and target slot; the exact observed target commit and
state; the source identity absorbed; the base evidence; the interpretation and rule inputs and any
author-supplied resolution; and — per the corrected §5 — the resulting **authored snapshot**, which is
the output authority, not a resolved graph alone.

**Publication is conditional on that exact observed target**, and a mismatch refuses rather than
publishing; the candidate is then not publishable and must be re-derived. A candidate is not a value
that stays valid; it is a value indexed by the state it was computed from.

**Successful publication binds the installed facts together.** The new commit M, its absorption receipt,
and the target advance become coherently available or none of them does — an uninstalled candidate or
receipt must never answer §D1's historical-integration query. Publication returns named success evidence,
and that evidence is what retirement consumes.

Four outcomes stay separately named, because each demands a different action:

- **`PublicationFailed { cause }`** — publication is ESTABLISHED not to have applied. Only an outcome
  that establishes that may carry this meaning; a realization that installs the state and then loses
  its acknowledgement must return `PublicationUnestablished` instead, and be reconciled rather than
  replayed. Either the realization guarantees the ambiguity cannot arise, or the unestablished outcome
  is preserved. A further attempt stays conditional on the exact observed target — "retry" never means
  blind replay.
- **`TargetGenerationStale { observed, current }`** — the target moved. Re-derive; do NOT retry.
- **`WrongTargetSlot { expected, actual }`** — publication was attempted against a slot other than
  the one observed. This is a caller error and must not be absorbed into staleness.
- **`SourceMovedSinceDerivation { observed_tip, current_tip }`** — the source line advanced after the
  candidate was built. The candidate absorbs less than the caller now means by "the source".

**Retirement checks the source AT RETIREMENT TIME.** Naming the absorbed source generation is necessary
and not sufficient, because this ordering is possible: derive a candidate for S, publish M successfully,
the source ref advances S → S₂, then retirement runs. Deletion is therefore conditional on that same
source slot still naming S. Otherwise the operation discards S₂ after honestly publishing only S.

The outcome when it does not hold is **`IntegrationCompletedRetirementWithheld { published, source_moved_to }`**
— integration succeeded and the source moved — which is neither a deletion of newer work nor a claim
that integration failed. Retirement also consumes the successful-publication evidence above and never a
derived candidate: retiring on a candidate that then fails to publish destroys the line and integrates
nothing. And later collection respects other live references — absorbing one line does not authorize
deleting content something else still holds.

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

- one control per `AncestryWalk` REFUSAL arm, per side — `AncestryStartNotInHistory`,
  `AncestryBrokenAt`, `AncestryRevisitsACommit` — each reaching a distinct, named merge refusal;
  `AncestryTraced` is the success arm, and the controls it owes are the four traced/traced outcomes
  below rather than a refusal;
- the traced/traced partition discriminated four ways: `SourceOnTargetParentChain`,
  `TargetIsAncestorOfSource`, `BaseDerived`, `NoCommonAncestor` — each from a real merge and from each
  other, and `SourceOnTargetParentChain` additionally discriminated from the step-(1) receipt answer,
  since those are two facts and the earlier `AlreadyAbsorbed` name held both;
- a merge whose result is asserted by CONTENT, not by "it succeeded" — the mutation that returns an
  empty or unchanged graph must go red;
- a refusal that stays a refusal: a conflict specimen with no author-supplied resolution must not
  produce a commit;
- the absorption receipt asserted to be present AND the ancestry asserted to remain single-parent,
  because the whole ruling of §2 is that those two facts coexist;
- **D1** — the receipt query answered on a target whose chain CONTAINS the absorbing commit and on one
  whose chain does not, plus `NoAbsorptionRecorded` discriminated from `TargetHistoryUnwalkable` and
  from `SourceNotInRepository`; and the absorbed-then-reverted specimen, which must report integrated
  historically and not-present currently, since collapsing those two is the defect §D1 names;
- **D2** — the no-reconciliation-needed case asserted to produce a commit with `DescendsFrom { parent:
  A }` and an absorption receipt naming S; the mutation that advances the head onto S's tip must go
  red, because that mutation is precisely the loophole;
- **D4** — a comment-only-edit specimen whose semantic roots are equal and whose authored snapshots
  differ, asserted to preserve the authored distinction; a changed-ancestors specimen (`f` and `g`
  edited on opposite sides) asserted NOT to report a root-grain conflict; and a shared-subtree
  specimen where editing one occurrence leaves the other unchanged. Each is a discriminating red for
  a correspondence claim, not a demonstration that a merge algorithm exists;
- **D1 at the DECISION, not only the query** — a repeated exact-source request after M is published,
  asserted to answer from the receipt rather than proceeding to another base-derived merge; the same
  with the source objects COLLECTED, which must still answer (the control that would have gone red on
  the earlier ordering); the same source requested against a DIFFERENT target lineage; a never-absorbed
  source whose fresh reconciliation happens to compute an unchanged result, asserted NOT to report
  historical integration; and a later target commit asserted not to disturb either answer;
- **D5** — a candidate published against a moved target asserted to refuse, discriminated from
  `PublicationFailed`, `WrongTargetSlot` and `SourceMovedSinceDerivation`; **ordinary publication into a
  HEALTHY repository asserted to succeed**, which is the control the superseded `supersede_head_slot`
  route would have failed with `StandingDoesNotPermit { standing: LoadComplete }`; commit, receipt and
  target advance asserted coherently available together, with an uninstalled receipt asserted not to
  answer the D1 query; source movement AFTER publication but BEFORE retirement asserted to yield
  `IntegrationCompletedRetirementWithheld` and to leave S₂ intact; and a failed publication asserted to
  leave the source ref unretired.

## 10. What this document does not claim

It does not claim a structural merge algorithm is BUILT, and §5a's tractability finding is derived from the node model rather than demonstrated by a running merge. It does not settle the ref model (§3 open) or object retention for absorbed lines (§2 open), or author and
timestamp (§8). It does not claim a bounded structural correspondence capability is feasible — §5a records that as an
obligation to demonstrate, not a result. Its performance claim is withdrawn (§5a) rather than weakened.

It does not authorize a merge implementation to be built. What it settles is the vocabulary those
consumers must preserve. **It does not gate `CorpusManifestObject`**: that object exists today in
`gunbc.scm.object_store`, this document's earlier claim to the contrary was false, and the ordering
constraint §5 actually establishes reaches only the permanent merge and publication consumer — not the
authored-source primitive, and not `add`/`commit`.
