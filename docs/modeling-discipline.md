# Modeling Discipline — Active Review Criteria

> Purpose: a short, review-focused checklist of modeling principles that
> every code change is checked against. This document is the active
> authority cited by review bots alongside THESIS.md and INVARIANTS.md.
>
> Full derivations, worked examples, and the background modeling analysis
> live in [v3-modeling-analysis.md](v3-modeling-analysis.md). This doc is
> the distilled review-checklist version.

## Six Principles

Every PR is checked against all six. A reviewer should name specifically
whether the diff satisfies each, where it could be violated, and whether
the existing checks are structural or merely behavioral.

### 1. Fail-closed

Every failure path goes through the diagnostic mechanism. No silent
`None` returns. No panics on user-reachable paths. No success results
with unresolved state.

**What to check:** If a code path fails, does it fail through a
diagnostic, or does it fail silently? If a function returns `Option<T>`
on error, can the caller distinguish "not yet computed" from "failed"?

**Example violation:** A function returns `None` on error without
writing a diagnostic. The caller sees `None` and may treat it as "not
yet computed."

**Fix:** Replace `None`-on-error with diagnostic writes. Return
`Option<T>` only when the absent case is a legitimate non-error state.

### 2. Illegal states unrepresentable

Data models must not admit combinations that the invariants forbid.
If the reviewer can imagine a combination of field values that
"shouldn't happen," the type is wrong.

**What to check:** Does any combination of field values represent an
illegal state? Is `Option<T>` being used to mean two different `None`s?
Are there product types with mutually exclusive field values?

**Example violation:** `Port { value_type: Option<TypeShape> }` where
`None` means both "not inferred yet" and "inference failed." The
invariant "`None` iff diagnostic exists" is runtime-checked rather
than type-enforced.

**Fix:** `PortState::Uninferred | Resolved(TypeShape) | Unresolved(PortId)`.

### 3. Facts flow forward

Every piece of structured information produced at one stage of the
compiler must be either consumed by the next stage, carried forward as
a field on downstream data structures, or explicitly discarded with a
comment explaining why the information is no longer needed. Silent
drops are violations.

**What to check:** For each cross-stage boundary touched in the diff
(parse→lower, lower→infer, infer→lens, lens→emit), enumerate the fields
on upstream types and verify each is handled downstream.

**Example violation:** Parser computes `SourceSpan`, lowering discards
it, inference cannot produce located diagnostics. The fact existed
upstream but didn't survive.

**Fix:** Add `span` field to every downstream behavior node. Facts
that die at a boundary either get carried or get justified.

### 4. Coproduct dissolution

Flat coproducts (Rust enums, tagged unions, sum types with N named
variants) are compressed references to richer structure. In a closed
system where we own all the definitions, most coproducts are unfinished
modeling — the richer structure exists, we just haven't written it down.

Every enum with N ≥ 2 variants must be classified as one of:

- **🟢 GREEN (terminal)** — no richer source exists. The variants trace
  to irreducible distinctions at the user-input boundary (literals,
  keywords, source locations). Requires a **ledger entry**: a written
  record of which dissolution patterns were attempted and why they
  failed.

- **🟡 YELLOW (scaffold)** — richer source exists but extracting it
  requires substrate work that isn't ready. Requires a **named
  trigger**: the specific condition under which dissolution becomes
  cheap.

- **🔴 RED (dissolvable-now)** — richer source exists and extraction
  is cheap. Do it immediately, before the next consumer is added.

**Four dissolution patterns to try in order** before classifying as
terminal:

1. **Fact placement.** Variants trace to different consumers or DAG
   locations. Move each variant's payload to the consumer that uses it.
   The coproduct dissolves into scattered fields.
2. **Variant-is-data.** Variants have the same structural shape with
   different labels. Promote the label to a field. Example:
   `LogLevel::Debug | Info | Warn | Error` → `LogLevel { name, priority }`.
   *Guardrail: only valid when the label space is closed and enumerable,
   not when it's free-form string.*
3. **Algebraic form.** Variants trace to different algebraic structures
   (intro/elim, ops over `std/` types). Express each as a reference to
   its `std/` source. Example: `ArithOp::Add | Sub | Mul | Div` →
   `Apply { function: FunctionRef }` pointing to `std::int::add` etc.
4. **Dimensional.** Variants are points in an M-dimensional space.
   Promote the dimensions to fields. Example:
   `EdgeKind::Consumed | Read | Threaded | Projected` →
   `Edge { source_effect, control_role }`.

**What to check:** Any new Rust enum with N ≥ 2 variants must have a
checkpoint comment naming its classification (🟢/🟡/🔴), with a ledger
entry if GREEN or a named trigger if YELLOW. Enums without any of these
are unfinished modeling and block review.

**Scaffold exception:** early-milestone code (marked `// scaffold:
<sunset-milestone>`) can skip the classification annotation until the
sunset milestone. Scaffolds must be revisited before sunset.

**Worked example (v2 retrospective):** `v2::ExprData` had 22 variants.
Failed pattern 1 (every consumer dispatches on all 22), pattern 2
(shapes genuinely differ), pattern 3 partial, pattern 4 partial. The
correct dissolution is pattern 3 for computation-carrying variants
(`Apply` over `std/` functions) and pattern 4 for control-carrying
variants (`Branch`/`Loop` as dimensional decompositions). Result: 22→5
reduction. Not "shorter code" — "same information, properly factored."

### 5. Single-authority metadata

Every piece of metadata about the program (types, diagnostics, spans,
provenance) must have exactly one canonical location. Duplicate
representations are violations. Mutator APIs that live on detachable
child objects violate single-authority because the child can be
separated from its parent.

**What to check:** Is there a second representation of any fact? Do
mutator methods live on child objects that hold references to parents?

**Example violation:** `DiagnosticTable::mark_unresolved(&mut dag, ...)`
where the method lives on a child object holding a reference to its
parent. Another `DiagnosticTable` instance could null the parent's
ports without going through the parent.

**Fix:** `Dag::mark_port_unresolved(&mut self, ...)` — method on the
parent. The child provides data; the parent owns mutation.

### 6. API-level enforcement over convention

When an invariant has to hold, the API should make violations
impossible, not merely undesirable. Convention-level enforcement
("please don't do X") fails under cognitive load. The type system
should stop violations, not documentation.

**What to check:** If a new contributor tried to violate the invariant,
would the type system stop them, or would they need to know the rule?

**Example violation:** `Dag::clear_port_type` is `pub(crate)` with a
doc comment saying "only call from `mark_unresolved`." If another
crate-internal caller forgets, the invariant breaks.

**Fix:** Make `clear_port_type` private to the diagnostic module, or
eliminate it entirely by making the state transition atomic at the
data-model level.

## Calibration: Blocking vs Non-blocking

A finding is **BLOCKING** if fixing it in a later PR would be meaningfully
harder than fixing it now — i.e., if merging this PR commits the project
to a shape that is expensive to change.

A finding is **NON-BLOCKING** if it's a cleanup that can land later at
roughly the same cost.

**Substrate-level issues are almost always BLOCKING** because the
substrate sets patterns that get copied. Once a bad shape propagates
through three consumers, changing it means changing all three plus the
substrate.

**Performance issues are almost always NON-BLOCKING** because they can
be optimized later without changing interfaces.

**Test coverage gaps depend:** gap in a high-risk invariant → BLOCKING;
gap in a low-risk area → NON-BLOCKING.

**When in doubt, prefer BLOCKING.** It is better to ask for a small
rework now than to accept a substrate bug that propagates through three
milestones before anyone notices.

## For Reviewers

A review must actively apply all six principles. For each:

1. Name specifically whether the diff satisfies it.
2. If violated, cite the exact file and line.
3. State whether the existing check is structural (type system enforced)
   or merely behavioral (convention).
4. For new enums: verify the 🟢/🟡/🔴 classification annotation.
5. For cross-stage boundaries: verify facts flow forward.
6. Classify every finding as BLOCKING or NON-BLOCKING per the calibration
   above.

This document is the distilled version of modeling principles. For the
full analysis and additional worked examples, see
[v3-modeling-analysis.md](v3-modeling-analysis.md).
