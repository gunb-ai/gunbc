# T-Substrate sibling sub-lane — parser extension for top-level map literals `(M, R2 substrate prereq)`

> **Director ad-hoc dispatch.** Sibling pre-requisite to the
> [`t-substrate-valuebody-map-worker.md`](t-substrate-valuebody-map-worker.md)
> substrate sub-lane. Authored 2026-04-25 post-`wise-boar-480`
> STOP-AND-ESCALATE on the bundled ValueBody::Map brief — worker
> verified that adding `SurfaceExpr::Map` + parse lookahead +
> `parse_parser_body.txt` edits is non-trivial and fits the bundled
> brief's STOP gate verbatim. Director picked **split** — this brief
> lands the parser side; the substrate brief narrows to "post-parser-
> extension" scope and dispatches sequentially.
>
> Reports back to Director (`zesty-bear-812`); Surface Manager territory
> overlap (parser surface-syntax authority).

## Read first

- **[`docs/briefs/t-substrate-valuebody-map-worker.md`](t-substrate-valuebody-map-worker.md)** — sibling substrate sub-lane brief (post-this-PR-narrowed). The substrate side blocks on this PR landing.
- **[`src/v3/std/parse_surface.dag`](../../src/v3/std/parse_surface.dag)** — `SurfaceExpr` declaration; today carries `Record` + `List` variants, **no `Map` variant**. New `SurfaceExpr::Map { entries: List<SurfaceMapEntry> }` (or equivalent — worker's call on element shape) lands here.
- **[`src/v3/compiler/src/parse_generated.rs`](../../src/v3/compiler/src/parse_generated.rs)** — auto-generated from `parse_surface.dag` + `parse_parser_body.txt`. Header line 1: *"Regenerate instead of hand-editing"*. Per `feedback_no_generated_code_on_disk`, edits flow through `parse_surface.dag` + `parse_parser_body.txt` authority then regen.
- **[`src/v3/compiler/parse_parser_body.txt`](../../src/v3/compiler/parse_parser_body.txt)** — parse-body algorithm authority. The `parse_data_item` path disambiguates record-vs-other via `looks_like_record_literal`, which requires `{ Ident :` (per `parse_generated.rs:262`). String-keyed map syntax `{ "Int": OrderedRingProfile, ... }` fails this lookahead and falls into `skip_brace_balanced` → `body: None` → `ValueBody::Unparsed`. Need a sibling `looks_like_map_literal` (e.g., `{ String :`) lookahead variant + a new `parse_map_literal` body.
- **[`dsl/std/algebra.dag:459-467`](../../dsl/std/algebra.dag)** — primary motivating consumer: `data kernel_algebra_profile: Map<String, AlgebraProfile> = {...}` (7 entries; map-literal body using string keys).
- **Cross-consumer survey from sibling substrate brief**: 22 top-level `Map<String, _>` declarations across `dsl/std/`, `dsl/extdeps/languages/*/emit.dag`, and `dsl/extdeps/languages/*/syntax.dag`. All use `Map<String, _>` — string-keyed. ROI is broad.
- **[`src/v3/compiler/src/lower.rs`](../../src/v3/compiler/src/lower.rs)** — exhaustive `match expr` sites (the `wise-boar-480` worker survey reported ~20+; **worker should re-confirm via `grep -n "match.*SurfaceExpr" src/v3/compiler/src/lower.rs` + sibling consumer files at dispatch** — survey number is from a snapshot, not authority). Adding a new `SurfaceExpr` variant means every exhaustive-match site must be updated to handle it (with a fall-through for non-data-body contexts where map literals aren't expected).

## Frame

The substrate `ValueBody::Map` extension cannot land standalone — the parser must first emit `SurfaceExpr::Map` for the lowerer to consume. Without parser support, top-level map literals never reach the new `ValueBody::Map` arm in `lower_data_item`; the substrate work would have no test surface beyond hand-built fake `SurfaceExpr` values.

This sub-lane lands the parser side. Output:

1. New `SurfaceExpr::Map { entries: List<SurfaceMapEntry> }`. **Element shape MUST structurally restrict the key to a string-literal carrier** (per non-goal "String-key maps only for this PR" — making non-string keys representable would force behavioral enforcement and violates illegal-states-unrepresentable). Specified shape: `SurfaceMapEntry { key: <surface-string-literal-carrier>, value: SurfaceExpr }` (worker confirms exact carrier name from `parse_surface.dag` — likely a sibling of `SurfaceLiteral::String` or similar). When non-string-key support lands as a future sibling extension, *that* PR widens the entry's key type — not this one.
2. Parser disambiguation (`looks_like_map_literal` sibling to `looks_like_record_literal`); body parser (`parse_map_literal`).
3. Regen `parse_generated.rs` from `parse_surface.dag` + `parse_parser_body.txt`.
4. Exhaustive-match audit: every `match SurfaceExpr` site in v3 substrate handles the new variant (with sensible fall-through where map literals aren't structurally expected — the substrate-side `lower_data_item` is the only site that meaningfully consumes; everything else either errors or carries through).

Today the parser falls into `skip_brace_balanced` for non-record `{...}` bodies; after this PR, string-keyed map literals route into `parse_map_literal` and emit `SurfaceExpr::Map`. The substrate side (sibling brief) consumes that.

**Top-level data declarations only** for this PR. Map literals inside function bodies, lambda bodies, etc. — out of scope unless the same parse path is structurally reused; worker's call on whether the parser scope expands to those positions or stays narrow to data-body context.

## Five consumer-side requirements

1. **`SurfaceExpr::Map` variant in `parse_surface.dag`.** New variant expressing *"map literal with key-value entries"*. **Element shape: `SurfaceMapEntry { key: <surface-string-literal-carrier>, value: SurfaceExpr }`** — key is structurally restricted to a string-literal carrier per the string-key-only non-goal (illegal-states-unrepresentable: making non-string keys representable forces behavioral enforcement instead of structural). Worker confirms the exact string-literal carrier name from `parse_surface.dag`. **Coproduct dissolution receipt** for the new `SurfaceExpr::Map` variant per `feedback_coproduct_dissolution` and the `LoopBound` precedent at `docs/design-mutual-recursion-lowering.md:117-134`. No silent stamp.
2. **`looks_like_map_literal` lookahead** (`{ String :` shape) sibling to `looks_like_record_literal` (`{ Ident :` shape). Disambiguation at the brace-body entry; routes into `parse_map_literal` instead of `skip_brace_balanced`.
3. **`parse_map_literal` body parser** producing `SurfaceExpr::Map` with the parsed entries. Handles trailing-comma + multi-entry; rejects empty `{}` (which would be ambiguous with empty record) — worker picks empty-map syntax (suggest `{:}` or similar) and surfaces in PR description.
4. **Regen flowed through.** `parse_generated.rs` regenerated from authority; **no hand edits to `parse_generated.rs`** per `feedback_no_generated_code_on_disk`.
5. **Exhaustive `SurfaceExpr` match audit + updates.** Every `match expr` site across `lower.rs` + lens consumers + serializer + cementer handles the new `Map` variant. Where map literals aren't structurally expected (e.g., expression-position-not-data-body), the arm either emits a structured "map-literal-not-allowed-here" diagnostic OR carries through fall-through depending on context. **Use exhaustive matches**; no wildcard `_` swallowing per `feedback_missing_checks_review_heuristic`.

## Slice — parser extension

1. Add `SurfaceExpr::Map { entries: List<SurfaceMapEntry> }` to `parse_surface.dag` (per req 1). Author coproduct dissolution receipt.
2. Add `SurfaceMapEntry { key: <surface-string-literal-carrier>, value: SurfaceExpr }` to `parse_surface.dag` per req 1's structural-restriction requirement.
3. Edit `parse_parser_body.txt` (per reqs 2 + 3): add `looks_like_map_literal` + `parse_map_literal`; route from the existing brace-body entry.
4. Regen `parse_generated.rs` (per req 4).
5. Exhaustive-match audit + updates across consumers (per req 5).
6. **Lowerer-side exhaustive-match consistency** (per req 5): add an explicit `SurfaceExpr::Map` arm to `lower_data_item` (and any other `match expr` site currently falling through via wildcard). The arm produces `ValueBody::Unparsed(span)` for now (sibling substrate sub-lane converts this to `ValueBody::Map`). **No wildcard `_` swallowing** — req 5 mandates exhaustive matches; the post-parser substrate sub-lane will replace this arm's body, not its existence.
7. Smoke test: parser accepts `data kernel_algebra_profile: Map<String, AlgebraProfile> = { "Int": OrderedRingProfile, ... }` and produces a `SurfaceExpr::Map` with 7 entries. **NOTE**: this PR does NOT lower `SurfaceExpr::Map` to `ValueBody::Map` (sibling substrate sub-lane does that); after this PR the parsed `SurfaceExpr::Map` flows through the **explicit `Map` arm added in step 6** → `ValueBody::Unparsed` → R14 hard-fail. That's expected; the sibling substrate sub-lane replaces that arm's body. Test should assert the parser-output shape, not end-to-end lowering.

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body.
- [ ] `SurfaceExpr::Map` + `SurfaceMapEntry` variants in `parse_surface.dag`; coproduct dissolution receipt landed.
- [ ] `looks_like_map_literal` + `parse_map_literal` in `parse_parser_body.txt`.
- [ ] `parse_generated.rs` regenerated; no hand edits.
- [ ] All `SurfaceExpr` exhaustive-match sites updated.
- [ ] Parser smoke test: `kernel_algebra_profile`-shaped data declaration parses to `SurfaceExpr::Map` with the asserted entries.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas: regen-output updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director.

- **Lookahead disambiguation reveals genuine ambiguity** (e.g., `{ "Int": ... }` could be a record with a string-keyed field; or some other syntactic form already in use) — STOP. Surface-syntax design call.
- **Empty-map syntax is contentious** (e.g., `{:}` conflicts with existing rules; `{}` is ambiguous with empty record) — STOP. Pick a shape with Director sign-off rather than worker discretion.
- **`parse_parser_body.txt` edits cascade beyond the brace-body entry** (e.g., changes to top-level statement parsing) — STOP. Cross-cutting parser changes need Surface Manager coordination.
- **Exhaustive-match audit reveals consumer using wildcard `_`** — STOP. Surface a fix decision (this PR vs follow-up).
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not adding `ValueBody::Map`.** That's the sibling substrate sub-lane.
- **Not lowering `SurfaceExpr::Map` to anything substantive.** This PR's lowerer behavior for `SurfaceExpr::Map` is "fall through to `ValueBody::Unparsed`" — same as today's behavior for unparsed map bodies. The sibling substrate sub-lane closes that.
- **Not changing record-literal parsing.** Records continue to disambiguate via `looks_like_record_literal` unchanged.
- **Not extending `Map<K, V>` to non-string keys.** String-key maps only for this PR.
- **Not retiring `kernel_algebra_profile` mirror.** Sibling substrate sub-lane.

## Reporting

- Single PR. Title: `feat(v3): T-Substrate parser-extension — SurfaceExpr::Map for top-level map literals (prereq for ValueBody::Map sub-lane)`.
- PR body cites this brief + addresses each of the 5 reqs + documents element-shape + empty-map-syntax choices.
- On merge: signal Director; Director signals sibling substrate sub-lane (`t-substrate-valuebody-map-worker.md`) is now dispatchable.

## Cross-manager note

- **Surface Manager**: heads-up at dispatch. This PR is Surface Manager territory by overlap (parser surface-syntax authority); coordinate at landing.
- **Zero-Floor Manager**: heads-up if `parse_surface.dag` changes touch substrate.dag-adjacent territory. Coproduct receipt requirement (req 1) is the substrate-discipline anchor.
- **Grounding Manager**: no current overlap.
