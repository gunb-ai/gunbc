# Node-grain merge and the confidentiality lifecycle

DRAFT, design-note-first. **No code lands from this note.** It records a model that changed shape
twice while being worked out, so its purpose is to be cheap to be wrong in. Every claim below is
marked as **verified** (read out of the live tree), **derived** (a consequence of a verified claim
plus an authority doc), or **proposed** (neither — a design position awaiting a discriminating
witness or an operator ruling).

Scope: what a `.dag`-native merge is, what identity it does and does not require, and what happens
when confidential material enters the history. It does not schedule work, and it deliberately does
not claim the constraint-satisfaction semantics `dag-scm-design.md` §4 describes — see §7.

---

## 1. The product requirement, restated

Operator directive: **eliminate rebasing.** Branching stays a real DAG; what disappears is the user
ever being asked to replay their work onto a moved base. A one-parent landing commit is a
*consequence* of that, not the mechanism — the intermediate states of a proposal were never the unit
of meaning, so nothing is lost by not preserving them in the target.

Rebase exists because git, holding only endpoints, must re-derive a diff whenever the base moves.
The design's answer is that a moved target is re-evaluated **internally** — `dag-scm-design.md`'s
`RetryStaleParent`, with timestamps and arrival explicitly non-authoritative. No user-facing merge,
rebase, cherry-pick, reset, conflict-marker editing, or force-push appears in a normal journey.

## 2. Five jobs, not one

Most of the confusion in working this out came from collapsing five independent problems into the
word "merge". They are separable and should stay separate:

| job | question |
|---|---|
| edge-local recursion | which parts of two trees correspond, and did this part change? |
| transformation lineage | is this the *same thing* renamed or moved, or a different thing? |
| subtree stability | can this whole region be skipped as unchanged? |
| semantic admission | is the merged program *valid*, independent of whether merging was clean? |
| publication | which bytes may an audience receive? |

§3–§5 cover the first three, §6 the fourth, §8 the fifth.

## 3. The merge key comes from the edge type, not from a chosen grain

**Verified.** `v2.std.node` already classifies every child edge:

```dag
type EdgeLabel = Named { name: Symbol } | Positional
type Edge { label: EdgeLabel, target: Node }
type Node { kind: NodeKind, children: List<Edge>, occurrence_id: NodeOccurrenceId }
```

This is per-edge and depth-independent — nothing in it mentions declarations. Named edges reach
deep: the v2 compiler constructs `match_arm_pattern`, `match_arm_body`, `binding`, `binding_id`,
`field`, `variant_marker`, and attaches a function declaration to its parent as
`Edge { label: Named { name: fn_name }, target: arrow }`. Calls are labeled too, which is why the
interpreter can refuse a *mislabeled* call.

**The rule, derived:** recurse through `Named` edges keyed by the edge name within its parent; at
`Positional` edges compare the region whole.

The positional half is not a limitation being tolerated. Where the substrate says positional,
**order carries meaning** — list literal elements, match arm order. Merging two edits there by
identity would fabricate a sequence nobody wrote, which is precisely what git does by line and calls
success. Refusing there is the safety half of the thesis, not a gap in it.

There is therefore **one grain — the node.** Recursion depth is not a parameter; it falls out of
where the substrate says identity is by name versus by order.

## 4. Cross-revision node identity is not required, and is not available

**Verified.** `std.occurrence_identity` `occurrence_identity_scope_law`: OccurrenceId values are
unique only inside one graph-scoped allocator; a standalone parse creates an isolated scope. Filename,
SourceSpan, authored name, structural Node equality, content hash, and module-local allocator reset
are **forbidden identity inputs**. Two parses of two revisions are two scopes, so the ids are
incomparable, and every shortcut for recovering identity from the bytes is closed by the same law.

**Derived, from DESIGN §3's fleet ruling** — identity is declaration-owned and allocator-minted,
never derived from unit/path/content, because inferring a rename from content similarity is the
heuristic §4 forbids in a closed system — cross-revision identity could only be **minted at
authoring time and carried in the source.** It cannot be recovered afterwards.

**And it is not needed**, which is the load-bearing conclusion. Identity's only job in a merge is to
**transport a patch to the right place**. A submitted revision is better read as a *state assertion*
— "the thing called `foo` should be this" — and assertions are intersected, not transported:

- assertions about **different names** → union, trivially consistent
- assertions about **the same name, same content** → consistent
- assertions about **the same name, different content** → a genuine disagreement about what the code
  should be, not a text collision

**Fungibility, scoped.** Content identity governs *storage sharing*; the *edit target* is a
containment path. Two structurally identical subtrees deduplicate in the store and remain
independently editable: a patch edits an edge occurrence at a path, never "all nodes with this hash."

**A base is still required — for scope, not for alignment.** Without it, "my revision does not
contain `baz`" is ambiguous between *`baz` should not exist* and *I did not touch `baz`*. Read the
first way, every concurrent addition to a shared module conflicts. So the base determines which
names an author actually asserted about. Base yes, identity no; the two are separable and were
conflated in earlier drafts of this analysis.

**Rename, consequently.** Renaming `foo`→`bar` asserts *`bar` exists with this body* and *`foo` does
not exist*. A concurrent *`foo` has this new body* contradicts it. That is an honest conflict with no
heuristic, and it is the design's `ChoiceRequired` shape: *"you renamed `foo` to `bar`; someone else
edited `foo` — apply their edit to `bar`, or keep `foo`?"* — one question, two concrete previews, in
domain language. Git cannot ask it, because git never knew a rename happened.

## 5. The value is node-local; the recursive digest is an accelerator

**Verified.** `v1.compiler.dag_collect_support` `dag_node_surface_fingerprint_rec` is fully
recursive — a node's digest mixes `dag_node_surface_leaf_mix` with every child and param digest,
recursively.

**Derived:** it is therefore *not* the merge value. One leaf edit dirties every ancestor to the root,
so any two edits under a shared ancestor would collide at every level between them. The value is the
**exact node-local authored state**, excluding source spans, formatting, file path, inferred caches,
and descendant content.

The recursive digest keeps a real job: **pruning.** Equal subtree digests skip the region; differing
digests are navigation evidence, not edit records. Where the digest is non-cryptographic — today
Fnv1a64 — equality must be confirmed exactly before a destructive transition, per §7's caution.

## 6. `std.change`'s three-way leaf is reconcile-shaped, not merge-shaped

**Verified.** `std.change` `keyed_three_way_leaf(base, observed, desired, equal)`:

```dag
if equal(observed, desired)   { KeyedUnchanged }
else if equal(observed, base) { KeyedApplyHunk { from: base, to: desired } }
else                          { KeyedConflict }
```

With `observed = A`, `desired = B`: A==B unchanged ✓; A==base ApplyHunk→B ✓; **B==base with A
changed → `KeyedConflict`** ✗; both changed → conflict ✓.

The third row is the most common merge case in existence — only one side touched this node — and it
reports a conflict. That asymmetry is *correct for reconciliation*, where `desired` is authority and
observed drift is reverted or flagged, and *wrong for merge*, where the two sides are peers.

**Derived:** the traversal and keying are shared with DESIGN's fleet-reconcile spine; the **verdict**
is domain-specific, because reconcile has an authority side and merge does not. A symmetric leaf
verdict is owed. This is small, but naming it now avoids discovering it as "why does every proposal
conflict."

## 7. What this model does *not* claim

Three limits, stated because each was reached by correcting an earlier overclaim in this same
analysis.

**It is not constraint satisfaction.** `dag-scm-design.md` §4 describes composing authored
transformations with checkable certificates over a bounded candidate closure. The model here opens
with `capture(base, proposal) → patch`, which is a base diff. This is **diff-and-patch with a
semantic key instead of a textual one** — a real improvement over git, and not the native model the
design specifies. Whether to pursue genuine transformation capture (design P2, parked) is an open
operator decision, not something this note settles.

**Merges are not "always safe."** Deep semantic understanding makes conflicts rarer and better
explained; it does not eliminate them. Two changes can each be valid alone and interact: a new call
to `foo` plus a narrowing of `foo`'s contract. Structural merge and **semantic admission** are two
phases and the second genuinely refuses. The honest product claim is *far fewer conflicts, each a
real question in the user's vocabulary* — never *always safe*.

**Structural cleanliness is not validity.** A structurally clean rename plus a stale reference merges
cleanly and produces a broken program. Admission, not merge, is what catches it.

## 8. The confidentiality lifecycle

Secrets are the forcing function for redaction, and redaction is where a content-addressed history
either has an answer or admits it does not.

**Git's answer is a non-answer**, and that is the design input. A secret in git history requires
rewriting every subsequent commit; every hash changes; every clone, fork and open PR diverges. On a
public repository this is not merely expensive but *ineffective* — the objects are already
distributed. The answer practitioners actually use is "rotate the credential and assume the bytes
are permanently public." We must not build a better version of that theater.

**Four operations git fuses into one:**

- **Prevention** — bytes never reach the public store. *The only one that delivers confidentiality.*
- **Retraction** — authoritative history no longer carries the material going forward: new clones do
  not receive it, the projection no longer contains it, the material is marked revoked. **Not
  erasure.**
- **Erasure** — bytes unrecoverable. Claimable only for material that never left a fully controlled
  store, and then only when every key, wrap, backup, cache, derivative, index and recovery path is
  inside the sanitization scope. Never "delete one key."
- **Invalidation** — rotate the credential so leaked bytes are worthless. **This repository already
  models it**: `gunbc.auth.secret_rotation` is a subject-agnostic kernel with exactly-once walls,
  receipt-backed retirement, and no stored payload digest.

**Derived rule:** on a committed secret, SCM performs *retraction*, the rotation kernel performs
*invalidation*, and *erasure* is claimable only in the private realm. **The SCM must never report
retraction as if it were erasure**, and `PubliclyIrrecoverable` must be a state the system can
*report*, not a failure it hides.

**Where the wall sits.** Merge-time or CI-time checking is structurally incapable of confidentiality
— the deleted Stage-0 placement gate established exactly this: a required check could refuse `main`,
but pushed objects had already reached public storage. The chronology must be private capture →
private merge and admission → derive an audience-authorized projection → serialize → write public
storage. The public writer must not accept an arbitrary `Node`, blob or patch, only a projection
minted by the authoritative private context. **Corollary:** a public PR branch cannot safely carry
secret bytes on the theory that squash-landing removes them.

**Retraction without rewriting.** Where a commit is *accepted parent plus sparse transformation*,
retraction can be an ordinary **appended transformation** rather than a rewrite — no existing
identity changes and no downstream clone is invalidated. **This holds only if public history is an
audience projection rather than a replayable copy of private transitions**; otherwise a reader
replaying from an early checkpoint reconstructs the secret on the way to the head. The proposed
resolution is an **audience-specific projection epoch with a sanitized anchor**: re-anchor the
*public projection*, never the authoritative history, accepting an explicit compatibility boundary
for older projections. *(Proposed — this is the least settled part of the note.)*

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

## 9. Discriminating scenario corpus

No merge kernel should be built without these. Each is a RED unless marked otherwise.

**Structural merge**

| scenario | required result |
|---|---|
| target changes an untouched named node | preserved, no conflict |
| proposal edits one named child, target edits another | both preserved |
| proposal deletes a subtree, target edits a descendant | conflict |
| concurrent same-name add, same kind, different children | conflict |
| one side changes node kind, other edits a child | conflict |
| both sides produce an identical kind-changed subtree | clean |
| unchanged-body rename | unique relocation candidate |
| rename versus edit under the old name | conflict; the edit is not lost |
| two sides move one subtree to different parents | conflict, never duplicated |
| duplicate named siblings in any input | input refusal |
| two different positional appends | conflict or explicit order choice |
| distinct positional ordinals edited without shape change | both preserved |
| only one side changed a node | **preserved — the §6 asymmetry regression control** |

**Admission and capture**

| scenario | required result |
|---|---|
| structurally clean rename plus stale reference | semantic admission refuses |
| occurrence ids collide numerically across allocator scopes | result reminted or remapped |
| comment or annotation absent from `Node` capture | opaque residue or capture refusal, never silent loss |

**Confidentiality**

| scenario | required result |
|---|---|
| secret bytes in a private candidate | the public writer never receives them |
| secret-content edit plus audience broadening | separate authority required |
| hidden-node conflict diagnostic to an unauthorized actor | authorized redaction, no existence leak |
| retraction reported to a user | never phrased as erasure |
| proposal authored against pre-retraction state | typed refusal, with a stated user-facing sentence |

## 10. Open questions

1. **Diff-and-patch versus transformation capture** (§7). The largest fork. Capture requires an
   authoring surface; the operator's stated present reality is that the interface is files.
2. **Retraction epochs** (§8) — the sanitized-anchor proposal is unverified and is the note's weakest
   claim.
3. **The symmetric merge verdict** (§6) — owed, small, and a prerequisite for anything executable.
4. **Positional append** — whether two appends are a legitimate commuting case. Special-casing it
   risks reintroducing the line-merge heuristic through the back door.
5. **Whether alpha-renaming is semantically neutral here.** Exported names are an external interface,
   and shadowing exists; the neutrality argument in §4 may hold only for non-exported,
   non-shadowing names. **Unresolved.**

## Dissolution trigger (DESIGN §6)

This note dissolves into the carriers it names when a merge kernel and a confidentiality lifecycle
land. Until then it is evidence about a model, never authority over one. The §9 corpus survives it:
those rows are re-enrolled against whatever kernel lands, per DESIGN §4b's rule that discriminating
evidence outlives the machinery that prompted it.
