# The 50-row `Nat` ← integer-literal refusal: what it is NOT

**Status 2026-08-17:** cause NOT established. Alternatives closed by execution
and by cross-tree comparison, recorded so the next person starts where this
stopped rather than re-walking it.

## The class

50 of the integration branch's ~192 diagnostics are one row:

```
expected 'Product(CommutativeSemiring)', got 'Primitive(Int)'
```

46 of the 50 are an integer literal landing in a `std.nat.Nat`-typed record
field or named argument. The dominant file is
`dag/extdeps/programmable_logic/ice40/subject.dag` (33 of 50):

```
type Ice40Device { logic_cells: std.nat.Nat, ram4k_blocks: std.nat.Nat, .. }
Ice40Device { logic_cells: 1280, ram4k_blocks: 16, .. }
```

`std.nat.Nat` is `CommutativeSemiring<std.magnitude.Magnitude>` — an alias whose
target is `std.algebra`'s four-field record (add/zero/mul/one).

## The two judges (established by quiet-deer-375, `v1.compiler.infer`)

- **fn-return**: `declared_type_conformance_diags`, which fires only when both
  sides are `conformance_ground_type`. `Nat` is neither, so `Nat` vs `Int` is
  **unjudged**. A green `-> std.nat.Nat { 1000 }` is SILENCE, not admission.
  This matters: it was the green I reasoned from, and it proved nothing.
- **field-init / named-arg**: `infer_record_lit_structural` post-checks with
  `peel_nominal_alias_identity` + `kernel_value_declared_type_mismatch`, which
  refuses a formal peeling to a `Conj`/`Disj` with children that is not a
  refinement-over-Int. Named-argument sites share the peel via the direct-call
  zipper. So this judge is the only one of the two actually looking.

## Closed alternatives — each by evidence, not argument

1. **Alias substitution generalizing the emit `resolved_type` hop.** Refuted by
   control: `measure.dag` `milliseconds_per_second() -> std.nat.Nat { 1000 }`
   and `Present { value: 1 }` compile green in a tree where the alias is
   present. (Weakened further by the point above: that green is silence.)
2. **Position-only defect (field-init has a hole).** Refuted: main compiles the
   same named-arg construct.
3. **The qualify pass caused it** (bare `Nat` → `std.nat.Nat`). Refuted:
   `origin/main` `dag/test/claim/budget_tree_witness_test.dag` already spells
   `amount: std.nat.Nat` fully qualified, and is green.
4. **Import binding preserved authored identity.** Refuted: that same main file
   does NOT import `std.nat` — its only import is `std.measure`.
5. **`std.magnitude` missing from the branch closure, so the peel cannot reach
   a refinement.** Refuted two ways: the error text prints a COMPLETED peel to
   the target (you only get `Product(CommutativeSemiring)` once the peel found
   algebra's record), and `magnitude.dag` is present in the frozen-141 READ set
   alongside `algebra.dag`.
6. **Pool membership generally.** Excluded on both sides: main's green closure
   contains the whole chain — `std/measure.dag` does `import std.nat { Nat }`
   (148 uses), and `nat` pulls `algebra` and `magnitude`.

## What is left

Same source spelling, same modules resident, opposite verdicts. So the
difference is the **resolver** — which is what the cut changed — and the
question is what `Node` identity is handed to `peel_nominal_alias_identity` /
`with_authored_identity`, not what the peel does with what it gets. In our tree
the alias TARGET is presented as the formal; in main's, the authored `Nat`.

## An open question this raises and does not answer

If the authored `Nat` is the correct formal, the branch is wrong. If the peeled
product is correct, then `logic_cells: std.nat.Nat = 1280` is ill-typed —
a literal is not a four-field semiring record — and these 50 rows are
PRE-EXISTING modeling errors that the import era never checked. That question
belongs with the two-`Nat`-authority fork (`std.nat.Nat` the algebra alias
versus `v2.std.nat.Nat` the Peano coproduct), which DESIGN already carries as
unconsolidated. Do not fix the 50 sites before deciding which way it points.
