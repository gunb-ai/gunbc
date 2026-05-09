# Numeric Construction Chain — Design Doc for T-Numeric-Construction (R3 Lane #6)

**Status:** PROPOSAL (locks substrate shape before T-Numeric-Construction worker dispatch). Authored 2026-05-01 by PM (deep-wolf-155) per Director ratification at [gunbc#828 comment 4357704426](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357704426).

**Authority:** single-source while open. Substrate Mgr 6Q audits land as substrate-introduction PRs against this doc. Worker brief authoring (Substrate Mgr territory) is gated on this doc + 6Q audits clearing.

**Supersedes:** Director's prior `OrderedRing<Magnitude>` shorthand ratification ([comment 4357686725](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357686725)) per [comment 4357704426](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357704426). The construction chain ℕ → ℤ → ℚ → ℝ is the structurally honest shape; the prior shorthand elided the math layering.

**Lane:** R3 Lane #6 (T-Int128 → T-Numeric-Construction; reframe per `docs/r3-structure.md` amendment landing in same PR as this doc).

## Frame

Today's `dsl/std/integer.dag:45` declares `type Int = Int64`, baking in 64-bit width at the default integer alias. The `integer.dag` MODELING NOTE itself flags this as a known issue waiting on language work:

> "Current representation: Int = OrderedRing<Word64>. This conflates carrier with witness ... The end-state model separates them ... Until the language supports trait/witness syntax, the direct alias is the honest intermediate."

This doc executes that end-state. The structural fix layers as the textbook math construction chain: model ℕ first as the foundational counting algebra, derive ℤ from ℕ via Grothendieck construction, derive ℚ from ℤ as the field of fractions, derive ℝ from ℚ via Cauchy completion (IEEE 754 = approximation thereof).

## The construction chain

```
Magnitude                              (terminal substrate — unbounded counting carrier)
   ↓
Nat = Semiring<Magnitude>              (ℕ — natural numbers; no neg, no div)
   ↓
Int                                   (ℤ via Grothendieck completion of Nat; AbelianGroup<Int> witness)
   ↓
Rational = Field<Int>                  (ℚ — fractions)
   ↓
Real = ApproximateField<Rational>      (ℝ; IEEE 754 = ApproximateField instance)
```

Refinements apply at any layer:

```dag
type Int<N>  = Int  where bits <= N         // bounded ℤ
type Nat<N>  = Nat  where bits <= N         // bounded ℕ
type Real<N> = Real where bits <= N         // bounded ℝ (with rounding semantics)

type Int8   = Int<8>
type Int128 = Int<128>
type UInt   = Nat                            // UInt IS Nat (aliasing-as-naming)
type UInt8  = Nat<8>
type Float32 = Real<32>                      // IEEE 754 fp32 with 23-bit mantissa, 8-bit exponent
type Float64 = Real<64>
// (no alias for unbounded Int / Nat / Real — that IS the abstract concept)
```

## Substrate-introductions needing 6Q audit

Per Director ratification, Substrate Mgr runs the 6-question audit (`feedback_substrate_principle_audit`) on each new substrate-introduction before authoring:

### 1. `Magnitude` — terminal substrate carrier

**Status:** NEW substrate. Most foundational primitive in std.

**Design call**: what is the structural shape of unbounded counting?

Three candidate encodings:

| Encoding | Shape | Tradeoffs |
|---|---|---|
| **Bit-stream** | `Magnitude = List<Bit>` (unbounded sequence of bits, big-endian) | Simple; matches existing `Bit` primitive at `dsl/std/bit.dag`. Cost-lens: O(n) per bit-position access |
| **Word-stream** | `Magnitude = List<Word64>` (unbounded sequence of 64-bit words) | Closer to hardware representation; native arithmetic on each word. Pre-bakes Word64 chunking |
| **Abstract counting** | `Magnitude` is a **terminal type** with no exposed structure; arithmetic operations defined extensionally via algebraic axioms (Semiring laws) | Cleanest mathematically; carrier is opaque; refinements `Nat<N>` give it bit-width when grounded. Cost-lens: O(1) per axiom-level operation; refinement determines target-realization cost |

**PM recommendation: abstract counting.** Carrier shape stays opaque; refinements `Nat<N>` / `Int<N>` give it concrete bit-width at grounding time. This matches `feedback_compositional_not_templating` (compositional preservation; refinement is child) + `feedback_naming_is_aliasing` (`Magnitude` is the namespace; refinements specialize it). Substrate Mgr 6Q audit verifies whether opaque-carrier representation is sound under the existing substrate machinery, OR whether one of the explicit-stream encodings is needed for cost-lens reasoning.

**Sized**: S substrate-introduction for declaration; M for design-call resolution if non-opaque encoding is chosen.

### 2. `AbelianGroup<G>` — verify existing

**Status:** Already declared at `dsl/std/algebra.dag:132` (`type AbelianGroup<T> { ... }`). Audit verifies it carries the structural facts needed:
- Identity element (additive 0)
- Inverse operation (additive negation)
- Commutativity axiom
- Associativity axiom

If audit confirms structural completeness, **no new substrate** for this layer.

### 3. Grothendieck construction for `Int`

**Status:** Design-call. Two canonical encodings:

| Encoding | Shape | Tradeoffs |
|---|---|---|
| **Quotient of pairs** | `Int = (Nat, Nat) / ~` where `(a, b) ~ (c, d) iff a + d = c + b`; pair `(a, b)` represents `a - b` | Mathematically canonical; preserves construction-chain integrity. **Substrate cost**: equivalence-class quotient is heavy substrate machinery |
| **Sign-magnitude** | `Int = (Sign, Nat)` where `Sign = Pos \| Neg` (with `Pos 0 == Neg 0` collapsed) | Same algebra as Grothendieck quotient (provably) without quotient machinery. Matches `feedback_naming_is_aliasing` cleanly |
| **Abstract Int via algebra** | `Int` is an abstract Grothendieck-completion carrier over `Nat`, with `AbelianGroup<Int>` as its algebra witness — no explicit construction encoding | Most v3-substrate-friendly; it does not commit to quotient or sign-magnitude representation; emission selects representation per target. Requires a distinct completion carrier or carrier/witness syntax before declaration |

**PM recommendation, narrowed by 6Q audit: Abstract Int via algebra (option 3).** The existing `AbelianGroup<T>` at `algebra.dag:132` is the additive-group witness shape. It is not itself a carrier constructor, so `type Int = AbelianGroup<Nat>` is only shorthand for the design intent and is not a valid current substrate declaration. The valid target is an integer carrier completed from `Nat`, witnessed by `AbelianGroup<Int>`, once the substrate can express either a distinct completion carrier or carrier/witness syntax. Per-target grounding then picks concrete representations: Rust `i128` is two's-complement; Python `int` and Go `math/big.Int` use implementation-owned arbitrary-precision representations. Substrate Mgr verifies this composition resolves at compile time.

**6Q audit receipt:** [`docs/audit/t-numeric-construction-grothendieck-6q.md`](audit/t-numeric-construction-grothendieck-6q.md) accepts abstract-via-algebra as the encoding decision and rejects quotient-of-pairs plus sign-magnitude on Q3/Q6 grounds. The audit also narrows the declaration target: current `AbelianGroup<T>` is an algebra witness over an existing carrier, not a Grothendieck-completion carrier constructor, so `type Int = AbelianGroup<Nat>` is deferred until the substrate can express either a distinct completion carrier or carrier/witness syntax. Sign, magnitude, pair coordinates, quotient normalization, and zero-collapse rules are target/grounding details unless a later substrate consumer proves otherwise.

**Sized**: docs-only for this audit; later declaration is S-M depending on whether carrier/witness syntax already exists or a distinct completion carrier must be introduced. M-L if Substrate Mgr decides explicit quotient machinery is needed for cost-lens or refinement-composition reasoning.

### 4. `Field<F>` — verify existing

**Status:** Already declared at `dsl/std/algebra.dag:198` (`type Field<T> { ... }`). Audit verifies it carries:
- Multiplicative identity (1)
- Multiplicative inverse for non-zero
- Distributivity over the underlying ring

If audit confirms, **no new substrate** for `Rational = Field<Int>`.

### 5. `ApproximateField<F>` — biggest substrate-introduction

**Status:** NEW substrate. Captures IEEE 754 nuances structurally rather than as a String tag.

**Design surface:**

```dag
type ApproximateField<F> {
  base: Field<F>                       // underlying field structure (e.g., Field<Rational>)
  rounding: RoundingMode               // how arithmetic results round to representable values
  special_values: SpecialValues        // NaN / ±∞ / signed zero / denormals
  precision: Precision                 // mantissa bits + exponent bits (or unbounded for true ℝ)
}

type RoundingMode = ToNearestEven | ToZero | ToPositiveInfinity | ToNegativeInfinity | ToAwayFromZero

type SpecialValues {
  has_nan: Bool                        // does this field admit NaN?
  has_signed_infinity: Bool
  has_signed_zero: Bool
  has_denormals: Bool
}

type Precision = Unbounded | IEEE754Width<N>   // N = total bits (32 / 64 / 128); mantissa/exponent split per IEEE 754 spec
```

**Design tradeoffs:**

| Aspect | Choice | Rationale |
|---|---|---|
| Why NOT `Real = Field<Rational>` directly | Field laws fail under IEEE 754 rounding (associativity of add fails on floats) | The existing `Float = Field<Word64>` at `float.dag:20` quietly lies about this. `ApproximateField` makes the rounding-induced lawlessness structural — the field-with-rounding inhabits a weaker algebra than pure Field |
| Why explicit `RoundingMode` enum | Different IEEE 754 contexts have different rounding semantics; Rust `f64` defaults to ToNearestEven; some scientific contexts want ToZero | Closed-system requires explicit choice; can't be implicit |
| Why explicit `SpecialValues` | NaN propagation, ±∞ handling, signed zero are all observable program facts that affect emission | Same as RoundingMode — explicit choice avoids hidden assumptions |
| Why `Precision = Unbounded \| IEEE754Width<N>` | Distinguishes "true ℝ" (Cauchy-complete; unbounded precision; impossible on hardware but useful as a substrate concept) from "IEEE 754 fp{32,64,128}" (bounded; hardware-realizable) | Refinement chain: `Real<N>` is `Real where precision = IEEE754Width<N>` |

**Sized**: M-L for this single substrate-introduction. The largest piece of NEW substrate in the lane.

### 6. Consumer migration of 8 types — mechanical cascade

3 direct (Int, UInt, Float at `integer.dag` + `float.dag`) + 5 inherited (Char, EpochMs, Duration, Milliseconds, Seconds at `types.dag`). Once `Int` becomes the Grothendieck-completion carrier over `Nat`, with its `AbelianGroup<Int>` witness, the inherited types automatically pick up the construction-chain `Int` — no per-type authoring needed.

**Sized**: M for direct (3 file edits); negligible for inherited (cascade).

## Refinement chain

Refinement composition applies bit-width bounds at any layer:

```dag
type Nat<N>  = Nat  where bits <= N         // bounded ℕ — N-bit unsigned
type Int<N>  = Int  where bits <= N         // bounded ℤ — N-bit signed
type Real<N> = Real where bits <= N         // bounded ℝ — N-bit IEEE 754

type Int8   = Int<8>     // alias for refinement
type Int128 = Int<128>
type Float32 = Real<32>  // IEEE 754 fp32: 23-bit mantissa + 8-bit exponent + 1 sign = 32 total
type Float64 = Real<64>
```

**Refinement-composition design call** (Substrate Mgr territory): how does `Int(0..)` parse?
- **Option A**: `Int(0..)` means "Int with low-bound 0" = abstract `Int` narrowed by range refinement (low-bound only; no upper bound) = effectively `Nat` (since ℕ is ℤ with low-bound 0)
- **Option B**: `Int(0..)` means "Int starting from 0" — same as Option A semantically
- **Option C**: `Int(0..N)` means width-refinement to `N` bits — different from low-bound

**PM recommendation**: keep `Int(low..high)` as range-refinement (per existing `dsl/std/types.dag:282` `Duration = Int where range(min: 0)` pattern); use `Int<N>` syntax for width-refinement; they compose: `Int<64>(0..1024)` = 64-bit Int narrowed to range [0, 1024]. Substrate Mgr verifies syntax composability at substrate level.

## Per-target grounding mapping

Target groundings consume refinements; one row per (target, refinement):

### Rust

| Refinement | Rust primitive | Cost-lens |
|---|---|---|
| `Int<8>` | `i8` | O(1) hardware arithmetic |
| `Int<16>` | `i16` | O(1) |
| `Int<32>` | `i32` | O(1) |
| `Int<64>` | `i64` | O(1) |
| `Int<128>` | `i128` | O(1) (compiler-supported) |
| `Int` (unbounded) | `num_bigint::BigInt` | O(n*m) for multiplication; O(max(n, m)) for add |
| `Nat<N>` | `u8`/`u16`/`u32`/`u64`/`u128` per N | O(1) |
| `Nat` (unbounded) | `num_bigint::BigUint` | O(n*m) for mul |
| `Real<32>` | `f32` | O(1) hardware FPU |
| `Real<64>` | `f64` | O(1) FPU |
| `Real<128>` | `f128` (Rust draft) OR `softfloat::F128` if hardware unavailable | O(1) hardware OR O(1) software-emulated with constant overhead |
| `Real` (unbounded — true ℝ) | NOT GROUNDABLE in Rust without external arbitrary-precision lib (e.g., `rug::Float`); design-call: bias to error-on-emission OR external-dep route | flag for Substrate Mgr |

### Python

| Refinement | Python primitive | Cost-lens |
|---|---|---|
| `Nat<N>` for any N | `int` | Native arbitrary-precision; size hint informs runtime checks but doesn't change carrier |
| `Int<N>` for any N | `int` | Same |
| `Int` / `Nat` (unbounded) | `int` | Naturally arbitrary-precision in Python |
| `Real<32>` / `Real<64>` | `float` (Python's float is C double = 64-bit IEEE 754) | f32 emits with explicit conversion; f64 is native |
| `Real<128>` | `decimal.Decimal` with 128-bit precision config OR external dep | design-call |
| `Real` (unbounded) | `decimal.Decimal` (configurable arbitrary-precision) OR external dep | design-call |

### Go

| Refinement | Go primitive | Cost-lens |
|---|---|---|
| `Int<8>`...`Int<64>` | `int8`...`int64` | O(1) |
| `Int<128>` | `math/big.Int` (heap-allocated) OR design-call: split-into-Word64-pair? | O(1) for split-pair; O(n*m) for math/big |
| `Int` (unbounded) | `math/big.Int` | O(n*m) for mul |
| `Nat<N>`...`Nat<128>` | `uint8`...`uint64`; `math/big.Int` for `>64` | O(1) for native widths |
| `Real<32>` / `Real<64>` | `float32` / `float64` | O(1) FPU |
| `Real<128>` | `math/big.Float` | O(1) software-emulated; configurable precision |
| `Real` (unbounded) | `math/big.Float` (configurable arbitrary-precision) | depends on configured precision |

**Cross-target consistency** is preserved: a `.dag` program reasoning about `Int<64>` gets `i64` / `int` / `int64` per target; a program reasoning about `Int` (unbounded) gets `BigInt` / `int` / `math/big.Int`. Cost-lens reads the refinement to derive realization cost; cross-target equivalence harness (R3 T-V-L5-Corpus) verifies semantic equivalence under the cost-bounded comparison.

## v2-refinement-syntax-blocker — path (a) coordination

Per [`dsl/std/types.dag:194-197`](../dsl/std/types.dag) MODELING NOTE:

> "Char = Brand('Char', Int) ... v2-compatible alias. The `where` constraints (brand, range) are future work — the v2 parser does not support field-level where syntax yet."

The refinement syntax (`Int<N>`, `Int where bits <= N`) is partially blocked on v2 parser limitations. Per [Director ratification at comment 4357696122](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357696122), **path (a) — v2 retirement lands in same wave** is the structurally honest default.

**Coordination shape** (Substrate Mgr ↔ PB Mgr):

| Step | Owner | Action |
|---|---|---|
| 1 | PB Mgr (T-V2-Retirement R3 lane #11) | T-FixedPoint + T-LensProducer-Retirement close → cascade-gate for v2 retirement opens |
| 2 | PB Mgr | T-V2-Retirement work begins (per `r3-pb-t-...-retirement` brief shape) |
| 3 | Substrate Mgr (T-Numeric-Construction) | While PB Mgr is dissolving v2, Substrate Mgr authors substrate-introductions (Magnitude, ApproximateField, Grothendieck disposition) — refinement syntax NOT YET REQUIRED at this stage |
| 4 | PB Mgr | T-V2-Retirement gates fire: `v2_oracle_no_remaining_test_consumers`, `v2_directory_deleted` |
| 5 | Substrate Mgr | v2 parser is gone; refinement syntax (`Int<N>`, `Int where bits <= N`) is now v3-native and unblocked |
| 6 | Substrate Mgr | T-Numeric-Construction worker dispatches consumer migration + refinement-chain authoring |

**Path (b) — author refinement syntax in v3 ahead of v2 retirement** is rejected per `feedback_no_textual_enforcement_bridges` + P5 dissolution discipline (creates a window where v2 cannot parse new substrate while v3 can; parallel-authority anti-pattern).

If v2 retirement schedule slips significantly, Substrate Mgr revisits path (b) as contingency, but path (a) is the default.

## Cost-lens implications per layer

The construction chain has different cost characteristics per layer that the cost-lens (T-CostLens-Composition R3 lane) must read:

| Layer | Operation | Cost (substrate-level) | Notes |
|---|---|---|---|
| **Magnitude** | (opaque carrier; no operations directly) | N/A | Refinements consume; Magnitude itself has no algebraic operations |
| **Nat = Semiring<Magnitude>** | `add(a, b)` | O(1) on the algebraic axiom; refinement determines target-realization cost | Per-target: O(1) for `Nat<64>` on hardware; O(max(a, b)) for unbounded `Nat` via BigUint |
| **Nat** | `mul(a, b)` | O(1) algebraic; target: O(1) for fixed-width; O(n*m) for arbitrary-precision | |
| **Int completion over Nat** | `add(a, b)` (via Grothendieck) | O(1) algebraic; same target cost as Nat | Sign handling adds constant factor; doesn't change asymptotic |
| **Int** | `negate(a)` | O(1) algebraic; O(1) target | Sign flip |
| **Rational = Field<Int>** | `mul(a, b)` (multiplying fractions) | O(1) algebraic; target: O(1) for fixed-width components; O(N) per gcd-reduction step | gcd dominates |
| **Rational** | `add(a, b)` (LCD'ing) | O(N) for LCD computation + O(1) for sum + gcd-reduction | |
| **Real = ApproximateField<Rational>** | All ops | Constant rounding overhead per op | IEEE 754: ~few cycles per op on FPU; software-emulated: ~100s of cycles |
| **Real** (unbounded) | All ops | Variable per configured precision | `decimal.Decimal` / `math/big.Float` / `rug::Float` — depends on precision config |

T-CostLens-Composition consumes these layer-cost facts to derive end-to-end program cost. Per `feedback_lenses_not_passes`: lens reads physics, no heuristics.

## What this design doc does NOT cover

These are deferred to Substrate Mgr 6Q audits + worker brief authoring:

1. **Concrete v3 syntax for refinement** (`Int<N>` declaration syntax, `where bits <= N` parser-grammar) — Substrate Mgr scope after T-V2-Retirement clears
2. **Per-target IntegerPrimitive row schema** — extension of existing `dsl/extdeps/languages/{rust,python,go}/primitives.dag` rows
3. **IntLit pipeline reshape** at parser/tokenizer level — separate slice; coordinates with T-V2-Retirement
4. **Cost-lens witness construction for ApproximateField** — T-CostLens-Composition lane consumer
5. **String audit** (separate Director-added scope) — closed by [`docs/audit/t-numeric-construction-string-audit-receipt.md`](audit/t-numeric-construction-string-audit-receipt.md): documented-no-change for `String` itself because `dsl/std/string_type.dag` already declares `String = FreeMonoid<Char>`; only `Char` remains in the inherited numeric-refinement scope.

## Cross-refs

- **Lane authority**: [`docs/r3-structure.md`](r3-structure.md) §"T-Numeric-Construction" (Lane #6, scope-reframed 2026-05-01)
- **Superseded brief**: `docs/briefs/t-int128-r3-initial-slice.md` (deleted in [PR #1364](https://github.com/gunb-ai/gunbc/pull/1364); pre-deletion content viewable at the PR's diff or `git show <PR-1364-parent>:docs/briefs/t-int128-r3-initial-slice.md`); scope absorbed into T-Numeric-Construction
- **Sibling**: [`docs/briefs/r2-substrate-cardinality-int-lit-worker.md`](briefs/r2-substrate-cardinality-int-lit-worker.md) — int-lit magnitude consumer (R2; this lane extends from i64-bounded to abstract Magnitude)
- **Director ratifications**:
  - [comment 4357560099](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357560099) — initial reframe proposal (3 direct bake-ins + abstract-plus-refinement shape)
  - [comment 4357565335](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357565335) — full audit addendum (8 types in scope)
  - [comment 4357570783](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357570783) — construction-chain refinement (PM)
  - [comment 4357686725](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357686725) — initial Director ratification (`OrderedRing<Magnitude>` shorthand; SUPERSEDED)
  - [comment 4357696122](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357696122) — addendum acknowledged; path (a) v2 same-wave
  - [comment 4357704426](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4357704426) — **canonical**: construction chain ratified verbatim; lane name lock; PM authors design doc + amendment
- **Modeling philosophy**: `feedback_modeling_philosophy`, `feedback_compositional_not_templating`, `feedback_naming_is_aliasing`, `feedback_no_metadata_markers`, `feedback_substrate_principle_audit`, `feedback_construction_over_ratchets`, `feedback_lenses_not_passes`
- **R2 substrate kindred**: `Secret<T>` (nominal-opaque graduation), `Dimension<Carrier>` (phantom-parameter refinement) — same shape (abstract carrier + refinement)
- **INVARIANTS**: §P5 (dispatch-discipline) + §P2 (no parallel authority)
- **Adjacent**: `Json` / `Bytes` opaque kernel types — separate substrate-completion class; ROADMAP `### Post-merge debt` row tracks; out of scope for this lane
