# Bare cross-module references: two diagnoses, not one class

Status: **census finding, nothing implemented.** Written because folding these
into a single "needs qualification" class hides that one of them is a silent
wrongness rather than a resolution failure.

The import cut exposed bare cross-module references that previously resolved by
pool-membership coincidence. They do not all have the same shape or the same
remedy.

---

## Diagnosis A — genuine homonym (the `Empty` class)

A name declared by a small number of modules, each meaning a **different
thing**. The author intended exactly one.

    Empty -> std.algebra (FreeMonoid constructor)
             std.stack   (Stack constructor)

Measured population: 151 sites, reported by the floor cut's whole-corpus strict
fold. Two candidates, neither on the referencing module's containment chain.

**Resolution verdict:** ambiguous, must refuse. **Remedy:** qualify at the use
site (`std.algebra.Empty`). A qualified reference IS the dependency edge, so
this is the intended end state rather than a workaround.

## Diagnosis B — per-module convention (the `extdeps_external_authority_anchor` class)

A name every module in a family declares **for itself**, as a convention. There
is no single intended referent to qualify toward, because each declarer's value
is different by design.

    extdeps_external_authority_anchor -> 551 declaring modules
    live_tree_disposition             -> 1097 declaring modules

These are not homonyms in the `Empty` sense. A bare reference to such a name
from outside the declaring module is not merely ambiguous, it is **meaningless
by construction**: the convention makes 551 candidates equally valid.

Measured: of 130 modules referencing the anchor, **128 also declare it**, so
their reference binds locally and is correct. Exactly **two** reference it
without declaring it:

- `dag/test/claim/claim_indexed_evidence_witness_test.dag`
- `dag/product/altra_motherboard/attachment_stack.dag`

### Why this one is worse than ambiguity

Each module's anchor holds a **different citation**. So a bare reference that
binds to an arbitrary declarer does not fail loudly — it yields a well-typed
`ExternalAuthority` value carrying the **wrong citation**. That is a plausible
wrong answer, not a refusal: the §5 fabricated-plausible-output class, sitting
in the corpus rather than in the compiler.

Under the pre-cut regime these two sites resolved by whichever anchor happened
to be in the pool, so which authority they cited was a property of the entry's
closure rather than of the source. **Not asserted here:** whether either site
currently cites a wrong authority on main. Establishing that requires resolving
each against its historical pool and comparing to intent; it has not been done.
What IS established is that the binding was not determined by the source text.

### The remedy differs from A

The intended referent is recoverable from context rather than needing a
judgement call. In the product specimen the reference sits beside a
`DeclarationRef` naming its own subject module:

    authority: extdeps_external_authority_anchor,
    subject: DeclarationRef {
      module_path: "extdeps.cpu.ampere_altra_package",

So the authority meant is that module's anchor, and the qualification is
determined, not chosen.

**Open question, deliberately not answered here:** whether 551 qualified
references are the right outcome, or whether a convention that makes a bare
reference meaningless should be examined instead — for example by making the
anchor a field of a single declared authority row rather than a name re-minted
per module. That is a modeling question about the convention, and it is not
this lane's to settle.

---

## Why the distinction is load-bearing for closure assembly

Closure width is driven entirely by diagnosis B. Measured on a five-line entry
over the whole corpus: 1376 of 3711 modules, of which one name
(`extdeps_external_authority_anchor`) accounts for 296 direct pulls because a
single bare reference pulls **every** declarer — the closure layer must not pick
a winner.

So the wide closure is not an algorithm defect. It is the closure faithfully
reporting a corpus authoring problem, and no narrowing of the reference walk can
fix it: subtracting binders and restricting to reference positions moved width
only 1503 -> 1376. Diagnosis A costs nothing in width (2 candidates); diagnosis
B costs everything (551 and 1097).
