# T-Substrate sub-lane 1 — cardinality subset for int-literal magnitude refinement `(M, R2 substrate)`

> **Director ad-hoc dispatch.** R2 T-Substrate sub-lane 1 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 1.
> Reports back to Director (`zesty-bear-812`); not under a standing
> manager. Cross-program heads-up to Zero-Floor Manager
> (`stern-swift-335`) at dispatch (substrate.dag-adjacent).

## Read first

- **[`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 1** — sub-lane scoping. *Does NOT commit to the full cardinality-substrate capability* (fixed-width-types by-construction, container cardinality bounds — those remain open design calls outside R2 scope).
- **[`ROADMAP.md` §"P4 type refinement" Surface int-literals row, ~`:355`](../../ROADMAP.md)** — originating analysis: *"Surface int-literals are concept-layer host-narrowed, not reconciliation-narrowed. Dissolution: `IntLit` carries an unbounded magnitude at the concept layer (candidate carrier: `std.natural` / magnitude-unbounded-natural type), narrows to the target int algebra at reconciliation."*
- **[`src/v3/std/tokenize.dag:30`](../../src/v3/std/tokenize.dag)** — current `IntLit(Int)` token; `Int = Int64 = OrderedRing<Word64>` (`dsl/std/integer.dag:43`). Source-layer host-narrowing already happens here; substrate must lift the magnitude *before* this point or carry it past tokenization.
- **[`src/v3/compiler/src/dag.rs:258-287`](../../src/v3/compiler/src/dag.rs)** + **[`dag_scalar_generated.rs:4-9`](../../src/v3/compiler/src/dag_scalar_generated.rs)** — `LiteralBits::Int(i64)` carries the int payload; no magnitude metadata.
- **[`src/v3/compiler/src/dag_scalar_generated.rs:20-25`](../../src/v3/compiler/src/dag_scalar_generated.rs)** — current `CardinalityBound { Exact(u32) | AtMostOne | Unbounded }`. **Repetition-only**, not magnitude-range. The substrate gap is exactly this.
- **[`src/v3/compiler/src/infer.rs:704-725`](../../src/v3/compiler/src/infer.rs)** — `decide` for `Behavior::Value`: all `LiteralBits::Int(_)` get `dag.int_shape()` (always `Int64`). **No narrowing today.** This is the reconciliation site that grows magnitude-aware logic.
- **[`src/v3/compiler/src/dag.rs:2210-2212`](../../src/v3/compiler/src/dag.rs)** — `int_shape()` returns the cached `TypeShape` for `std::integer::Int`; the hardcoded heuristic site.
- **[`dsl/extdeps/languages/rust/primitives.dag:1-247`](../../dsl/extdeps/languages/rust/primitives.dag)** — target `IntegerPrimitive` declarations carry `target_name` + `algebra` + `carrier` + `overflow`. Carrier names imply ranges (Byte=±128, Word32=±2³¹, …) but **substrate does not declare ranges as facts** today — the pilot mirrors them in Rust constants.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame

Today every integer literal pre-narrows to `i64` at the tokenizer boundary (`tokenize.dag:30`); reconciliation at `infer.rs:704-725` rubber-stamps that decision via `dag.int_shape()` (always `Int64`). The substrate cannot express *"this literal has magnitude that fits `i8`"* or *"this literal exceeds `u32::MAX`"* — `CardinalityBound` exists but only models repetition counts, not numeric ranges. The result: `i64::MIN` is unrepresentable as a single token (workaround: additive-inverse via `OrderedRing`); `let x: u8 = 5` works only by coincidence; out-of-range literal silently truncates.

Sub-lane closes the magnitude carrier + reconciliation narrowing. **Scope is sufficient-to-unblock T-Modeling int-lit** — not full cardinality substrate. Container-cardinality-bounds, fixed-width-types-by-construction, and other cardinality-substrate capabilities are out of scope.

## Five consumer-side requirements

1. **`IntLit` carries a magnitude carrier separable from i64-pre-narrowed bits.** Candidate: `std.natural`-style unbounded-magnitude carrier (positive magnitude + sign) as a new `LiteralBits` variant or as a parallel field. Worker picks shape; STOP-AND-ESCALATE if the choice has cross-consumer implications beyond int-lit.
2. **Substrate models numeric ranges as declared facts on integer algebras.** Each `IntegerPrimitive` (i8/i16/.../u64) gets a range fact (e.g., `range: Range<Magnitude>` or `min`+`max` fields) in `dsl/extdeps/languages/rust/primitives.dag`. Pilot's Rust-mirror range knowledge dissolves into the substrate.
3. **Reconciliation narrows magnitude-aware.** `infer.rs:704-725` `decide` arm grows: query the literal's magnitude + the call-site's expected target algebra; pick the narrowest fitting target; emit a structured `MagnitudeOutOfRange` diagnostic if no target fits. Default-when-unconstrained policy stays explicit (e.g., still default to `Int64` if no narrowing target is named) — surface the choice in PR description.
4. **`i64::MIN` representable end-to-end.** Smoke test: a `data x: Int = -9223372036854775808` declaration lowers + reconciles + emits without the additive-inverse workaround. This is the canonical regression that motivated the lane.
5. **Out-of-range literal emits structured diagnostic.** Smoke test: `data x: u8 = 256` produces a diagnostic naming the literal's magnitude + the target's range + a fix-hint suggesting a wider target. No silent truncation.

## Slice — magnitude carrier + reconciliation narrowing

1. Add the magnitude carrier shape (per req 1) to `LiteralBits` or as a parallel `IntLitMagnitude` carrier. Update `dag_scalar_generated.rs` regen path; update tokenize to populate the magnitude alongside (or instead of) `i64` pre-narrowing.
2. Add range facts to integer algebras (per req 2) in `dsl/extdeps/languages/rust/primitives.dag` (and Python/Go siblings if they declare integer primitives). Pilot mirror at `src/v3/grounding_pilot/src/lib.rs` updates accordingly.
3. Extend `infer.rs:704-725` decide arm with magnitude-aware narrowing (per req 3). Keep the default-when-unconstrained policy explicit.
4. Diagnostic for out-of-range literals (per req 5) — add a new `Diagnostic::MagnitudeOutOfRange` variant or equivalent at the appropriate location.
5. Smoke tests for reqs 4 + 5; integration test for end-to-end narrowing across multiple target algebras.

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body.
- [ ] Magnitude carrier shape lands; doc-comment explains the dissolution path (eventually `std.natural` once Class 5 substrate lifts further).
- [ ] Range facts on integer algebras (substrate-declared, not Rust-mirrored).
- [ ] Reconciliation narrows magnitude-aware; default-when-unconstrained policy documented.
- [ ] `data x: Int = -9223372036854775808` works end-to-end (`i64::MIN` smoke).
- [ ] `data x: u8 = 256` emits structured `MagnitudeOutOfRange` diagnostic.
- [ ] Existing int-literal tests pass without modification (no silent regressions).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes; `clippy --all-targets -- -D warnings` clean; `fmt --all --check` clean.
- [ ] **DB-8 `self_host_fixed_point` converges bit-identically.**
- [ ] SG-0 census deltas: any retired hand-Rust off the list; regen snapshot updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director.

- **Magnitude carrier shape is consumer-visible** — if the chosen carrier (e.g., a new `LiteralBits` variant vs a parallel field) leaks into lens producers / serializer / cementer in ways that need cross-consumer redesign, STOP.
- **Range-fact substrate touches `substrate.dag`** — coordinates with PB-Substrate (Zero-Floor); STOP for cross-program coordination.
- **Default-when-unconstrained policy** — if changing it (e.g., dropping the `Int64` default in favor of "unconstrained → diagnostic") is the cleaner shape, STOP. Director call.
- **DB-8 fixed-point drifts** — STOP immediately.
- **Magnitude carrier needs cardinality-substrate capability beyond range facts** (e.g., needs the full `Cardinality<T, Bound>` machinery to even express the range) — STOP. May indicate the sub-lane scoping was too narrow.

## Non-goals

- **Not the full cardinality-substrate capability.** Container cardinality bounds, fixed-width-types-by-construction, etc. — out of scope.
- **Not implementing T-Modeling int-lit.** That's the consumer; dispatches against this sub-lane post-merge.
- **Not refactoring the integer algebra hierarchy** beyond adding range facts.
- **Not changing the surface syntax** for integer literals.

## Reporting

- Single PR. Title: `feat(v3): T-Substrate cardinality-for-int-lit — magnitude carrier + reconciliation narrowing (unblocks T-Modeling int-lit)`.
- PR body cites this brief + addresses each of the 5 reqs + documents the chosen magnitude carrier shape + the default-when-unconstrained policy.
- On merge: signal Director; Director dispatches T-Modeling int-lit worker brief authoring (paired-blocked on this landing).

## Cross-manager note

- **Zero-Floor Manager**: heads-up at dispatch. If req 2 surfaces substrate.dag-declaration changes, coordinate.
- **Grounding Manager**: no current overlap; pilot's Rust-mirror range knowledge dissolves but pilot's structural surface stays.
