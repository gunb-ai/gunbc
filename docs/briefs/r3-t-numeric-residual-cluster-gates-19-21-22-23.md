# R3 T-Numeric Residual Cluster Gates 19/21/22/23

**Lane:** T-Numeric-Construction
**Issue:** gunbc#2612
**Date:** 2026-05-10

## Scope

This worker slice closes the residual consumer gap for four R3 §1.8
T-Numeric-Construction gates sharing the same substrate cause: fixed-width
integer names are refinements over the abstract numeric chain, and literal
magnitude checks consume those structural range facts.

Covered gates:

- `numeric_aliases_align_to_refinements` (#19)
- `int_refinement_overflow_proven_parametric` (#21)
- `int_lit_full_magnitude_consumer` (#22)
- `string_audit_receipt` (#23)

## Substrate Evidence

The integer arms are already present at HEAD:

- `dsl/std/integer.dag` declares `Int8`..`Int128` as
  `Compose<Int, MachineWidth<...>>`.
- `dsl/std/integer.dag` declares `UInt8`..`UInt128` as
  `Compose<UInt, MachineWidth<...>>`.
- `type Int = AbelianGroup<GroupCompletion<Nat>>`.
- `type UInt = Nat`.

The Float arm is intentionally out of this worker scope because the dispatch
brief records the remaining parallel-authority alias pending S8
ApproximateField follow-up. This PR only claims the Int/UInt arms for #19.

The String audit is documented-no-change: `dsl/std/string_type.dag` declares
`String = FreeMonoid<Char>`, so String itself is not a numeric carrier; only
`Char` remains in inherited numeric scope.

## Executable Consumers

This slice adds/extends integration receipts:

- `bootstrap_integer_aliases_align_to_refinements_per_gate_19` checks every
  Int/UInt fixed-width alias resolves to `Compose<abstract, MachineWidth<W>>`.
- `int_refinement_overflow_is_proven_parametric_for_representable_widths`
  already checks signed and unsigned widths through the shared
  `MagnitudeOutOfRange` path, including alias representatives.
- `uint128_full_magnitude_literal_tokenizes_and_narrows` proves the full
  `u128::MAX` decimal literal remains preserved and narrows to `UInt128`.
- `bootstrap_string_audit_receipt_per_gate_23` checks `String` remains
  `FreeMonoid<Char>`.

## Gate Disposition

With these consumers passing, gates #19/#21/#22/#23 have executable coverage
for the scoped substrate facts. The canonical §1.8 ledger promotion can follow
as the post-merge status-receipt update.
