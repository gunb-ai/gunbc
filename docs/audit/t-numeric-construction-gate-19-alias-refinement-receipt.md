# T-Numeric Construction Gate #19 Alias Refinement Receipt

**Gate:** `numeric_aliases_align_to_refinements`
**Lane:** T-Numeric-Construction
**Date:** 2026-05-11

## Receipt

The fixed-width integer aliases are refinements over the abstract numeric
carriers, not parallel algebra substrate:

- `dsl/std/integer.dag` declares `Int8`, `Int16`, `Int32`, `Int64`, and
  `Int128` as `Compose<Int, MachineWidth<...>>`.
- `dsl/std/integer.dag` declares `UInt8`, `UInt16`, `UInt32`, `UInt64`, and
  `UInt128` as `Compose<UInt, MachineWidth<...>>`.
- `Int` remains the abstract integer construction
  `AbelianGroup<GroupCompletion<Nat>>`.
- `UInt` remains the abstract unsigned construction `Nat`.

The executable consumer is
`bootstrap_integer_aliases_align_to_refinements_per_gate_19`, backed by
`assert_bootstrap_integer_aliases_align_to_refinements`. It checks every
fixed-width signed and unsigned alias listed above resolves to
`Compose<abstract-carrier, MachineWidth<word-carrier>>`.

## Scope Boundary

This receipt closes the Int/UInt fixed-width alias arm of gate #19. Float
aliases already route through `Real32` / `Real64` in `dsl/std/float.dag`;
broader Real/Float demonstration and default-width alias questions remain
under the existing S9 / Shape-A follow-on lanes and are not claimed here.
