# Node/subtree visibility grants — a `Reference` verb in the effect-grant algebra

Status: DRAFT for operator review (2026-07-21, session gentle-otter-138). Origin: brief to
design private/public toggling over the containment tree as a consequence of the
effect-namespace-grants thread rather than a parallel mechanism. No code lands from this doc.
It rides `docs/plans/effect-namespace-grants.md` (owned silent-ibex-417) and
`docs/plans/namespace-resolution-design.md` (the containment-tree authority) — both cited, not
duplicated.

## 0. Displaced cost (§6 — the pain this removes)

Two facts already in the repo justify this, not a speculative want:

- **DESIGN §3 states a rule it cannot enforce today:** "below-boundary representation is
  opaque (the rename test)" — `extdeps/` internals should be freely renameable because nothing
  outside the boundary depends on them. That is currently *diligence*, not a wall. Nothing stops
  a distant caller from reaching into an `extdeps/` internal, a `_go`/`_rec` accumulator helper
  (`admit_effect_go`, `path_is_prefix`'s recursion, the dozens like them), or a below-std
  representation, and once it does, the rename test silently fails — the caller *is* a second
  authority on that shape (§3's own definition of a fork), discovered only when someone tries to
  rename and breaks a caller nobody modeled as a dependent. This is a live, named gap, not a
  hypothetical one.
- **`namespace-resolution-design.md` §11 records the current state explicitly:** "interface/export
  boundary — absent (keyword set has no pub/export/private; reachability is lexical)." There is
  no privacy concept in the language today. The census in that doc (98% of names globally unique)
  means this has not bitten yet at scale, but the two use-cases above — hiding accumulator
  helpers, and making the extdeps opacity rule real — are the displaced cost, and they compound
  as `std`/`extdeps` grow (§6: don't wait for the bottleneck to name the fix).

The point of riding effect-namespace-grants rather than inventing a new mechanism: a second,
parallel access-control vocabulary would itself be the §3 violation this doc exists to prevent.

## 1. The concept, stated once

**A reference is an edge** (DESIGN §4: a program is `Node` + `Edge`; namespace-resolution-design
§3.3: "`.` = projection one level down that tree ... descend into a named child" is the one
operation that constructs such an edge, whether the child is a field, a module member, or a
variant). Visibility is the question **"may this edge be constructed?"** — decided once, at
projection time, between the referencing position and the referenced position. That is exactly
the shape `std.effect_grant.admit_effect` already answers for effect targets: *is a verb, from a
frame, onto a target position, admitted by a grant set?* Node visibility is the same question
asked of a different verb and a different edge-direction, not a different question.

**What this is not.** Visibility gates whether a reference edge may be *formed* — it says
nothing about what happens once the referenced node is reached. Reading, writing, or executing
the *resolved value* stays `std.effect_grant`'s existing `Read`/`Write`/`Execute` concern; a
public `fn` is still just a `fn`, subject to whatever effect grants apply when it runs. The two
axes compose (resolve, then act) but neither absorbs the other — keeping them separate is itself
a §3 discipline (distinct concepts, distinct carriers), not an oversight.

**What this is also not.** `dag/std/cache_interface.dag`'s `VisibilityScope = Repo | Org |
Network | World` is a false friend — same word, different concept: it answers "how widely may a
*cache entry* be shared across trust boundaries," not "may this *name* be referenced." No
convergence is proposed; flagging the near-miss so a future reader doesn't try to unify them.

## 2. End shape

**Reuse, verbatim, from `std/effect_grant.dag`:** `NamespacePosition { tree, path }` (in
particular `tree: CodeNameTree`, already enumerated and unused until now — the code-name
containment tree namespace-resolution-design makes the single naming authority),
`path_is_prefix` / `position_under`, and the `Grant { verb, root, ... }` /
`admit_effect_go`-style fold. **Added:** one `Verb` variant.

```
type Verb = Read | Write | Execute | Reference
```

`Reference` is grounded the same way `Execute` was (`verb_of_effect_shape`): it names the act of
forming a projection edge, not a REST-derived CRUD shape — a peer coarsening, not a fork.

**Directionality is inverted from effect grants, deliberately — the dual reading.** An effect
grant answers "what may this *frame* reach *out to*" (root = target the actor may touch). A
visibility grant answers "what may reach *in to* this *node*" (root = the subtree of referrers
admitted to hold a `Reference` edge to it). This is the same relation asked backwards — exactly
the pattern DESIGN §4 already names for `emit`/`ingest` ("one grammar, read in both directions ...
the structural inverse, not a second emitter") and for `coercion_fold` ("one procedure asked in
three directions, not three procedures"). Visibility is grant admission's fourth direction, not
a fourth mechanism.

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
  not yet migrated resolves exactly as it does now, from anywhere, unconditionally. This is the
  same shape as the self-host frontier (DESIGN §7: `SelfEmitted | SeedRetained`, "a declared row
  with a reason and a migration trigger — countable, prioritizable — never a silent escape
  hatch") and `module-identity-storage-binding-design.md`'s per-surface authority rows: a typed,
  counted frontier state, not a default grant. Nothing is *fail-open by omission* — the omission
  itself is a named, migratable state.
- **`Declared { grants }`** is fail-closed exactly like `admit_effect_go`: once a node opts in,
  a referrer outside every declared grant root is refused. **One implicit exception, needing no
  grant row:** a referrer whose position is under the node's own immediate declaring container is
  always admitted — this is not a grant, it is namespace-resolution-design's existing rule 2
  ("a sibling module is visible ... that sibling's members are not bare — you project into it"
  read at the declaring container's own grain: the lexical ancestor-chain walk that already
  exists is unaffected by this doc). `Declared { grants: [] }` is therefore genuine, strict
  privacy: visible only within the declaring container, nowhere else.
- **Public / private / friend fall out as grant-root presets, not new types** — the same move
  `effect-namespace-grants.md` §2 uses for `Hermetic | Wet | Record` ("named envelope presets,
  kept during migration, deleted at the end"):
  - *Private* = `Declared { grants: [] }` (implicit container-only admission).
  - *Public* = `Declared { grants: [Grant { verb: Reference, root: <CodeNameTree root>, ... }] }`.
  - *Friend / package-private / `pub(crate)`-shaped* = `Declared { grants: [Grant { verb:
    Reference, root: <some ancestor subtree narrower than the tree root> }] }` — Rust's binary
    `pub`/not-`pub` and its `pub(in path)` refinement are **both** the same mechanism at
    different root depths, not two features. A node may also hold more than one grant (e.g.
    visible to two named subtrees without being fully public) — the existing `List<Grant>` fold
    already expresses a union of admitted roots with no new plumbing.

```
fn visible_from(nv: NodeVisibility, referrer: NamespacePosition) -> Bool {
  match nv.status {
    LegacyOpen => true
    Declared { grants: grants } =>
      position_under(target: referrer, root: declaring_container(nv.node))
        || admit_effect_go(grants: grants, verb: Reference, target: referrer, frame: <unused>)
             is Admitted
  }
}
```

(sketched for shape, not for landing verbatim — `admit_effect_go`'s `frame` parameter is
carried only for the refusal payload today; a visibility-flavored fold would drop it or the
signature would be factored to share the walk without forcing an unused `Frame`. That factoring
is a P-B question, not a modeling one.)

## 3. Where the check fires

Namespace-resolution-design §3 already names the one seam that constructs a reference edge:
**`.` projection** ("field, module member, variant: the same operation, descend into a named
child"). The lexical ancestor-chain walk (bare-name lookup) never leaves the referrer's own
containment lineage, so it never needs this check — by construction, everything a bare name can
reach is already within the implicit container-visible set. The check applies exactly where
`namespace-resolution-design.md` says qualification is required to leave that lineage: a `.`
projection whose target subtree is not on the referrer's ancestor chain. One seam, matching that
design's own claim that resolution has "no knobs to tune" — this doc adds one predicate at the
existing seam, not a new pass.

`.` is also, per that doc, the `std.induction` sub-value relation (structural descent evidence).
Visibility is a different question about the same edge (may it be formed, vs. does it prove
termination) — the two stay separate consumers of one edge, not a merge, matching the "one
structure, N consumers" framing effect-namespace-grants.md already applies to the containment
tree (resolution, content-addressing, termination, effects — visibility becomes a fifth).

## 4. Convergence map (§2/§3 — DFS'd before minting)

| element | existing carrier | relationship |
|---|---|---|
| position / subtree / `⊑` | `std.effect_grant.NamespacePosition`, `position_under` (`CodeNameTree` variant, already enumerated, unused until now) | reuse verbatim |
| grant / admission fold | `std.effect_grant.Grant`, `admit_effect_go` | reuse the algebra; new `Verb::Reference`; direction inverted (documented, not hidden) |
| the edge being gated | `.` projection (namespace-resolution-design §3.3) | reuse — the one seam that constructs a reference edge |
| undeclared/migrating state | self-host frontier (DESIGN §7: `SelfEmitted \| SeedRetained`) | pattern reuse for `LegacyOpen \| Declared` |
| effect verbs on the resolved value | `std.effect_grant.Verb = Read \| Write \| Execute` | orthogonal, composes after resolution, not touched |
| cache-entry sharing scope | `std.cache_interface.VisibilityScope` | false friend — different question, no convergence, flagged so it isn't minted twice under one name |
| "below-boundary representation is opaque" | DESIGN §3, stated rule, currently unenforced | this doc is the enforcement mechanism named against that gap |

## 5. Phases

Mirrors `effect-namespace-grants.md`'s own phase discipline — each green-by-execution with REDs,
under-scope counted and never silent.

- **P-A (model, no behavior change):** `Verb::Reference` added to `std/effect_grant.dag`;
  `NodeVisibility` / `VisibilityStatus` modeled with witnesses over synthetic trees, including
  REDs (referrer outside every grant → typed refusal; container-internal referrer → admitted
  with no grant needed; `LegacyOpen` → always admitted). No resolver wiring yet.
- **P-B (first enforcement seam):** the `.` projection path in the resolver (namespace-resolution
  lane's `SymbolIndex` walk) calls `visible_from` when the projected child's position leaves the
  referrer's ancestor chain; refusal is a typed, located `NameNotVisible` alongside the existing
  `Ambiguous` class namespace-resolution-design already defines. Every currently-resolvable name
  stays resolvable — every node is `LegacyOpen` until an author explicitly declares one — so P-B
  is a zero-behavior-change landing by construction, not by discipline.
- **P-C (surface syntax, later, gated on P-B and on the namespace-resolution terminal step):**
  a keyword or header row that toggles a node from `LegacyOpen` to `Declared`. Per
  namespace-resolution-design §11's own steer ("prefer keyword-free bare containers ... nothing
  ever needs renaming"), this is a grammar-row question for whoever owns that surface at the
  time, not decided here. `namespace-resolution-design.md` §11's "interface/export boundary —
  absent" line updates to point at this doc once P-B lands.
- **P-D (migrate the two named displaced costs from §0):** `extdeps/` internal helpers and the
  `_go`/`_rec` accumulator family get `Declared { grants: [] }` rows, each a per-site opt-in
  priced by an actual rename or a caught unintended dependency — never a bulk sweep (§6: priced
  by displaced cost, not swept speculatively).

## 6. Open questions (operator)

1. **Verb name** — `Reference` (this doc's pick, paired with the existing action-verb shape of
   `Read`/`Write`/`Execute`) vs. `Visible` (adjectival, reads better at call sites like
   `Grant { verb: Visible, ... }` but breaks the verb-shape pattern). No behavior difference.
2. **Directionality sketch** — confirm the inverted reading (grant lives on the declared node,
   root = admitted referrer subtree) rather than forcing visibility onto the existing
   `Frame`/`Envelope` pair. The alternative was rejected in §2: `FrameKind` is closed over four
   materialization-scoped variants (`SharedStateFrame`/`IsolatedChildrenFrame`/`ReplayedFrame`/
   `UnboundedSiblingsFrame`) with no compile-time-scope member, and forcing a fifth would be the
   state-space conflation DESIGN §3/§5 name as the recurring failure — reusing `Grant`/`Verb`/
   `NamespacePosition` is the actual "not a parallel mechanism" commitment; reusing `Frame` for a
   shape it doesn't fit would not be.
3. **`LegacyOpen` dissolve-on** — left open-ended (per-node opt-in only, §5 P-D) rather than a
   scheduled bulk migration. Confirm that's the intended shape, vs. an operator-set target date
   for corpus-wide default-private (a much larger, differently-priced move this doc does not
   attempt to justify).
