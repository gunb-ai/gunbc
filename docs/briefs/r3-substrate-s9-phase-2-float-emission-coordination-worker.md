---
status: draft (worker brief; queued — dispatch gates on BOTH S8 `ApproximateField<F>` carrier landing AND S9 Phase-1 Step 3 emission-entries pattern landing)
authority parent: R3 Substrate Manager (#1739)
ratification: dispatchable per Q-MachineConstraint sub-decisions RATIFIED at gunbc#828 #issuecomment-4385530115; cross-program with Grounding Mgr (#1745); coordinates with S8
roadmap row: T-Numeric-Construction (S9 Phase-2; companion to Phase-1 Step 3) + §1.8 ledger row #19 (float_construction_landed) + Acceptance bullet #67
authority docs:
  - docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md (parent S9 brief; Phase-2 scope)
  - docs/briefs/r3-substrate-s9-phase-1-step-3-emission-entries-worker.md (Phase-1 Step 3 — Int<N>/UInt<N> emission-entry shape; Phase-2 mirrors)
  - docs/briefs/r3-substrate-s8-approximate-field-float-migration-worker.md (S8 — `ApproximateField<F>` + base-carrier landing)
  - PR #1856 (S3 Q-MachineConstraint substrate — `Compose<Algebra, MachineConstraint>` MERGED)
  - gunbc#828 #issuecomment-4385530115 (Q-MachineConstraint sub-decisions ratification)
gates:
  - §1.8 ledger row #19 (float_construction_landed)
  - §67 (numeric_construction_demonstration — Real<64> half of Acceptance bullet)
worker pin: TBD (queued post-S8 + post-Phase-1-Step-3)
---

# R3 Substrate S9 Phase-2 — Float emission-entry coordination + `Real<64>` demonstration

## Context

### T-V2-Retirement coordination gate (precondition framing)

Per S9 Phase-1 Step 3 brief (lines 22-38) the historical
T-V2-Retirement-landing-first cascade gate is **implicitly
superseded** by Director Q-MachineConstraint sub-decision 6
(UNIVERSAL substrate posture) at gunbc#828
#issuecomment-4385530115. Phase-2 inherits the same supersession;
T-V2-Retirement coordination is **not** a hard precondition.

If the supersession is contested at dispatch, surface to
Substrate Mgr (#1739) as STOP — Director re-ratification needed.

### Substrate state precondition

Phase-2 dispatch gates on **both** of the following landing:

1. **S8 — `ApproximateField<F>` carrier + base-carrier** (`Real` /
   `Rational`) per `r3-substrate-s8-approximate-field-float-migration-worker.md`
2. **S9 Phase-1 Step 3 emission entries** (Int<N>/UInt<N> via
   `Compose<Algebra, MachineConstraint>`) per
   `r3-substrate-s9-phase-1-step-3-emission-entries-worker.md` —
   establishes the parametric-emission-entry pattern that Phase-2 mirrors

If either is not landed at dispatch time, STOP and surface — Phase-2
is a coordination/synthesis brief, not a substrate-producer brief on
either axis.

## Scope

### Deliverable 1 — `Float<N>` emission entries (mirrors Phase-1 Step 3)

Author parametric instantiations in `dsl/std/float.dag` (or canonical
equivalent — worker greps for the file used by the Float migration in
S8, and authors emission entries adjacent to it):

- `Real<32>` ≡ `Compose<Real, MachineWidth<32>>` → emits Rust `f32`
- `Real<64>` ≡ `Compose<Real, MachineWidth<64>>` → emits Rust `f64`

**Spelling note**: per Q-MC sub-decision 3 critical correction at
gunbc#828 #issuecomment-4385530115, the user-facing surface is
`Real<N>` (not `Float<N>`); `Float` is target-language name for the
Rust primitive, `Real` is the algebraic-concept name. Substrate
elaboration is `Compose<Real, MachineWidth<N>>` — first slot is
the fully-applied algebraic concept `Real` (= `ApproximateField<FieldOfFractions<Int>>`
per the Option A STOP resolution), NOT bare `ApproximateField` witness or `ApproximateField<Real>`
parameterization. Same correction shape that applies to Int/UInt
slot-1 spellings: witness shape doesn't go in slot-1; the named
concept does.

Q-ApproximateField-Axiom-Set ratification (S8) is **prerequisite** —
if Director has not chosen the relaxation set at dispatch time,
Phase-2 STOPs until it lands. Brief absorbs ratified shape verbatim
before authoring.

### Deliverable 2 — `Real<64>` end-to-end demonstration (Acceptance bullet #67 second half)

Per S9 parent Acceptance + Phase-1 Step 3 brief deferral: Real<64>
round-trip demonstration runs via E6-G0d evaluator + Grounding Rust
emission. Phase-1 Step 3 landed Int<32> half; this brief lands the
Real<64> half, completing §1.8 #67 Acceptance bullet.

Demonstration shape:
- Source DSL declaration using `Float<64>` (or `Real` alias if
  ratified) → lower → execute via E6-G0d evaluator → emit Rust `f64`
  → numeric value correct under round-trip

### Deliverable 3 — Cross-program emission consumer wiring (Grounding G2)

Per Q-MachineConstraint sub-decision 6 (UNIVERSAL substrate posture):
- Grounding Mgr (#1745) consumes the Float<N> instantiations to emit
  Rust target primitives
- Targets without native IEEE-754 semantics handle omission at
  Grounding-level discharge — target-conditioned **lowering**, NOT
  target-conditioned substrate

Cross-program handoff receipt to Grounding Mgr (#1745) in PR body.

## Slice — single PR

Phase ordering (PR-internal):
1. Verify S8 + Phase-1-Step-3 landed at HEAD (precondition gate)
2. Author Float<N> instantiations (Deliverable 1)
3. Cross-program handoff receipt to Grounding Mgr (#1745)
4. End-to-end Real<64> round-trip demonstration (Deliverable 2)
5. Bootstrap snapshot regen + parse corpus manifest refresh

## Acceptance

- 2 concrete `Real<N>` emission entries landed: `Real<32>` / `Real<64>`
  via `Compose<Real, MachineWidth<N>>` (slot-1 is the algebraic-concept
  name `Real`, not the witness `ApproximateField` or its parameterization)
- Algebra-side spelling matches S8 ratified shape
  (`Real = ApproximateField<FieldOfFractions<Int>>` per the Option A STOP
  resolution — brief absorbs verbatim before authoring)
- Machine-axis spelling consistent: `MachineWidth<bits>` per S3
  ratified shape
- Cross-program handoff receipt to Grounding Mgr (#1745) for G2
  consumer wiring (Float lowering rules)
- `Real<64>` (or canonical Float<64> spelling) round-trip
  demonstration runs: source DSL → lower → execute → emit Rust f64 →
  numeric value correct
- §1.8 ledger row #19 (`float_construction_landed`) advances DECLARED
  → PRODUCER_LANDED upon merge (not CONSUMER_LANDED — Grounding G2
  follow-on PR carrying per-pair lowering rules + emitted-Rust-primitive
  (`f32`/`f64`) verification advances the row to CONSUMER_LANDED;
  this brief produces substrate emission entries, the consumer is
  owned by Grounding Mgr #1745). Split-PR producer-then-consumer per
  bundled-scope discipline (gunbc#1739 #issuecomment-4392225548)
- §1.8 ledger row #67 (`numeric_construction_demonstration`)
  Acceptance bullet **complete** (Phase-1 Step 3 landed Int<32> half;
  this PR lands Real<64> half)
- `cargo test --workspace --exclude v2-compiler-tests` green
  (3 pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`:
  section anchors / rule-text quotes only; no bare `:NNN`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **S8 not landed at dispatch**: Phase-2 cannot author Float<N>
  entries without `ApproximateField<F>` + `Real` base carrier. STOP —
  surface to Substrate Mgr; reopen on S8 landing.
- **Phase-1 Step 3 not landed at dispatch**: Phase-2 mirrors that
  brief's emission-entry pattern. STOP — surface; reopen on
  Phase-1-Step-3 merge.
- **Q-ApproximateField-Axiom-Set not ratified**: Float<N> emission
  shape depends on the relaxation-set Director chooses (which axioms
  the algebra-side carrier names). STOP — surface; absorb ratified
  shape before authoring.
- **Grounding emission rule shape not yet defined for Float pair
  consumption** (G2 doesn't have per-pair lowering rules for
  `Compose<Real, MachineWidth<N>>`): STOP —
  coordinate with Grounding Mgr (#1745); cross-program handoff
  receipt needs concrete consumer surface, not aspirational.
- **End-to-end demonstration fails at evaluator or emission**: STOP —
  root-cause; do NOT bridge with placeholder lowering. Per
  `feedback_fail_closed_discipline`: production code shouldn't ship
  demo-stub-shaped Acceptance Pass-bullets.

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Compose<Algebra, MachineConstraint>` + `MachineWidth<bits>`
     landed at PR #1856 ✓
   - `ApproximateField<F>` + `Real` base carrier — gates on S8
     landing (worker re-greps `dsl/std/` at dispatch)
   - Float<N> parametric emission entries do NOT yet exist — this
     brief is producer
2. **Existing brief?** S9 parent brief Phase 2 section (lines 95-100)
   names Phase-2 scope; S8 brief covers algebra-side carrier landing.
   This brief is the worker dispatch packet for the Phase-2
   coordination/synthesis step; not a competing authority.
3. **Design-doc match?** Q-MachineConstraint ratification (gunbc#828
   #issuecomment-4385530115) sub-decision 5 ("≥3 algebra × constraint
   pairs minimum"); Phase-1 Step 3 + Phase-2 together carry 8 pairs
   (Int<32/64/128> + UInt<32/64/128> + Float<32/64>).
4. **Citations live?** Worker re-verifies at dispatch — S8 + Phase-1
   Step 3 landings are the precondition gate.
5. **Carrier dissolves the bridge?** Yes — Float<N> emission entries
   are the end-state of the substrate-carrier port program for IEEE-754
   floats. The "bridge" is the gap between the abstract
   `Real = ApproximateField<FieldOfFractions<Int>>` algebra carrier + machine-width carrier
   and concrete target-Rust-primitives; parametric `Compose<...>`
   instantiations dissolve via emission-rule lookup at Grounding
   lowering layer.

## Provenance

Drafted 2026-05-06 per Tier-1 brief-queue commitment at gunbc#1858
(R3 Substrate Mgr open assignment) + auto-nudge cadence. Cross-references
S3 (`MachineConstraint<C>` carrier — landed at #1856), S8
(`ApproximateField<F>` Float migration — pending), and S9 Phase-1
Step 3 (Int<N>/UInt<N> emission entries — pending). Brief queues
post-both-precondition-landings; worker pin assigned at that time.
