# T-Numeric-Construction — `FieldOfFractions<R>` 6Q substrate-introduction audit

**Lane:** R3 #6 (T-Numeric-Construction). **Authority:** [`docs/design-numeric-construction.md`](../design-numeric-construction.md). **Subject:** `FieldOfFractions<R>` — a new algebra-surface declaring "the carrier of the field of fractions derived from an integral domain `R`." Prerequisite for the Slice 4 alias-pivot `type Rational = Field<FieldOfFractions<Int>>`.

**Why this audit exists.** Slice 4's first attempt at `type Rational = Field<Int>` ([#1470](https://github.com/gunb-ai/gunbc/pull/1470)) was reverted after Director ratified the same M9 modeling-faithfulness finding the substrate has now seen for both `AbelianGroup<Nat>` and (here) `Field<Int>`: under the standard parametric reading of `Field<T>`, `T` is the carrier of field operations including `reciprocal: fn(T) -> T`. With `T = Int`, this asserts `reciprocal: fn(Int) -> Int` — denotationally false. Only ±1 are units in ℤ; reciprocal is undefined for every other integer (0 included). The field of fractions ℚ derived from ℤ has a carrier *distinct from* `Int`.

Director's decision (inbox #1288 [#4362643231](https://github.com/gunb-ai/gunbc/issues/1288#issuecomment-4362643231)): keep `Field<T>` standard (T is the carrier; reciprocal/division close over T); introduce a separate `FieldOfFractions<R>` algebra-surface (analogous to `GroupCompletion<M>` from [#1448](https://github.com/gunb-ai/gunbc/pull/1448)) that is honest about taking an integral domain `R` and producing the carrier of its field of fractions. Then Slice 4 honestly becomes `type Rational = Field<FieldOfFractions<Int>>`.

**Hard boundaries (per dispatch):**
- "No pair/quotient numerator-denominator representation unless explicitly ratified."
- "No NonZero/Result reciprocal semantics in this prerequisite unless the audit chooses that path."
- "Prefer the opaque derived-carrier construction first, mirroring GroupCompletion."

This audit captures the algebra surface, not the carrier representation. Carrier representation is per-target grounding (Rust `num_rational::Rational`, Python `fractions.Fraction`, etc. — emission selects).

## Design call — what is `FieldOfFractions<R>`?

Three candidate shapes (parallel to the GroupCompletion audit at `docs/audit/t-numeric-construction-group-completion-6q.md`):

| Shape | Declaration | Tradeoffs |
|---|---|---|
| **A — algebra-only** | `type FieldOfFractions<R> { /* field ops over a derived carrier; carrier left unspecified */ }` | Risk: substrate has no language for "carrier derived from R" without committing to representation. |
| **B — algebra-with-explicit-derived-carrier** | `type FieldOfFractions<R> { carrier: ?, add, zero, mul, one, reciprocal, ... }` | Pulls in carrier-representation facts the design doc and dispatch explicitly reject (numerator/denominator pair). |
| **C — abstract atom + algebra-witness pattern** | `type FieldOfFractions<R>` (opaque atom; no fields) + a separate inhabitance witness pattern `type Rational = Field<FieldOfFractions<Int>>` at the use site | Closest to GroupCompletion's shape. The atom names the construction; the Field witness over the abstract `FieldOfFractions<R>` carrier is structurally honest because the carrier is opaque, not Int. |

**Recommendation: Shape C as a carrier construction** — opaque-atom `type FieldOfFractions<R>` parameterized by an integral-domain type, denoting the **carrier** of "the field of fractions derived from `R`." `FieldOfFractions<R>` is **not** a field-with-derived-carrier; it is the carrier alone. The algebra witness is named separately at the use site as standard `Field<T>` with `T = FieldOfFractions<R>`.

This matches:
- `GroupCompletion<M>` precedent (#1448) — exact same shape and design rationale; Slice 4 prerequisite mirrors Slice 3 prerequisite.
- Slice 1's `Magnitude` precedent (abstract opaque atom; carrier shape at the algebraic-axiom layer; algebra inhabitance attached at use sites).
- Design doc §"The construction chain" — ℚ as field of fractions is exactly this shape.
- Director's "keep `Field<T>` standard: T is the carrier of field operations" boundary (inbox #1288 [#4362643231](https://github.com/gunb-ai/gunbc/issues/1288#issuecomment-4362643231)).
- Director's "no pair/quotient numerator-denominator representation in this prerequisite" boundary.

**Canonical Slice 4 form:** `type Rational = Field<FieldOfFractions<Int>>`.

Type-correctness under the standard parametric reading:
- `T = FieldOfFractions<Int>` is an opaque atom denoting "the field-of-fractions carrier derived from Int."
- `Field<T>`'s structural shape `{ add, zero, negate, mul, one, reciprocal, compare }` instantiates over `FieldOfFractions<Int>`. `reciprocal: fn(FieldOfFractions<Int>) -> FieldOfFractions<Int>` is honest by construction — every non-zero element of the field-of-fractions carrier has a multiplicative inverse (the field axioms are total over the carrier; the zero element is structurally distinguished but reciprocal-undefined-at-zero is the same gap any concrete field has, addressed at the runtime/refinement layer rather than the algebra-surface layer).
- The derivation rule (how `R`'s integral-domain structure produces the field of fractions) lives in a future inhabitance lens, not in the substrate shape.

**Q6 single-authority resolution.** The simpler form `type Rational = FieldOfFractions<Int>` (collapsing carrier + algebra into a single named type, treating `FieldOfFractions<R>` as both the derived carrier and an implied Field algebra witness) admits two structurally distinct shapes for the same Slice 4 fact and would violate Q6 representation-duality. **Rejected** by analogy with the GroupCompletion audit's identical Q6 resolution. Canonical form is the explicit two-step `Field<FieldOfFractions<Int>>`: `FieldOfFractions<R>` is **only** the carrier, and the algebra witness is **only** the standard `Field<T>`.

## The 6 questions

### Q1 — Cardinality invariants
Does the type admit `[]` when invariant says ≥1, or singletons when ≥2?

**Answer: N/A under Shape C — opaque atom with no fields.** PASS by construction.

### Q2 — Index/handle types
Does a raw `Int` / `NodeId` encode something with a domain restriction?

**Answer: N/A under Shape C.** No fields. The single type-parameter `<R>` is a substrate type-reference, not a raw index. PASS.

### Q3 — Duplicated fact
Does Field A duplicate what's derivable from Field B?

**Answer: PASS.** Under Shape C, `FieldOfFractions<R>` shares no structural surface with existing carriers. It is parametrically distinct from:
- `Field<T>` (standard parametric reading: T is the field's carrier; not the same concept as "derived from T").
- `GroupCompletion<M>` (different category — Grothendieck construction over commutative monoid → abelian group; localization construction over integral domain → field of fractions; both opaque atoms but parameterized over different algebra layers).
- `Magnitude`, `Nat`, `Int` (concrete carrier names, not parametric constructions).
- `Word*` (storage carriers).

The construction is named exactly once in std; future Slice 4 (`type Rational = Field<FieldOfFractions<Int>>`) is the unique consumer for this slice.

### Q4 — Coproduct compression
Does one variant compress N distinct causes that downstream needs to distinguish?

**Answer: N/A under Shape C (opaque atom; not a sum type).** PASS.

### Q5 — Construction authority
Are multiple call sites independently constructing the same fact?

**Answer: PASS.** Single declaration in `dsl/std/algebra.dag` (mirroring GroupCompletion's preferred home). No consumers in this prerequisite slice. Slice 4 (post-prerequisite) consumes via `type Rational = Field<FieldOfFractions<Int>>` as the unique authority for ℚ-as-field-of-fractions-of-ℤ.

### Q6 — Representation duality
Can the same fact be expressed in two structurally different shapes?

**Answer: PASS for Shape C.** The opaque-atom form has exactly one structural shape. The compact alternative `Rational = FieldOfFractions<Int>` is named explicitly and rejected (above). The carrier-representation choice (numerator/denominator pair vs decimal-fraction vs continued-fraction etc.) is deferred to per-target emission, not exposed in std/.

## Constrained-inhabitance gap (P5 tracked scaffold)

Same shape as GroupCompletion's audit:

`FieldOfFractions<R>` is **denotationally** parameterized over "an integral domain `R`" — the localization construction is well-defined only when `R` carries the integral-domain laws (commutative ring with no zero divisors). The current substrate has no parametric where-clause syntax (`<R> where R : IntegralDomain<_>`), so the recommended Shape C declaration accepts any type-reference for `<R>` at the parser/lower level.

**Bounded denotationally:** Slice 4's only intended consumer is `FieldOfFractions<Int>`, and `Int = AbelianGroup<GroupCompletion<Nat>>` (Slice 3) carries the abelian group structure under addition; with multiplication inherited from the Nat semiring path, `Int` is denotationally a commutative ring with no zero divisors (an integral domain). The single intended consumer denotationally satisfies the precondition.

**Dissolution trigger:** when constrained-inhabitance / parametric where-clause syntax lands in the substrate (separate substrate-feature lane, not T-Numeric-Construction's scope), tighten `FieldOfFractions<R>` to require `R : IntegralDomain` (or `R : CommutativeRing` with appropriate refinement). Existing call sites `Field<FieldOfFractions<Int>>` continue to type-check.

**Mitigation in this slice:** the substrate-introduction PR for `FieldOfFractions<R>` lands alongside a structural ratchet pinning `FieldOfFractions<Int>` as the only intended consumer at bootstrap time — making the denotational precondition observable as a use-site fact rather than an unconstrained parametric admission. Same pattern as GroupCompletion's audit.

## Where does it live?

Recommend `dsl/std/algebra.dag` for proximity to existing algebra-construction surfaces (`GroupCompletion<M>` already lives there per the prior audit's preferred-home recommendation, and `Field<T>` is the consumed algebra). Cost-of-change: one file edit; bootstrap regen.

## What this audit does NOT cover

- **Carrier representation.** Per Director's hard boundary, no numerator/denominator pair. Per-target representation is emission's job (Rust `num_rational::Rational`, Python `fractions.Fraction`, etc.).
- **Algebra inhabitance proof.** Whether `FieldOfFractions<R>` mechanically derives a `Field` witness over its carrier is a follow-up modeling question; this audit pins the substrate shape, not the inhabitance lens.
- **Slice 4 alias-pivot edit.** Authoring `type Rational = Field<FieldOfFractions<Int>>` is the post-prerequisite slice; this audit gates that.
- **Refinement syntax.** `Rational<N>` for bounded precision is gated on T-V2-Retirement.
- **Parametric where-clause / constrained inhabitance.** Required to structurally enforce `<R> : IntegralDomain`. Tracked above as a substrate-feature gap.
- **Reciprocal/division refinement.** The prior #1370 audit's executable-completeness gap (reciprocal undefined at zero) is a runtime-layer concern; the substrate-shape question this audit answers is whether the algebra-surface CAN be honestly expressed (yes, via the derived-carrier opaque atom).

## Verdict

**PASS — proceed with `type FieldOfFractions<R>` opaque-atom (Shape C) as a substrate-introduction PR.**

- Sized: **S** — single new type declaration in `dsl/std/algebra.dag`. No fields, no algebra-inhabitance shape declared at this layer (left to future inhabitance lens). Bootstrap regen.
- Hard boundaries: no carrier-representation facts; no NonZero/Result reciprocal semantics; no migration of `Float`-backed surface; no tokenizer/literal-grammar work.
- Once landed, Slice 4 becomes a one-line edit in a new `dsl/std/rational.dag` (`type Rational = Field<FieldOfFractions<Int>>`) plus structural ratchet — no further substrate-introduction work needed.

## Cross-refs

- `docs/design-numeric-construction.md` §"The construction chain" (layer 4: ℚ as field of fractions of ℤ).
- `docs/audit/t-numeric-construction-group-completion-6q.md` (sibling audit; identical pattern; landed in #1448).
- `docs/audit/t-numeric-construction-magnitude-6q.md` (Slice 1 audit; opaque-atom precedent).
- `feedback_substrate_principle_audit` (the 6Q rule).
- `feedback_compositional_not_templating`, `feedback_naming_is_aliasing` (refinement-as-child rationale).
- Director ratifications:
  - inbox #1288 #4362611659 (Slice 4 alias-pivot dispatch; preferred path).
  - inbox #1288 #4362643231 (Option 2 substrate-split decision after M9 finding; this audit's authority).
- M9 reviewer findings on PR #1470:
  - https://github.com/gunb-ai/gunbc/pull/1470 (the reverted alias-pivot attempt).
- Prior `Field` verification audit ([#1370](https://github.com/gunb-ai/gunbc/pull/1370)) — acknowledged the executable-completeness gap (reciprocal undefined at zero); this audit is the substrate-shape companion.
