---
status: draft (worker brief; ENGAGE-NOW parallel with S3 per Substrate Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074769)
authority parent: R3 Substrate Manager (#1739)
ratification: parallel-author authorized per Substrate canvas B2 + Grounding G2 + Substrate Mgr partition; `MachineConstraint<C>` and `ApproximateField<F>` are independent axes (machine width vs algebra approximation)
roadmap row: T-Numeric-Construction (Lane B2) + §1.8 ledger rows #17-#24
authority docs:
  - docs/r3-design-schedule-2026-05-06.md §1 S8
  - Substrate canvas B2 (post at gunbc#846 #issuecomment-4385074769) — partition response naming S8 parallel with S3
  - docs/r3-structure.md (T-Numeric-Construction lane row)
gates:
  - one or more of §1.8 ledger rows #17-#24 (T-Numeric-Construction 8 gates) — specifically the float-row gates
---

# R3 Substrate S8 — `ApproximateField<F>` Float migration worker brief

## Context

Float32/Float64 currently model as `Field<Word32>` / `Field<Word64>`
(or equivalent concrete carrier). Per Substrate canvas B2 + Grounding
G2 audit: the modeling is wrong on two axes —

1. **Algebra**: floats do not satisfy `Field` axioms (associativity
   fails under rounding; identity fails for `+0.0` vs `-0.0`;
   distributivity fails). They satisfy a weaker structure —
   `ApproximateField<F>` — which is the substrate target.
2. **Width independence**: `Word32` / `Word64` is the wrong axis.
   Per S3 (`MachineConstraint<C>` carrier), machine width is an
   independent axis from algebra. Float32 = `Real` ×
   `MachineWidth<32>`; Float64 = `Real` × `MachineWidth<64>`, where
   `Real = ApproximateField<FieldOfFractions<Int>>`. In
   `ApproximateField<F>`, `F` is the exact-field carrier slot
   (`FieldOfFractions<Int>` for IEEE-754 real approximations). The
   witness alias `Rational = Field<FieldOfFractions<Int>>` describes
   the exact field over that carrier; it is not the `ApproximateField`
   type argument. Other approximation regimes may introduce different
   exact-field carriers, such as a future complex carrier.

This brief is **parallel with S3** per Substrate Mgr partition
response 2026-05-06 — they're independent axes. Brief cross-references
S3 carriers; consumer work synthesizes both axes at the
`Compose&lt;Algebra, MachineConstraint&gt;` interaction-lookup substrate.

## Slice

### Phase 1 — `ApproximateField<F>` carrier landing

1. **Author `ApproximateField<F>` declaration** in
   `dsl/std/algebraic_structures.dag` (or co-located with `Field` /
   `Ring` / `Group` if such file exists; worker greps for canonical
   location at dispatch).

   Practice 4 classification: **🟡 SCAFFOLD** — the axiom-set
   relaxation from `Field` is hand-modeled until property-based
   tests (T-Tests-As-Data-Completeness `ForAll` quantifier substrate)
   land that derive `ApproximateField` from named relaxation rules
   (associativity-up-to-rounding etc.). Named dissolution trigger:
   `forall_exists_quantifier_substrate_landed` (§1.8 ledger row).

   Surface **Q-ApproximateField-Axiom-Set** for Director ratification:
   - Which axioms relax (associativity / identity / distributivity)?
   - Which retain (commutativity / closure)?
   - Are rounding-mode parameters part of the carrier or a separate axis?

2. **Author exact-field carrier / `Rational` witness prerequisites** if not landed:

   ```
   data Rational  // 🟢 PRIMITIVE — algebraic rationals; structural fact
   ```

   The `F` parameter on `ApproximateField<F>` is the field being
   approximated. Per Q-MachineConstraint sub-decision 4 RATIFIED
   (gunbc#828 #issuecomment-4385530115): `Real<64>` ≡
   `Compose<Real, MachineWidth<64>>`, with `Real =
   ApproximateField<FieldOfFractions<Int>>` —
   `FieldOfFractions<Int>` is the exact-field carrier, `MachineWidth<N>` is the machine
   side, and `ApproximateField<...>` carries the algebra
   approximation. Both algebra-approximation and machine-
   approximation compose as independent axes per S3 ratified
   `Compose<Algebra, MachineConstraint>`.

   Phase-1 lands `Rational` only; `Real` is NOT a primary
   substrate entity (concrete `Real<N>` types emit from the
   `Compose<...>` interaction).

### Phase 2 — Float consumer migration

1. **Catalog existing `Field<Word32>` / `Field<Word64>` references**
   at HEAD via grep across `dsl/std/`, `src/v3/std/`, `src/v3/compiler/`,
   `src/v3/grounding_*`. Worker re-greps at dispatch; existing memory
   notes record `Field<Word*>` shape but state may have drifted.

2. **Migrate each consumer** from `Field<Word32>` /
   `Field<Word64>` → parametric `Compose<Real,
   MachineWidth<N>>` per S3 sub-decision 2 RATIFIED. Consumers
   fall into two classes:
   - **Algebra-only consumers** (e.g., type inference, lens
     analysis): consume `Real = ApproximateField<FieldOfFractions<Int>>`; do not need
     machine width.
   - **Emission consumers** (e.g., Grounding Rust target): consume
     the parametric instantiation `Compose<Real,
     MachineWidth<N>>` (S3 carrier).

3. **Cross-reference S3** parametric instantiations:

   ```
   Float<32> ≡ Compose<Real, MachineWidth<32>>  // → Rust f32
   Float<64> ≡ Compose<Real, MachineWidth<64>>  // → Rust f64
   ```

   These are **demonstration entries** for Class 1 Pass criterion
   (≥3 algebra × constraint pairs); float entries are ON TOP of
   S3's int entries (Int × Width<32> → i32, etc.). Combined float
   + int demonstrations close Phase-3 of S3.

### Phase 3 — Real / base-carrier convention documentation

1. **`docs/modeling-discipline.md` patch** documenting the
   "approximation regime" pattern: `ApproximateField<F>` × machine
   axis × rounding-mode (if separate) compose to the concrete
   primitive. Establishes pattern for future `ApproximateRing<F>`,
   `ApproximateGroup<G>` extensions.

2. **Q-ApproximateField-Rounding-Mode** — surface for Director
   ratification: is rounding mode part of `ApproximateField<F>`
   (parametrized) or a separate `RoundingMode<R>` axis (composing
   like `MachineConstraint<C>`)? Recommendation: separate axis,
   following the "independent axes" thesis from Brian directive
   (algebra / machine / rounding all independent).

## Acceptance

- `ApproximateField<F>` declaration landed with Practice 4 SCAFFOLD
  classification receipt + named dissolution trigger.
- `Real` base-carrier landed (PRIMITIVE) if not already present.
- `Field<Word32>` / `Field<Word64>` consumer migration complete;
  zero remaining references in non-test, non-archive `.dag` /
  `.rs` source.
- `Compose&lt;Algebra, MachineConstraint&gt;` interaction entries for Float32 / Float64
  added (cross-references S3 substrate; coordinate timing —
  S3 must land carriers Phase-1 first).
- Q-ApproximateField-Axiom-Set + Q-ApproximateField-Rounding-Mode
  surfaced in PR body for Director ratification.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- §1.8 ledger float-row gates advance from DECLARED → CONSUMER_LANDED.
- ROADMAP T-Numeric-Construction Float row updates from PARTIAL →
  Retired-with-residual (residual: rounding-mode axis pending
  Q-ApproximateField-Rounding-Mode resolution if Director carries
  to follow-up).

## STOP-AND-ESCALATE

- **`Field<Word32>` / `Field<Word64>` consumers in v2 compiler
  paths** that block migration cleanly: the lane intersects
  T-V2-Retirement (PB Mgr P3). STOP; coordinate cross-program
  with PB Mgr (#1742) on sequencing. Float migration may need
  to wait for v2 retirement of consumer paths, or v2 retirement
  may need to consume migrated paths first.
- **Property-based test substrate (`ForAll` quantifier) lands
  during this brief's wait window**: dissolution trigger fires
  for SCAFFOLD classification; revisit `ApproximateField<F>`
  classification (🟡 → 🟢 if axiom set fully derived). Coordinate
  with Verification Mgr (V4 T-Tests-As-Data-Completeness).
- **S3 `MachineConstraint<C>` shape lands differently than
  this brief assumes** (e.g., `MachineWidth<bits>` carrier
  renamed or restructured): re-frame Phase-2 consumer migration
  to match S3 ratified shape. Brief assumes `MachineWidth<bits>`
  + `Compose&lt;Algebra, MachineConstraint&gt;` table per S3 Phase-1; if S3 shifts,
  this brief shifts.
- **Director ratifies `ApproximateField<F>` is not the right
  algebra-side shape** (e.g., prefers `IEEE754Float<N>` or
  `BoundedField<F, ε>`): STOP. Re-canvas; carrier authoring
  follows ratification.
- **Rounding mode emerges as load-bearing for ≥1 emission
  consumer** (e.g., complexity lens needs rounding-mode-aware
  cost model): Q-ApproximateField-Rounding-Mode becomes
  blocking; surface to Director immediately, do not bridge
  with default rounding mode.
- **`Real` carrier conflicts with existing declaration** at
  HEAD: re-use existing carrier; re-frame Phase-1 step 2 as
  consumer migration not landing.

## Authority audit receipt

1. **Substrate exists?** Per memory + draft-time grep:
   `Field<Word32>` / `Field<Word64>` references exist (memory
   notes "Word*" usage). `ApproximateField<F>` does NOT exist
   in `dsl/std/`. `Real` carrier may or may not exist —
   worker greps at dispatch. No new top-level carrier conflict
   anticipated.
2. **Existing brief?** None for `ApproximateField<F>` migration.
   T-Numeric-Construction lane covers Float migration as part
   of broader scope (S9); this brief is the focused float-migration
   slice. Brief co-references S9 (T-Numeric-Construction full
   13-type scope); S8 is the float-specific carrier landing.
3. **Design-doc match?** No design-doc precedent specifically
   for `ApproximateField<F>`. Substrate canvas B2 + Grounding
   G2 audit are the originating authority. Worker re-reads
   audit notes at dispatch.
4. **Citations live?** Substrate Mgr partition response cited
   at gunbc#846 #issuecomment-4385074769 — verified at HEAD
   2026-05-06.
5. **Carrier dissolves the bridge?** Yes — `Field<Word*>` is
   a substrate-modeling fault (Field axioms fail for floats;
   width is wrong axis). `ApproximateField<F>` × `MachineWidth<N>`
   product correctly models the algebra-machine factoring.
   Migration retires the faulty carrier and lands the correct
   one. Cementing test (Phase 3 PR documentation) anchors the
   pattern.

## Provenance

Drafted 2026-05-06 per R3 design schedule §1 S8 (PR #1810) +
Substrate Mgr partition response 2026-05-06 (gunbc#846
#issuecomment-4385074769) authorizing parallel-author with S3.
Cross-references S3 (`MachineConstraint<C>`) carrier brief —
both briefs are Mgr-tier design now; cross-reference at
brief-landing.
