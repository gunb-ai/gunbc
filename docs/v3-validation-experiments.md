# v3 Validation Experiments

> Part of: [v3-spec.md](v3-spec.md)
>
> **Purpose:** prove or disprove the v3 spec's claims inside v2
> before committing to a v3 build. Each experiment is bounded,
> has a clear pass/fail criterion, and teaches us something
> specific about the design.

## Experiment 1: Lambda → Bind + Define

**What it tests:** the kernel shape (5 behaviors) and the
"sameness" principle (lambda = function, no special handling).

**Scope:** in v2, make ExprLambda desugar to the same path as
a named function during DAG construction. Delete lambda-specific
downstream logic.

**Steps:**
1. In 02_parse.dag or 03_normalize.dag: when the parser sees
   `x => expr`, emit the same structure as `fn anonymous(x) { expr }`.
   Free variables (captures) become explicit input edges.
2. Delete ExprLambda variant from ExprData (or make it sugar that
   immediately lowers to ExprCall + a generated function).
3. Delete lambda-specific paths in 04_infer.dag (~77 lines),
   ownership.dag (~10 lines), 05_emit_rust.dag (3 emission modes).
4. Run the test suite. Run bootstrap.

**Pass criterion:**
- All 394 tests pass
- Bootstrap converges (regen → diff → empty)
- Net lines deleted > lines added
- No downstream code asks "is this a lambda?"

**What we learn if it passes:** the 5-behavior kernel works.
Transform with Define rule is sufficient for both lambdas and
named functions. Consumer code simplifies.

**What we learn if it fails:** lambdas have structural
differences we haven't accounted for. The spec needs to
explain what those differences are and whether they represent
genuine behaviors or missing physics.

**Estimated size:** 1-2 PRs, medium complexity. Touches parse,
infer, ownership, emit. But each touch is DELETION.

---

## Experiment 2: Carry one provenance fact, delete one CX heuristic

**What it tests:** the physics+lens principle. If the IR carries
the fact, the lens (complexity) can read it without reconstruction.

**Scope:** pick ONE function where complexity.dag reconstructs
"where did this value come from" via heuristics. Instead, carry
that fact through TypeBinding. Delete the heuristic.

**Candidate:** `classify_let_value()` in 04_infer.dag. This
function classifies whether a let-binding's value is a sub-value
of a parameter (for structural descent). CX reconstructs this
in `annotate_descent()`. Instead:
1. In 04_infer.dag: when creating the TypeBinding for a let,
   compute the SubValueRelation and store it on the binding.
2. In complexity.dag: read the SubValueRelation from the binding
   instead of reconstructing it. Delete the reconstruction logic.

**Pass criterion:**
- CX violation count does not increase (ratchet holds)
- At least one reconstruction function in complexity.dag is deleted
  or simplified
- Net lines deleted > lines added
- Bootstrap converges

**What we learn if it passes:** facts carried through bindings
dissolve downstream heuristics. The lens reads structure that
already exists. The core v2 diagnosis (construct-discard-reconstruct)
is confirmed as fixable.

**What we learn if it fails:** the binding boundary is harder to
enrich than expected, or the reconstruction does something the
simple provenance fact doesn't capture. The spec needs to address
whatever that is.

**Estimated size:** 1 PR, small-medium. Touches 04_infer.dag
(binding creation) and complexity.dag (reading). This is
essentially what Track 1 has been attempting — but scoped to
ONE function, not the whole pipeline.

---

## Experiment 3: Add a new transform, zero consumer edits

**What it tests:** the "variation is data" principle. New
transforms should be rule table entries, not structural changes
that ripple through every consumer.

**Scope:** add one new builtin operation to the .dag language.
Measure how many files need editing.

**Candidate:** add a `clamp(value, min, max)` builtin that
restricts a number to a range. This is a pure function with
known cost (O(1)), known effect (Pure), and known type
(Int → Int or Float → Float).

**Steps:**
1. Add `clamp` to the relevant extdeps or std/ authority
2. Wire it through the pipeline
3. Count every file that needed editing

**Pass criterion:**
- clamp works end-to-end (compile, emit, test)
- Files edited ≤ 3 (declaration, type rule, maybe emit template)
- complexity.dag: zero edits
- ownership.dag: zero edits
- No new match arms in any consumer

**What we learn if it passes:** the rule table approach works.
New operations are data. Consumers are generic over operations.

**What we learn if it fails:** consumers pattern-match on
specific operations somewhere. The spec needs to identify
where and explain how to make those consumers generic.

**Estimated size:** 1 PR, small.

---

## Experiment 4: Add one observational lens, zero compiler changes

**What it tests:** the lens extensibility principle. Users can
define new analyses without touching compiler code.

**Scope:** define a simple lens that reads the existing v2 IR
and produces a useful observation. Implement it as a .dag
function that the interpreter can run, NOT as compiler code.

**Candidate:** a "purity lens" — for each function, determine
whether it is pure (no service calls, no mutation). This
information is already in the IR (transport declarations,
service operations). The lens just reads it.

**Steps:**
1. Write a .dag function `is_pure(func: Node) -> Bool` that
   inspects the function body for service calls / transport
   declarations
2. Write a .dag function `purity_report(modules: List<Node>) -> List<PurityResult>`
   that applies is_pure to every function
3. Run it via `dag run purity_report.dag` on the compiler's own source

**Pass criterion:**
- The purity lens produces correct results for known-pure and
  known-impure functions
- Zero compiler .dag files edited
- Zero stage0 changes
- The lens runs via the interpreter, not a compiler pass

**What we learn if it passes:** lenses over the IR work without
compiler changes. The IR has enough structure for external
observation. User-defined lenses are feasible.

**What we learn if it fails:** the IR doesn't expose enough
structure for external observation. The spec needs to identify
what's missing and whether it's a physics gap or an access gap.

**Estimated size:** 1 PR, small. New .dag file only.

---

## Sequencing

```
Experiment 1 (lambda)     ← highest value, tests kernel shape
     |
Experiment 2 (provenance) ← tests physics+lens core claim
     |                       (can run in parallel with 1)
Experiment 3 (new transform) ← tests variation-as-data
     |                         (can run in parallel with 1+2)
Experiment 4 (purity lens)   ← tests lens extensibility
                               (can run in parallel with all)
```

All four are independent. Can run in parallel. Each is 1-2 PRs.
All must keep bootstrap green.

## After validation

If all four pass → the spec's core claims are validated.
Proceed to v3 build with confidence.

If any fail → the spec has a gap. Fix the spec, then re-validate.
Cheaper to learn this now than after building half a compiler.
