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

---

## RESOLVED 2026-08-17: the declaration is the defect, not the 50 sites

Cause established by reading the two declarations side by side. The question
this note opened with — "is `logic_cells: std.nat.Nat = 1280` a latent modeling
error, or is the authored `Nat` the correct formal?" — has a third answer that
is better than either: **the literal is correct and `type Nat` is wrong.**

### The evidence

`CommutativeSemiring<T>` is an operations RECORD (`dag/std/algebra.dag`):

```
type CommutativeSemiring<T> {
  add: fn(T, T) -> T
  zero: T
  mul: fn(T, T) -> T
  one: T
}
```

`dag/std/nat.dag` then declares:

```
type Nat = CommutativeSemiring<std.magnitude.Magnitude>
```

which says: **a natural number IS a table containing an add function, a zero, a
mul, and a one.** It names the STRUCTURE where it should name the CARRIER.

### The file contradicts its own declaration

Two functions in the same file are unwritable under it:

```
fn nat_compare(a: Nat, b: Nat) -> Ordering { if a < b { ... } }
fn nat_max(a: Nat, b: Nat) -> Nat { if a > b { a } else { b } }
```

You cannot order two operation-records with `<`. Every USE in `std.nat` treats
`Nat` as a magnitude; only the declaration says otherwise. So the declaration
was never consistent with its own module — the import era simply never made a
consumer check it.

### v2 already models this correctly, which makes it a §3 fork

`src/v2/std/nat.dag`:

```
type Nat = Zero | Succ { prev: Nat }

data nat_semiring: CommutativeSemiring<Nat> = CommutativeSemiring {
  add: nat_add, zero: Zero, mul: nat_mul, one: Succ { prev: Zero }
}
```

Carrier is the type; the semiring is a VALUE the carrier inhabits. That is
DESIGN §4 exactly — "operations come from inhabitance (no per-type ops)", the
same shape as `Int.add` deriving from `Int` inhabiting a ring. `std.nat` states
the equation the other way round and thereby fuses two concepts under one name,
which is the §3 nicknaming violation with the roles swapped: not two names for
one concept, but one name for two.

Note also `type Magnitude` is BODYLESS (`dag/std/magnitude.dag:3`), so the
type argument is itself the hollow-alias class DESIGN tracks as an open thread.

### Why exactly now

The field-init judge (`peel_nominal_alias_identity` +
`kernel_value_declared_type_mismatch`) fires on this; the fn-return judge
(`declared_type_conformance_diags`) does not, because it only runs when both
sides are `conformance_ground_type` and `Nat`-vs-`Int` is not. So the historical
green was SILENCE, not admission — the cut did not introduce this, it removed
the last thing hiding it.

### What is decided, and what is still the operator's

DECIDED by evidence: the 50 sites are correct authoring. Do NOT "fix" them by
wrapping literals or by changing `ice40/subject.dag`. That would edit 50 correct
declarations to satisfy one incorrect one, and it is the shape §5 warns about —
the check can be satisfied by editing the declaration while the model stays wrong.

STILL OPEN, because more than one repair is defensible and this is load-bearing:

1. Ground `std.nat.Nat` on the carrier and add a separate semiring VALUE,
   mirroring `v2.std.nat`. Correct, and largest blast radius (45 `Nat`
   annotations in tree).
2. Dissolve the fork per §3 — one `Nat` authority, v2's, with `std.nat`
   consuming it. Terminal shape, and couples to the `Int = GroupCompletion<Nat>`
   grounding already named in DESIGN's open threads.
3. Ground `Magnitude` first, since a bodyless carrier makes (1) hollow anyway.

These differ in blast radius and in which open thread they discharge, not in
whether the current declaration is wrong. That part is settled.

---

## REPAIRED 2026-08-17: `type Nat = Int where range(min: 0)`

The repair choice left open above is taken, under the operator's ruling that the
bar is compilation and tests rather than reference-identity preservation.

WHICH OPTION, AND WHY IT COLLAPSED TO THE SMALLEST ONE. Option 1 (ground the
carrier, add a semiring value) is correct; the semiring VALUE half of it turned
out to be unwarranted. A census of `.add`/`.zero`/`.mul`/`.one` against anything
Nat-shaped returns 7 hits, all unrelated -- prose in design_document.dag and a
`rows.add`/`rows.zero` helper in body_lowering. NOTHING in the corpus consumes a
Nat as an operations record. Declaring one to mirror v2 would be adding structure
no consumer wants, which §2 prices as redundant work, so the repair is the carrier
alone.

Option 2 (dissolve onto v2's `Zero | Succ`) is the terminal §3 shape and is NOT
taken here, for a reason that is decisive rather than a preference: a Peano
coproduct does not admit an integer literal at a field-init site, so it would
leave all 50 rows red -- it fixes the fork while failing the bar. It stays a
DESIGN open thread, coupled to `Int = GroupCompletion<Nat>`.

Option 3 (ground Magnitude first) is dissolved rather than deferred: with the
carrier grounded on Int, the bodyless `Magnitude` is no longer load-bearing for
Nat at all, so the hollow-alias thread and this one are now independent.

WHY THIS SPELLING. `Int where range(min: 0)` is the tree's existing idiom for
exactly this concept -- std.types declares EpochSecs, EpochMs, Port, HttpStatus,
RetryCount and Char the same way -- so it introduces no new vocabulary, and a
refinement-over-Int is what the field-init judge already admits. It also makes
std.nat self-consistent for the first time: nat_compare, nat_max and nat_min
order two Nats with `<` and `>`, which the previous declaration made unwritable.

WHAT IS NOT CLAIMED. Compilation of the 50 rows is EXPECTED, not yet observed --
the verifying run is dispatched separately, and this section will be corrected
rather than quietly amended if the expectation fails. This also does not close
the two-Nat-authority fork; it corrects the wrong declaration in one of them.
1,571 `std.nat.Nat` annotations exist in tree and every one of them reads as a
magnitude, which is the population this repair serves.
