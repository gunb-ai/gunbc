# v4 Decisions

This file is a bounded decision record for the P1-KEYSTONE fact-bundle
reseed. It is not a revived comment-ledger: inline model marks, gate
slugs, dissolution classifications, and per-carrier receipts stay on the
owning `.dag` declarations, in `src/v4/TASKS.md`, or in PR review as
required by `docs/modeling-discipline.md`.

## D2 REVERSAL + FACT-BUNDLE RESEED - RATIFIED 2026-05-17

This section supersedes the earlier D2 / D4 alias-identity shape. D2
ratified an `extdeps` primitive form where a language file could declare
`type <Lang>X = StdX` and let algebra "flow through the alias"; D4 then
homed shared resolver rows for that form. The operator reversed that
decision on 2026-05-17: a bare alias is not modeling. It asserts an
unproven identity between an external spec primitive and a `std/` carrier
while reading zero facts from the external spec.

### What Reversed

The retired shape was:

```dag
type RustI32 = Int32
```

That declaration models nothing about Rust `i32`: not width, signedness,
range, representation, overflow behavior, or spelling. It only asserts
that Rust `i32` is the same thing as the `std/` carrier. The assertion is
allowed only after proof, not before modeling.

The replacement is fact-bundle modeling:

- Model the facts first. A target primitive carries a fact-bundle made of
  the facts its own specification states.
- Express those facts in the shared `std/` vocabulary so mechanical
  comparison is possible.
- Deduplicate to a `std/` carrier only when the language bundle and
  `std/` bundle are proven to coincide.
- Never redeclare a parallel per-language algebra substrate. `INVARIANTS.md`
  P1 forbids duplicate `OrderedRing<Word*>`-style authorities; it does
  not license hollow aliases.

### Coincide

Two models coincide when their canonical `Node` groundings are
structurally equal under the B1-CANON contract:

```text
content_hash = merkle_fold o canonical
```

Coincidence is mechanical structural equality, not a free-form judgment
that two things "mean the same." The coercion fold compares the two
groundings by walking the same shared vocabulary. If the groundings use
different private vocabularies for the same fact, the fold correctly sees
different `Node` shapes; the model must be repaired, not papered over.

Proven coincidence is the only license to collapse an external primitive
onto a `std/` carrier. A bare alias asserts coincidence without proving
it, which is the D2 error.

### Reseed Plan

**Phase 0 - stop the bleeding.** Close or hold work that extends the
alias-identity form. Forward-fix existing merged language vocabulary;
do not revert broad commits when they contain surviving modeled facts.

**Phase 1 - doc keystone and review floor.** The following already-landed
documents are the P1-KEYSTONE rubric T-4 authors and reviewers must use:

- `INVARIANTS.md` P1: the hollow-alias problem shape and the integer
  worked example now state fact-bundle modeling and proven coincidence.
- `MODELING.md` M1: compositional types use named-edge `Conj` /
  fact-bundle carriers; bare aliases are under-modeled.
- `docs/modeling-discipline.md` Practice 8: reviewer-facing
  fact-bundle rule, hollow-alias discriminator, and default
  SEPARATE-for-extdeps / REUSE-for-internal policy.
- `docs/modeling/grounding-worked-examples.md`: good and bad concrete
  target forms, including the definition of coincidence.

This phase supplies the convention-tier bad example that D2 lacked. It
is not the final enforcement tier.

**T-30 - structural enforcement.** The structural fact-density /
hollow-alias gate is a hard prerequisite of T-4. Per-language
fact-bundle work must not run under convention-only enforcement; the
checker makes a hollow alias fail closed instead of relying on review
memory.

**Phase 2 - shared vocabulary.** T-4 fact-bundles need the shared axes
they ground into: signedness, representation, machine width, numeric
spine, refinement carriers, exact-real / physical-quantity carriers
where applicable, and `ModelCore` factoring. Missing vocabulary is a
feeder dependency, not permission to alias.

**Phase 3 - per-target execution.** Each `extdeps/languages/*` slice
models its primitives as fact-bundles from its own spec, then supplies
conformance evidence through the `LanguageBehaviorEquivalence` corpus
owned by the T-19 / T-21 testgen and affected-set path. Rust is the
pattern anchor; the same rule applies to Python, Go, C++, TypeScript,
and later Shape-A language targets.

### T-4 Authoring Contract

For each external primitive:

1. Read the target's own specification.
2. Declare a fact-bundle whose named fields are the facts the spec
   states: width, signedness, representation, range, encoding, overflow
   / NaN / Inf disposition, spelling, or the equivalent facts for that
   domain.
3. Ground each field into the shared `std/` vocabulary.
4. Cite coincidence evidence before reusing a `std/` carrier directly.
5. Keep algebra inhabitance single-authority: consume the shared algebra
   structures and Node constructors; do not declare a parallel
   per-language algebra.
6. If a needed carrier is missing, mark the model as gated on the named
   feeder task. Do not substitute a hollow alias.

Kernel-ambient atoms such as `Bool` and `Char` remain exempt where the
external spec carries no further facts to drop. The discriminator is
whether the spec says facts the declaration omits.

### Status

Ratified 2026-05-17. The P1-KEYSTONE doc rubric is landed; T-30 is the
structural enforcement gate; T-4 consumes this decision together with
`INVARIANTS.md`, `MODELING.md`, `docs/modeling-discipline.md`, and
`docs/modeling/grounding-worked-examples.md`.
