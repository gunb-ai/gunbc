# Dissolution Findings — a review-finding family

> **Status: proposal — for still-hawk-102 to audit, verify, and decide
> integration.** This doc names a family of review findings so reviewers
> have the discriminants and the *confidence* to recommend them. It is a
> companion to [modeling-discipline.md](modeling-discipline.md); still-hawk
> owns the decision of whether these fold into that keystone as named
> Practices or stay a referenced companion.

## Why this exists

The project thesis is **the derived homomorphism** ([THESIS.md](../THESIS.md),
[the-derived-homomorphism.md](thesis/the-derived-homomorphism.md)): a
correct model lets the compiler *derive* the structure-preserving map
between targets. A **hand-rolled walker** — a function that walks a
structure by hand — is an *un-derived homomorphism*: the same map,
re-implemented per stage instead of derived once from the model.

Reviewers were missing these. A hand-rolled traversal *looks* fine —
"that's just how you write it" — so reviewers had no license to flag it.
This doc supplies the license: a **name**, a **recognition test**, a
**confidence test**, and a **recommendation** for each finding.

The umbrella: a *dissolution finding* names a hand-rolled construct that
**re-derives what the model should provide**. The fix always has the
same shape — the construct *dissolves* into **(a substrate primitive) +
(model data)**.

## The family

| # | finding | smell | dissolves into |
|---|---|---|---|
| 1 | **coproduct dissolution** *(existing)* | `type X = A \| B` enumerating cases the model should answer with facts | facts |
| 2 | **walker dissolution** | a function that hand-rolls recursion over a structural type | std catamorphism (`fold_node`) + a supplied algebra |
| 3 | **traverse dissolution** | a `match acc { Rejected => … }` short-circuit ladder inside a fold | `traverse` / `sequence` over the effect carrier |
| 4 | **predicate dissolution** | a `match`/`if` on kind or symbol that *derives a property* | a fact / field carried by the model |
| 5 | **carrier dissolution** | a local coproduct that re-implements an existing std carrier | the std carrier (e.g. `Outcome<T>`) |

Coproduct dissolution is already an established finding (per-coproduct
`// 🟢/🟡/🔴 coproduct dissolution` tag + DECISIONS.md ledger). Findings
2–5 are proposed here.

## The findings

Each finding has the same rubric shape: **Recognize** (what to look for),
**Confidence test** (when to recommend it, with confidence), **Not when**
(false positives), **Recommendation** (what the reviewer writes).

A recommendation may resolve to one of two outcomes — this is the part
that gives reviewers confidence, because they are never wrongly demanding
the impossible:

- **fix-now** — the substrate the construct should use *already exists*;
  the fix is mechanical.
- **substrate-sequencing finding** — the substrate primitive does *not*
  exist yet. The finding is then "name the missing primitive"; the
  hand-rolled construct is *not* accepted as the end state, but the fix
  is upstream, not in this PR.

### 2. Walker dissolution

**Recognize.** Any of: a function that recurses over a structural type
(`Node`, `ParseTree`, AST) by destructuring children and calling itself;
a per-node-kind `match` whose arms each re-walk children; a hand-written
`fold(children, acc, …)` re-implementing a fold; `gather`/`harvest`/
`collect` helpers that traverse to pull out sub-structure.

**Confidence test.** It *is* walker dissolution when all hold: (1) the
recursion's shape is fixed by the *structure being walked*, not by logic
unique to this call site — the per-kind behavior could be supplied as an
algebra; (2) the behavior that varies per node-kind is a *fact about the
language/structure* currently encoded as `match` arms; (3) bonus signal:
two+ stages hand-roll the *same* traversal shape.

**Not when** the recursion is genuinely irregular — the call graph is not
the data graph.

**Recommendation.** "This hand-rolls a structural homomorphism. Express
it as: read the [structural fact] → apply a derived homomorphism (std
catamorphism + supplied algebra). If that catamorphism does not yet exist
in `std/`, this is a substrate-sequencing finding — name the missing
primitive; do not accept the hand-rolled walker as the end state."

### 3. Traverse dissolution

**Recognize.** A `fold` whose body is a `match acc { Rejected => …
propagate … ; Ok => … continue … }` ladder — effect-threading (failure,
short-circuit, accumulation) inlined by hand.

**Confidence test.** The fold threads an *effect* (an `Outcome`-shaped
result, a short-circuit) that is uniform across call sites — i.e. it is
`traverse`/`sequence` for some carrier.

**Not when** the per-element step has genuinely position-dependent logic
that is not expressible as a uniform effect (rare; usually still a
`traverse` with a richer algebra).

**Recommendation.** "This hand-inlines `traverse` over [carrier]. Replace
with `traverse`/`sequence`. If a fail-closed `traverse` does not exist for
this carrier in `std/`, name the missing primitive." Note: traverse
dissolution almost always co-occurs with walker dissolution — the fold
body both recurses *and* threads the effect.

### 4. Predicate dissolution

**Recognize.** A `match`/`if` on a node-kind, connective, or symbol whose
purpose is to *answer a question* — "is this a binder?", "which sugar is
this?", "what are the canonical symbols?" — rather than to do
structurally distinct work.

**Confidence test.** The question is a *fact about the language or
structure* that the model could carry as data (a field, a `Map`, a rule
table). The `match` enumerates cases the model already knows.

**Not when** the branches do genuinely distinct work that is not reducible
to reading a value.

**Recommendation.** "This `match` derives [property] by hand. [property]
is a fact — carry it on [the model] and *read* it. If the model has no
slot for it, this is a substrate-sequencing finding — name the missing
fact."

### 5. Carrier dissolution

**Recognize.** A locally-declared coproduct whose shape mirrors an
existing std carrier — most often a `Foo { value } | FooRejected
{ diagnostic }` that is `Outcome<T>` under another name. Also: accumulator
types that exist only to thread a flag/counter through a hand-rolled fold.

**Confidence test.** A std carrier with the same shape already exists, and
the local type is converted *to* that carrier at the function boundary
anyway — the local type is pure friction.

**Not when** the local type carries a genuinely distinct payload the std
carrier cannot express (verify by checking the std carrier's parameters).

**Recommendation.** "This re-implements [std carrier]. Delete it and use
[std carrier] directly." Carrier dissolution is *almost always* fix-now —
the std carrier, by definition, already exists.

> Carrier dissolution is a sharpened sub-case of coproduct dissolution: a
> coproduct flagged for dissolution should first be checked against the
> std carrier set — if it matches one, the finding is the sharper, more
> mechanical "carrier dissolution", not generic "model this as facts".

## Worked examples — PR #3225 (CP-1b normalize/resolve)

The three files of #3225 (`compiler/03_normalize.dag`,
`compiler/03_resolve.dag`, `extdeps/languages/dag.dag`) are a near-perfect
teaching corpus. Line numbers are as of #3225's branch
(`session/lane-a-3211`); reference functions by name once it merges.

**Walker dissolution** — hand-rolled `Node` catamorphism:
- `resolve.dag` `resolve_node` (the canonical one: `match n.kind` →
  destructure `.children` → self-call), `resolve_children_homogeneous_scope`,
  `resolve_edges_first_outer_then_inner`, `resolve_bind_edges`
- `resolve.dag` `add_module_named_exports`, `add_arrow_domain_named_params`,
  `add_bind_atom_binder` — "harvester" walkers
- `normalize.dag` `normalize_edge` / `normalize_children` / `normalize_node`
  — a mutually-recursive hand-rolled `Node` catamorphism

**Traverse dissolution** — hand-inlined `traverse` (co-occurs with every
walker above): `normalize_children`, `resolve_children_homogeneous_scope`,
`resolve_edges_first_outer_then_inner`, `resolve_bind_edges` — each fold
body carries a `match acc { Rejected => propagate ; Ok => continue }`
ladder.

**Predicate dissolution** — facts hidden in a `match`:
- `normalize.dag` `classify_sugar` — an `if id == sugar_service … else if
  …` ladder; should be a `Map<Symbol, SurfaceSugarKind>` carried by the
  `DagLanguageModel` (which already enumerates the four sugar symbols).
- `resolve.dag` `add_arrow_domain_named_params` / `add_bind_atom_binder`
  and `resolve_node`'s `match n.kind { Arrow => … ; Bind => … }` — they
  `match` on `Arrow`/`Bind` to decide which child slots introduce binders
  and which see them. That is a **binding/scoping fact** the
  `LanguageModel` should carry.
- `dag.dag` `dag_language_model_canonical_symbols` — reverse-engineers the
  canonical-symbol set from node *shape* (`if is_wave1_void_shape …`); the
  model should *carry* its canonical symbols as a fact.

**Carrier dissolution** — the three types currently tagged "coproduct
dissolution" are sharper than that:
- `resolve.dag` `ResolveResult` **is** `Outcome<Node>`;
  `ResolveChildrenResult` and `normalize.dag` `NormalizeChildrenResult`
  **are** `Outcome<List<Edge>>`. They are not arbitrary sums — they are
  clones of the landed std carrier (`std/diagnostic.dag` `Outcome<T>`),
  and each is converted to `Outcome` at the function boundary anyway.
- `resolve.dag` `EdgeResolveAcc` / `BindEdgeAcc` exist *only* to thread a
  flag/counter through a hand-rolled fold; under `traverse` + `fold_node`
  they vanish.

## What the marking revealed

Marking #3225 collapsed the findings to **four fixes**, and the fix-now /
substrate-gap discriminant sorted itself:

| fix | status | location |
|---|---|---|
| use `Outcome<T>` | **exists** — carrier dissolution is fix-now | `std/diagnostic.dag` |
| `traverse` / fail-closed sequencing over `List` | substrate gap | `std/collection.dag` |
| `fold_node` catamorphism over `Node` | substrate gap | `std/node.dag` |
| a binding/scoping fact on the `LanguageModel` | substrate gap | `extdeps/languages/dag.dag` |

This validates the rubric on a real PR: carrier dissolution is a "fix-now"
finding; the other three are substrate-sequencing findings. The drift in
#3225 is **upstream sequencing** — the fact substrate landed after the
stages that need it — not worker error. #3225 is correct work given the
substrate available; it should land as an honest scaffold, and the
substrate-first followup migrates it.

## For the audit

Requested of still-hawk-102:

1. **Verify the family** — are findings 2–5 correctly distinguished? Is
   the recognize/confidence/recommend rubric sound?
2. **Look for other patterns** — does the `dissolution finding` umbrella
   have members this doc missed?
3. **Integration decision** — fold findings 2–5 into
   `modeling-discipline.md` as named Practices, or keep this as a
   referenced companion? (still-hawk owns `modeling-discipline.md`.)
4. **Retroactive v4 audit** — sweep landed and in-flight v4 work
   (`src/v4/compiler/*`, `src/v4/extdeps/**`, `src/v4/std/**`) for
   dissolution findings, and decide what — if anything — needs
   retroactive correction versus what is correctly deferred to the
   substrate-first sequence.
