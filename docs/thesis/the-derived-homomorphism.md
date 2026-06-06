# The derived homomorphism — model local, derive global

A `gunbc` thesis derivation. Parent: [THESIS.md](../../THESIS.md) → "The derived homomorphism".

## The compiler's job, stated once

Every target — every language, format, service, persistence layer — is
**modeled once, in one shared substrate** (the connectives, `Nat`, `Bool`,
the algebra). A target model encodes that target's facts: its carriers,
conventions, arity, and operation semantics (overflow disposition, NaN/Inf
handling, …) — as far as the modeler knows them.

Given that, **translation between any two targets is a homomorphism the
compiler *derives* — never an adapter anyone *authors*.** Because both
targets ground into the same substrate, the compiler finds the
structure-preserving map by comparing groundings and composing the
destination's modeled primitives. Rust's wrapping `+` becomes the Python
composition `((x + 2^31) mod 2^32) - 2^31` — *derived*, because `Wrap`
decomposes into substrate primitives and Python's `+`/`mod` are modeled.

## Three consequences — this is the theme

**Integration is local, not global.** You model *your own* target. You never
write — never even see — the N×M cross-target adapters. The Rust modeler
knows Rust; the SQL modeler knows SQL; neither needs the Rust↔SQL
translation. You author N target-models; the compiler derives the N×M
homomorphisms. Integration cost collapses N×M → N+M.

**Coercion verifies; the homomorphism is the operation; the unfold is the
fan-out.** Coercion is the mechanical check that a candidate map preserves
structure. The homomorphism is the whole
structure-preserving translation. Unfolding one intent across N targets is
omni-emission.

**Gaps become visible, never silent.** Where no structure-preserving map
exists — a target genuinely cannot express a behavior — the compiler
fail-closes with an explicit, located `Diagnostic`. A translation gap
converts from a silent production bug into a surfaced compile-time fact.
There is never an unfaithful silent emit.

## The bet this rests on

All of the above is mechanical *given correct, complete target models*. The
compiler owns the homomorphism; the modeler owns the model. The thesis
therefore reduces to one claim: **target modeling is doable well.** That is a
defensible bet — modeling is:

- **separable** — each modeler needs only their own target;
- **bounded** — model the target's versioned spec (finite), not its library
  universe (the L-2 spec-first discipline);
- **checkable** — the fact-bundle discipline and the structural fact-density
  gate make "modeled honestly" enforceable.

A wrong model yields a confidently-wrong homomorphism. The modeling
discipline ([MODELING.md](../../MODELING.md)) is therefore not
peripheral — it is what the thesis stands on.

## Why this makes the rest of the design make sense

Fact-bundle modeling, the no-prose / no-templating rules, the fact-density
gate, the L-2 spec-first discipline — every modeling rule exists in service
of one thing: making each target's model correct, complete, and honest
enough that the derived homomorphism is sound. A reviewer reading any
modeling rule should read it as: *this protects the homomorphism.*
