# Node/subtree visibility grants — two verbs in the effect-grant algebra, not a parallel mechanism

Status: DRAFT for operator review (2026-07-21, session gentle-otter-138). Origin: private/public
toggling over the containment tree — motivated generically by (a) hiding implementation-internal
declarations from cross-container reference, and (b) partitioning a source tree across storage
roots of differing audience, a need any project with a public mirror and a private overlay has,
and which forge-style per-subtree visibility toggles are a known point of prior art for. No code
lands from this doc. It rides `docs/plans/effect-namespace-grants.md` (owned silent-ibex-417) and
`docs/plans/namespace-resolution-design.md` (the containment-tree authority), both cited, not
duplicated.

## 0. The anti-fork verdict (DESIGN §3 — required before minting anything)

The operator concern this section answers first, explicitly: does visibility need a new
authority, or does an existing shape already carry it? Method: shape-search, not name-search —
each candidate below was read for its actual type definition, not matched on the word
"visibility."

| candidate | actual shape | carries the visibility axis? |
|---|---|---|
| `std.effect_grant` (`Grant{verb,root,binding,lifecycle}`, `admit_effect`, `NamespacePosition`/`position_under`) | permission = (verb × subtree) admission over a containment prefix relation | **yes — this is the hit.** Verb is open (`Read\|Write\|Execute`, already extended once for `Execute`); adding a verb is the designed extension point, not a fork. |
| `std.resources.ResourceHandle` (`{type, resource_id, key, cap: Secret}`) | a runtime credential/capability carrier for *acquiring* a resource | no — identity/auth-side, answers "can this process open the handle," not "may this name be referenced." Already named as a peer, not a merge target, in effect-namespace-grants.md §3. |
| `src/v2/std/live_tree.dag` `LiveTreeDisposition = ReadsLiveTree \| SubstrateInputsOnly` | per-witness declaration of whether a test reads host state, feeding affected-set selection eligibility | no — a witness-selection axis, unrelated in kind. Read in full; confirmed a false friend by shape, not just by name. |
| `std.realization_ladder`/`Hermetic \| Wet \| Record` presets | named envelope presets over effect grants (input closure / output reach) | no new axis — but structurally the precedent for "public/private/friend as named grant-root presets," reused below. |
| `std.realization` (`RealizedStep`, `Placement = LocalInProcess\|LocalFilesystem\|RemoteNetwork\|LocalAccelerator`, `Materialization = Recompute\|Memoize\|Share`, `RealizationPlan`/`Schedule`) | a query-planner over **redundant computation**: given one graph fact demanded N times, decide compute-vs-share and where the bytes physically sit during a *run* | **no, on the literal module.** `Placement`'s four variants describe compute locality for a cache tier, not a trust/audience boundary — forcing a fifth variant onto it would be exactly the state-space conflation DESIGN §3/§5 name as the recurring failure (the same trap §2 of this doc's Q2 already refused for `FrameKind`). See §1.3 below for what *does* transfer. |
| `gunbc.ownership` (`Ownership = Owned \| Ensured`) | fleet-reconcile teardown responsibility (who tears a deployed member down) | no — deploy-spine concept, unrelated axis. |
| `std.cache_interface.VisibilityScope = Repo \| Org \| Network \| World` | audience/trust-boundary scope for **cache-entry sharing** | **initially flagged a false friend for the `Reference` verb below (different question: cache read-sharing vs. name resolution) — but on completing the search for the storage/audience half of this brief, it is the closest existing shape for that half.** Corrected finding, not a new mint: see §1.4. |

**Verdict on the brief's hypothesis:**

- **(a) "one new verb on the same grant rows, zero new grant machinery" — HOLDS**, and in fact
  holds *twice*: this doc needed two verbs, not one, because it decomposes into two genuinely
  separate questions (§1.1–§1.2), and both verbs land on the unmodified `Grant`/`Envelope`/
  `admit_effect` shape from `std/effect_grant.dag`.
- **(b) "the public repo IS `realize(graph, audience=World)` on existing realization/
  materialization interfaces" — PARTIALLY HOLDS, with the divergence named precisely.** The
  capitalized **Realization pattern** DESIGN §2 names methodologically ("content-addressed
  pure-spec → host-effect; one kernel, N handlers") is exactly the right frame for "one graph, N
  storage projections" (§2 below). But the concrete `std.realization` **module** — `Placement`,
  `Materialization`, `RealizationPlan`/`Schedule` — is shaped for redundant-*computation*
  scheduling (when to recompute vs. share a result across demands), a different axis than "which
  git remote a subtree's *source* is stored in for a given audience." Reusing the pattern is
  right; reusing the module's concrete types would be a forced fit. The divergence is real and is
  documented, not smoothed over: this doc does **not** propose adding an audience arm to
  `Placement`.

## 1. Two axes, not one — both verbs, both on unmodified `Grant`

**1.1 — `Reference`: may this edge be *formed*, within one compiled program.** Unchanged from
this doc's prior draft (kept below, §3): a `.` projection (namespace-resolution-design §3.3 —
field / module-member / variant, one operation) constructs a reference edge; `Reference`-verb
grants decide whether the referrer's position is admitted to hold it. Compile-time, symbol-grain,
audience-blind — a private helper stays uncallable from outside its container *regardless of
which storage root either side lives in*.

**1.2 — `Publish`: may this node's *source* be present in a given storage realization at all.**
New in this draft, answering the brief's storage/distribution half. A node's declaring author
grants `Publish` to an **audience** — not a code position, a trust-scope position (§1.4) — and a
storage realization for a given audience includes exactly the subtree reachable under grants it
holds for that audience.

**These are independent axes, and the independence is the point, not an oversight:** a node can
be `Reference`-public (any code in the tree may call it) yet `Publish`-private (its source never
leaves the private storage root — an internal library callable freely inside the monorepo but
never mirrored out); or `Reference`-private (a Rust-style implementation-detail fn, uncallable
from outside its module) yet `Publish`-public (its source text *is* in the open mirror, just not
part of the callable surface). Fusing them into one verb would be the exact state-space
conflation DESIGN §3/§5 name — "an `Option`/`None` meaning >2 things" — so they stay two verbs on
one algebra, composing rather than merging.

**1.3 — What transfers from `std.realization` even though the module doesn't.** The reconcile
pattern in `std.realization.reconcile` — many logical demands, one produced artifact, `Share`
when two demands prove identical — is the right *shape* for "one graph, N storage projections
of the same content-addressed subtree," just needing its axis renamed: a storage realization is
not sharing a *computed result* across redundant demands, it is sharing *source content* across
audiences, keyed the same way (`ContentHash`) for the same reason (one authority, never a byte
fork between the public mirror and the private root for the subtree they do share). §2 uses this
framing without importing `RealizedStep`/`Placement` themselves.

**1.4 — `VisibilityScope` converges into the `Publish` grant's audience tree.** Correcting this
doc's earlier "false friend, no convergence" call (that call was right for `Reference`, wrong to
generalize to "never converges anywhere" — the search continued for `Publish` and found a real
hit): `Repo | Org | Network | World` is already a **totally ordered audience-breadth axis** —
exactly nested containment (`Repo ⊂ Org ⊂ Network ⊂ World`). Read as a small, fixed
`NamespacePosition` tree (`tree: AudienceScopeTree`, four canonical positions, each path a prefix
of the next), `Publish` grants reuse `position_under`/`admit_effect_go` **verbatim**, the same
fold `Reference` uses, just against a different cited tree — the "N cited trees" list
`std/effect_grant.dag` already carries (`FilesystemPathTree | UriTree | ProcTree |
ServiceOpTree | CodeNameTree`) gains a sixth member, `AudienceScopeTree`, and
`std.cache_interface.VisibilityScope` becomes a **projection** of its four positions rather than
a parallel enum (dissolve-on: the projection lands, `VisibilityScope`'s own definition stays as
the cache-lane's local name for the same four positions, or is retired in favor of the shared
tree — an open question, §7).

## 2. One graph, N storage projections

A private repo is **not** a second graph — it is a **storage realization of the subtrees whose
`Publish` grant excludes `World`**. The current single public repo becomes, under this model, the
`audience=World` projection of the same containment tree; a private overlay repo is the
`audience={Org, Repo, ...}` projection carrying everything `World` cannot see. Both are
**materializations of one graph fact**, never two authorities — this is what keeps the affected
set, content-addressing, and CI from ever having to reconcile two divergent trees, because there
is structurally only one (§2/§3 DESIGN discipline: single authority, multiple realizations, the
same move `dag/std/integer.dag`'s ten machine-width rows already make for one `Int` axis). Local
development mounts both storage roots at once — the existing multi-root precedent
(`--source-root src/v2 --source-root dag`, already load-bearing in `gunbc ci`'s compile-clean
gate) generalizes to "mount every storage root you're authorized to read," not a new mounting
concept.

## 3. `Reference` verb (carried over, unchanged in substance)

**Reuse, verbatim, from `std/effect_grant.dag`:** `NamespacePosition { tree, path }` (in
particular `tree: CodeNameTree`, already enumerated and unused until now — the code-name
containment tree namespace-resolution-design makes the single naming authority),
`path_is_prefix` / `position_under`, and the `Grant { verb, root, ... }` /
`admit_effect_go`-style fold. **Added:** one `Verb` variant.

```
type Verb = Read | Write | Execute | Reference | Publish
```

`Reference` is grounded the same way `Execute` was (`verb_of_effect_shape`): it names the act of
forming a projection edge, not a REST-derived CRUD shape — a peer coarsening, not a fork.

**Directionality is inverted from effect grants, deliberately — the dual reading.** An effect
grant answers "what may this *frame* reach *out to*" (root = target the actor may touch). A
`Reference` grant answers "what may reach *in to* this *node*" (root = the subtree of referrers
admitted to hold a reference edge to it). This is the same relation asked backwards — exactly the
pattern DESIGN §4 already names for `emit`/`ingest` ("one grammar, read in both directions ... the
structural inverse, not a second emitter") and for `coercion_fold` ("one procedure asked in three
directions, not three procedures"). Visibility is grant admission's fourth direction, not a fourth
mechanism. `Publish` (§4) reuses this same inverted reading, over a different cited tree.

```
type VisibilityStatus
  = LegacyOpen
  | Declared { grants: List<Grant> }

type NodeVisibility {
  node: NamespacePosition   -- tree: CodeNameTree
  status: VisibilityStatus
}
```

- **`LegacyOpen`** is today's actual, honest behavior — no privacy concept exists, so every node
  not yet migrated resolves exactly as it does now, from anywhere, unconditionally. Same shape as
  the self-host frontier (DESIGN §7: `SelfEmitted | SeedRetained`, "a declared row with a reason
  and a migration trigger — countable, prioritizable — never a silent escape hatch"): a typed,
  counted frontier state, not a default grant.
- **`Declared { grants }`** is fail-closed exactly like `admit_effect_go`: once a node opts in, a
  referrer outside every declared grant root is refused. One implicit exception needing no grant
  row: a referrer under the node's own immediate declaring container is always admitted — not a
  grant, namespace-resolution-design's existing ancestor-visibility rule read at the declaring
  container's own grain. `Declared { grants: [] }` is therefore genuine, strict privacy: visible
  only within the declaring container.
- **Public / private / friend are grant-root presets, not new types** — the same move
  `effect-namespace-grants.md` §2 uses for `Hermetic | Wet | Record`. *Private* =
  `Declared { grants: [] }`. *Public* = a grant rooted at the `CodeNameTree` root. *Friend /
  `pub(crate)`-shaped* = a grant rooted at some ancestor subtree narrower than the tree root —
  Rust's binary `pub` and its `pub(in path)` refinement are the same mechanism at different root
  depths, not two features.

The `visible_from` fold, resolver seam (`.` projection), and phase plan for this verb are
unchanged from the prior draft and are folded into §6's combined phase list below rather than
repeated twice.

## 4. `Publish` verb — audience-parameterized storage admission

```
type AudienceScopeTree = AudienceScopeTree   -- sixth cited NamespaceTree variant

type Audience = Repo | Org | Network | World   -- canonical AudienceScopeTree positions,
                                                -- nested: World ⊃ Network ⊃ Org ⊃ Repo
```

A `Publish` grant is `Grant { verb: Publish, root: <AudienceScopeTree position>, ... }`, attached
to the declaring node exactly like a `Reference` grant, using the identical inverted-direction
reading (§3): root = the audience subtree admitted to see this node's source. `LegacyOpen |
Declared` applies identically — a node with no declared `Publish` grant is not silently public or
silently private; it is the same typed, counted frontier state as `Reference`'s `LegacyOpen`
(§6 fixes the *default* frontier disposition explicitly, since storage has a live incident class
`Reference`'s frontier did not: see §5).

**Rule 1 — visibility monotone along the import DAG.** A node's `Publish` audience must be at
least as broad as every node it references: a `World`-published node may only reference other
`World`-published nodes; a `Repo`-scoped (most private) node may reference anything. Checkable
over the existing import-closure machinery (the same reference edges `Reference`-verb grants
gate, walked for a different purpose here). This is what keeps a storage projection
**self-contained** — the reason it matters is concrete: if a public node referenced a private one,
the `World` storage realization would ship a dangling reference (unresolvable in the public-only
build; CI, the affected set, and content-addressing all silently assume a closed tree today and
would break on the first violation). Violation is a typed, located refusal (`PublishClosureGap`),
never a warning.

**Rule 2 — privacy downward-closed on containment.** A node's `Publish` audience must be no
broader than its declaring ancestor's. Rationale is name-leakage, not just policy taste: a
qualified name **is** a containment path (namespace-resolution-design §3, point 1), so a `World`-
published child under a `Repo`-scoped parent would leak the parent's existence and name the
moment the child's qualified path is written down anywhere in the public projection — the private
parent's name is *in* the child's own address. Widening below a tighter ancestor is refused
(`PublishAncestorNarrower`). **Nearest-ancestor grant wins** when a subtree has no grant of its
own — the identical tie-break namespace-resolution-design already uses for lexical lookup
("nearest enclosing binding," here read as "nearest enclosing audience"), not a new rule invented
for this doc.

**Grant declared on the node; storage placement derived, never independently declared.** The
`Publish` grant is the single authority; which physical storage root (git remote) holds a node's
source is a **projection** of its resolved audience, computed, not separately written anywhere. A
construction wall — `VisibilityPlacementMismatch`, typed and located, fired at push time — refuses
whenever a file about to land in a storage root carries content whose resolved `Publish` grant
does not admit that root's audience. This is the incident class named in the brief and it is
worth stating precisely: **a file with no declared `Publish` fact, pushed to the public root, is
not "assumed public"** — per §4/§8's frontier default, undeclared content defaults to the *narrowest*
storage root until declared, so the failure mode of "forgot to mark it" is exclusion from the
public mirror (loud, safe, reversible by declaring the grant) rather than accidental publication
(silent, unsafe, irreversible — history is public forever once pushed). This is the fail-closed
choice DESIGN §5 requires whenever a default must be picked at all.

## 5. World projection renders absence, not a redacted stub — decided, not left open

A subtree whose `Publish` grant excludes `World` is **entirely absent** from the `World` storage
realization: no file, no directory, no commit touching it, ever. A redacted stub (a placeholder
file, a "N lines omitted" marker, a commit that touches a path without its content) is rejected
as a design option, not merely deferred, because a stub leaks exactly what absence hides — file
count, churn frequency, commit cadence, and often the name itself, none of which are inert
metadata once made public.

**Content-hash stubs are named as a distinct, later, explicitly-opted-in mode for integrity-
continuity use cases** (e.g., proving a locally-resolved private node matches what a hypothetical
public build would have content-addressed, without exposing its bytes) — never the default, never
built in Stage 0/1 (§6), and requiring its own per-node opt-in the same way `Declared` requires
opting a node into `Reference`/`Publish` at all. The default for every node, declared or
`LegacyOpen`, is plain absence. Stating this now, even though the stub mode is not built, is the
explicit decision the brief asks for: the axis has a named default and a named (unbuilt)
alternative, not an open question.

## 6. Staging

Mirrors `effect-namespace-grants.md`'s own phase discipline — each green-by-execution with REDs,
under-scope counted and never silent. `Reference` (§3) and `Publish` (§4) share P-A/P-B shape;
listed together, noting where they diverge.

- **P-A (model, no behavior change):** `Verb::Reference` and `Verb::Publish` added to
  `std/effect_grant.dag`; `NodeVisibility`/`VisibilityStatus` modeled for both, including the
  `AudienceScopeTree` positions and Rules 1–2 as pure predicates over synthetic trees, with REDs
  (referrer/audience outside every grant → typed refusal; container-internal referrer → admitted
  with no grant; ancestor-narrower child → `PublishAncestorNarrower`; closure gap →
  `PublishClosureGap`). No storage or resolver wiring yet.
- **P-B (first enforcement seam, Stage 0 — file granularity):** two git storage roots (public +
  private overlay), file-grain `Publish` declaration. A push-time guard wall computes
  `VisibilityPlacementMismatch` (§4) and a public-to-private dangling-reference check (Rule 1,
  degenerate to file grain). The resolver's `.` projection path calls `Reference`'s `visible_from`
  when the projected child leaves the referrer's ancestor chain (unchanged from the prior draft);
  refusal is `NameNotVisible`. For `Reference` this is a zero-behavior-change landing by
  construction: every node is `LegacyOpen` until an author explicitly declares one, and
  `LegacyOpen`'s undeclared default stays unconditional resolvability. `Publish` is **not**
  zero-behavior-change the same way (§10 Q5 names this explicitly) — its undeclared default is
  narrowest-storage-root (§4/§8), which would exclude every currently-public file the instant the
  guard goes live unless the landing step itself stamps the existing corpus. So P-B's `Publish`
  half lands with a one-time bulk migration as part of the same change: every file already
  committed to the public root is declared `Publish { audience: World }` explicitly (not left
  `LegacyOpen`) at cutover, so the narrowest-default rule only ever bites *new* undeclared content
  added after P-B lands, never retroactively excludes what was already public. That migration step
  is what makes "today's all-public-repo content stays public" true; it is a one-time cost, not an
  ongoing one, and is the concrete answer to Q5's workflow-cost concern (new files need an explicit
  grant going forward; existing ones do not need one retroactively).
- **P-C (Stage 1 — the composed-graph wall):** Rules 1–2 run as compile-time refusals over the
  **composed** graph (both storage roots mounted, the §2 multi-root precedent), not just the
  push-time file check — turning "the public repo happened to not dangle" into "the public repo
  cannot dangle." Gated on P-B landing and on whichever dev/CI context can afford to mount the
  private root (§7 states the resulting enforcement-strength split explicitly).
- **P-D (Stage 2 — node granularity):** rides the module-identity many-to-many storage-binding
  thread (`module-identity-storage-binding-design.md`) so one file can carry nodes of different
  `Publish` grants; the public tree becomes a **generated projection** rather than a hand-
  maintained overlay repo. Not attempted before that thread's binding authority lands — this doc
  takes a dependency on it rather than re-deriving file↔node binding here (§3 single authority).
- **P-E (surface syntax, later):** a keyword or header row toggling `LegacyOpen → Declared` for
  either verb. Per namespace-resolution-design §11's own steer ("prefer keyword-free bare
  containers ... nothing ever needs renaming"), a grammar-row question for whoever owns that
  surface then, not decided here.
- **Declared, not built — the serve-time extension seam:** a per-principal (not just per-audience-
  tier) visibility check at serve time — the generalization forge-style tooling points at — is
  named as a future consumer of the same `Grant`/`admit_effect` shape (a `Frame` representing a
  requesting principal, `Publish`-checked per request instead of per storage realization) and
  explicitly **not built**: no code, no stub, no placeholder type. Declaring the seam here is what
  keeps a future implementer from reinventing the algebra; building it now would be scope this
  brief did not ask for.
- **Migrate the two `Reference` displaced costs (prior draft's §0):** `extdeps/` internal helpers
  and the `_go`/`_rec` accumulator family get `Declared { grants: [] }` `Reference` rows, each a
  per-site opt-in priced by an actual rename or a caught unintended dependency — never a bulk
  sweep.

## 7. The grant-row-location question, treated explicitly

A private subtree's own `Publish` grant row is itself content in that private subtree — Rule 2
applies recursively to the marker, not just the code it marks. Consequence: **the public-repo-only
guard (P-B) can only enforce what it can see** — its own declared markers, plus a dangling-
reference check against what the public tree references. It cannot see a private node's grant row
to know Rule 1/Rule 2 hold *from the private side*, because that row does not exist in its
mount. **The composed-root wall (P-C) is where completeness actually lives** — enforcement
strength is proportional to how much of the tree a given check can mount, which is inherent to a
genuinely split-storage system, not a gap in this design. This is acceptable **provided the
asymmetry is never mistaken for completeness**: P-B's guard must report exactly what it checked
(own markers + dangling references), never claim the full Rule 1/Rule 2 closure it cannot see —
a check that silently under-reports its own coverage is the DESIGN §5 absorbing-fallback trap
("everything is affected" and "I could not compute what is affected" are different states) read
at the coverage-claim level instead of the answer level. Concretely: P-B's refusal type and P-C's
refusal type stay **distinct** (`PublishGuardLocalViolation` vs. `PublishClosureGap`/
`PublishAncestorNarrower`), so a reader of a refusal always knows which mount produced it and
therefore how complete that check was entitled to be.

## 8. Fail-closed discipline (DESIGN §5, restated for this doc's two verbs)

- Every refusal is typed and located (`NameNotVisible`, `VisibilityPlacementMismatch`,
  `PublishClosureGap`, `PublishAncestorNarrower`, `PublishGuardLocalViolation`) and counted —
  no bucket is a silent catch-all.
- No escape-hatch toggle: nothing "proceeds as if the refusal had not fired." An author who wants
  a node visible changes its `Grant` row (the single authority), never a flag that bypasses the
  check.
- The `LegacyOpen` frontier is the one place a default is permissive (§3), and it is honest about
  it: it is what already happens today, named and counted rather than hidden, with the migration
  priced per-site rather than swept. `Publish`'s undeclared-content default (§4) is the opposite
  choice, deliberately, because unlike a stale in-tree symbol reference, an accidental public push
  is irreversible — the two frontiers are not the same shape and this doc does not pretend they
  are.

## 9. Convergence map (§2/§3 — DFS'd before minting; supersedes the prior draft's table)

| element | existing carrier | relationship |
|---|---|---|
| position / subtree / `⊑` (code) | `std.effect_grant.NamespacePosition`, `position_under` (`CodeNameTree`) | reuse verbatim |
| position / subtree / `⊑` (audience) | `std.cache_interface.VisibilityScope`'s four values, reframed | converge — projected onto a sixth `NamespaceTree` variant, `AudienceScopeTree` (§1.4, corrected from the prior draft's "false friend" call) |
| grant / admission fold | `std.effect_grant.Grant`, `admit_effect_go` | reuse the algebra unmodified; two new verbs (`Reference`, `Publish`), direction inverted for both (documented, not hidden) |
| the edge `Reference` gates | `.` projection (namespace-resolution-design §3.3) | reuse — the one seam that constructs a reference edge |
| the edges `Publish`'s Rule 1 walks | existing import-closure machinery | reuse — same reference edges, different question asked of them |
| containment monotonicity tie-break | namespace-resolution-design's nearest-enclosing-binding rule | reuse verbatim for "nearest-ancestor grant wins" (Rule 2) |
| undeclared/migrating state | self-host frontier (DESIGN §7: `SelfEmitted \| SeedRetained`) | pattern reuse for `LegacyOpen \| Declared`, on both verbs, with deliberately different default polarity (§8) |
| "one graph, N storage realizations" | DESIGN §2's capitalized Realization pattern; `std.realization.reconcile`'s share-on-identity shape (content-addressed, not the module's concrete `Placement`/`Materialization` types) | pattern reuse, module reuse explicitly declined (§0/§1.3 — the named divergence) |
| effect verbs on the resolved value | `std.effect_grant.Verb = Read \| Write \| Execute` | orthogonal, composes after resolution, not touched |
| resource acquisition credential | `std.resources.ResourceHandle` | orthogonal (already named as a peer in effect-namespace-grants.md §3), not touched |
| fleet-reconcile teardown ownership | `gunbc.ownership.Ownership` | unrelated axis, ruled out by shape |
| witness live-read selection eligibility | `LiveTreeDisposition` | unrelated axis, ruled out by shape |
| "below-boundary representation is opaque" | DESIGN §3, stated rule, currently unenforced | `Reference` (§3) is the enforcement mechanism named against that gap |

## 10. Open questions (operator)

1. **Verb name for §3** — `Reference` (paired with the existing action-verb shape of
   `Read`/`Write`/`Execute`/`Publish`) vs. `Visible` (adjectival). No behavior difference.
2. **`AudienceScopeTree` encoding** — a genuine, fixed four-position nested tree as sketched in
   §1.4/§4, vs. keeping `Audience` a flat enum and giving `Grant.root` a second, non-
   `NamespacePosition` variant for it. The tree encoding is preferred (buys `position_under` for
   free, zero new comparison logic) but the fixed-cardinality, non-code-shaped nature of audience
   positions is different enough from the other five `NamespaceTree` members that forcing it in
   deserves explicit sign-off rather than silent adoption.
3. **`Reference`'s directionality vs. `Frame`/`Envelope`** — confirmed in the prior draft:
   `FrameKind` is closed over four materialization-scoped variants with no compile-time-scope
   member, so grants live on the declared node, not a `Frame`. Carried forward unchanged; open
   question is whether `Publish` should follow the same call (yes, by symmetry — flagged here only
   so it is ruled on together with #1, not assumed).
4. **`Reference`'s `LegacyOpen` dissolve-on** — left open-ended (per-node opt-in only) rather than
   a scheduled bulk migration; confirm vs. an operator-set target date for corpus-wide
   default-private.
5. **`Publish` frontier default (§4/§8)** — this doc picks "undeclared defaults to narrowest
   storage root" as the fail-closed choice. Confirm, since it is a behavior change the moment
   Stage 0 lands (today, *everything* is in the one public repo; after P-B, undeclared new content
   defaults private-until-marked) — a decision with real workflow cost (every new public file
   needs an explicit grant) that the operator should rule on knowingly rather than inherit from
   this doc's §5 fail-closed default.
6. **`VisibilityScope` retirement** — once the `AudienceScopeTree` projection lands (§1.4), does
   `std.cache_interface.VisibilityScope` get deleted in favor of the shared tree, or kept as the
   cache lane's local name projected from it? Either is consistent with §3 (below-boundary
   representation is opaque); picking one avoids a second live name for one fact indefinitely.
