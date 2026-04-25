# T-Substrate 4th sub-lane — top-level `ValueBody::List` extension `(M, R2 substrate)`

> **Director ad-hoc dispatch.** R2 T-Substrate sub-lane per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" (4th sub-lane:
> Top-level `ValueBody` list/sum subset). Reports back to Director
> (`zesty-bear-812`); not under a standing manager. Cross-program
> heads-up at dispatch to Grounding Manager (`crisp-seal-366`) and
> Surface Manager (whoever owns the live tokenizer-charclass-phase-2
> queue), since both their downstream lanes unblock the moment this lands.

## Read first

- **[`docs/r2-structure.md`](../r2-structure.md) §"Goal 3"** — the 4th T-Substrate sub-lane definition and the dual-consumer scoping (post-#782 cascade: 2 consumers, both list-of-sum; `kernel_algebra_profile` excluded as map-shaped).
- **[`src/v3/compiler/src/dag.rs:258-287`](../../src/v3/compiler/src/dag.rs)** — current `ValueBody` enum: 3 variants (`Unparsed(SourceSpan)` catch-all; `Structural { fields: Vec<(String, FieldValue)> }` for record bodies; `Scalar(LiteralBits)` for scalar constants). The doc-comment at `:259-262` explicitly names the M2+ parser extension as the dissolution trigger.
- **[`src/v3/compiler/src/dag.rs:327-352`](../../src/v3/compiler/src/dag.rs)** — `FieldValue` enum, including the existing nested `List(Vec<FieldValue>)` variant at `:343`. The natural element shape for the new top-level `ValueBody::List` mirrors this — `Vec<FieldValue>` keeps element semantics uniform with record-field values and variant payloads.
- **[`src/v3/compiler/src/lower.rs:2378-2436`](../../src/v3/compiler/src/lower.rs)** — `lower_data_item` produces `ValueBody`. Today the match at `:2411-2432` handles `SurfaceExpr::Record` → `lower_record_to_structural` (`:2413`) and scalar literals (`:2416`). Anything else (list literal `[...]`, map literal, opaque body) falls through to `Unparsed(body_span)` at `:2434-2436`. **A new arm here is where `ValueBody::List` gets produced.**
- **[`src/v3/compiler/src/lower.rs:2224-2273`](../../src/v3/compiler/src/lower.rs)** — `reject_user_unparsed_scaffolds` is the R14 hard-fail path. Detection at `:2245`; rejection emits a `Diagnostic::ResolveError` naming "M1(2.8) user code cannot yet use record / list / map literals inside data bodies (see DOWNSTREAM_REQUIREMENTS.md class-5 gap #3)". Once the new variant lands, list-bodied data declarations stop falling through and stop being rejected.
- **[`dsl/extdeps/languages/rust/primitives.dag:196`](../../dsl/extdeps/languages/rust/primitives.dag)** — Engine consumer: `data rust_pilot_primitives: List<RustPrimitive> = [...]` (10-element list of sum-typed entries). Lowers to `Unparsed` today; should lower to `ValueBody::List(Vec<FieldValue>)` after this PR.
- **[`src/v3/compiler/tokenize.dag:69-79`](../../src/v3/compiler/tokenize.dag)** — Tokenizer consumer comment: names `data ascii_scan_order: List<CharClass> = [...]` as the Phase 2 dissolution trigger waiting on this substrate work. The data declaration itself does not yet exist; landing this substrate work unblocks its authoring (handed off to whoever owns tokenizer-charclass-phase-2 dispatch — not part of this sub-lane).
- **[`src/v3/compiler/src/bootstrap.rs:37-46`](../../src/v3/compiler/src/bootstrap.rs)** (post-#776 merged) — comment block documenting the loader-close worker's discovery of this gap and the Phase 1/2 split that hangs on this substrate sub-lane closing.
- **[`docs/briefs/t-ground-engine-phase-1-typestructure.md`](t-ground-engine-phase-1-typestructure.md)** — the Engine Phase 1 brief that consumes the loader's accessor today; Phase 2 (full enumeration) blocks on this PR.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)** — same governing rules.

## Frame

v3's `ValueBody` enum supports record bodies (`Structural`) and scalar bodies (`Scalar`) but not list bodies. Top-level data declarations whose RHS is a list literal (`= [...]`) fall through to `ValueBody::Unparsed(SourceSpan)`, which `reject_user_unparsed_scaffolds` then hard-fails with R14 / class-5-gap-3 diagnostics.

This blocks two named consumers, both list-of-sum shaped:
1. **Engine sharpened-(b) Phase 2** — full pilot enumeration via symbolic walk of `rust_pilot_primitives: List<RustPrimitive>`. (Engine Phase 1 walks the type structure today; Phase 2 needs the actual list to enumerate.)
2. **Tokenizer charclass phase-2** — `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` and similar list-of-sum top-level declarations. (Surface Manager's deferral target post-PR #762.)

This sub-lane closes the substrate gap. Engine Phase 2 + tokenizer charclass phase-2 dispatch the moment this lands. **Out of scope explicitly: `kernel_algebra_profile: Map<String, AlgebraProfile>`** — map-shaped, needs distinct `ValueBody::Map(...)` substrate work, tracked as a sibling future T-Substrate sub-lane per #782 cascade.

## Seven consumer-side requirements (bake in upfront)

1. **`ValueBody::List(Vec<FieldValue>)` variant added.** Element shape is `FieldValue` (not a new type), mirroring the existing nested `FieldValue::List(Vec<FieldValue>)` at `dag.rs:343`. Structural uniformity: list elements follow the same value-shape rules as record-field values, variant payloads, and other `FieldValue` carriers. If the worker concludes a different element shape is structurally cleaner (e.g., a new `ListElement` carrier with provenance), STOP-AND-ESCALATE — the choice has cross-consumer implications.
2. **Lowerer arm in `lower_data_item` produces `ValueBody::List` for list-shaped surface expressions.** Mirrors the existing `SurfaceExpr::Record` → `lower_record_to_structural` arm at `lower.rs:2413`. Each list element lowers to a `FieldValue` per its surface shape (variant constructor → `FieldValue::Variant`, scalar → `FieldValue::Literal`, nested record → `FieldValue::Record`, identifier reference → `FieldValue::Reference`). Existing element-shape lowering reuses the same machinery as record-field-value lowering.
3. **R14 hard-fail path naturally narrows.** `reject_user_unparsed_scaffolds` at `lower.rs:2224-2273` continues to reject `ValueBody::Unparsed`; list-bodied declarations stop producing `Unparsed` and therefore stop being rejected. **No edits to the R14 path itself** — the gap closes by construction (new variant lifts these cases out of the catch-all). If the worker finds the R14 diagnostic message needs to update its error text (it currently says "list / map literals inside data bodies"), narrow the message to mention only `map literals` post-PR.
4. **Both named consumers' data declarations end-to-end visible structurally.** Acceptance proof: the Engine pilot consumer at `dsl/extdeps/languages/rust/primitives.dag:196` (`rust_pilot_primitives`) lowers to `ValueBody::List` with 10 `FieldValue::Variant` elements; integration test asserts the count + the first/last variant constructor identity. (Tokenizer's `ascii_scan_order` declaration does not exist yet — its authoring is downstream tokenizer-charclass-phase-2 work; this PR demonstrates the substrate works via the Engine consumer.)
5. **Compiler dissolution trigger annotation updated in same PR.** The doc-comment at `dag.rs:259-262` ("The body's shape (record / map / list / variant literal) awaits M2+ parser extension") must update to reflect what landed (record + list now structural; map still pending — call out the `kernel_algebra_profile` sibling sub-lane as the next dissolution target). Don't leave contradictory documentation on the substrate type itself.
6. **`std.unicode` bootstrap/load-set inclusion** (per `docs/r2-structure.md` §"Goal 3" 4th sub-lane title — *"Top-level `ValueBody` list/sum subset + `std.unicode` bootstrap inclusion"*). The tokenizer-charclass-phase-2 unblock claim depends on `std.unicode::CharClass` (and the rest of `std.unicode`) being resolvable from the `Dag::new()` bootstrap path so the future `data ascii_scan_order: List<CharClass> = [...]` declaration's element type resolves to the substrate-known `CharClass` type. Add `std.unicode` to the bootstrap fixture set (likely via `EXTDEPS_BOOTSTRAP_FIXTURES` extension that landed via PR #776 for `rust/primitives.dag`, or via the staged-files set, depending on where `std.unicode` naturally lives). **If `std.unicode` doesn't exist yet as a `.dag` file**: STOP-AND-ESCALATE — its authoring is upstream of this sub-lane's tokenizer-unblock claim, and may need to be a sibling pre-req. If the worker concludes the cleanest path is to **narrow this PR's unblock claim to Engine-only and defer the `std.unicode` bootstrap inclusion + tokenizer-unblock to a sibling sub-lane**, that's a Director-call STOP. Acceptance: integration test confirms `std.unicode::CharClass` (or whatever `std.unicode` content this PR includes) resolves from a fresh `Dag::new()`-loaded compilation context. Without this requirement satisfied, the brief's claim that this PR unblocks tokenizer-charclass-phase-2 is unfounded.
7. **Coproduct dissolution receipt + four-pattern check.** Per `feedback_coproduct_dissolution` and the in-tree pattern at [`docs/design-mutual-recursion-lowering.md:117-134`](../design-mutual-recursion-lowering.md) (the `LoopBound` coproduct receipt is the canonical form): every new substrate coproduct variant must carry a four-pattern dissolution receipt before being stamped 🟢 TERMINAL. `ValueBody` is a substrate coproduct (lives on `Declaration` in `dag.rs`); adding the `List` variant is exactly the kind of substrate-coproduct extension that needs the receipt. Receipt structure (per `LoopBound` precedent): explicitly walk each of the four dissolution patterns and stamp the variant's disposition; document either dissolution to a sibling structural shape OR justification why the variant stays as-coproduct. Land the receipt as a doc-comment on `ValueBody` itself OR in a sibling design doc cited from the variant's doc-comment — worker's call on placement, but the receipt must be authored in this PR. **No silent stamp-without-receipt.** Precedent for why this requirement is load-bearing: PR #589 surface-coproduct violations on 2026-04-20 (six new coproduct declarations landed without dissolution receipts; required follow-up correction).

## Slice — top-level `ValueBody::List` extension

**Goal:** make top-level `data X: List<T> = [...]` declarations lower to `ValueBody::List(Vec<FieldValue>)` rather than `ValueBody::Unparsed(SourceSpan)`. Engine sharpened-(b) Phase 2 + tokenizer charclass phase-2 dispatch unblocked; `kernel_algebra_profile` map-shaped consumer remains out of scope.

**Round-trip:**

1. Add `ValueBody::List(Vec<FieldValue>)` variant at `dag.rs:258-287`. Update the doc-comment at `:259-262` per req 5.
2. Implement `lower_list_to_structural` (or equivalent — worker's call on naming) in `lower.rs`, mirroring `lower_record_to_structural`. Add the new arm to `lower_data_item`'s match at `:2411-2432` (before the `Unparsed` fallback).
3. Update any consumer that pattern-matches on `ValueBody` to handle the new variant (audit by `grep` for `ValueBody::` callsites; lens consumers / serializer / cementer / DB-8 fixed-point machinery may need new arms). **Use exhaustive matches**; prefer adding `match` arms over wildcard `_` to surface every consumer that needs to think about the new variant — per `feedback_missing_checks_review_heuristic`.
4. R14 diagnostic message at `lower.rs:2261-2271` narrows to mention only `map literals` (record + list now supported).
5. Integration test asserting Engine pilot consumer (`rust_pilot_primitives`) lowers structurally:
   - `Dag::new()` loads `rust_pilot_primitives` without R14 hard-fail.
   - `value_body` is `Some(ValueBody::List(elements))` with `elements.len() == 10`.
   - Each element is `FieldValue::Variant { constructor, payload }`; first element's constructor identity is `IntegerPrimitive`, last element's constructor identity is `NonIntegerPrimitive` (or whichever per the actual file content).
6. **Add `std.unicode` to bootstrap fixture set per req 6** (or STOP-and-document if `std.unicode` doesn't exist as `.dag` yet → narrow this PR's unblock claim to Engine-only). Integration test confirms `std.unicode::CharClass` resolves from `Dag::new()`.
7. **Author the four-pattern coproduct dissolution receipt for `ValueBody::List` per req 7.** Land as doc-comment on `ValueBody` or in a sibling design doc cited from the variant. Walk all four patterns; stamp the variant's disposition.
8. Regen the snapshot (`bootstrap_generated.rs`) since the substrate ratchets shift.

## Acceptance

- [ ] All 7 consumer-side requirements satisfied + documented in PR body.
- [ ] `ValueBody::List(Vec<FieldValue>)` variant lands in `dag.rs:258-287`; doc-comment updated.
- [ ] `lower_data_item` arm produces `ValueBody::List` for list-shaped surface expressions.
- [ ] R14 diagnostic message narrowed (no longer mentions list literals).
- [ ] Integration test passes: `rust_pilot_primitives` lowers structurally with the asserted element shape.
- [ ] **`std.unicode` (or worker-narrowed Engine-only deferral) — req 6 disposition documented.** If included: integration test confirms `std.unicode::CharClass` resolves from `Dag::new()`. If deferred: STOP-and-Director-call receipt cited.
- [ ] **Coproduct dissolution receipt for the new `List` variant — req 7.** Four-pattern walk authored either as `ValueBody` doc-comment or in a sibling design doc cited from the variant. No silent stamp.
- [ ] All `ValueBody::` exhaustive matches across the codebase updated to handle the new variant.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] **DB-8 `self_host_fixed_point` converges bit-identically** (the no-compromise gate; same as PB-1's).
- [ ] SG-0 census deltas: any retired hand-Rust off the list (unlikely for this PR, but check); regen snapshot updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director (`zesty-bear-812`); do not absorb scope.

- **If element shape `Vec<FieldValue>` doesn't carry enough information** — e.g., need explicit element-position provenance, or the consumers want a typed `ListElement` carrier with metadata. STOP — the choice is consumer-visible and the cross-consumer implications need Director sign-off.
- **If the lowerer needs a new surface-expr variant the parser doesn't produce yet** — e.g., the parser collapses list literals into something other than the expected `SurfaceExpr` shape. STOP — surface the parser-side gap; may need a coupled parser PR or a re-scoping decision.
- **If `ValueBody::` exhaustive-match audit reveals a consumer that pattern-matches via wildcard `_` and silently swallows the new variant** — STOP. Prior reviewer pattern caught this on similar substrate extensions; surfacing it here lets us decide whether to convert to exhaustive in this PR or open a follow-up.
- **If the new variant requires changes to substrate.dag declarations** (i.e., the `.dag`-level declaration of what ValueBody is) — STOP. That's PB-Substrate proper territory (Zero-Floor Manager); coordinate before landing.
- **If DB-8 fixed-point drifts** — STOP immediately. Same no-compromise gate as PB-1, PB-1-e, AtomPayload.
- **If serialization / cementer / fixed-point machinery doesn't naturally extend** — STOP. The new variant must round-trip through the snapshot pipeline; if the existing pattern doesn't generalize, that's a substrate-shape question.

## Non-goals

- **Not adding `ValueBody::Map(...)`.** That's the sibling `kernel_algebra_profile` sub-lane, distinct substrate shape, tracked separately. Out of scope per #782 cascade re-scoping.
- **Not authoring `data ascii_scan_order: List<CharClass> = [...]`.** That declaration is downstream tokenizer-charclass-phase-2 work, not this substrate sub-lane. This PR only proves the substrate works via the Engine consumer that already exists.
- **Not implementing Engine sharpened-(b) Phase 2.** Engine Phase 2 is owned by R2 Grounding Manager and dispatches against this substrate landing. Out of scope.
- **Not touching emit pipeline.** Emit consumers of `ValueBody` may need new arms — that's a mechanical addition (req 3 covers it), but no emit-side semantic changes are in scope.
- **Not migrating `kernel_algebra_profile`** — same exclusion as the consumer scoping.
- **Not changing the R14 detection logic** — only the diagnostic *message* text narrows.

## Reporting

- Single PR. Title pattern: `feat(v3): T-Substrate ValueBody::List — top-level list/aggregate variant unblocks Engine Phase 2 + tokenizer charclass phase-2`.
- PR description cites this brief + addresses each of the 5 consumer requirements explicitly + documents the chosen element shape (`Vec<FieldValue>` vs alternative if the worker took a different call).
- On merge: signal Director (`zesty-bear-812`); Director signals Grounding Manager (`crisp-seal-366`) to dispatch Engine Phase 2 worker brief, and signals whoever owns tokenizer-charclass-phase-2 dispatch (Surface Manager / Director ad-hoc) to author/dispatch that work.
- On STOP-AND-ESCALATE: surface to Director; Director resolves before resuming.

## Cross-manager note

- **Grounding Manager (`crisp-seal-366`)**: heads-up at dispatch. Engine Phase 2 brief at [`t-ground-engine-phase-2-enumeration.md`](t-ground-engine-phase-2-enumeration.md) (forward-planned, not yet authored per PR #785) will be authorable against the live `ValueBody::List` shape after this PR lands. Pilot `RUST_PILOT_PRIMITIVES` mirror retirement happens in Phase 2.
- **Zero-Floor Manager (`stern-swift-335`)**: heads-up at dispatch. This PR doesn't touch substrate.dag declarations, so no PB-Substrate conflict. If req 5's doc-comment update or the exhaustive-match audit surfaces a substrate.dag declaration that needs updating, coordinate (the substrate.dag carrier may need a parallel update).
- **Surface Manager** (current owner of the tokenizer charclass phase-2 deferral queue): heads-up at dispatch. Tokenizer's `ascii_scan_order` declaration becomes authorable post-this-PR; the substrate gap that motivated the phase-2 deferral closes.
