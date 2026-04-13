> Part of: [THESIS.md](../THESIS.md) > **Core abstraction** + **Correctness dimensions**
> See also: [dimensions-design.md](../src/v2/dimensions-design.md), [cx-design.md](../src/v2/cx-design.md)

# Binding Unification: 7 → 2

## Context

Conversation on 2026-04-12 auditing thesis claims against the live
codebase surfaced a structural observation: the 7 "binding sites"
documented in `dimensions-design.md` are really 2 fundamental forms
with syntactic variations. This has implications for the dimension
mechanism, provenance, and the cost-of-change invariant.

## The observation

`dimensions-design.md` lists 7 binding-creation sites that each
need per-dimension provenance handling:

1. Function parameter
2. Let-binding
3. Match arm binding
4. Lambda param (collection context)
5. Lambda param (callable contract)
6. Lambda param (no context)
7. For-each variable

But:
- 4, 5, 6 are all **lambda parameter** — same form, different
  caller context
- 7 (for-each) is `collection |> fold(init: (), f: (_, elem) => body)`
  — a lambda parameter in collection context. Same as 4.
- 3 (match arm) is destructuring — equivalent to
  `let x = scrutinee.Variant.field`. Same as 2 with field access.

**Two fundamental binding forms:**

1. **Function/lambda parameter** — "I receive a value from my caller"
2. **Let-binding** — "I name a computed value"

Everything else is syntax over these two.

## Why this matters

### For provenance (Track 1)

S1-S7 had to visit 7 separate code paths in inference because each
syntactic form creates TypeBinding independently. If the IR desugared
to 2 forms before inference, provenance handling would be written once
per form:

- **Parameter**: provenance = what my caller passes me (walk edge
  back to call site)
- **Let-binding**: provenance = what expression computes my value
  (walk edge back to computation)

The "caller context" distinction (collection vs callable vs unknown)
is a property of the **call site**, not the binding form. The call
site already knows what it's passing (fold passes IteratedSubValue,
descend passes StrictSubValue). The lambda parameter binding doesn't
need to know — it reads what it receives.

### For dimensions generally

Every future dimension (ownership, purity, effects, user-defined)
would implement 2 binding-site rules instead of 7. The dimension
mechanism cost drops from O(7 × dimensions) to O(2 × dimensions).

### For cost of change

Adding a new syntactic form (hypothetical `with`, `guard`, `await`)
becomes: parser + desugaring to existing forms. No changes to
inference, CX, ownership, emission. The THESIS claim "cost of
change: 1 file" becomes structurally true instead of aspirational.

### For the "will there be more cases" question

With 7 binding sites, new syntax can introduce an 8th. With 2
fundamental forms, new syntax desugars — there is no 8th. The
binding enumeration is closed by construction, not by discipline.

## What the desugaring looks like

```dag
// for-each → fold
for x in list { body }
  ↓
list |> fold(init: (), f: (_, x) => body; ())

// match arm → let + field access (conceptually)
match scrutinee { Foo { x, y } => body }
  ↓
// arm selection (exhaustiveness) stays structural
// but binding creation is: let x = scrutinee.Foo.x; let y = scrutinee.Foo.y
// provenance follows from field access on scrutinee
```

### Preserving surface information

Desugaring discards information useful for error messages and
idiomatic code emission. Two strategies:

**Option A: Metadata on the desugared form.** The way `ExprData`
already carries `LambdaSemantics` and `CallSemantics` — keep the
original syntactic intent as metadata without downstream code
branching on it. Inference, CX, ownership see 2 forms. Emission
and diagnostics read the metadata for presentation.

**Option B: Desugar late (after inference, before CX/ownership).**
Inference handles the surface forms for good error locality.
Desugaring happens once, then CX/ownership/dimensions see the
reduced IR. This preserves error quality without multiplying
downstream code paths.

Option B is probably more practical given current architecture —
inference is already written for the surface forms, and rewriting
it is high-risk. The win is in downstream consumers (CX, ownership,
emission, future dimensions) seeing fewer forms.

## Relationship to existing work

- **Track 1 (provenance):** Currently threads SubValueRelation
  through 7 binding sites. With unification, threads through 2.
  The remaining S-steps and M1 plan don't change — the composition
  algebra is the same. The win is: once M1 reaches 0 and the gate
  is blocking, new syntax can't re-introduce violations because it
  desugars to existing forms.

- **Track 13 (single emitter):** Every `ExprForEach | ExprLambda |
  ExprMatch | ExprLet` match in the emitter collapses. Python, Go,
  and Rust emitters each have these matches — unification reduces
  per-language surface area.

- **INVARIANTS "no duplicate representations":** 7 code paths for
  2 fundamental mechanisms is a duplicate representation. The
  binding site enumeration should reflect the actual mechanism
  count.

## Open questions

1. **Exhaustiveness checking** — match arms are structurally
   meaningful beyond binding creation. The coproduct exhaustiveness
   check needs the arm structure even after binding desugaring. The
   desugaring must preserve the arm→variant relationship.

2. **Fold accumulator** — for-each has no accumulator (imperative
   style). Desugaring to fold introduces a synthetic `()` accumulator.
   Does this create noise in ownership/provenance analysis?

3. **When to desugar** — Option A (early, before inference) vs
   Option B (late, after inference). Trade-off: early desugaring
   simplifies ALL downstream; late desugaring preserves error
   quality for inference diagnostics.

4. **Migration scope** — this is a large refactor touching the IR
   boundary. Should it happen before or after M1 (CX gate → 0)?
   Argument for before: reduces M1 surface area. Argument for
   after: M1 is on a clear path, don't disrupt.

## Structural modeling (2026-04-12)

The 7 → 2 reduction is grounded in `SubValueRelation` — the
existing DAG edge vocabulary:

- **SubValueRelation IS the edge vocabulary.** It classifies how a
  target value relates to its source along a DAG edge. It already
  exists in `std/induction.dag` with lattice operations and
  composition. No new edge type needed.

- **Binding form is not a type — it's a question about the edge:**
  "Is the SVR provided by the caller (parameter), or derived from
  the expression (let-binding)?" This is derivable from which Node
  field the binding sits in (`params` = boundary crossing,
  `body`/`children` = local).

- **Dimension computation is SVR-keyed.** One row per SVR variant,
  one column per dimension. All four classification systems in the
  compiler (`classify_binding_provenance`, `classify_let_value`,
  `classify_body_provenance`, `classify_self_call_evidence`) become
  thin readers of SVR on bindings.

- **UsageEdge is the only new concept.** What happens to the value
  at each use site (`Consumed | Read | Projected | Threaded`).
  Currently `EdgeKind` in the compiler; belongs in `std/`.

Full proposal: [binding-model-proposal.md](binding-model-proposal.md).
Full accounting: [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md).

### Open questions resolved

1. **Exhaustiveness checking:** Match arms remain structural
   `ExprMatch` in the IR. Only BINDING CREATION inside arms
   is a local-position edge with `StrictSubValue` SVR (field
   access on scrutinee). The arm→variant relationship and
   exhaustiveness check are untouched.

2. **Fold accumulator:** The fold passes `PreservedValue` SVR to
   the accumulator parameter and `IteratedSubValue` to the element
   parameter. Inside the body, the accumulator has `Threaded`
   UsageEdge — unique access per iteration. No separate
   `AccumulatorOwnership` type needed.

3. **When to desugar:** Option B (late, after inference). Surface
   syntax metadata (`BindingSurface`) is carried for diagnostics.
   Dimension computation uses only SVR on the edge.

4. **Migration scope:** Incremental, parallel with M1. Each
   desugaring is a separate PR:
   - PR 1: for-each → fold (most obvious)
   - PR 2: match arm bindings → local + field access SVR
   - PR 3: lambda param variants → single boundary-crossing edge

## Status

**On roadmap.** Theme 1 (Close the Binding Model) in ROADMAP.md.
Incremental migration path defined. Next step: first implementation
PR (for-each → fold desugaring).
