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

1. **REQ 1 RE-SCOPED 2026-04-25 (post-`wise-pike-578` STOP-AND-ESCALATE).** Worker verified at HEAD that `dsl/std/substrate.dag:31` has `LitInt(Int)` with `Int = Int64`; no `Int128`/`UInt128` types exist; `dsl/extdeps/languages/rust/primitives.dag:134-136` closes `TargetCarrier` at `Word64Carrier` (no `Word128Carrier`). Widening the canonical carrier to i128 requires either (a) adding new substrate types `Int128`/`UInt128` + `Word128Carrier` (hierarchy refactor — contradicts non-goal "Not refactoring the integer algebra hierarchy beyond adding range facts") or (b) keeping `LitInt(Int)` substrate-side while emitting Rust `i128` (representation drift between substrate declaration and emitted code — violates substrate honesty). **Director re-scope: drop the canonical-carrier-widening from req 1.** The lane's value comes from reqs 2+3+5 (range facts + reconciliation narrowing + out-of-range diagnostic), all of which work against existing `i64`/`Int64`. The `i64::MIN` smoke (req 4) is **deferred to a sibling sub-lane** that does proper Int128/Word128/Word128Carrier substrate work alongside.

   **Re-scoped req 1**: keep `LiteralBits::Int(i64)` on substrate side. Reconciliation narrowing (req 3) operates against `i64`-bounded literals; out-of-i64-range literals are rejected at tokenize via the existing additive-inverse workaround pathway (no change required). The single-authority discipline still applies to reqs 2+3+5: **range facts (req 2) use String-decimal representation** (width-independent; covers `u64::MAX` which doesn't fit in i64); reconciliation (req 3) compares String-decimal bounds against the i64 literal magnitude via host-narrowing-time parse into a wider comparison primitive (i128 host or equivalent). The literal *payload* stays `i64`; the *range-bound representation* is String to preserve fail-closed declared-facts discipline. Carrier and range-bound representations are distinct fields with distinct shapes; both serve req 1's "no carrier widening" boundary.

   **Sibling sub-lane (deferred; named dissolution trigger)**: a future T-Substrate sub-lane authors `Int128`/`UInt128` types in `dsl/std/integer.dag` + `Word128Carrier` in `primitives.dag` + widens `LitInt(Int)` to use the new `Int128` type. That sub-lane closes the `i64::MIN`-as-single-token gap and enables narrowing literals beyond the i64 range. **Do not author or imply that sibling sub-lane in this PR's body**; it's tracked separately.
2. **Substrate models numeric ranges as declared facts on integer algebras using a width-independent String-decimal representation** (consequence of req 1's carrier-non-widening). Each `IntegerPrimitive` (i8/i16/.../**u64**) gets `range_min_inclusive: String` + `range_max_inclusive: String` fields in `dsl/extdeps/languages/rust/primitives.dag` carrying the decimal magnitude (e.g., `"-128"`/`"127"` for i8; `"0"`/`"18446744073709551615"` for u64). **String-decimal, NOT i64-typed**: u64's max (`2^64 - 1`) doesn't fit in i64, so binding range bounds to the i64 literal carrier would force truncation, omission, or Rust-mirror drift — all of which would violate fail-closed declared-facts discipline. String-decimal sidesteps the carrier mismatch entirely; reconciliation (req 3) parses both the range bounds and the literal's i64 magnitude for comparison. This is the bridge form pending the sibling Int128/Word128 sub-lane that lands a typed unbounded magnitude carrier — at which point both range bounds and literal payload migrate to that typed carrier and the String-decimal bridge dissolves. Pilot's Rust-mirror range knowledge dissolves into the substrate.
3. **Reconciliation narrows magnitude-aware via String-decimal range comparison.** `infer.rs:704-725` `decide` arm grows: query the literal's i64 magnitude + the call-site's expected target algebra's `range_min_inclusive` / `range_max_inclusive` String-decimal bounds (req 2); compare by parsing the bounds (`String → i128` or equivalent host comparison primitive) and the literal magnitude into a common comparison space; pick the narrowest fitting target; emit a structured `MagnitudeOutOfRange` diagnostic if no target fits. **Comparison-space note**: parsing String-decimal bounds into i128 (or wider host primitive) is a *narrowing-time host comparison*, NOT carrier widening — the literal payload stays `i64` per req 1. The host parse is bounded by what the literal can express (any `i64`-representable literal compares against any width's String-decimal bound). Default-when-unconstrained policy stays explicit (still default to `Int64` if no narrowing target is named) — surface the choice in PR description, and **add a tracked-follow-up note** (in PR body or as a ROADMAP/active-deferral entry) that the `Int64` default is itself a host-narrowing heuristic the lane is otherwise dissolving; full dissolution is out of scope here.
4. **DEFERRED — `i64::MIN` smoke test moves to sibling sub-lane.** Per req 1's re-scope, this lane retains the existing `i64`-bounded carrier; `i64::MIN` as a single token still requires the additive-inverse workaround. The sibling sub-lane (Int128/UInt128/Word128Carrier substrate work) closes this gap. **Do not implement req 4 in this PR**; document its deferral in PR body with the sibling-sub-lane reference.
5. **Out-of-range literal emits structured diagnostic.** Smoke test: `data x: u8 = 256` produces a diagnostic naming the literal's magnitude + the target's range + a fix-hint suggesting a wider target. No silent truncation.

## Slice — range facts + reconciliation narrowing (against existing i64 carrier)

**Note (post-2026-04-25 re-scope)**: per req 1, the `LiteralBits::Int(i64)` carrier is **NOT widened** in this lane; carrier-widening defers to a sibling Int128/Word128 sub-lane. This slice operates against the existing `i64` carrier.

1. **(NOT in scope — deferred)** Carrier widening from `LiteralBits::Int(i64)` to a wider canonical form. This is the deferred req 1 work; sibling sub-lane handles. Worker should NOT touch `LiteralBits::Int` shape, `dag_scalar_generated.rs` regen for that variant, or tokenize's i64 parse path.
2. Add range facts to integer algebras (per req 2) in `dsl/extdeps/languages/rust/primitives.dag` (and Python/Go siblings if they declare integer primitives). **Range bounds use String-decimal representation per req 2** (e.g., `range_min_inclusive: "0"`, `range_max_inclusive: "18446744073709551615"` for u64) — width-independent so u64 bounds (which exceed i64) are faithfully representable. Pilot mirror at `src/v3/grounding_pilot/src/lib.rs` updates accordingly.
3. Extend `infer.rs:704-725` decide arm with magnitude-aware narrowing (per req 3). Use the existing `i64`-bounded literal payload + the new substrate-declared range facts. Keep the default-when-unconstrained policy explicit.
4. Diagnostic for out-of-range literals (per req 5) — add a new `Diagnostic::MagnitudeOutOfRange` variant or equivalent at the appropriate location. Triggered when an `i64`-representable literal exceeds the call-site's target-algebra range (e.g., `data x: u8 = 256`).
5. Smoke tests for req 5 only; **req 4 (`i64::MIN`) is deferred** — no smoke test for it in this PR; document deferral in PR body. Integration test for end-to-end narrowing across multiple target algebras (i8, i16, i32, i64, u8, u16, u32, u64) using `i64`-representable literals.

## Acceptance

- [ ] Reqs 2, 3, 5 satisfied + documented in PR body. Req 1 explicitly noted as RE-SCOPED (carrier-widening deferred to sibling sub-lane). Req 4 explicitly noted as DEFERRED (depends on sibling sub-lane).
- [ ] `LiteralBits::Int(i64)` carrier untouched (no widening; no parallel; no `dag_scalar_generated.rs` shape change for this variant).
- [ ] Range facts on integer algebras (substrate-declared, not Rust-mirrored) using **String-decimal representation** (`range_min_inclusive: String` + `range_max_inclusive: String`) — width-independent; u64 bounds expressible without truncation.
- [ ] Reconciliation narrows magnitude-aware against existing `i64` carrier; default-when-unconstrained policy documented.
- [ ] **DEFERRED**: `data x: Int = -9223372036854775808` (`i64::MIN` smoke; sibling sub-lane will implement).
- [ ] `data x: u8 = 256` emits structured `MagnitudeOutOfRange` diagnostic.
- [ ] Existing int-literal tests pass without modification (no silent regressions).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes; `clippy --all-targets -- -D warnings` clean; `fmt --all --check` clean.
- [ ] **DB-8 `self_host_fixed_point` converges bit-identically.**
- [ ] SG-0 census + regen-output deltas: this lane is **adding** substrate (range facts only — carrier untouched), not retiring hand-Rust files; expected census movement is **the pilot's Rust-mirror range constants becoming substrate-declared** (so the corresponding pilot Rust code that previously hard-coded ranges either retires or shrinks). Any regen snapshot updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director.

- **Pressure to widen the carrier** — req 1's re-scope explicitly defers carrier-widening to a sibling sub-lane. If execution surfaces that range-fact narrowing requires touching `LiteralBits::Int(i64)` shape (e.g., narrowing logic needs a wider literal payload to represent magnitudes that the user wrote), STOP. That's the boundary the re-scope drew; carrier-widening belongs in the sibling Int128/Word128 sub-lane.
- **Range-fact substrate touches `dsl/std/substrate.dag` (NOT `dsl/extdeps/languages/*/primitives.dag`)** — extdeps language files are in-scope for this lane; only edits to `dsl/std/substrate.dag` itself trigger the STOP for cross-program coordination with PB-Substrate (Zero-Floor). Range facts on `IntegerPrimitive` in `dsl/extdeps/languages/rust/primitives.dag` (req 2) are NOT substrate.dag changes and do NOT require Zero-Floor coordination.
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
