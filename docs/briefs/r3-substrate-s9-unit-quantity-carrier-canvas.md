---
status: draft (Mgr-tier canvas; surfaces design question for Director ratification per S9 Phase-3 boundary feedback)
authority parent: R3 Substrate Manager (#1739)
ratification: Director feedback on S9 at gunbc#828 inbox response 2026-05-06 (zesty-bear-812) — "several Phase-3 refinements are likely dimensional/unit-of-measure, not value-restriction predicates ... the right shape is likely a Unit<Quantity, Scale> axis"
roadmap row: cross-references docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md Phase-3
authority docs:
  - docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md (parent S9 brief)
  - feedback_reason_not_label (the discipline this canvas applies)
  - INVARIANTS.md#p1-modeling-faithfulness (substrate-fact-introduction procedure)
  - docs/r4-carve-out-routing.md C6 (R4 routing if Unit/Quantity defers)
gates: cross-references §1.8 ledger rows #20-#23 (refinements that may reframe)
---

# R3 Substrate S9 Phase-3 — Unit/Quantity carrier canvas

## Purpose

Director feedback on S9 (gunbc#828 inbox response 2026-05-06) flagged
that several Phase-3 refinements are not value-restriction predicates
but **dimensional / unit-of-measure** semantics:

- `Duration` — carries time-quantity
- `Milliseconds` — carries time-quantity at milli-scale
- `Seconds` — carries time-quantity at unit-scale
- `EpochMs` — carries time-instant at milli-scale (different
  conceptual axis: instant vs duration)

Forcing these through `Refined<Int, predicate>` is a
`feedback_reason_not_label` violation: the predicate name is the
label, not the reason. The reason for `Milliseconds` is "this
carries time-unit semantics at milli scale", not "this Int satisfies
some Boolean predicate P".

This canvas surfaces the design question for Director ratification.
**Output**: a ratified `Unit<Quantity, Scale>` (or equivalent)
carrier shape; S9 Phase-3 reframes from `Refined<Int, ...>` for
the dimensional refinements; remaining 6 refinements (`Char`,
`HttpStatus`, `Port`, `RetryCount`, `PositiveInt`, `NonNegativeInt`)
proceed under `Refined<Base, predicate>` shape unchanged.

## Candidate carrier shape

### Sketch — 2-axis `Quantity` × `Scale`

```
data Quantity = Time | Length | Mass | Memory | Information
              | DataRate | Frequency | Count | Currency

data Scale =
  | Atto    // 10^-18
  | Femto   // 10^-15
  | Pico    // 10^-12
  | Nano    // 10^-9
  | Micro   // 10^-6
  | Milli   // 10^-3
  | Unit    // 10^0
  | Kilo    // 10^3
  | Mega    // 10^6
  | Giga    // 10^9
  | Tera    // 10^12
  | Peta    // 10^15
  | Exa     // 10^18

data Unit<Q, S> = Phantom  // 🟢 PRIMITIVE — Q: Quantity, S: Scale
```

Phase-3 dimensional refinements emit as products:

```
Duration     ≡ Quantity<Time>      ⊗ Scale<Unit>   ⊗ underlying<Int>
Seconds      ≡ Quantity<Time>      ⊗ Scale<Unit>   ⊗ underlying<Int>
Milliseconds ≡ Quantity<Time>      ⊗ Scale<Milli>  ⊗ underlying<Int>
```

(`Duration` and `Seconds` collapse under this shape. May or may
not be intended — see Q-Unit-3 below.)

### Open question: Instant vs Duration axis

`EpochMs` is conceptually different from `Duration`: it's a
**time-instant** (point on a timeline relative to epoch), not a
**time-duration** (interval magnitude). Common modeling pattern:

```
data PointKind = Instant | Duration
data Time<P, S> = Phantom  // P: PointKind, S: Scale
```

So:
```
Duration     ≡ Time<Duration, Unit>   ⊗ underlying<Int>
Seconds      ≡ Time<Duration, Unit>   ⊗ underlying<Int>
Milliseconds ≡ Time<Duration, Milli>  ⊗ underlying<Int>
EpochMs      ≡ Time<Instant,  Milli>  ⊗ underlying<Int>  // relative to epoch
```

Or generalized further:

```
data Quantity = Time | Length | ... 
data Aspect = Magnitude | Instant | Rate
data Unit<Q, A, S> = Phantom
```

Adding `Aspect` widens the carrier to 3-axis. Q-Unit-3 ratifies
whether to land 2-axis or 3-axis Phase-1.

## Director-side ratification questions

### Q-Unit-1 — Carrier name + scope

Is `Unit<Quantity, Scale>` the right name? Alternatives:
- `Dimension<Q, S>` (collides with existing `Dimension<C>` lens
  carrier — likely conflicts; not recommended)
- `Quantity<Q, S>` (folds Quantity name into outer carrier)
- `UnitOfMeasure<Q, S>`

**Recommendation**: `Unit<Quantity, Scale>` — short, conventional,
no naming collision with existing `Dimension<C>`.

### Q-Unit-2 — Phase-1 axes (2-axis vs 3-axis)

Should Phase-1 land 2-axis (`Q × S`) or 3-axis (`Q × Aspect × S`)?

- **2-axis** lands minimum carrier; `EpochMs` (instant) defers
  to follow-up that adds the `Aspect` axis. Risk: known-needed
  3rd axis defers; future migration churn.
- **3-axis** lands Aspect now; supports `EpochMs` immediately
  but introduces an axis with limited initial demand.

**Recommendation**: **2-axis Phase-1**, with `EpochMs` deferred to
follow-up. Per `feedback_construction_over_ratchets`: don't
build axes ahead of consumer demand. `EpochMs` is single consumer;
the 3rd axis can land when a 2nd instant-shaped consumer demands
it (e.g., absolute timestamps, Unix epochs, Schedule deadlines).

### Q-Unit-3 — Duration / Seconds collapse

Under 2-axis shape, `Duration` and `Seconds` collapse to the same
type (`Time × Unit`). Is this intended?

- **Yes (collapse)**: `Duration` is the canonical name; `Seconds`
  is a type alias — single carrier, no semantic distinction.
- **No (keep distinct)**: `Duration` is dimensional-time-magnitude
  (could be in any time scale at runtime); `Seconds` pins scale.
  Requires a `BoundScale` type-level constraint distinguishing
  "scale variable at runtime" from "scale = Unit".

**Recommendation**: collapse — `Seconds = type Duration<Scale = Unit>`
or `Seconds = Duration` with `Duration` always at `Scale<Unit>`.
The "scale variable at runtime" form is a generalization; if no
consumer demands it, do not introduce.

### Q-Unit-4 — Refined predicates over Unit-typed bases

Under Unit-typed shape, predicates like `non_negative` /
`gt_zero` still apply. Composition shape:

```
Duration         ≡ Refined<Unit<Time, Unit>, non_negative>  ⊗ underlying<Int>
PositiveDuration ≡ Refined<Unit<Time, Unit>, gt_zero>       ⊗ underlying<Int>
```

Is `Refined<Unit<...>, predicate>` the right composition order?
Or should `Refined<...>` wrap the underlying primitive and
`Unit<...>` annotate the type-tag externally?

**Recommendation**: **outer Refined wraps inner Unit** —
predicates apply at the Unit-typed level (predicates over
"durations" make semantic sense; predicates over "raw Ints
that happen to be milliseconds" do not). Refined-outer / Unit-inner
preserves the semantic chain.

### Q-Unit-5 — Carrier classification

Practice 4 classification for `Unit<Q, S>`:
- 🟢 PRIMITIVE — phantom-parameter carrier; widening adds Quantity
  values or Scale values, not new variants. P1 substrate-fact-
  introduction procedure required for new Quantity values
  (each Quantity is a substrate-fact: "this measurement axis
  is meaningful in our system").

`Quantity` and `Scale` themselves:
- 🟡 SCAFFOLD until enumerated set is ratified — adding Quantity
  or Scale values is substrate-fact-introduction, not silent
  widening. Named dissolution trigger: "all Quantity / Scale
  values consumed by ≥ 1 emission rule" (i.e., the enumeration is
  load-bearing for Grounding emission, not speculative).

## Phase-3 reframe shape

Under ratified Unit/Quantity carrier:

| S9 Phase-3 declaration | Reframed shape | Practice 4 |
|---|---|---|
| `Char` | `Refined<Int, valid_unicode_codepoint>` | 🟢 PRIMITIVE |
| `EpochMs` | **deferred to Aspect-axis follow-up** | (deferred) |
| `Duration` | `Refined<Unit<Time, Unit>, non_negative>` | 🟢 |
| `Milliseconds` | `Refined<Unit<Time, Milli>, non_negative>` | 🟢 |
| `Seconds` | type alias for `Duration` (Q-Unit-3 = collapse) | 🟢 |
| `RetryCount` | `Refined<NonNegativeInt, retry_semantics>` | 🟢 |
| `HttpStatus` | `Refined<Int, range_100_599>` | 🟢 |
| `Port` | `Refined<UInt, range_0_65535>` | 🟢 |
| `PositiveInt` | `Refined<Int, gt_zero>` | 🟢 |
| `NonNegativeInt` | `Refined<Int, gte_zero>` | 🟢 |

The 6 non-dimensional refinements proceed unchanged. Dimensional
refinements reframe under Unit. `EpochMs` defers to R4 carve-out
(C6 in `r4-carve-out-routing.md` already names it).

## STOP-AND-ESCALATE

- **Director rules `Unit<Q, S>` carrier shape is wrong**
  (e.g., prefers explicit dimensional types like
  `data Time = ...; data Length = ...` without product axis):
  STOP. Re-canvas. The two-axis product shape is recommendation;
  Director-side may prefer a different decomposition.
- **`feedback_construction_over_ratchets` flags Quantity / Scale
  enumeration as overbuilding**: SCAFFOLD classification + named
  dissolution trigger ("consumed by ≥1 emission rule") is the
  guardrail. If even the SCAFFOLD enumeration is too speculative,
  narrow Phase-1 to "only Time / Milli / Unit" and grow per
  consumer demand.
- **Existing `Dimension<C>` lens carrier conflicts with `Unit<Q, S>`
  naming**: re-canvas; the recommendation `Unit<Q, S>` is named
  to avoid collision but worker greps at dispatch.

## Authority audit receipt

1. **Substrate exists?** `Refined<Base, predicate>` substrate
   landed (annotation-elimination Wave 1 per memory). `Unit<Q, S>`
   does NOT exist; `Quantity` / `Scale` enumerations do NOT exist.
   This canvas is substrate-fact-introduction proposal.
2. **Existing brief?** None for Unit/Quantity carrier. S9 Phase-3
   parent brief STOP-AND-ESCALATEs to this canvas. R4 carve-out
   routing C6 names this canvas as upstream input.
3. **Design-doc match?** Director feedback at gunbc#828 inbox
   response 2026-05-06 anchors the design question. No prior
   design-doc; this canvas IS the design surface.
4. **Citations live?** Verified at HEAD 2026-05-06.
5. **Carrier dissolves the bridge?** Yes — current S9 Phase-3
   shape would force dimensional refinements through value-restriction
   predicate carrier (`Refined<Int, ...>`); this is
   `feedback_reason_not_label` violation. Unit/Quantity carrier
   provides the correct algebra-axis shape; Refined wraps Unit
   for predicate-bearing dimensional types (e.g.,
   non-negative durations).

## Provenance

Drafted 2026-05-06 per Director feedback on S9 at gunbc#828 inbox
response 2026-05-06 (zesty-bear-812). Surfaces Q-Unit-1 through
Q-Unit-5 for ratification. Output: ratified carrier shape →
S9 Phase-3 reframe → R4 carve-out routing C6 closes (or stays
carved if landing defers to R4).
