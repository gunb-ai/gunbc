# Width admission at declared positions: the contract (WORK NOTE, prep; no carrier code)

Base: #15571's model at 5bb4171d5b7a. Wiring into `declared_type_inhabitance` waits for bold-dove-340's carrier (single implementer).
Fixtures: `dag/test/claim/width_admission_declared_type_witness.dag` (20 cells, authored, UNRUN, NOT enrolled).

## The three authorities (one each, nothing new minted)

1. **The crossing (value of a width type at a width type).** `std.literal_elaboration kernel_grounding_pair_crossing(source, target)` is the ONE pair fold.
   `kernel_grounding_crossing` calls it for declaration-keyed rows; a resolved width reading calls it directly with two
   `integer_width_grounding(source: <resolved DeclarationRef>, signedness, bits)` rows (a shared `Compose` head cannot key a declaration-only map).
   It answers `CrossingTotalWidening` or `CrossingRefused{WidthSignednessChange | WidthExtentNotIncluded | ExtentNotIncluded}`.
2. **The literal extent (an Int literal at a width type).** `kernel_grounding_admits(row, value)` over the consolidated `IntegerExtent`
   (`std.integer_interval`), `row` = `integer_width_grounding` of the DESTINATION's reading. No range table in v1.
3. **The identity bridge (reading a type node as `(signedness, bits)`).** The v1 seed realization of v2 `infer_machine_width_of`:
   the same three cases and identities. `NotWidthConstrained` (head is not `std.machine_constraints Compose`, or its second argument does
   not name `MachineWidth`); `WidthReadable{signedness, bits}` (`Compose<A, MachineWidth<N>>`, N a static Nat literal, A the kernel `Int` => Signed
   or `std.nat Nat` => Unsigned, by declaration identity; `UInt = Nat` unfolds by the alias rule first); `WidthUnreadable{reason}` (malformed
   Compose, non-static width such as `MachineWidth<PointerWidth>`, carrier signedness undeclared) which REFUSES typed, never admitted by guess
   and never handed to ordinary grounding. `integer_standard_type_from_width` is the roster-free reverse lookup where a standard name is needed.
   #15571's frontier names this seed realization as this lane's.

## Two arms, not three (redirect from keen-heron-809): literal INTRODUCTION plus relation CROSSING, one shared identity bridge

- **Literal arm lives in literal introduction, not the relation.** Judging a literal against its expected type's extent is introduction's job:
  E1b's `grounded_literal_boundary` (`literal_boundary_elaboration`, branch `q1b-e1b-grounded-literal`, sunny-newt-661, landing bundled with E1) already
  sees the literal's value AND the expected type through `kernel_grounding_admits`. Width literals join THERE once the consolidated `IntegerExtent` gives a
  width its bounded extent (authority (2), `integer_width_grounding` of the destination's reading via the bridge (3)): 300 at UInt8 refuses at introduction;
  200 at UInt8 types as UInt8. No value-carrying field is added to `DeclaredTypeObligation` (that would be a second literal path, DESIGN section 3);
  the earlier "obligation carries no value" gap is therefore dissolved, not closed.
- **Relation arm judges TYPES only.** In `declared_type_inhabitance`, placed BEFORE the two admitting arms (`produced_is_kernel_grounded_carrier_or_its_alias`,
  `declared_realizes_as_kernel_numeric`, which today admits any numeral at a width type): declared and produced both `WidthReadable` (bridge (3) on both sides)
  then `kernel_grounding_pair_crossing` (1): `CrossingTotalWidening` => Inhabits; `WidthSignednessChange` / `WidthExtentNotIncluded` => `InhabitanceRefused`.
  Either side `WidthUnreadable` => typed `InhabitanceRefused`, never falling through to the kernel-numeric arm. `NotWidthConstrained` on both: unchanged.
  New refusal reasons are rows on `InhabitanceRefusedReason`. A kernel `Int`/`Nat` produced at a width type refuses by the same pair fold (unbounded extent not within).
- **Shared:** the identity bridge (3), one v1 seed realization of `infer_machine_width_of`, read by both arms.

## Which arm owns which fixture cell (20 cells, all unrun)

| cells | owner | depends on |
|---|---|---|
| RED literal 300 at UInt8 (argument, field, return, default); CONTROL literal 200 at UInt8 (same four) | literal introduction (E1b `grounded_literal_boundary`) | E1b (q1b-e1b-grounded-literal) plus the consolidated extent |
| RED Int32 at UInt8; RED Int8 at UInt8 (sign change); CONTROL UInt8 at UInt16 (each at the four positions) | relation crossing (`declared_type_inhabitance` width arm) | the carrier plus the relation wiring |

Positions: argument = `PositionDirectCallArgument`; field = `PositionRecordLiteralField`; declared return = `PositionDeclaredReturn`; default = `PositionParameterDefault` (#15547).

## Pairing obligation (DESIGN §3)

The fixtures SUPPLY source and judge the real acceptance path through `compile_dag_diagnostic_census` (the real producer), so the last execution of the real path
stays. The RED/control cells are the discriminators; enrol only after a run on a binary that contains the arm, and record the before/after on the same cells.
