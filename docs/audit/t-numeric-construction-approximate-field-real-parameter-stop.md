# STOP — `Real = ApproximateField<Rational>` vs landed `Rational` witness (T-Numeric prep)

**Lane:** R3 #6 (T-Numeric-Construction). **Artifacts:**
[`docs/design-numeric-construction.md`](../design-numeric-construction.md),
[`docs/audit/t-numeric-construction-approximate-field-6q.md`](t-numeric-construction-approximate-field-6q.md),
landed axes `src/v3/std/approximate_field.dag` (#1427), landed
`dsl/std/rational.dag` — `Rational = Field<FieldOfFractions<Int>>` (#1508).

**Status:** **STOP for the Real alias slice** until the type-parameter convention is
resolved. This receipt does **not** block the parametric carrier
`ApproximateField<F>` / `SpecialValues` precursor (orthogonal substrate facts).

## What is honest today

- `Field<T>` in `dsl/std/algebra.dag` is parameterized by **carrier** `T`
  (elements of the field).
- Slice 4 pins `Rational` as **`Field<FieldOfFractions<Int>>`** — i.e. `Rational`
  names the **algebra witness** applied to carrier `FieldOfFractions<Int>`, not a
  bare carrier (see `dsl/std/rational.dag` and FoF 6Q Q6).
- The 6Q carrier sketch fixes `ApproximateField<F>` with **`base: Field<F>`** where
  **`F` is the same carrier slot as in `Field<F>`** — exact field operations live
  under `Field<F>`; rounding / precision / specials / subnormals are orthogonal.

## Why `Real = ApproximateField<Rational>` is not structurally honest as written

If one substitutes **`F := Rational`** into `ApproximateField<F>`:

1. `base` becomes **`Field<Rational>`**.
2. Under substrate parsing, that expands to **`Field< Field<FieldOfFractions<Int>> >`** —
   `Field` applied to a type that is **already** a `Field<…>` instantiation.
3. That is **not** the intended math layering (“approximate ℚ then ℝ”): it nests a
   second `Field` around the **witness type**, not around the **fraction carrier**.

So the phrase **`ApproximateField<Rational>`** collides with **`Rational =
Field<FieldOfFractions<Int>>`**: `Rational` is not in the same semantic class as
the `F` in `Field<F>`.

The design doc chain still shows **`Real = ApproximateField<Rational>`** and an
outdated `Rational = Field<Int>` sketch (`docs/design-numeric-construction.md` §chain).
Those lines need a **single-authority edit** together with the convention picked
below — **not** attempted silently in the carrier-only slice.

## Missing sub-carriers / decisions (before Real alias lands)

Pick **one** convention and encode it in design + ratchet tests:

| Option | Real spelling (example) | Notes |
|--------|-------------------------|--------|
| **A — Carrier-parameter honesty** | `Real = ApproximateField<FieldOfFractions<Int>>` | Matches `base: Field<FieldOfFractions<Int>>` ≡ alias `Rational`. User-facing name “Real approximates ℚ” is documentation + alias ergonomics, not the type argument to `ApproximateField`. |
| **B — Separate carrier alias** | Introduce e.g. `RationalCarrier = FieldOfFractions<Int>` in `dsl/std/rational.dag`, keep `Rational = Field<RationalCarrier>`, then `Real = ApproximateField<RationalCarrier>` | Keeps `ApproximateField`’s `F` visibly “carrier-shaped”; avoids repeating `FieldOfFractions<Int>` at Real. |
| **C — Reparameterize ApproximateField** | Change carrier shape so the parameter means “exact field alias” instead of `Field`’s carrier | Larger substrate/design churn; must restate 6Q §carrier sketch and any ratchets already authored against `base: Field<F>`. |

Until **one row** is ratified, **`Real = ApproximateField<Rational>`** must not ship as
a substrate claim — it either mis-instantiates `Field<F>` or smuggles witness/carrier
confusion.

## Explicit non-goals (Director dispatch)

Unchanged from approximate-field 6Q and inbox: **no** Float migration, **no** target
mirrors, **no** tolerance runner / cross-target float equivalence, **no** consumer
cascade in the carrier-prep lane.

## Dissolution trigger

Close this STOP when:

1. Design doc + numeric-construction chain reflect Slice 4 `Rational` and the
   chosen Real spelling; and
2. A substrate ratchet test asserts the `ApproximateField` parameter binds to the
   chosen carrier (or the revised carrier convention), analogous to Slice 4’s
   two-step `Rational` witness ratchet.
