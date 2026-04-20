### Two groundings: static validation vs efficient realization

Every concept in gunbc has **two groundings** that must both be
present, in the thesis's end-state, and must be consistent with
each other. These are different things — they have different
shapes, different purposes, and different completeness
requirements. Earlier drafts of the thesis blurred them; this
section pins the distinction down.

**Milestone scope note.** The thesis's top-level claim ("if it
compiles, the intent is sound and will execute as declared")
applies when both groundings are in place. **At M1(2.5), only
the static grounding is validated.** Realization grounding — the
target-language mapping that makes concepts emittable — lands at
M1(3)+ as language-spec declarations in
`dsl/extdeps/languages/`. During the M1(2.5) → M3 transition,
some primitive Arrows carry a scaffolded `Pending` realization
state: their concept decomposes completely via inhabitance, but
their target-language realization is declared later. A CI
ratchet (tracked in `INVARIANTS.md` §"No short-term solutions"
exception 2) requires that the Pending count monotonically
decreases and every Pending Arrow resolves to `UserDefined` or
`ExternalRealization` before M3 completes. The top-level executability claim is
therefore **milestone-conditional**: fully satisfied once M3's
realization declarations land, not before.

**Grounding #1: static grounding (`.dag` level).** Every concept
walks all the way down to the root primitives (Classical Bit +
Conj + Disj + Atom) via structural composition. The chain is
complete, deep, and bounded — every step is a substrate edge,
and every walk terminates at the roots. There are no escape
hatches, no opaque intrinsics, no concepts whose decomposition
is unfinished. This is the grounding §"Epistemic stacking"
insists on and the grounding §"The substrate"'s load-bearing
test validates.

The purpose of static grounding is **correctness proof**. It
answers questions like: Is `Int.add` a valid concept? Does it
type-check? Does it terminate? Does adding `Int256` work without
touching the compiler? Every answer comes from walking the
decomposition chain; nothing depends on the target world. Static
grounding is pure, closed-system, compile-time-only.

**Grounding #2: realization grounding (target level).** When the
compiler emits to a target language, every concept maps to the
nearest efficient target primitive via a language spec
declaration. This grounding is **shallow and target-specific**.
It is not a decomposition — it is a projection onto the target's
native capabilities. Concrete examples:

- `.dag Int64` realizes as Rust `i64`, NOT as a struct of 64
  individual Bit nodes. The compiler does not emit carry-
  propagation loops to add two integers — it emits `a + b` using
  the target's native machine instruction.
- `.dag Bool` realizes as Rust `bool`, not as a two-variant enum
  that dispatches through match arms for every operation.
- `.dag String` realizes as Rust `String`, not as a `FreeMonoid<
  Char>` traversal.
- `.dag List<T>` realizes as Rust `Vec<T>`, not as a cons-cell
  chain.

The language spec (`dsl/extdeps/languages/rust.dag` and friends)
is the **mapping from `.dag` concepts to efficient target
primitives**. The emitter uses this mapping directly; it does
not walk down the decomposition chain and reconstruct. The
target's native primitives are the realization boundary — the
point where the compiler hands off to the target world.

**The two groundings have different shapes and different
completeness rules:**

| Property | Static grounding | Realization grounding |
|---|---|---|
| Depth | Deep (walks to Classical + Conj + Disj) | Shallow (one hop to target primitive) |
| Purpose | Correctness proof | Efficient execution |
| Completeness | Must be total | Must exist for every concept the target emits |
| Time | Compile-time, closed-system | Compile-time declaration, target-time execution |
| Substrate impact | None (concept decomposition) | Language spec declarations only |
| Consistency requirement | Type-preserving with declared laws | Semantically equivalent to static chain (L4 verifies) |

**The two groundings must be consistent by construction + verified
by L4.** The language spec's realization claim is structurally
consistent with the `.dag` declaration's algebraic laws: Rust's
`i64` satisfies the OrderedRing axioms modulo two's-complement
overflow (which is a handled special case with explicit bounds
on safe inputs). L4 verification (see §"Tier 3: Verification
from structure") tests this by generating witness values,
evaluating them at the `.dag` level via the deep chain, emitting
them via the shallow chain to the target, and comparing results.
When L4 says a concept's static and realization groundings agree,
the language spec's realization claim is certified.

**Why this distinction resolves the "ungrounded concept"
question.** Earlier drafts of §"Epistemic stacking" declared that
every opaque intrinsic, runtime-native helper, or bolt-on
analyzer is "unfinished composition." That reading is correct for
the static grounding — at the `.dag` level, every concept must
decompose completely. But it's wrong if you apply it to
realization: `.dag Int.add` at M1(2.5) has its realization
binding in a **`Pending`** state (one of the four `ArrowBody`
variants defined in `src/v3/compiler/src/dag.rs`) because its
target-world mapping lives in `dsl/extdeps/languages/rust.dag`
— which will be declared as an ordinary Declaration reachable
via a typed edge, not a name-based lookup.

> **Bootstrap staging note.** During the v3 M1(2.5)–M1(3)
> bootstrap window, v3's Rust compiler reads its realization
> fixture from `src/v3/spec/rust.dag` as a temporary staging
> location (the shipped file at that path is what the bootstrap
> currently loads). The **canonical home** for Shape A language
> specs is `dsl/extdeps/languages/` per §"Targets are
> declarations" below; the `src/v3/spec/` location is a
> bootstrap-local artifact, not a second authority. v3's
> migration of the realization fixture to its canonical
> extdeps home is tracked as class-5 follow-up work. Everywhere
> in this thesis, "the language spec lives in
> `dsl/extdeps/languages/`" is the conceptual claim, and the
> current-implementation path is an implementation detail of
> the bootstrap. That is NOT an ungrounded concept; it is
a **concept whose static grounding is complete and whose
realization grounding edge is pending declaration loading.**
The stop signal applies to static grounding (no concept without
a compositional chain), not to realization grounding (which is
deliberately target-specific and shallow by design, but still
structural when it lands).

**For M1(2.5):** primitive Arrows in `dsl/std/algebra.dag` carry
`body: Pending` — a scaffolded bootstrap state. Their static
grounding walks through inhabitance (e.g., `Int.add` walks
through `Instantiation(OrderedRing, [T := Word64])`); their
realization grounding will be declared in the Rust language spec
when that work lands, after which `Pending` resolves to either
`UserDefined(NodeId)` or `ExternalRealization(DeclarationId)`
— both structural edges. A CI ratchet enforces that no `Pending`
remains once the M3 extdeps work completes. The C-checkpoint for
"ungrounded concept" fires only when a concept cannot be walked
through its static chain — which `Int.add` can, via
`OrderedRing<Word64>`.

