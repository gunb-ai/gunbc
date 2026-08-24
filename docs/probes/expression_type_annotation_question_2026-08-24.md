# Should `.dag` have an expression-position type annotation? Three instances, and a test that says no

*2026-08-24. An open design question, not a decision. Instances from smart-wolf-868, witty-badger-734
and bold-pike-475; the deciding test is deep-ant-102's. Recorded because it was assembled across
several message threads and would otherwise be lost.*

## The strongest single fact, stated first

`v1.compiler.infer`'s `ExprListLit` arm answers `unit_type` — with **no diagnostic** — for an empty
list literal with no expected type. *"I could not determine the element type"* rendered as *"the
element type is unit"*, which is §5's fabricated plausible output.

It fabricates for **24 sites in one closure alone** (gunbc#9099, affected-set closure; a lower bound,
not a corpus count). One lane hit it, measured it, and declined to route around it.

**Had an expression-position type annotation existed, that lane writes `: List<Foo>`, moves on, and
the twenty-four stay fabricated.** That is what the absence bought.

## Three instances, three distinct causes

They share a symptom — *a type is absent at the position that needed it* — and nothing else. An
earlier draft collapsed them into one gap; that was over-synthesis and is retracted here, because the
shared symptom is exactly what makes one annotation look like one fix for three problems.

| | cause | repair taken |
|---|---|---|
| gunbc#9075 | **surface syntax** — `.dag` lambda parameters have no type slot: `collect_fn_lambda_params` builds `ParserParam { name, span }`, and a corpus search for a typed lambda parameter returns zero | named helper `fn(T) -> P`, called from an untyped lambda |
| gunbc#9101 | **emitter fork** — dropping `.cloned()` for an unused element changes the item type `T` → `&T`; the closure's parameter keeps its by-value annotation because the second derivation is never told | emit `_`, let inference supply it |
| gunbc#9099 | **inference ordering** — the element type *is* determinable from the outer fold's accumulator; the callee's type variable is unsolved when the literal is judged | none; refusing locally costs 12 blocking rows to fix 1, documented instead |

Nothing is missing from the language in the second or the third.

## The test: does the annotation ADD information, or RESTATE it?

*deep-ant-102's, and it decides this more cleanly than the escape-hatch framing it replaces.*

- **#9075 — ADDS.** The type is genuinely absent from the program. `ParserParam` carries a name and a
  span; there is nowhere else the type lives. A **missing capability**.
- **#9101 — RESTATES, and restates it wrong.** The type is determined by the iterator. The annotation
  is a second copy, and at the moment the emitter fork makes the two disagree it is the *stale* copy.
- **#9099 — RESTATES.** The type is determined by the outer accumulator. It merely arrives late.

## Why that is a §5 objection rather than a preference

§5: *a check that re-states a constraint the model already carries is a second representation of it,
so prefer a single authority from which the realization is derived over a check that flags it after
the fact.*

**Substitute "annotation" for "check" and the paragraph reads unchanged.** A general expression
annotation is not merely a hatch that happens to conceal defects — it is structurally a parallel
representation of a fact the type system already owns, and the concealment is the *consequence*
rather than the objection. That distinction matters: a consequence invites cost-benefit haggling,
a second representation does not.

The concealment is still worth stating because it is measured: with the hatch available, #9101's
emitter fork survives papered over at one call site, and #9099's 24 fabrications are never found.
Two of these three defects were found **only because there was no way to route around them.**

## The boundary this yields, and it is principled rather than convenient

> **Declaration positions declare. Expression positions are derived.**

A lambda parameter is a *declaration* — the one place a binding enters scope with no other source for
its type, and every other declaration in the language carries one. Its absence is a §3 inconsistency,
not a discipline.

An expression already sits in a context that determines it; annotating there is convention standing
where necessity was available — §1's reduce-convention-to-necessity, read at the grammar layer.

Stating the boundary this way matters: without a principle, "add it for lambda parameters only" reads
as carving out the case that inconvenienced us.

## The two questions, and what is NOT established

1. Does `.dag` get expression-position type annotations at all, or is their absence load-bearing?
2. If the answer to (1) is no, is the lambda-parameter slot separable — a missing capability rather
   than a hatch?

**Nobody involved knows whether the omission was designed**, and this document does not claim it was.
It is *consistent with* the design either way: §4's claim that a heuristic is never necessary in a
closed system has exactly this shape, and an author-supplied annotation at a determined position is
the author acting as the heuristic. Consistent-with is not designed-for, and the question of intent
belongs to whoever owns the grammar.

**No lane is blocked by this.** All three found correct repairs. It is filed so the question is
answerable later rather than re-derived from three PR bodies.
