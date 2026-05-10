---
status: draft (worker brief; ENGAGE-NOW per Brian directive 2026-05-06)
authority parent: R3 Substrate Manager (#1739)
ratification: Brian directive 2026-05-06 (chat) — algebra × machine-constraints interaction modeling; concrete types as products of independent axes
roadmap row: docs/r3-program-plan.md §4.1 Class 1 (parser/grammar surface) + §1.8 ledger row #60
authority docs:
  - docs/r3-program-plan.md §4.1 (Class 1 modeling target — independent constraint models)
  - docs/r3-program-plan.md §1.4 (substrate_gap_parser_grammar_closed)
  - docs/r3-design-schedule-2026-05-06.md §1 S3
  - docs/audit/r3-debt-sweep-2026-05-06.md (Class 1 5-criteria Pass)
gates:
  - substrate_gap_parser_grammar_closed (#60)
worker pin: valiant-ant-72 (per S7 brief partition; valiant-ant-72 reserved for S3 implementation post-design ratification)
---

# R3 Substrate S3 — `MachineConstraint<C>` carrier worker brief

## Context

Per Brian directive 2026-05-06 (chat): concrete types like `i64`
emerge as products of independent constraint axes — algebra (`Int`,
the additive Abelian group on integers `Z`) × machine constraint
(`MachineWidth<64>`) — **NOT primary substrate entities**. (The
algebra side is a load-bearing instance of an algebraic structure;
the relationship of `Int` to `Nat` is explicit group-completion
per S9, NOT `AbelianGroup<Nat>` — Nat lacks additive inverses.) The current substrate treats
some concrete primitives (e.g., `Int<32>`, `Float<64>`) as primary
declarations; modeling target reframes them as projections of
algebra × machine-constraint products.

Class 1 (`substrate_gap_parser_grammar_closed`, ledger row #60) closes
when:
1. Substrate carriers for machine constraints exist as independent axes.
2. Parser handles generic interaction syntax (e.g., `Int @ Width<32>`
   or equivalent — bikeshed in design phase).
3. ≥ 3 algebra × constraint pairs emit to target primitives
   (Rust `i32` / `i64` / `u64` minimum).
4. Target primitives are NOT primary substrate entities — they
   are emitted from the interaction.
5. v2-oracle parity on the same source program produces equivalent
   target-language output.

This brief lands the **substrate carrier slice**: `MachineWidth<bits>`
+ parametric `Compose<Algebra, MachineConstraint>` interaction shape
per Q-MachineConstraint-Carrier RATIFIED at gunbc#828
#issuecomment-4385530115 (Brian directive 2026-05-06: *"universal
substrate, ratify defaults"*; 6 sub-decisions — see PR #1817 + this
brief's Phase-1).

**Universal-substrate posture (Brian directive)**: every target
carries machine-constraint facts as substrate. Targets lacking
native machine-width semantics (Python `int` / `float`) handle
omission at **Grounding-level discharge** — target-conditioned
**lowering**, NOT target-conditioned substrate. The carrier is
the same regardless of target.

Companion lanes land parser-grammar (separate downstream PR) +
emitter wiring (Grounding Mgr cross-program).

## Slice

### Phase 1 — Carrier shape (gate scaffolding for #60)

1. **Author `dsl/std/machine_constraints.dag`** introducing:

   Each new coproduct / sum declaration MUST carry a 🟢/🟡/🔴
   checkpoint-comment classification per
   `docs/modeling-discipline.md#4-coproduct-dissolution`
   (Practice 4). Per the "What to check" rule there: *"Any new
   Rust enum with N ≥ 2 variants must have a checkpoint comment
   naming its classification (🟢/🟡/🔴), with a ledger entry if
   GREEN or a named trigger if YELLOW."* Implements
   `INVARIANTS.md` P1 (Modeling Faithfulness).

   - `MachineWidth<bits>` — phantom-parameter carrier; `bits: Nat`.
     **🟢 PRIMITIVE** — width values are Nat-valued; the carrier
     names "this width matters for emission" with the value carried
     in the phantom parameter. No variant enumeration; new widths
     are not new variants but new instances.

   Per Q-MachineConstraint sub-decision 1 RATIFIED: `MachineWidth<bits>`
   is the **only** machine-axis carrier in R3. `RegisterClass<R>` /
   `EndianMode<E>` / `Alignment<bytes>` / signedness-as-axis defer
   post-R3 (no speculative landing per `feedback_construction_over_ratchets`).
   Sub-decision 1 supersedes the prior "introduce
   `MachineConstraint<C>` as top-level subsumer" framing — there
   is no separate `MachineConstraint<C>` carrier; `MachineWidth<bits>`
   IS the machine-axis carrier in R3 scope.

2. **Parametric `Compose<Algebra, MachineConstraint>` interaction
   shape** per Q-MachineConstraint sub-decision 2 RATIFIED.
   Lookup-maps (e.g., a record-keyed `AlgebraMachineProduct` table)
   are EXPLICITLY REJECTED. Instead the interaction is
   **type-level parametric composition**:

   ```
   data Compose<Algebra, MachineConstraint> = Phantom
   ```

   Type spelling per sub-decision 3: `Int<64>` ≡
   `Compose<AbelianGroup, MachineWidth<64>>`. `Real<64>` ≡
   `Compose<Real, MachineWidth<64>>`
   (sub-decision 4 — algebra approximation + machine approximation
   compose as independent axes; both carried explicitly).

   Practice 4 classification: `Compose<A, M>` is **🟢 PRIMITIVE**
   — phantom-parameter carrier; widening adds Algebra / MachineConstraint
   instantiations, not new variants.

3. **Annotation classifications** for new declarations follow
   Practice 4 with **in-source checkpoint comments** on the live
   declarations in `dsl/std/machine_constraints.dag` (the
   load-bearing artifact per
   `docs/modeling-discipline.md#4-coproduct-dissolution` "What
   to check"). PR-body summary is supplementary, not substitute.

4. **Algebra-side independence** per sub-decision 3: equivalent
   under both algebra-side options from PR #1815 (Option A
   canonical AbelianGroup vs Option B GroupCompletion-of-
   CommutativeMonoid<Nat>). S3 brief does NOT block on S9 / Option-
   A-vs-B selection — `Compose<AbelianGroup, MachineWidth<64>>`
   reads identically under either Int algebra-side shape.

### Phase 2 — Parser-grammar surface (separate downstream PR)

This brief does NOT land parser changes. Per "carrier slice only"
discipline, parser-grammar work is sequenced after Director
ratification of carrier shape. A follow-up brief (Substrate
S3-Phase-2) lands grammar production + AST nodes for the interaction
syntax (bikeshed: `Int @ Width<32>` vs `Int with MachineWidth<32>`
vs operator-form).

### Phase 3 — Emit-shim consumption (Grounding Mgr coordination)

Cross-program. Grounding Mgr (#1745) consumes the parametric
`Compose<Algebra, MachineConstraint>` instantiations to emit
target primitives. Brief G2 (`r3-structure.md` Lane T-Ground-Rust)
is the consumer.

Per sub-decision 6 (UNIVERSAL substrate, Brian directive): every
target carries machine-constraint facts as substrate. Targets
without native machine-width semantics (Python `int` / `float`)
handle omission at **Grounding-level discharge** — target-conditioned
lowering only, NOT target-conditioned substrate. The carrier
shape is identical regardless of target.

### Phase 4 — ≥3 algebra × constraint pairs landed (Pass criterion 3)

Per sub-decision 5: ≥3 pairs is **minimum, not target**. Worker
demonstrates Class 1 closure with at least three concrete
instantiations:

1. `Compose<AbelianGroup, MachineWidth<32>>` (≡ `Int<32>`) → Rust `i32`
2. `Compose<AbelianGroup, MachineWidth<64>>` (≡ `Int<64>`) → Rust `i64`
3. `Compose<CommutativeMonoid, MachineWidth<64>>` (≡ `UInt<64>`) → Rust `u64`

(Alternative: `Compose<Real, MachineWidth<64>>`
≡ `Real<64>` → Rust `f64` per sub-decision 4 covers the algebra-
approximation × machine-approximation independent-composition case.
Coordinate with S8 worker (quiet-boar-160) for the float entry.)

These exercise the substrate but **target-primitive emission lives
in Grounding Mgr lane**, not this brief. This brief produces the
substrate; Grounding consumes. Coordinate Phase-3/4 hand-off receipts.

## Acceptance

- `dsl/std/machine_constraints.dag` lands with `MachineWidth<bits>`
  + `Compose<Algebra, MachineConstraint>` parametric carrier
  (phantom carriers; no lookup-map; no parser surface).
- Practice 4 classification receipts in PR body for each new
  declaration.
- Q-MachineConstraint-Axis-Enumeration surfaced as a §note in PR
  body for Director ratification of further axes (RegisterClass /
  EndianMode / Alignment).
- Grounding Mgr cross-lane handoff documented (G2 brief consumes
  `AlgebraMachineProduct` substrate).
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- ROADMAP "Class 1 (parser/grammar surface)" row updates to reflect
  carrier-slice landed (remaining: parser surface + emit consumption).
- §1.8 row #60 status moves DECLARED → CONSUMER_LANDED upon Phases
  1+3 coordination; PASSING upon Phase 4 demonstration.

## STOP-AND-ESCALATE

- **Director rules `MachineConstraint<C>` axis enumeration is
  not the right shape**: STOP. Re-canvas. Brian directive surfaces
  algebra × machine-constraint as the modeling target; if Director
  identifies a third axis (e.g., compile-target-OS conditioning),
  carrier shape may need to expand beyond `MachineWidth` /
  `RegisterClass` / `EndianMode` / `Alignment`.
- **Concrete primitive (`Int<32>`, `Float<64>`, etc.) is currently
  primary substrate entity**: per Brian directive, target-primitive
  is NOT primary. If existing substrate has primary `Int<N>` /
  `Float<N>` declarations consumed at lower or emit time, those
  consumers must migrate to read `AlgebraMachineProduct` interaction
  table — sequenced **after** carrier slice lands. STOP if migration
  scope is non-trivial; surface to Substrate Mgr (#1739) for
  scope decision (carve to S9 T-Numeric-Construction).
- **Parser-grammar interaction syntax lands during Phase-1 worker
  scope creep**: STOP — brief is carrier slice only. Parser work
  is Phase-2 sequenced after Director carrier-shape ratification.
- **`AlgebraMachineProduct` interaction-lookup table grows beyond
  3 entries during Phase-1**: STOP — Phase-1 is substrate carrier
  only; populating the table is Grounding Mgr (Phase-3) work.
- **Substrate-fact-introduction P1 procedure flags
  `MachineConstraint<C>` axis as DAG-ancestor of an existing
  declaration**: STOP — re-frame as consumer migration not new
  carrier. Per `INVARIANTS.md#p1-modeling-faithfulness` DFS the
  concept DAG before introducing.

## Authority audit receipt

1. **Substrate exists?** Per memory + grep at draft time:
   `dsl/std/` lacks `MachineConstraint`, `MachineWidth`, or
   `AlgebraMachineProduct` declarations. `Int` / `Float` exist as
   concrete carriers; this brief reframes them as projections of
   algebra × machine-constraint product. Worker re-greps at dispatch.
2. **Existing brief?** None for machine-constraint axis. S9
   (T-Numeric-Construction) is the algebra-side companion — it
   lands `Int` algebra-side shape (group-completion of Nat-monoid;
   see S9 Phase-1); this brief lands the
   machine-side shape. Brief co-references S9; the two briefs
   are independent axes (Substrate Mgr partition response 2026-05-06).
3. **Design-doc match?** Brian directive 2026-05-06 (chat) is the
   primary authority. No design-doc precedent; this brief IS the
   design surface. Director ratification of carrier shape required
   before parser-grammar Phase-2 dispatch.
4. **Citations live?** `r3-program-plan.md` §4.1 + §1.4 and design
   schedule §1 S3 verified at HEAD 2026-05-06.
5. **Carrier dissolves the bridge?** Yes — `substrate_gap_parser_grammar_closed`
   #60 5-criteria Pass requires (a) substrate carriers
   [this brief: Phase 1] + (b) parser handles generic interaction
   syntax [Phase 2] + (c) ≥3 emission pairs [Phase 3+4
   cross-program] + (d) target primitives NOT primary substrate
   entities [structural property of carrier shape] + (e) v2-oracle
   parity. This brief covers (a) and structurally enables (d).

## Provenance

Drafted 2026-05-06 per Brian directive (chat) + R3 design schedule
§1 S3 (PR #1810). Substrate Mgr authors carrier-slice scope; worker
pin = valiant-ant-72 per S3 implementation reservation
(Substrate Mgr partition response 2026-05-06). Dispatched as
foundational substrate work; gates Class 1 closure.
