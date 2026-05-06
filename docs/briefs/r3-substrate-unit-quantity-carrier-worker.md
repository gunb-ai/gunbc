---
status: draft (worker brief; Q-Unit-1..5 RATIFIED at gunbc#828 #issuecomment-4385412256 (2026-05-06); dispatchable when worker pin assigned)
authority parent: R3 Substrate Manager (#1739)
ratification: Director ratified Q-Unit-1..5 at gunbc#828 #issuecomment-4385412256 (2026-05-06) (zesty-bear-812). Scale value `Unit` renamed to `One` per Q-Unit-1.
roadmap row: cross-references docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md Phase-3
authority docs:
  - docs/briefs/r3-substrate-s9-unit-quantity-carrier-canvas.md (RATIFIED canvas; this brief consumes the ratified shape)
  - docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md (parent Phase-3 dimensional refinements)
  - docs/r4-carve-out-routing.md C6 (R4 routing for Aspect-axis follow-up; EpochMs deferred)
  - INVARIANTS.md `#p1-modeling-faithfulness` (substrate-fact-introduction procedure)
gates:
  - cross-references §1.8 ledger rows #20-#23 (S9 Phase-3 dimensional refinement gates)
---

# R3 Substrate Unit/Quantity carrier — worker brief

## Context

Unit/Quantity carrier landing per Q-Unit-1..5 RATIFIED. The shape
emerged from S9 Phase-3 boundary feedback: `Duration` /
`Milliseconds` / `Seconds` / `EpochMs` are dimensional / unit-of-
measure semantics, not value-restriction predicates over `Int`.
Per `feedback_reason_not_label`: the predicate name is the label,
the reason is "this carries time-unit semantics".

This brief lands the **2-axis carrier** (`Unit<Quantity, Scale>`)
and updates S9 Phase-3 reframe table consumers. EpochMs (Aspect-
axis-shaped) defers to R4 C6 — explicitly out of scope for this
brief.

**Cross-program**: Grounding Mgr (#1745) consumes Unit-typed
emission for dimensional refinements; coordinate emission scope
at brief-landing.

## Slice

### Phase 1 — Carrier landing

1. **Author `dsl/std/units.dag`** (or canonical equivalent —
   worker greps `dsl/std/` for existing Quantity / Scale / Unit
   declarations at dispatch; if any pre-exist, this brief is
   migration not landing).

   Shape (verbatim per Q-Unit-1..5 ratification):

   ```
   data Quantity =
       Time
     | Length
     | Mass
     | Memory
     | Information
     | DataRate
     | Frequency
     | Count
     | Currency

   data Scale =
       Atto    // 10^-18
     | Femto   // 10^-15
     | Pico    // 10^-12
     | Nano    // 10^-9
     | Micro   // 10^-6
     | Milli   // 10^-3
     | One     // 10^0  (RATIFIED rename from `Unit` per Q-Unit-1)
     | Kilo    // 10^3
     | Mega    // 10^6
     | Giga    // 10^9
     | Tera    // 10^12
     | Peta    // 10^15
     | Exa     // 10^18

   data Unit<Q, S> = Phantom
   ```

   Practice 4 classification per
   `docs/modeling-discipline.md#4-coproduct-dissolution` —
   *"Any new Rust enum with N ≥ 2 variants must have a checkpoint
   comment naming its classification (🟢/🟡/🔴), with a ledger
   entry if GREEN or a named trigger if YELLOW."* Marks per
   Q-Unit-5 RATIFIED:

   - `Unit<Q, S>` → 🟢 PRIMITIVE (phantom-parameter carrier;
     widening adds Quantity / Scale values, not new variants)
   - `Quantity` → 🟡 SCAFFOLD with named dissolution trigger:
     "all Quantity values consumed by ≥ 1 emission rule"
   - `Scale` → 🟡 SCAFFOLD with same dissolution trigger

   Scaffold marks reflect that the enumeration is hand-modeled;
   when each value gates a real emission rule (Grounding consumer
   demonstrates need), the carrier is no longer speculative. Per
   `feedback_construction_over_ratchets` — adding values without
   a corresponding emission rule violates the dissolution-trigger
   contract.

2. **NOT in scope**:
   - **Aspect axis** (`PointKind`, `Magnitude` / `Instant` / `Rate`):
     deferred to R4 C6 per Q-Unit-2 RATIFIED. EpochMs (instant-
     shaped) carves to R4 alongside.
   - **Scale-agnostic `Duration<S>`** (parameterized for "any
     time-magnitude regardless of scale"): deferred per Q-Unit-3
     forward-flag. Until ≥ 2 consumers ask for the parametric form,
     collapsed `Duration ≡ Unit<Time, One>` is correct.
   - **3-axis `AlgebraMachineRoundingProduct`**: deferred to R4 C5
     (separate cascade; covered by S8 Q-ApproximateField-Rounding-
     Mode RATIFIED as separate axis).

### Phase 2 — S9 Phase-3 dimensional reframe consumer

Update S9 Phase-3 dimensional refinements to consume Unit-typed
shape. Per Q-Unit-3 RATIFIED collapse + Q-Unit-4 outer-Refined /
inner-Unit composition order:

| S9 Phase-3 declaration | Unit-typed shape | Practice 4 |
|---|---|---|
| `Duration` | `Refined<Unit<Time, One>, non_negative>` | 🟢 PRIMITIVE |
| `Seconds` | type alias for `Duration` (collapse per Q-Unit-3) | 🟢 |
| `Milliseconds` | `Refined<Unit<Time, Milli>, non_negative>` | 🟢 |
| `EpochMs` | **DEFERRED** to R4 C6 (Aspect-axis follow-up) | — |

Refined predicate `non_negative` applies at the Unit-typed level
(predicates over "durations" are semantically coherent; predicates
over "raw Ints that happen to be milliseconds" are not).

Worker authors the 3 in-scope refinements (`Duration` /
`Seconds` / `Milliseconds`) inheriting the Unit shape. `EpochMs`
**stays unchanged or carries a SCAFFOLD comment naming the R4
trigger** — do NOT land it under a forced shape.

### Phase 3 — Cross-program emission coordination

Coordinate with Grounding Mgr (#1745) on dimensional emission:

1. Grounding consumes `Unit<Q, S>`-typed values; emission rules
   project Unit-tagged carriers to target-language types
   (Rust: `Duration` → `std::time::Duration` or `i64`-with-unit-
   tag; Python: `datetime.timedelta`; Go: `time.Duration`).
2. Each emission rule that consumes a Unit value FOR A SPECIFIC
   `(Quantity, Scale)` pair is a dissolution-trigger consumer
   for the corresponding `Quantity` / `Scale` enum value. Worker
   does NOT author emission rules in this brief; that's
   Grounding G2 follow-on. Worker DOES document the handoff
   receipt in PR body.
3. If Grounding's Phase-1 emission scope does not consume any
   Unit-typed values yet, the SCAFFOLD trigger on `Quantity` /
   `Scale` does not fire — that's expected. The trigger fires
   per-consumer, not all-or-nothing.

## Acceptance

- `dsl/std/units.dag` (or canonical equivalent path) lands with
  `Unit<Q, S>` + `Quantity` + `Scale` declarations at the
  ratified shape.
- Practice 4 classification receipts in PR body: `Unit<Q, S>`
  🟢, `Quantity` 🟡 SCAFFOLD, `Scale` 🟡 SCAFFOLD with named
  dissolution trigger.
- S9 Phase-3 `Duration` / `Seconds` / `Milliseconds`
  declarations updated to Unit-typed `Refined<Unit<Time, ...>, ...>`
  shape (Q-Unit-4 RATIFIED outer-Refined / inner-Unit).
- `EpochMs` retained with SCAFFOLD comment naming R4 C6 trigger;
  NOT migrated.
- Grounding Mgr (#1745) cross-program handoff receipt in PR body
  (G2 / dimensional emission consumes Unit-typed values).
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile
  diagnostic ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- §1.8 ledger row #20 (or appropriate dimensional-refinement
  row) advances per Phase-2 consumer landing.
- Citation discipline per `brief-authoring-checklist.md`:
  cross-doc cites use anchors or rule-text quotes; bare
  `:NNN` PROHIBITED.

## STOP-AND-ESCALATE

- **Existing `Quantity` / `Scale` / `Unit` declarations exist
  in `dsl/std/`**: re-frame as consumer migration, not landing.
  PR receipt names existing carriers; worker grep at dispatch
  is mandatory.
- **`Unit<Q, S>` collides with existing carrier name** (e.g.,
  `Dimension<C>` lens carrier or any other `Unit`-named
  declaration): STOP. Re-canvas; alternative names per Q-Unit-1
  surfaced (`UnitOfMeasure<Q, S>` / `Quantity<Q, S>`). Director
  re-ratification needed before landing under a different name.
- **Grounding emission consumer needs an Aspect-axis distinction
  during Phase-3** (e.g., a Grounding row distinguishes "duration
  emission" vs "instant emission" and the brief's collapsed
  Phase-1 form cannot serve): Aspect-axis becomes load-bearing
  earlier than R4. STOP — surface to Substrate Mgr (#1739) for
  re-canvas; this is consumer-demand-driven landing of C6.
- **A Phase-3 dimensional refinement requires a `Quantity` value
  not in the ratified enumeration** (e.g., a refinement is
  modeled as `Volume` or `Temperature`): the value is substrate-
  fact-introduction. Per `feedback_construction_over_ratchets`,
  do NOT silently grow `Quantity`. STOP — surface to Substrate
  Mgr; new value requires P1 procedure receipt + named consumer
  demand.
- **Refined<Unit<...>, predicate> composition fails to type-check
  in lowerer**: the substrate's `Refined<Base, predicate>`
  expects `Base` to be a concrete underlying type, not a phantom
  carrier. STOP — surface as substrate-extension question;
  Refined may need to consume Unit-typed bases via a separate
  bridge or carrier shape. Coordinate with Substrate Mgr.
- **`Seconds = Duration` collapse breaks an existing consumer**
  (e.g., a consumer disambiguates `Seconds` from `Duration` at
  type level): per Q-Unit-3 RATIFIED collapse, the existing
  consumer is the drift — surface as consumer-migration finding,
  do not unwind the collapse without Director re-canvas.

## Authority audit receipt

1. **Substrate exists?** Per memory + draft-time grep:
   `Refined<Base, predicate>` substrate landed (annotation-
   elimination Wave 1; memory note 2026-02-26). `Unit<Q, S>` /
   `Quantity` / `Scale` do NOT exist in `dsl/std/`. This brief
   is substrate-fact-introduction (P1 procedure for `Quantity`
   + `Scale` enums; phantom carrier for `Unit<Q, S>`). Worker
   re-greps at dispatch.
2. **Existing brief?** Canvas
   `r3-substrate-s9-unit-quantity-carrier-canvas.md` is the
   ratified design surface this brief consumes. S9 parent brief
   names Phase-3 reframe table; this brief lands the carrier
   to satisfy the table. No competing brief.
3. **Design-doc match?** Director ratification on canvas
   (gunbc#828 #issuecomment-4385412256 (2026-05-06)) is the design-doc
   anchor. Worker re-reads canvas RATIFICATION section before
   authoring.
4. **Citations live?** Canvas commit `5f22fd06e` records the
   ratification + rename. R4 carve-out routing C6 names this
   work as upstream input. Verified at HEAD.
5. **Carrier dissolves the bridge?** Yes — the bridge is
   `feedback_reason_not_label` violation in S9 Phase-3 forced
   `Refined<Int, predicate>` shape for dimensional refinements.
   Unit/Quantity carrier provides the algebra-axis shape that
   correctly carries dimensional semantics. Outer-Refined /
   inner-Unit composition preserves the predicate-bearing
   semantic chain (predicates over Unit-typed bases, not over
   raw Ints).

## Provenance

Drafted 2026-05-06 per Q-Unit-1..5 RATIFIED at gunbc#828
#issuecomment-4385412256 (2026-05-06; zesty-bear-812). Canvas at
`docs/briefs/r3-substrate-s9-unit-quantity-carrier-canvas.md`
ratification banner records the 5-Q ratification + Scale rename.
Worker pin TBD — dispatchable when idle pool refreshes
(loyal-wolf-828 + quick-koi-190 reserved for #1782 merge cascade;
valiant-ibex-312 next-available post-#1803 merge but no specific
reservation; assignment per Substrate Mgr at dispatch time).
