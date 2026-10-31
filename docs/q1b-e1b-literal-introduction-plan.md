# Q1b E1b: expected-type-directed integer-literal introduction in v1 (design, no code)

Read-only design. Base origin/main. Prerequisite of Q1b E1 (replace v1.compiler.infer `declared_realizes_as_kernel_numeric` with the grounding crossing authority).

## 1. Where v1 types an integer literal today
- v1.compiler.infer `infer_expr_body`, arm `ExprLiteral`: always builds `plain` with `inferred = infer_literal_node(lit)` (a LitInt is the kernel Int), then asks `literal_boundary_elaboration(lit, expected, scope)`.
- `literal_boundary_elaboration` consults only std.literal_elaboration `elaborate_literal_at` over `literal_homomorphism_rows`. For a destination that `provenance_realizes_natively` (std.nat Nat) it answers `DirectLiteral` => `LiteralBoundaryPlain`: the literal stays typed Int, at any value. It never consults `kernel_grounding_rows`.
- `literal_introduction_type_mismatch` answers `false` for `ExprLiteral { value: LitInt }` unconditionally, so no call-site/list-element judgment sees the literal either.
- The Int-literal-at-Nat admission therefore lives entirely in `declared_type_inhabitance` arm `declared_realizes_as_kernel_numeric` (a realization fact: "declaring module is kernel-backed") deciding a typing question. That is the 33-of-50 direct-call sites the comment above `product_versus_coproduct_head_mismatch` records.

## 2. The existing authority and whether v1 consumes it as is
- std.literal_elaboration: `KernelGrounding { source_kind, carrier, extent, view }`, `kernel_grounding_for(rows, source_kind, carrier)` -> `KernelGroundingFound | Absent | Ambiguous`, `kernel_grounding_admits(row, value)` (reads `IntegerExtent`: `UnboundedIntegers` true, `UnboundedNaturals` value >= 0). Rows: gunbc.structural_realization_bindings `kernel_grounding_rows`.
- v1.compiler.infer already imports all of it (`KernelGrounding`, `kernel_grounding_for`, the three lookup arms, `kernel_grounding_rows`) and already applies `kernel_grounding_for` once (grounded-literal fold in the emission path) and `kernel_grounding_crossing` in `grounded_cast_verdict`. v2 already consumes it at this exact point: v2.compiler.infer imports `kernel_grounding_admits, kernel_grounding_for` (witness src/v2/test/claim/compiler/literal_elaboration.dag: `-1` refuses at Nat from the declared range).
- So v1 can consume it as is. No new rule, no new row. Gap: the extent arms are only UnboundedIntegers/UnboundedNaturals; machine widths (UInt8...) have no row, so they stay un-judged exactly as today (no widening of scope, and no fabricated bound).

## 3. Design
In `literal_boundary_elaboration`, before the homomorphism path, when `lit` is `LitInt { value: n }` and `expected` is Present with `destination` resolved by `type_reference_declaration_ref`:
1. `kernel_grounding_for(rows: kernel_grounding_rows, source_kind: KernelIntLiteral, carrier: destination)`.
2. `Found { row }` and `kernel_grounding_admits(row, n)` -> new `LiteralBoundary` arm `LiteralBoundaryGrounded { destination_type: exp }`; `infer_expr_body` keeps the `ExprLiteral` node but sets `inferred = Resolved { destination_type }` (typed AS the carrier; no image, no elaboration: the carrier realizes as the kernel value, per the KernelGrounding comment).
3. `Found` and not admitted -> `LiteralBoundaryRefused` with a message derived from the row (carrier + extent), e.g. `-5` at std.nat Nat.
4. `Absent` -> existing path unchanged (no row = no grounded carrier, e.g. String, user types, widths). `Ambiguous` -> refuse as `HomomorphismDuplicated` does (typed, located).
5. Expected-absent (`Absent`) and Optional-cardinality expected stay Plain (Int), as now.
Consequence: the literal denotes a Nat before inhabitance sees it, so `alias_identity_in_judgment` answers Inhabits and E1 can delete `declared_realizes_as_kernel_numeric`; an Int-typed VALUE no longer matches (it reaches `grounded_cast_verdict`/crossing, which refuses Int -> Nat as `ExtentNotIncluded`). `literal_introduction_type_mismatch`'s LitInt arm stays false (the literal is now typed correctly, so there is nothing for it to add); it must not grow a Nat arm of its own.
Open point to measure before E1 (not guessable): every one of the 33 sites must pass the formal as `expected` into `infer_expr`. The list-literal path does (`infer_list_literal_element`); verify direct-call args, record fields, return position, `let x: Nat = 3`, match arms and binop operands by the existing census of sites, since a site that omits `expected` types the literal Int and would then refuse after E1.

## 4. Witness cells
int_argument_at_nat_parameter_witness (origin/int-at-nat-red):
- `a_negative_literal_at_a_nat_parameter_must_refuse`: RED today, GREEN by step 3 above (the row's extent, not a local check).
- `a_non_negative_literal_at_a_nat_parameter_is_accepted`: control, must stay green (step 2).
- `a_nat_value_at_an_int_parameter_is_accepted` (Nat -> Int): control, must stay green; it goes through `kernel_grounding_crossing` (CrossingTotalWidening), untouched.
- `an_int_value_at_a_nat_parameter_must_refuse` (E1 cell, not E1b): stays RED until E1 removes the numeric arm.
- `a_nat_value_at_a_nat_parameter_is_accepted`: control.
Untouched: `a_grounded_natural_cast_onto_the_kernel_integer_is_unjudged` (cast path, `grounded_cast_verdict`, not changed here).
Inhabitance claim owed (DESIGN s3 pairing): one claim that the real v1 path types `3` at Nat as Nat (route, not only verdict), so removing the new arm turns a control red.

## 5. Not in scope
No change to extents, rows, widths, v2, stage0 mirrors (heal regenerates), or the numeric arm (E1).
