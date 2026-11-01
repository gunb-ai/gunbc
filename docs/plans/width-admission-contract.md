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

## Which arm of `declared_type_inhabitance` reads which

Today (origin/main `src/v1/04_infer.dag`) a width type reaches the verdict through two ADMITTING arms, neither width-aware:
`produced_is_kernel_grounded_carrier_or_its_alias` (keyed on `env_name_is_kernel_grounded_carrier`) and `declared_realizes_as_kernel_numeric`
(`kernel_carrier_admits_numeral`, which answers true for any numeral at a corpus-declared type). Hence the REDs are expected accepted today.

| obligation shape | arm (new, placed BEFORE both admitting arms, after alias identity) | authority |
|---|---|---|
| declared reads `WidthReadable`, produced reads `WidthReadable` | width crossing arm | (3) bridge on both sides, then (1) pair fold: `CrossingTotalWidening` => Inhabits; `WidthSignednessChange` / `WidthExtentNotIncluded` => `InhabitanceRefused` |
| declared reads `WidthReadable`, produced is an Int literal | literal extent arm | (3) on declared, then (2) `kernel_grounding_admits`; outside => `InhabitanceRefused` |
| declared or produced reads `WidthUnreadable` | unreadable arm | typed `InhabitanceRefused`; never falls through to the kernel-numeric arm |
| `NotWidthConstrained` on both | unchanged | existing arms |

A kernel `Int` / `Nat` produced at a width declared type is NOT this arm's: it is the unbounded-to-bounded direction and refuses by the same pair fold
(`Int` extent is not within the width extent). New refusal reasons extend `InhabitanceRefusedReason`; each is a row, not a branch.

## Two carrier gaps the wiring must close (stated, not fixed here)

- **The obligation carries types, not values.** `DeclaredTypeObligation { position, subject, declared, produced, span }`: an Int literal's value is not in
  `produced`, so arm (2) needs the literal's value delivered to the judgment. Smallest honest change: one optional field on the obligation (the literal's
  value when the producer is a kernel Int literal) filled by each producer, not a per-position check. DESIGN §3: the field is read by this one relation.
- **Positions.** Argument = `PositionDirectCallArgument`; record field = `PositionRecordLiteralField`; declared return = `PositionDeclaredReturn`;
  parameter default = `PositionParameterDefault` (#15547). The fixtures cover exactly these four; other positions inherit by the same relation.

## Pairing obligation (DESIGN §3)

The fixtures SUPPLY source and judge the real acceptance path through `compile_dag_diagnostic_census` (the real producer), so the last execution of the real path
stays. The RED/control cells are the discriminators; enrol only after a run on a binary that contains the arm, and record the before/after on the same cells.
