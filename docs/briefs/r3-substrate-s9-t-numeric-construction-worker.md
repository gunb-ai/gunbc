---
status: draft (Mgr-tier worker brief; ENGAGE-NOW parallel-author authorized per Substrate canvas B2 ratified)
authority parent: R3 Substrate Manager (#1739)
ratification: B2 ratified per Substrate Mgr canvas + Director ratification of T-Numeric-Construction lane scope
roadmap row: T-Numeric-Construction (REFRAMED 2026-05-01 from T-Int128) + §1.8 ledger rows #17-#24
authority docs:
  - docs/r3-design-schedule-2026-05-06.md §1 S9
  - docs/r3-structure.md (T-Numeric-Construction lane row + 8 gates)
  - Substrate canvas B2 (parallel-author authorization)
  - docs/r3-program-plan.md §1.8 rows #17-#24
gates:
  - 8 T-Numeric-Construction gates (§1.8 rows #17-#24) including:
    - int_construction_landed
    - uint_construction_landed
    - float_construction_landed (paired with S8)
    - char_refinement_landed (Int-inherited)
    - epoch_ms_refinement_landed (Int-inherited)
    - duration_refinement_landed (Int-inherited)
    - http_status_refinement_landed (Int-inherited)
    - port_refinement_landed (Int-inherited)
  - cross-references: numeric_construction_demonstration (#67 — folded into Acceptance)
---

# R3 Substrate S9 — T-Numeric-Construction Mgr-tier worker brief

## Context

T-Numeric-Construction (REFRAMED 2026-05-01 from prior T-Int128 lane)
covers the full numeric-primitive construction surface: 13 in-scope
types + Slice 2 Nat-alignment migration. Per B2 canvas + Substrate
Mgr partition response 2026-05-06, this brief authorizes parallel
authoring NOW (foundational; no Evaluator gating).

**Scope inventory (13 types)**:
- **Direct** (3): `Int`, `UInt`, `Float`
- **Int-inherited refinements** (10): `Char`, `EpochMs`, `Duration`,
  `Milliseconds`, `Seconds`, `RetryCount`, `HttpStatus`, `Port`,
  `PositiveInt`, `NonNegativeInt`

**Cross-references**:
- **S3** `MachineConstraint<C>` — machine-axis carriers; Int /
  UInt / Float concrete primitives emit via `AlgebraMachineProduct`.
- **S8** `ApproximateField<F>` — Float-specific algebra carrier.
  S8 owns Float migration; this brief covers Int / UInt + 10
  refinements.

## Slice

This is **L** sized — multi-PR delivery. Worker stages into 4
phases below; Substrate Mgr ratifies phase-by-phase.

### Phase 1 — Int / UInt direct construction (gates #17 / #18)

1. **`Int` algebra carrier — group-completion of `Nat`-monoid**.
   Author or verify `Int` declaration as projection of algebraic
   structure + machine axis (`MachineConstraint<C>`). Per
   substrate-modeling discipline: `Nat` is a commutative monoid
   under addition (closure / associativity / identity / commutativity;
   no inverses). `Int` is the additive Abelian group on integers
   (carrier `Z`); the algebraic relationship to `Nat` is **explicit
   group-completion** (Grothendieck construction —
   `Int ≡ GroupCompletion<CommutativeMonoid<Nat>>`), NOT a
   parameterization `AbelianGroup<Nat>` (which is unfaithful — Nat
   does not satisfy group axioms; additive inverses are not
   representable in Nat). Worker EITHER models `Int` directly as
   `AbelianGroup` (terminal Abelian-group instance over `Z`,
   `Nat` not a parameter) OR introduces explicit
   `GroupCompletion<M>` substrate carrier consuming
   commutative-monoid `M`. Decision lands in Phase-1 PR body
   per P1 substrate-fact-introduction procedure.
   Practice 4 classification: 🟢 PRIMITIVE if algebra-only;
   🟡 SCAFFOLD if machine-axis composition is hand-modeled.
2. **`UInt = Monoid<Nat>` algebra carrier**. Companion to `Int`;
   non-negative monoid structure.
3. **Concrete emission entries** in `AlgebraMachineProduct`
   (cross-program with Grounding Mgr):
   - `Int × MachineWidth<32> → Rust i32`
   - `Int × MachineWidth<64> → Rust i64`
   - `Int × MachineWidth<128> → Rust i128`
   - `UInt × MachineWidth<32> → Rust u32`
   - `UInt × MachineWidth<64> → Rust u64`

   Phase-1 lands the substrate; emission consumer landing is
   Grounding Mgr G2 (T-Ground-Rust).

### Phase 2 — Float construction (gate #19; coordinate with S8)

This phase is **owned by S8** — `ApproximateField<F>` migration
+ `Real` base-carrier + `Float = ApproximateField<Real> × MachineWidth<N>`.
This brief references S8 closure rather than re-doing the work.
Worker confirms S8 has landed before declaring Phase 2 complete.

### Phase 3 — Int-inherited refinements (gates #20-#24)

Refinements are SECONDARY-PARAMETER projections of `Int`. Per
modeling-discipline P1: refinement = predicate over base type, not
new substrate fact. Each refinement lands as:

```
data Char = Refined<Int, valid_unicode_codepoint>
data EpochMs = Refined<Int, milliseconds_since_epoch>
data Duration = Refined<Int, non_negative>
data Milliseconds = Refined<Duration, milliseconds_unit>
data Seconds = Refined<Duration, seconds_unit>
data RetryCount = Refined<NonNegativeInt, retry_semantics>
data HttpStatus = Refined<Int, range_100_599>
data Port = Refined<UInt, range_0_65535>
data PositiveInt = Refined<Int, gt_zero>
data NonNegativeInt = Refined<Int, gte_zero>
```

(Exact `Refined<Base, predicate>` shape depends on landed
`TypeExpr::Refined` substrate — annotation elimination Wave 1
landed the AST per memory note 2026-02-26.)

Practice 4 classification: each refinement is 🟢 PRIMITIVE
(structural refinement of base type; no new sum-type variants).

### Phase 4 — Slice 2 Nat-alignment migration

Per B2 canvas + roadmap: existing `Nat` references in substrate
need migration to canonical Nat-via-PeanoNat or Nat-via-Refined<Int, gte_zero>
shape. Worker:

1. Greps existing `Nat` references at dispatch.
2. Catalogs which use Nat as algebra-axis-carrier (correct usage)
   vs which use Nat as concrete primitive (drift; should migrate
   to `UInt × MachineWidth<N>` product).
3. Migrates concrete-primitive uses to product form. Algebra-axis
   uses are preserved (Nat as commutative-monoid carrier in
   group-completion / `CommutativeMonoid<Nat>` shapes — NOT
   `AbelianGroup<Nat>`, which would be unfaithful).

## Acceptance

- 13 in-scope types landed per Phase mapping (3 direct + 10
  refinements; Phase 2 = S8 cross-coordination).
- Slice 2 Nat-alignment migration complete; no concrete-primitive
  Nat references in non-test source outside algebra-axis usage.
- Practice 4 classification receipts for each new declaration.
- Cross-program receipts with Grounding Mgr (#1745) for emission
  consumer entries in `AlgebraMachineProduct`.
- §1.8 gates #17-#24 advance DECLARED → CONSUMER_LANDED phase by
  phase.
- §1.8 gate #67 (`numeric_construction_demonstration`) folded into
  Acceptance per Substrate Mgr partition response 2026-05-06: the
  end-to-end `Int<32>` + `Real<64>` round-trip demonstration is an
  Acceptance bullet on this brief, not a separate dispatch.
  Demonstration runs via E6-G0d evaluator + Grounding Rust emission.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- ROADMAP T-Numeric-Construction row updates Partial → Retired
  upon all 4 phases landing; residual rows for any deferred
  refinement (e.g., if Refined<...> substrate lacks predicate
  expressivity for a specific refinement, that one defers).

## STOP-AND-ESCALATE

- **`MachineConstraint<C>` (S3) does not land before Phase-1
  emission entries**: this brief depends on S3 substrate for
  `AlgebraMachineProduct` interaction table. Phase-1 algebra
  carriers can land independently; emission entries STOP until
  S3 lands. Coordinate sequencing with Substrate Mgr (#1739).
- **`ApproximateField<F>` (S8) doesn't land before Phase-2
  Float demonstration**: Phase 2 is S8-owned. STOP if S8 work
  blocks; Phase 2 reopens upon S8 landing.
- **Refined<Base, predicate> substrate not expressive enough**
  for a specific refinement (e.g., `EpochMs` predicate
  "milliseconds-since-epoch" cannot be expressed as a closed
  predicate over `Int`): re-frame as algebra-axis (i.e.,
  `EpochMs` is a separate algebra, not a refinement). STOP;
  surface to Substrate Mgr.
- **v2-refinement-syntax-blocker reopens** (per
  `r3-structure.md` T-Numeric-Construction internal cascade
  on T-V2-Retirement): coordinate with PB Mgr (#1742) on
  v2 retirement sequencing. T-Numeric-Construction has named
  T-V2-Retirement-landing-first dependency for path-(a)
  v2-refinement-syntax-blocker resolution.
- **Practice 4 classification flags a refinement as
  hand-declared mirror of compiler-internal type**: STOP.
  Per modeling-discipline P1, hand-declared mirrors require
  named dissolution trigger. Author the dissolution trigger
  before landing the SCAFFOLD-marked carrier.
- **`Char` refinement requires unicode-codepoint validity
  predicate that exceeds Refined<...> substrate**: defer
  to a `Char`-specific carrier (e.g.,
  `data Char = UnicodeCodepoint<u32>`) following P1 procedure.

## Authority audit receipt

1. **Substrate exists?** Algebra carriers (`Group`, `AbelianGroup`,
   `Monoid`) likely exist in `dsl/std/algebraic_structures.dag`
   (worker re-greps at dispatch). `Int` / `UInt` may exist as
   concrete primitives needing migration; `Refined<Base, predicate>`
   substrate landed per annotation-elimination Wave 1 (memory note
   2026-02-26). The 10 Int-inherited refinements may have partial
   landed surfaces (e.g., `HttpStatus`, `Port` may exist in HTTP
   domain modules); worker greps + reframes as migration where
   applicable.
2. **Existing brief?** S3 (`MachineConstraint<C>`) and S8
   (`ApproximateField<F>` Float migration) are companion briefs.
   This brief is the broader T-Numeric-Construction parent;
   S3 + S8 cover specific axes. Brief explicitly references
   both; sequencing handoffs documented in STOP-AND-ESCALATE.
3. **Design-doc match?** No design-doc specifically for
   T-Numeric-Construction; lane reframe (2026-05-01 from
   T-Int128) is the originating authority. Substrate Mgr canvas
   B2 + roadmap row are the design surface.
4. **Citations live?** `r3-design-schedule-2026-05-06.md §1 S9`
   verified at HEAD 2026-05-06.
5. **Carrier dissolves the bridge?** Yes — current substrate
   models `Int` / `UInt` / `Float` as primary primitives
   (carrier-modeling fault per Brian directive). This brief
   reframes them as products of algebra × machine axes.
   Refinements consume `Refined<Base, predicate>` substrate
   consistently. Cementing demonstration is gate #67
   (numeric_construction_demonstration), folded into Acceptance.

## Provenance

Drafted 2026-05-06 per R3 design schedule §1 S9 (PR #1810) +
Substrate Mgr canvas B2 ratified for parallel authoring.
Coordinates with S3 + S8 cross-axis briefs. Worker pin TBD
(idle pool: loyal-wolf-828 reserved for S7 PR-F; valiant-ant-72
reserved for S3 implementation; smart-ram-167 reserved for S11
Slice C). T-Numeric-Construction worker pin assignment surfaces
post-S3/S8 dispatch as idle pool refreshes.
