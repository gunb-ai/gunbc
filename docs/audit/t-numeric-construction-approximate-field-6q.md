# T-Numeric-Construction — `ApproximateField<F>` 6Q Audit

**Lane:** R3 #6 (T-Numeric-Construction). **Authority:**
[`docs/design-numeric-construction.md`](../design-numeric-construction.md)
for numeric-construction shape and
[`docs/r3-structure.md`](../r3-structure.md) for R3 lane ownership. ROADMAP.md
is not the milestone authority for this reframed R3 lane. **Subject:**
`ApproximateField<F>` — the structural carrier for `Real =
ApproximateField<FieldOfFractions<Int>>` and IEEE-754-style bounded real
approximations.

This is a design-decision receipt only. It intentionally does not introduce
the carrier, migrate `Float`, author target-specific mirrors, or touch
refinement syntax.

## Recommendation

Proceed with `ApproximateField<F>` as a substrate-introduction, but slice it.
The full design is **M-L** because rounding behavior, precision, special
values, subnormal policy, comparison policy, and target grounding all become
observable facts. A single broad carrier PR would mix too many new facts.

Recommended substrate shape, preserving the design doc's intent while making
all observable IEEE-754 surfaces structural:

```dag
type ApproximateField<F> {
  base: Field<F>
  rounding: RoundingMode
  precision: Precision
  special_values: SpecialValues
  subnormal_policy: SubnormalPolicy
}

type RoundingMode
  = ToNearestEven
  | ToZero
  | ToPositiveInfinity
  | ToNegativeInfinity
  | ToAwayFromZero

type Precision
  = Unbounded
  | BinaryPrecision { significand_bits: Int, exponent_bits: Int }
  | DecimalPrecision { digits: Int, exponent_digits: Int }

type SpecialValues {
  nan: NanPolicy
  infinity: InfinityPolicy
  signed_zero: SignedZeroPolicy
}

type NanPolicy = NoNaN | QuietNaN | QuietAndSignalingNaN
type InfinityPolicy = NoInfinity | SignedInfinity
type SignedZeroPolicy = NoSignedZero | SignedZero
type SubnormalPolicy = NoSubnormals | GradualUnderflow | FlushToZero
```

This keeps the design doc's required facts — rounding, precision, NaN,
infinities, signed zero, and denormals/subnormals — as typed carriers, not
strings or target-name tags. It also separates subnormal handling from the
generic `SpecialValues` record because denormal policy is not merely "has a
value"; it changes underflow behavior.

## Slice Plan

1. **Enums-only precursor (S-M):** land `RoundingMode`, `Precision`,
   `NanPolicy`, `InfinityPolicy`, `SignedZeroPolicy`, and
   `SubnormalPolicy`, plus carrier-shape ratchets. No `Float` migration.
2. **Carrier slice (M):** land `SpecialValues` and `ApproximateField<F>`.
   Ratchet that `ApproximateField` points to `Field<F>` and carries the five
   structural axes above.
3. **Real alias slice (S-M):** introduce `Real =
   ApproximateField<FieldOfFractions<Int>>` only after the Rational/Field gap
   is resolved. Do not pass the witness alias `Rational =
   Field<FieldOfFractions<Int>>` as the `ApproximateField<F>` carrier argument.
4. **Grounding slices (per target):** map `Real<32>`, `Real<64>`, etc. to
   Rust/Python/Go target facts. These are target-realization facts, not the
   substrate carrier itself.
5. **Comparison/equivalence slice:** connect float equality/tolerance to the
   cross-target equivalence policy. `docs/design-cross-target-equivalence.md`
   currently excludes floats from strict L5 by default until a typed policy
   exists.

## The 6 Questions

### Q1 — Cardinality invariants

Does the type admit `[]` when invariant says >=1, or singletons when >=2?

**Answer: PASS with the recommended shape.** The proposed carrier uses records
and closed sums, not list encodings. No list field admits empty/duplicate
states for precision or special-value policy. `BinaryPrecision` explicitly
names `significand_bits` and `exponent_bits`; it does not compress the pair
into a list of unnamed integers.

### Q2 — Index/handle types

Does a raw `Int` / `NodeId` encode something with a domain restriction?

**Answer: GAP unless precision dimensions are refined.** `significand_bits`,
`exponent_bits`, `digits`, and `exponent_digits` are positive counts. Bare
`Int` is the only currently available syntax-compatible placeholder, but the
carrier slice should name the follow-on refinement:

- `PositiveInt` or a dedicated `PositiveBitCount` for all precision counts.

This is not a blocker for the audit receipt, but the carrier PR should not
claim that unconstrained negative precision is semantically valid.

### Q3 — Duplicated fact

Does Field A duplicate what is derivable from Field B?

**Answer: PASS if the axes remain orthogonal.** `base: Field<F>` carries the
underlying exact algebra. `rounding`, `precision`, `special_values`, and
`subnormal_policy` carry approximation facts not derivable from `Field<F>`.
They must not be duplicated later as target-specific strings such as
`"f64"` or `"ieee754"` on the same carrier.

Target grounding can derive Rust/Python/Go implementation choices from these
facts; it should not restate them in parallel mirrors.

### Q4 — Coproduct compression

Does one variant compress N distinct causes that downstream needs to
distinguish?

**Answer: PASS with one refinement to the design sketch.** The design doc's
`Precision = Unbounded | IEEE754Width<N>` is too compressed for the consumers
named in the same doc: `Float32` and `Float64` need mantissa/significand and
exponent facts, not only total width. The recommended shape splits that into
`BinaryPrecision { significand_bits, exponent_bits }`.

Similarly, booleans such as `has_nan` are enough to express admission but not
enough to express quiet vs signaling NaN behavior. The audit recommends
closed policy sums (`NanPolicy`, `InfinityPolicy`, `SignedZeroPolicy`,
`SubnormalPolicy`) so downstream equivalence and grounding code can fail
closed instead of interpreting booleans differently.

### Q5 — Construction authority

Are multiple call sites independently constructing the same fact?

**Answer: PASS if construction is staged.** The substrate authority should be
one std carrier file and its generated bootstrap reflection. Per-target rows
must reference the carrier facts rather than reconstructing precision,
rounding, and special values independently.

The first carrier PR should add only type declarations and carrier-shape
ratchets. Concrete `Float32`, `Float64`, `Real<N>`, and per-target realization
rows belong to later consumer/grounding slices.

### Q6 — Representation duality

Can the same fact be expressed in two structurally different shapes that
comparison treats differently?

**Answer: current repo has representation duality; this design dissolves it.**
`dsl/std/float.dag` currently says `Float32 = Field<Word32>` and `Float64 =
Field<Word64>`, while the numeric construction design requires approximate
real semantics. That existing shape quietly models floats as exact fields
over storage words. `ApproximateField<F>` is the correct replacement
authority, but it must land before `Float` migration and then retire the old
`Field<Word*>` shape.

Until that migration, audit and design docs should treat the current
`Float = Field<Word64>` as legacy debt, not a valid parallel authority.

## Verdict

**PASS for design, STOP for broad implementation in this slice.**

The carrier is necessary and should land, but not as one large mixed PR. The
next implementation should be the enums-only precursor or, at most, the
carrier slice with no `Float`/`Real` migration and no target-specific mirrors.

Named follow-on substrate facts:

- `PositiveBitCount` / precision-count refinement for precision fields.
- `RoundingMode` closed sum.
- `Precision` closed sum with binary and decimal precision payloads.
- `NanPolicy`, `InfinityPolicy`, `SignedZeroPolicy`, `SubnormalPolicy`.
- `SpecialValues` record over those policy sums.
- `ApproximateField<F>` record over `Field<F>` plus the structural axes.
- Typed float comparison/equivalence policy before strict cross-target L5
  admits float programs.

## Boundaries

- No refinement syntax work in this audit.
- No consumer migration from `Float = Field<Word64>` in this audit.
- No target-specific Rust/Python/Go mirrors in this audit.
- No tolerance/epsilon runner behavior in this audit.
