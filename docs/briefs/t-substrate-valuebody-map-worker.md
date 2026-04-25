# T-Substrate sibling sub-lane — top-level `ValueBody::Map` extension `(M, R2 substrate)`

> **Director ad-hoc dispatch.** Sibling future T-Substrate sub-lane to
> the `ValueBody::List` sub-lane (worker brief on [PR #790](https://github.com/gunb-ai/gunbc/pull/790),
> branch `briefs/t-substrate-valuebody-list`; lands at
> `docs/briefs/t-substrate-valuebody-list-worker.md` once #790 merges)
> per [`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" 4th
> sub-lane *"Excluded from this sub-lane (substrate shape differs)"*
> callout. Map-shaped consumers tracked separately because they need
> distinct `ValueBody::Map(...)` substrate work, not the
> `ValueBody::List(...)` extension that closes tokenizer + Engine.

## Read first

- **The sibling `ValueBody::List` worker brief.** Lives on [PR #790](https://github.com/gunb-ai/gunbc/pull/790) (branch `briefs/t-substrate-valuebody-list`); will land at `docs/briefs/t-substrate-valuebody-list-worker.md` once #790 merges. **If #790 has merged**: read it for the discipline pattern this brief inherits. **If #790 has not yet merged**: read it directly off the branch (`git show origin/briefs/t-substrate-valuebody-list:docs/briefs/t-substrate-valuebody-list-worker.md`) or via the PR diff. Inherited pattern (summarized inline so this brief is self-contained even when #790 isn't on main yet): (a) extend `ValueBody` enum with the new variant (variant placement at `dag.rs:258-287`); (b) add a new arm to `lower_data_item` at `lower.rs:2378-2436` (before the `Unparsed` fallback); (c) R14 hard-fail path naturally narrows by construction (no edits to `reject_user_unparsed_scaffolds` detection logic — only the diagnostic message text narrows precisely to the still-unsupported residual); (d) exhaustive `ValueBody::` match audit across the codebase (no wildcard `_` swallowing the new variant); (e) coproduct dissolution receipt + four-pattern check per `feedback_coproduct_dissolution` and the `LoopBound` precedent at [`docs/design-mutual-recursion-lowering.md:117-134`](../design-mutual-recursion-lowering.md); (f) DB-8 fixed-point as no-compromise gate; (g) substrate-doc-comment update on `ValueBody` itself reflecting the new variant. The map sub-lane differs from list only in the variant shape + consumer set + the load-bearing parser-extension dependency (see below).
- **[`src/v3/compiler/src/dag.rs:258-287`](../../src/v3/compiler/src/dag.rs)** — current `ValueBody` enum. New variant placement.
- **[`src/v3/compiler/src/dag.rs:326-352`](../../src/v3/compiler/src/dag.rs)** — current `FieldValue` enum (5 variants: `Literal | Reference | Record | List | Variant`). **No `Map` variant exists today.** New nested `FieldValue::Map(Vec<(String, FieldValue)>)` — string-key form preferred since all 22 candidate consumers use `Map<String, _>` keys (see consumer survey below).
- **[`src/v3/compiler/src/lower.rs:2378-2436`](../../src/v3/compiler/src/lower.rs)** — `lower_data_item`. New arm for map-shaped surface expressions, alongside the `ValueBody::List` arm landing in #790 / parallel sub-lane.
- **[`src/v3/compiler/src/lower.rs:2224-2273`](../../src/v3/compiler/src/lower.rs)** — `reject_user_unparsed_scaffolds` (R14). Diagnostic message text narrows again ("map literals" → "" once both list + map land; or remove the literal-list-of-not-yet-supported entirely).
- **[`src/v3/compiler/src/dag.rs:1419-1430`](../../src/v3/compiler/src/dag.rs)** — **the existing P2 drift ratchet** (`kernel_algebra_profile` Rust mirror; not at `:1530` as the cascade text approximated). Mirror function carries the seven-variant map; current ratchet test `m2_substrate_inhabitance_test::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`. Mirror dissolves once `ValueBody::Map` lands and the `.dag` table lowers structurally.
- **[`dsl/std/algebra.dag:459-467`](../../dsl/std/algebra.dag)** — primary consumer: `data kernel_algebra_profile: Map<String, AlgebraProfile> = {...}` (7 entries; map-literal body).
- **Parser prerequisite split into sibling sub-lane post-`wise-boar-480` STOP-AND-ESCALATE (2026-04-25).** Original bundled brief presumed parser extension was absorbable; worker investigation showed it's non-trivial (`SurfaceExpr::Map` + `looks_like_map_literal` lookahead + `parse_map_literal` body + regen + ~20+ exhaustive-match-site updates). **Parser side now lives in [`docs/briefs/t-substrate-valuebody-map-parser-worker.md`](t-substrate-valuebody-map-parser-worker.md) as a sibling pre-requisite sub-lane.** This substrate brief is now scoped to **post-parser-extension state**: assumes `SurfaceExpr::Map` is already emitted by the parser; this PR consumes that via a new `lower_data_item` arm + `ValueBody::Map`. **Dispatch sequence**: parser sub-lane lands first; this substrate sub-lane dispatches against the post-parser state.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame

Sibling to the `ValueBody::List` sub-lane. `Map<K, V>` top-level data declarations fall through to `ValueBody::Unparsed` and trigger R14 hard-fail; the substrate gap is exactly the missing `ValueBody::Map(...)` variant + its `FieldValue::Map(...)` element-shape sibling + a `lower_data_item` arm.

**ROI is broad** — the consumer survey found 22 top-level `Map<String, _>` declarations across the tree (vs the 2 list-of-sum consumers for the sibling sub-lane). One substrate work simultaneously unblocks: the `kernel_algebra_profile` P2 drift ratchet (primary), 5 other `dsl/std/` map declarations, 11 `dsl/extdeps/languages/*/emit.dag` per-language template tables, and 5 `dsl/extdeps/languages/*/syntax.dag` keyword/literal tables.

**Parser prerequisite resolved via split (2026-04-25).** Original brief bundled the parser extension; `wise-boar-480` worker correctly STOP-AND-ESCALATE'd that the parser scope wasn't absorbable. Director split into [`t-substrate-valuebody-map-parser-worker.md`](t-substrate-valuebody-map-parser-worker.md) (parser side, dispatched first) + this brief (substrate side, post-parser). Worker dispatching this brief assumes the parser sub-lane has landed and `SurfaceExpr::Map` is available.

## Five consumer-side requirements

1. **`ValueBody::Map(Vec<(String, FieldValue)>)` variant.** String-key form preferred (all 22 consumers use `Map<String, _>`); supports the consumer set without inventing a key-FieldValue carrier. If a consumer eventually needs non-string keys, that's a follow-up sibling extension. Worker decides whether to land the more general `Vec<(FieldValue, FieldValue)>` form upfront — surface choice in PR description.
2. **`FieldValue::Map(...)` nested element-shape variant.** Mirrors `FieldValue::List(Vec<FieldValue>)` for structural uniformity. Same key-shape choice as req 1.
3. **Parser prerequisite — already landed via sibling sub-lane.** This brief assumes the parser sub-lane ([`t-substrate-valuebody-map-parser-worker.md`](t-substrate-valuebody-map-parser-worker.md)) has landed first and `SurfaceExpr::Map` is emitted by the parser. **STOP-AND-ESCALATE if dispatching against this brief and the parser sub-lane has NOT landed** — that's a sequencing error.
4. **Lowerer arm in `lower_data_item`** producing `ValueBody::Map`. Mirrors the `ValueBody::List` arm.
5. **`kernel_algebra_profile` mirror dissolution + Rust mirror retirement.** Smoke + integration test asserting: `Dag::new()` loads `kernel_algebra_profile` without R14; `value_body` is `Some(ValueBody::Map(entries))` with 7 entries; the seven-variant mirror function at `dag.rs:1419-1430` retires; the `m2_substrate_inhabitance_test::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` ratchet's parallel-representation drift surface dissolves. PR body lists which other 21 consumers' Rust mirrors (if any) similarly retire vs which stay (substrate-shape audit).

## Slice — `ValueBody::Map` extension

1. Extend parser per req 3 (or STOP if scope balloons).
2. Add `ValueBody::Map(Vec<(String, FieldValue)>)` per req 1; `FieldValue::Map(...)` per req 2.
3. Add lowerer arm per req 4.
4. R14 diagnostic message narrows.
5. `kernel_algebra_profile` mirror retires per req 5; the substrate-declared map becomes the single authority.
6. Audit + retire other 21 consumers' Rust mirrors where the substrate shape now reads them; document which stay (and why) in PR body.
7. Doc-comment update at `dag.rs:259-262` (similarly to `ValueBody::List` brief — call out remaining map-shape variants as next dissolution targets, if any, e.g., non-string-key maps).

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body.
- [ ] `ValueBody::Map(Vec<(String, FieldValue)>)` + `FieldValue::Map(...)` lands; doc-comment updated.
- [ ] Parser prerequisite confirmed landed (sibling sub-lane PR merged); `SurfaceExpr::Map` available pre-PR.
- [ ] Lowerer arm produces `ValueBody::Map` for map-shaped surface exprs.
- [ ] R14 diagnostic message narrows accordingly.
- [ ] `kernel_algebra_profile` lowers to `ValueBody::Map` with 7 entries; Rust mirror at `dag.rs:1419-1430` retires; ratchet test updated or retired.
- [ ] Consumer-mirror audit completed for the other 21 declarations; each either retired (substrate now reads) or documented-stay (with reason).
- [ ] All `ValueBody::` exhaustive matches across codebase updated.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas: any retired hand-Rust off the list.

## STOP-AND-ESCALATE

Surface to Director.

- **Parser sub-lane has not landed** — sequencing error per the parser-prerequisite split. STOP and verify the parser sub-lane PR has merged before proceeding.
- **Element-shape choice (string-key vs general key)** — if a consumer in the audit needs non-string keys (none surfaced today, but possible), STOP. Director-call on whether to land general form upfront.
- **`ValueBody::` exhaustive-match wildcard `_` swallowing the new variant** — same gate as the sibling list sub-lane. STOP if found; convert to exhaustive in this PR or follow-up.
- **Substrate.dag declaration changes** — coordinate with PB-Substrate (Zero-Floor); STOP.
- **Consumer-mirror retirement breaks downstream** — if retiring a Rust mirror reveals a consumer pattern beyond simple substrate-walk, STOP. Audit reveals work, not just retirement.
- **DB-8 fixed-point drifts** — STOP immediately.
- **Serializer / cementer / fixed-point machinery doesn't extend** — same gate as sibling sub-lane.

## Non-goals

- **Not adding non-string-key Map support.** Defer to follow-up if needed.
- **Not migrating consumers beyond mirror retirement** — semantic consumption stays as-is; only the storage shifts from Rust mirror to substrate.
- **Not implementing `kernel_algebra_profile` consumers' richer features** — algebra-profile semantics unchanged; only the carrier shifts.
- **Not closing the parser-emit-shape design questions** for map syntax beyond what's needed for top-level data declarations.

## Reporting

- Single PR. Title: `feat(v3): T-Substrate ValueBody::Map — top-level map variant (retires kernel_algebra_profile mirror; unblocks 22 consumers)`.
- PR body cites this brief + addresses each of the 5 reqs + documents element-shape choice + lists consumer-mirror audit results.
- On merge: signal Director; Director may follow up with map-specific consumer-migration briefs if any consumers from the 21 sibling list need separate workstreams.

## Cross-manager note

- **Zero-Floor Manager**: heads-up. Substrate.dag-adjacent; `kernel_algebra_profile` mirror retirement is a ratchet-relevant change.
- **Grounding Manager**: no current overlap.
- **Surface Manager / parser owners**: heads-up — req 3 parser extension overlaps surface-syntax authority. Coordinate at dispatch if parser changes are non-trivial.
