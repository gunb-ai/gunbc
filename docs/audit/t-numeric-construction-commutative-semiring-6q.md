# T-Numeric-Construction — CommutativeSemiring 6Q

**Date:** 2026-05-02
**Authority:** [`docs/design-numeric-construction.md`](../design-numeric-construction.md) and #1399 review follow-up.
**Scope:** substrate algebra-strength declaration only. No `Nat` alias sharpening in this PR.

## Decision

Add `CommutativeSemiring<T>` to `dsl/std/algebra.dag` as the algebra-strength surface between `Semiring<T>` and future commutative ring surfaces.

Shape:

```dag
type CommutativeSemiring<T> {
  add: fn(T, T) -> T
  zero: T
  mul: fn(T, T) -> T
  one: T
}
```

The fields intentionally match `Semiring<T>`. The new information is the law that the multiplicative monoid is commutative: `mul(a, b) == mul(b, a)`. As with `CommutativeMonoid<T>`, the law is not stored as a field.

This PR should not also change `Nat = Semiring<Magnitude>`. The default split is cleaner: first land the algebra type, then sharpen `Nat` in a follow-up that changes the alias and the `nat_resolves_to_semiring_over_magnitude` ratchet together.

## 6Q

### Q1 - Carrier Invariants

**PASS.** The carrier has the same four operation/identity fields as `Semiring<T>`. No nullable law fields, booleans, or side tables are added.

### Q2 - Index / Handle Types

**N/A.** No new references or ids are introduced.

### Q3 - Duplicated Fact

**PASS.** This does not duplicate `Semiring<T>`; it names a strictly stronger algebraic law. `Semiring<T>` remains the non-commutative multiplication surface. `CommutativeSemiring<T>` is needed because Nat's multiplication is commutative and the denotational table already names that strength.

### Q4 - Coproduct Compression

**N/A.** This is a record, not a coproduct.

### Q5 - Construction Authority

**PASS.** The declaration lives beside the rest of the algebra hierarchy in `dsl/std/algebra.dag`. It does not populate target rows, operator realizations, or Nat aliases.

### Q6 - Representation Duality

**PASS.** No alternate Nat representation is introduced. `dsl/std/nat.dag` remains `Semiring<Magnitude>` in this PR with an updated tracked trigger naming the follow-up alias change.

## Follow-ups

- Sharpen `Nat` from `Semiring<Magnitude>` to `CommutativeSemiring<Magnitude>` and update the existing Nat ratchet in the same PR.
- Add `CommutativeRing<T>` only when a concrete consumer needs ring-with-commutative-multiplication strength.
- Add `OrderedCommutativeRing<T>` only when the Int chain or target selection needs that exact strength.

Those sibling types are intentionally not implemented here; adding the semiring layer is bounded and does not force the entire algebra hierarchy to move in one PR.
