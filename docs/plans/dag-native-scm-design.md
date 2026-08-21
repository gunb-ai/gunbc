# A `.dag`-native SCM — model, vocabulary, and first executable slice

DRAFT, design-note-first. **No code lands from this note.** Claims are marked **verified** (read
out of the live tree), **derived** (a consequence of a verified claim plus an authority doc), or
**proposed** (a design position awaiting a discriminating witness or an operator ruling).

This note supersedes an earlier draft on the same branch that modelled merge as a three-way diff
over parsed trees. §9 records what was rejected and why, because that reasoning is the most useful
part of the history; the scenario corpus from that draft survives unchanged in §8.

---

## 1. Vocabulary — git's words, git's constraints deliberately not inherited

Operator ruling: **anchor on git's terms.** Not from familiarity — from anti-forking. Coining a new
word for a concept that already has an industry-standard one is the nicknaming DESIGN §3 forbids,
and git's vocabulary is the shared one. New vocabulary must earn its place by naming something git
genuinely fuses or lacks.

The rule that separates the two goals: **keep the word for what it *means*; drop what git *does* to
implement it.** "Commit" means *record a state I can point at later* — that survives entirely.
Git's mechanism for it — a whole-tree snapshot fused with one parent, an author and a timestamp —
is one realization, not the meaning. This is the interface/realization split of DESIGN §3 applied
to naming.

| term | meaning here |
|---|---|
| **object** | immutable language content, content-addressed |
| **proposal** | requested obligations — what an author asks the program to contain |
| **branch** | a line of proposals and commits; a grouping label, contributing nothing to identity |
| **merge** | combine proposals |
| **commit** | the exact frozen program |
| **acceptance** | agreement to use a commit — the one new term |
| **deploy** | make an accepted commit active |

`acceptance` earns its place because git's `commit` fuses three questions that are separate here:
*is this program valid?*, *have we agreed to use it?*, and *is it what is running?* Merging those
back into one word would re-create the fusion this model exists to remove.

**Terms deliberately refused**, and the pattern is worth noticing — every one names a *mechanism*
rather than something a person wants, and each exists to repair a model that loses information:

| refused | why |
|---|---|
| `rebase` | "rewrite my work as though it started elsewhere" is a procedure, not a goal; the goal — make my work apply to the current state — happens without it |
| merge base / common ancestor | requires a total order that need not exist |
| `HEAD~3`, `HEAD^` | parent arithmetic assumes a single line |
| force push | acceptances only append; there is nothing to force |
| index / staging area | an artifact of the working-copy model |
| `reset --hard` | mutating a pointer |
| `cherry-pick` | patch transport, which this model does not do |
| conflict markers | a textual representation of an unresolved choice |
| `squash` | merge already produces one exact commit; squash is a git-export description |

## 2. Comments, formatting, and everything that is not the program

Stated first because it decides what a commit *is*, and because getting it wrong is how a system
silently loses what people wrote.

**Verified.** `std.source_annotation` already models this: `AnnotationPlacement`,
`UnboundAnnotationCapture`, `SourceAnnotationDebt`, `SourceAnnotationGraph`,
`AnnotationAttachmentRefusal`. Comments are captured *data attached to structural subjects*, not
discarded trivia and not semantic program content.

DESIGN §4c rules the two facts this design needs. Adding, deleting or moving an annotation **cannot
alter any semantic occurrence identity, semantic graph, resolution result, semantic hash, or
target-program bytes**; and semantic passes receive the annotation-erased projection while *"source
authoring, formatting, **SCM**, and annotation lenses consume and preserve the authored wrapper."*

**Derived — two identities, deliberately:**

- **semantic identity** — over the annotation-erased graph. Annotation-invariant *by rule*.
- **authored identity** — the semantic graph together with its annotation graph. What was written.

Four consequences, all of them features:

1. **A comment-only change is provably a semantic no-op** — new authored identity, identical
   semantic identity. The system states this as a fact rather than guessing from bytes.
2. **Comments never cause semantic conflicts.** Annotations are keyed by their subject, so two
   people annotating different declarations never interact.
3. **An annotation collision is its own lower-stakes conflict**, reportable separately from a
   semantic one.
4. **Reformatting is free**, and the authored form still round-trips because captures carry the raw
   `lexeme` — the delimiter included — with normalization performed on read, never on capture. That
   field distinction is load-bearing: its own carrier records that a realization stripping the
   delimiter where another does not makes parse→render→parse either double it or silently change
   content.

**Comments are eternal carried debt** (operator framing, and the carrier agrees — the type is
literally `SourceAnnotationDebt`). DESIGN calls plain annotations modeling debt: prose that has not
yet migrated into typed carriers, carried forward in the hope that someone models it properly. The
SCM's obligation is therefore to **carry them faithfully without enshrining them** as permanently
first-class, so the debt stays payable rather than becoming load-bearing.

## 3. The model

**The store is append-only and immutable.** Editing never modifies anything; it creates another
object. Objects are content-addressed, so identical content is automatically one object. Nothing in
the store ever conflicts.

**A program is an act of exclusion, and that is why the store alone is not enough.** The store only
grows — adding an object never invalidates anything — but choosing one `foo` means not the other.
A monotonic store cannot by itself produce that non-monotonic fact, so something must *close* a
selection and be nameable. This is A3 (agreement holds only on what stays stable across time)
reaching the design directly.

**A commit is that closed thing: one root object identity whose every reachable edge names exact
content.** It has no parent list, no author, no timestamp, no message, no branch. Those are facts
about proposals and acceptances, not about the program.

**Selection must not stay live after a commit.** If a program re-queried the store at use time,
later additions would retroactively change what it means. The output of merge is an immutable exact
graph, which is why the unit of agreement is a commit root rather than a choice function.

Five facts, kept apart:

```
store        possibility          — every object that exists
commit       a program            — one exact closed graph
certificate  validity             — does this program hold together
acceptance   agreement            — we have agreed to use it
deploy       now                  — it is what is running
```

**Identity does not need to survive across revisions, and cannot.** *Verified:*
`std.occurrence_identity` `occurrence_identity_scope_law` makes `OccurrenceId` unique only inside
one graph-scoped allocator and forbids filename, span, authored name, structural equality and
content hash as identity inputs. *Derived:* cross-revision identity could therefore only be minted
at authoring time and carried in the source — and DESIGN §3's fleet ruling says inferring it from
content similarity is the heuristic §4 forbids. **It is also unnecessary:** identity's only job in a
merge is transporting a patch to the right place, and combining proposals transports nothing.

**A name identifies a binding role in a context, not a thing.** Renaming changes a binding; the
value is untouched. This is why content equality does not imply one program occurrence — one object
may sit at several positions, sharing storage while remaining independently addressable by path.

**"Latest" is a query, not an identity.** This repository already ruled it: `extdeps.pin`
`pin_selection_note` — *execution always resolves an exact identity, because resolving "latest" at
execution is nondeterminism at the substrate boundary; re-resolution is an explicit action producing
a reviewable diff.* Pinning a moving reference is not a contradiction: you resolve the moving
reference to an exact identity and record that it came from a channel. `Pin<Subject>` is already the
shape for that, already subject-generic by operator ruling, and `Pin<NodeName>` is an instantiation
rather than a new concept.

## 4. What merge is

Two proposals combine by taking both. A conflict exists only where one binding is required to hold
two different objects — a genuine disagreement about what the code should be, never a collision of
positions.

Where the substrate says an edge is `Named`, recursion is keyed by that name; where it says
`Positional`, **order carries meaning** — list elements, match-arm order — and the region is
compared whole. Refusing there is the safety half of the design: merging positional edits by
identity would fabricate a sequence nobody wrote, which is precisely what line-based merge does and
calls success. *Verified:* `v2.std.node` already carries `EdgeLabel = Named { name } | Positional`
per-edge at every depth, and named edges reach deep — `match_arm_pattern`, `binding`, `field`, and a
function attaching to its parent as `Named { name: fn_name }`.

**One grain — the node.** Depth is not a parameter; it falls out of where the substrate says
identity is by name versus by order.

## 5. What this model does not claim

**Merges are not "always safe."** Deep semantic understanding makes conflicts rarer and better
explained; it does not remove them. Two changes can each be valid alone and interact — a new call to
`foo` plus a narrowing of `foo`'s contract. Structural combination and **validity** are separate,
and the second genuinely refuses: a structurally clean rename plus a stale reference produces a
broken program. The honest claim is *far fewer conflicts, each a real question in the user's own
vocabulary.*

**The hard part is not removed, only the accidental part.** Two proposals may admit no common
program, or several materially distinct ones. That residue is the same compatibility problem merge
always addressed. What is gone is everything around it: mutable files, patch transport, branch
topology, global history, working-tree alignment, rebasing, and cloning.

**Identity is currently weaker than the model needs.** *Verified:* `v2.std.node` `Hash` is
`Fnv1a64Structural` — 64-bit, non-cryptographic — and no SHA-256 *computation* exists in `.dag`
(`std.content_hash` `sha256_hex_digest` and `extdeps.crypto.hash` `sha256_digest` validate hex; they
do not hash bytes). A 64-bit non-cryptographic digest is a **locator, not a durable intersubjective
identity**, and adding a host builtin is closed because DESIGN freezes the v1 seed's growth
surfaces. **Declared rung:** the first slice uses the available digest and states this limitation
rather than implying cross-party agreement it cannot support. **Dissolve-on:** a computing
cryptographic digest reachable from `.dag`.

## 6. Confidentiality

Secrets are the forcing function for redaction, and redaction is where a content-addressed history
either has an answer or admits it does not.

**Git's answer is a non-answer**, and that is the design input. A secret in git history requires
rewriting every subsequent commit; every hash changes; every clone, fork and open PR diverges. On a
public repository this is not merely expensive but *ineffective* — the objects are already
distributed. The answer practitioners actually use is "rotate the credential and assume the bytes
are permanently public." We must not build a better version of that theater.

**Five independent facts git fuses into one.** Prevention, current-state retraction,
audience-history retraction, credential invalidation, and bounded erasure are separate, and the
distinction between the two retractions is load-bearing rather than pedantic:

- **Prevention** — bytes never reach the public store. *The only operation that preserves
  confidentiality.*
- **Current-state retraction** — the head no longer contains the material. **This is not enough**:
  a clone that replays from the original root still recovers it.
- **Audience-history retraction** — new readers begin from a sanitized anchor and never receive the
  material at all. This is what people mean by redaction, and it is a different fact from the above.
- **Invalidation** — rotate the credential so leaked bytes are worthless. **Already modeled**:
  `gunbc.auth.secret_rotation`, with exactly-once walls, receipt-backed retirement, and no stored
  payload digest.
- **Erasure** — bytes unrecoverable. Claimable only inside a closed sanitization scope covering every
  key, wrap, backup, cache, derivative, index and recovery path. Never "delete one key."

**Derived rule:** on a committed secret, SCM performs retraction, the rotation kernel performs
invalidation, and erasure is claimable only in the private realm. **The SCM must never report
retraction as erasure.** Once disclosure escapes a controlled scope, `UncontrolledCopiesMayRemain` is
a **terminal standing** and **no global-erasure constructor exists** — the impossible claim is made
unwritable rather than merely discouraged.

Two consequences that are easy to get wrong in opposite directions. Invalidation is **not**
established by the secret manager disabling a version: the underlying service may still accept the
leaked token, so invalidation needs a subject-specific negative authentication probe. And a rewrite
that lacks a complete sanitization receipt is `PrivateHistoryReanchored`, **not** erased.

**Where the wall sits.** Merge-time or CI-time checking is structurally incapable of confidentiality
— the deleted Stage-0 placement gate established exactly this: a required check could refuse `main`,
but pushed objects had already reached public storage. The chronology must be private capture →
private merge and admission → derive an audience-authorized projection → serialize → write public
storage. The public writer must not accept an arbitrary `Node`, blob or patch, only a projection
minted by the authoritative private context. **Corollary:** a public PR branch cannot safely carry
secret bytes on the theory that squash-landing removes them. And the surface is wider than file
content: a secret in a commit message, path, diagnostic, workflow log, or projection metadata is the
same disclosure.

**Retraction without rewriting.** Where a commit is *accepted parent plus sparse transformation*,
retraction can be an ordinary **appended transformation** — no existing identity changes, no
downstream clone is invalidated. **This holds only if public history is an audience projection rather
than a replayable copy of private transitions**; otherwise replay from an early checkpoint
reconstructs the secret on the way to the head, and what was achieved is current-state retraction
only. The resolution is an **audience-specific projection epoch with a sanitized anchor**: re-anchor
the *public projection*, never the authoritative history.

Three constraints on that epoch, each of which an earlier draft of this note got wrong by being
vague. The sanitized epoch must have **no public predecessor relation** to the contaminated one — a
tombstone naming the old root, the path, or the incident reason is itself the leak, so the mapping is
retained privately or not at all. The old epoch must be **retired from authoritative service**, since
a clean head while the old refs are still advertised is not retraction. And **hiding a ref is not
retraction** if direct object retrieval still serves the bytes. *(Proposed — the least settled part
of this note.)*

**No confidentiality receipt carries secret bytes or a secret-derived digest**, following the
rotation kernel's existing rule. Receipts retain opaque incident identity, subject references,
non-secret-derived transition identities, verification verdicts, audience standings, and bounded
sanitization evidence.

**Diagnostics are an information-flow surface.** If an old-root lookup answers "retracted" where a
nonexistent root answers "unknown", that difference is an existence oracle; where existence is
confidential the unauthorized responses must be indistinguishable. Over-redaction is the paired
failure — an authorized responder must still receive located detail — so both directions need
controls.

**At the user boundary:** "keep the revoked secret" is never offered as a `ChoiceRequired`. It is a
hard `CouldNotLand`.

**`.env` does not belong in the graph.** Secret *values* live in a secret store; the graph holds
`SecretRef` nodes, schema, and required binding identities, and `.env` becomes a local
materialization rather than source-control authority. A marker inside the file cannot self-authorize:
removing `secret=true` must be an authorized policy change, not an ordinary content edit.

**Node-grain withholding survives only at a real interface/realization boundary**, consistent with
the existing refusal of per-statement holes. Public signature with encrypted body: allowed. Public
body with one expression silently missing: refused.

**Erasure is not strictly harder than git's** — git is already a Merkle DAG, and finer grain can help
because a secret subtree can carry its own key instead of requiring a whole file's destruction. But
*accounting* is harder, and two hazards are specific to this design: never publish the private
canonical root hash as the public root, and **never deduplicate secret plaintext across audience
realms — cross-audience content-address equality is an oracle.**

## 7. `NATIVE-COMMIT-0` — the first executable slice

**Honest feasibility ruling first.** An executable slice exists without the parked authoring-capture
surface, but it **cannot honestly merge arbitrary concurrent edits to existing `.dag` files**.
Without capture, nothing can tell from two file endpoints which proposal, deletion, rename or frame
the author intended, and claiming otherwise reconstructs patch inference under new vocabulary. So
the first slice merges **explicitly authored proposals**, and says so.

Three operations. No base, no ancestor, no parent, no branch pointer, no working copy, no patch.
**This is the shipped signature, corrected 2026-08-21 against `dag/gunbc/scm/merge.dag` as merged in
#8719** — the draft below it specified a two-proposal call and a `ChoiceRequired` arm, neither of
which exists, and a stale authority describing an operation nobody can invoke is the premise
contamination DESIGN documents against its own CI paragraph:

```
merge(store, target, target_dependencies, proposals)
  -> Merged { store, commit, roles }
   | MergeRolesContested { role, alternatives }
   | MergeRefused { cause }
accept(store, commit, authority, contract) -> appended; never changes the commit
checkout(store, commit) -> CheckedOut { program } | CheckoutRefused { cause }
```

**Why the signature grew a target and lost a proposal count** (operator direction, 2026-08-20). The
two-proposal form had nothing to preserve *from*, so the only thing it could do with two proposals
was union their bindings — which is neither the literal closed-scope reading nor a dependency model,
and which ships a program referencing a deleted node when one proposal adds `k` depending on `f`
while the other deletes `f`. The target supplies every fact outside the implication frontier, and
**silence means preserve it**. Proposals became a list because a contest is a property of the whole
population rather than of a pair.

**Why the outcome arms are named for less than they conclude.** `MergeRefused` rather than
`CouldNotLand`: the engine cannot establish terminality, since an unsupported dependency kind may be
unmodelled implementation and a missing target root may be an incomplete fetch.
`MergeRolesContested` rather than `ChoiceRequired`: that distinct alternatives exist is what this
layer establishes; that a *user* must choose requires proving they survive the admitted equivalence
quotient, that the candidate population is closed, and that no further machine work resolves them —
none of which lives here. **`MergeRolesContested` is complete over the proposal population supplied
to that call and proves nothing about global candidate-space closure.** That boundary is currently
held by this paragraph and by the type name, not structurally; the landing seam is where it becomes
structural.

A proposal is a set of authored requirements — `RequireBinding { role, value }` or
`RequireBindingAbsent { role }` — meaning *the resulting program must contain this exact value at
this named binding*, or *must not bind this role at all*. Absence is authored rather than implied,
because silence already means preserve: a delete expressed by omission would read as "leave it
alone", the exact inversion of the request. Both sides normalize to a requested state
(`DesiredRoleValue`) before any contest is decided, so identical authorings are **agreement**, not a
question with identical alternatives. Distinct bindings combine by construction; two different
requested states at one binding produce `MergeRolesContested` carrying **every** distinct
alternative — accumulated over the whole population so arrival order cannot decide which evidence
survives, and never latest-wins, first-wins, or branch-priority-wins.

`checkout` follows exact object links from the commit root and consults **nothing else** — not
names, branches, proposals, acceptances, or the live store beyond the identities it was given.

**Storage:** objects keyed by identity, plus append-only acceptances. Canonical encoding excludes
occurrence provenance, sorts named edges, retains positional order, and tags constructors distinctly.
In-memory first; persistence, packing, networking and collection are separate realizations
deliberately kept out of the semantic slice.

**Witnesses**, each with a deliberately-wrong control so that a passing suite means something:

| claim | witness | wrong control that must fail |
|---|---|---|
| a later object cannot change an agreed program | commit, accept, add a second object at the same name, re-checkout | a checkout that queries live names |
| a commit rebuilds from its own closure | discard proposals, branches, acceptances and indexes; checkout from the root alone | — |
| a missing object refuses | remove one reachable object, re-checkout | any fallback substitution |
| proposals combine without node identity | remint every occurrence id, re-merge | a merge requiring matching occurrence ids |
| conflicting bindings ask rather than guess | same name, different objects | any winner rule |
| arrival order is not authority | merge (a,b) and (b,a) | — |
| merge does not imply agreement | merge without accept | — |

**Deliberate non-goals:** no proposal inference from file edits, no capture, no diffing, no
deletion/rename/move, no repository-wide candidate search, no general constraint solving, no
positional-sequence merge, no git import or export, no `GIT-PLUMBING-0` dependency, no refs, no
compare-and-swap, no cross-process persistence, no deploy, no confidentiality, and **no merging of
this repository's live concurrent edits.**

**Known scope limit, stated because it affects what the slice demonstrates.** Proposals are keyed by
top-level binding, so two proposals touching one binding differently ask a question even when they
changed unrelated sub-nodes. **The headline win — two people editing different functions in one
module combining cleanly — is the *next* slice**, which recurses §4's named-edge rule. This slice
proves the foundation: content-addressed objects, stability under growth, exact reconstruction, and
combination without identity.

**Dogfood disposition:** a small controlled subject first. Making this repository's concurrent edits
the first consumer would fuse a model proof with unrelated frontend and performance work.

**Direction after `NATIVE-COMMIT-0`, operator ruling 2026-08-21: a CLI vertical before the depth
recursion**, which reverses the "headline win is the next slice" ordering stated above. The reason is
DESIGN §5's specification-without-execution trap rather than a change of view about which slice is
more valuable. Nothing has ever *consumed* this kernel: the 15 claims in
`test.claim.scm_merge_witness` are assertions **about** it, authored alongside it, and a witness
suite is not a consumer. A CLI is the first artifact that uses the store, checkout, identity and
merge together under conditions nobody authored to make them pass.

The ordering is safe because **depth changes how `merge` combines, not how `add` authors**. A user
adds a module either way; whether merge recurses into it is orthogonal, so the command surface built
now survives the recursion landing later.

The known cost, stated so it is designed for rather than discovered: a CLI makes the
top-level-binding limitation *user-visible*, since the first thing anyone tries is editing two
functions in one module — which today asks a question instead of combining. That refusal must say
exactly that, in those terms, rather than reading as a defect.

## 8. Scenario corpus (retained)

No merge kernel should be built without these. Each is a RED unless marked otherwise.

**Structural merge — with measured coverage as of 2026-08-21 (#8719 merged).** The corpus opens "no
merge kernel should be built without these", and a kernel was built, so the honest thing is to state
which of them it actually answers rather than leave the reader to assume. **4 of 13 are covered.**
The `witness` column names the claim in `test.claim.scm_merge_witness`; the `blocked on` column says
what would close each gap, because "uncovered" collapses three different situations — a gap the
current grain could close today, a gap that needs the sub-node recursion, and a gap that needs
vocabulary the model does not yet have at all.

| scenario | required result | covered | witness / blocked on |
|---|---|---|---|
| target changes an untouched named node | preserved, no conflict | **yes** | `an_independent_sibling_is_preserved_exactly`, `an_empty_proposal_set_preserves_the_target` — preserved at *identity* grain, so it is object reuse rather than a recomputed equal |
| proposal edits one named child, target edits another | both preserved | no | **sub-node recursion.** This is the headline win; the kernel is top-level-binding keyed and cannot express it |
| proposal deletes a subtree, target edits a descendant | conflict | no | **sub-node recursion.** `dependent_add_and_delete_refuses` is the role-grain analogue — delete `f` while `k` requires it — and is *not* this scenario |
| concurrent same-name add, same kind, different children | conflict | **yes** | `two_distinct_requests_for_one_role_are_contested` — different children give a different content hash, hence a distinct `DesiredRoleValue` |
| one side changes node kind, other edits a child | conflict | no | **sub-node recursion** |
| both sides produce an identical kind-changed subtree | clean | **yes**, degenerately | `the_same_request_authored_twice_is_not_a_contest` — identical results agree at top-level grain; the sub-node path is untested |
| unchanged-body rename | unique relocation candidate | no | **no rename vocabulary.** Neither `Requirement` nor the outcome can express relocation |
| rename versus edit under the old name | conflict; the edit is not lost | no | **no rename vocabulary** |
| two sides move one subtree to different parents | conflict, never duplicated | no | **no move vocabulary** |
| duplicate named siblings in any input | input refusal | no | **closable at the current grain.** Measured 2026-08-21: nothing in `gunbc.scm.*` or `v2.std.node` refuses a node carrying two children with one name. The nearest live behaviour is content-hash canonicalization *sorting* named edges, which orders duplicates rather than refusing them. This is the one structural gap that needs no new grain and no new vocabulary |
| two different positional appends | conflict or explicit order choice | no | **positional merge**, an explicit §7 non-goal for this slice |
| distinct positional ordinals edited without shape change | both preserved | no | **positional merge** |
| only one side changed a node | **preserved — the §6 asymmetry regression control** | **yes** | `an_independent_sibling_is_preserved_exactly` |

**What the partition says about the next slice.** Four gaps are one job — the sub-node recursion of
§4's named-edge rule. Three are a second job needing rename/move vocabulary the model does not have.
Two are the positional-merge non-goal. **One — duplicate named siblings — is closable now**, and it
is the only structural scenario that neither waits on depth nor on new vocabulary, which makes it
the cheapest real coverage available.

**Coverage claims about this table must be measured, not recalled.** The number above was produced
by joining the 15 claims in `test.claim.scm_merge_witness` against these rows one at a time. An
earlier estimate from reading was "roughly five", which was close enough to feel safe and wrong
enough to have mis-scoped the next slice.

**Admission and capture**

| scenario | required result |
|---|---|
| structurally clean rename plus stale reference | semantic admission refuses |
| occurrence ids collide numerically across allocator scopes | result reminted or remapped |
| comment or annotation absent from `Node` capture | opaque residue or capture refusal, never silent loss |

**Confidentiality.** Every refusal needs a nearby positive control, so that "always refuse" cannot
satisfy the suite.

| scenario | required result |
|---|---|
| raw payload reaches the public writer before policy evaluation | writer invocation count stays zero; `PublicationPrevented` |
| publication authority unavailable | refuse before write; never classify as clean |
| secret inside opaque/unparsed file residue | capture-fidelity refusal before serialization |
| secret in commit message, path, diagnostic, log, or projection metadata | refuse or redact; no public object carries it |
| bytes already public but lifecycle reports `PublicationPrevented` | impossible state |
| append a retraction, then clone by replaying from the original root | secret recovered — only `CurrentStateRetracted`, never audience retraction |
| sanitized epoch names the contaminated predecessor | commitment/existence leak |
| old ref hidden but direct object retrieval still serves the bytes | retraction incomplete |
| clean head while the old epoch is still advertised | retraction incomplete |
| any receipt stores a payload digest | type uninhabitable |
| one backup, cache, wrap, derivative or recovery path unobserved | `ErasureUnestablished` |
| rewrite succeeds without a sanitization receipt | `PrivateHistoryReanchored`, not erased |
| private realm erased after public exposure | private erasure **plus** `UncontrolledCopiesMayRemain` |
| secret-manager version disabled but the service still accepts the token | not invalidated |
| unauthorized diagnostic names retracted path, secret kind, or incident time | audience-projection refusal |
| authorized responder denied located detail | over-redaction control |
| retracted-vs-nonexistent lookups distinguishable to an unauthorized reader | existence oracle |
| public graph retains a dangling reference to retracted material | public projection refuses |
| proposal from a revoked epoch persisted publicly before inspection | ingress wall |
| proposal independent of revoked material blanket-refused | recovery-expectation failure |
| proposal relying on revoked material lands via ordinary stale-parent retry | admission failure |
| "keep the revoked secret" offered as a choice | user-boundary failure — must be `CouldNotLand` |
| retraction reported to a user as erasure | never |

## 9. What was rejected, and why

Kept because the reasoning is the most reusable part of this note.

**Three-way merge over parsed trees.** The first draft modelled merge as `capture(base, proposal)`
producing a patch, keyed by declaration name. It works, and it is better than line-based merge, but
it is diff-and-patch with a semantic key — it reconstructs intent from endpoints because the
information was destroyed before it was seen. It also inherits a base, and through the base a
timeline.

**Cross-revision node identity.** Pursued for most of a day as the way to make renames carry. Closed
by `occurrence_identity_scope_law` and, more importantly, shown to be unnecessary: identity exists to
transport patches.

**`std.change` `keyed_three_way_fold` as the merge kernel.** *Verified:* its leaf verdict returns
`KeyedConflict` when only one side changed a value — correct for reconciliation, where `desired` is
authority, and wrong for merge, where the sides are peers. The traversal is shared with the
fleet-reconcile spine; the verdict is not.

**The seven inherited assumptions.** Text as storage, commit-as-whole-tree-snapshot, one global
timeline, branch-as-mutable-pointer, merge-as-invoked-event, the working copy, and the repository as
a unit. Three are simply wrong here. Four contain a real requirement wearing a git costume: a state
commitment is load-bearing even though the snapshot is not; arbitration is real at a unique effectful
target even though compare-and-swap is not fundamental to content; a sparse working *context* is real
even though a materialized tree is not; and authority domain, confidentiality realm, retention scope
and trust policy are real boundaries that must not be re-fused into a "repository" type.

**The existing P1 kernel.** `gunbc.source_integration_proof_kernel` is built around an
`accepted_parent` and a before/after delta — the mutation-and-parent assumptions this model rejects.
*Proposed:* freeze it as quarry rather than extend it. This touches an active roadmap node and is an
operator decision, not this note's.

## 10. Open questions

1. **Authoring capture.** The largest fork, and the reason §7 is scoped as it is. Until an authoring
   surface records what an author *did*, proposals must be stated explicitly rather than inferred.
2. **Durable identity** (§5) — the declared rung. Needs a computing cryptographic digest.
3. **Recursion into named edges** — the next slice, and the one that demonstrates the differentiator.
4. **Retraction epochs** (§6) — the weakest claim in the note.
5. **Positional append** — whether two appends commute. Special-casing risks re-importing the
   line-merge heuristic through the back door.
6. **Alpha-renaming neutrality** — may hold only for non-exported, non-shadowing names, since an
   exported name is an external interface.

## Dissolution trigger (DESIGN §6)

This note dissolves into the carriers it names when a merge kernel and a confidentiality lifecycle
land. Until then it is evidence about a model, never authority over one. The §8 corpus outlives it:
those rows re-enroll against whatever kernel lands, per DESIGN §4b's rule that discriminating
evidence survives the machinery that prompted it.
