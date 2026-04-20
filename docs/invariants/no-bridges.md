### No bridges

A **bridge** is an adapter function, helper module, or translation
layer that exists purely to convert one representation of a fact
into another representation of the same fact. Bridges are introduced
when a refactor lands the *new* representation but can't yet touch
every *old* consumer, so an adapter is added "temporarily" to keep
the old consumers working while the rest of the migration is
tracked as follow-up work.

**Bridges are forbidden. Do not introduce them, no matter how
well-tracked. And if you find one already in the codebase — even
in a file you were passing through — do not silently route
around it. Raise it as an alarm signal per §"No short-term
solutions."**

The refactoring cost that would make the bridge "temporary" is
exactly the refactoring cost that the bridge is supposed to defer.
The bridge doesn't reduce the cost — it just rewrites it as
"someone else's later problem," and tracked bridges calcify because
every downstream consumer learns the adapter shape, not the new
representation. By the time the dissolution trigger fires, removing
the bridge means reworking every consumer AND every consumer
downstream of those consumers that inherited the adapter's
assumptions. The debt compounds.

**Historical example (2026-04-14):** v3's `declaration_to_type_shape`
was introduced as a "localized" adapter from `DeclarationId` to
`TypeShape::Primitive(Prim)` because the substrate rework landed
a rich declaration table but didn't refactor port-level `TypeShape`.
The function matched declaration names against a hardcoded string
list (`"Int" | "Int64" | "Word64" | "Word32" | ...` → `Prim::Int`)
and was tracked as "scope-bound, dissolves in M2." It violated
three invariants at once: no duplicate representations (DeclarationId
+ Prim for the same type identity), no name-based dispatch (string
match on declaration names), and facts flow forward (the rich
declaration identity was collapsed to a coarse tag at the boundary).
Had it survived, every M1(3)+ consumer of `TypeShape` would have
learned the Prim-tagged shape instead of the declaration-carrying
shape, and the M2 rework would have had to edit every consumer
plus the adapter plus any new consumers that appeared in between.

**The test:** does the change introduce a function whose purpose is
to translate between two representations of the same fact? Signs
to look for:

- Function name or docstring matches `*_to_*`, `convert_*`,
  `adapt_*`, `bridge_*`, `as_*`.
- Body does a match on names, indices, or tags that came from one
  representation and produces a corresponding value in another.
- Comment says "localized," "scope-bound," "dissolves in M2+," or
  "the last bridge."
- Caller code "just needs" the adapter to unblock work in one area
  without touching another.

If any of these apply, the change is introducing a bridge. Stop.

**The rule:** the representation change and every consumer update
must land in the same PR. If that PR is too large, split the
representation change into a smaller one that doesn't require
adapters — but do not split it into "new representation now, rework
later." The only acceptable split is the one that keeps every
consumer consistent at every commit boundary.

**The fix when you've already written one:** back out the adapter
and the representation change together. Rework the representation
change into something that every consumer can adopt in one push.
Do not merge an adapter and track its removal — track the smaller
representation change instead.

**Structural prevention (future):** a CI audit on every M1+ PR
grep-matches function signatures and docstrings for the adapter
pattern above and fails the build if new matches appear. Until
that audit exists, this invariant is enforced by code review —
any PR reviewer can veto an adapter with a reference to this
section.

**Exception:** there is exactly one boundary where an adapter is
unavoidable: emission into a target language. The emitter converts
Node trees into target source code via a language spec — that's
`coercion = emission`, and the "conversion" is the whole point of
the emitter. But the emitter is not a bridge under this invariant
because (a) the output is in a different target world entirely, not
in the compiler's own representation, and (b) it is driven by a
declared language spec, not a compiler-internal adapter function.
Test: if the adapter's output is consumed by another part of the
compiler (not by a target world), it is a bridge and is forbidden.
If the output is target source code produced via a declared
language spec, it is emission and is allowed.

