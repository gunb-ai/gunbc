# T-ImpossibleBugs unenumerated effects — parser extension `(M, R2 prereq)`

> **Director ad-hoc dispatch.** Sibling pre-requisite to the
> [`t-impossiblebugs-unenumerated-effects-worker.md`](t-impossiblebugs-unenumerated-effects-worker.md)
> substrate sub-lane. Authored 2026-04-25 post-`sunny-otter-128`
> STOP-AND-ESCALATE on the bundled effects brief — worker verified
> that adding a declared-effect carrier to function type signatures
> requires net-new parser surface (`SurfaceType.Arrow` and
> `SurfaceItem.Fn` have zero effect slots today). Director picked
> **split** — this brief lands the parser side; the substrate brief
> narrows to "post-parser-extension" scope and dispatches sequentially.
>
> Reports back to Director (`zesty-bear-812`); Surface Manager
> territory overlap (parser surface-syntax authority).
>
> **Precedent**: this split mirrors the `ValueBody::Map` parser
> sub-lane pattern from PR #797 ([`t-substrate-valuebody-map-parser-worker.md`](t-substrate-valuebody-map-parser-worker.md)).

## Read first

- **[`docs/briefs/t-impossiblebugs-unenumerated-effects-worker.md`](t-impossiblebugs-unenumerated-effects-worker.md)** — sibling substrate sub-lane brief (post-this-PR-narrowed). The substrate side blocks on this PR landing.
- **[`docs/briefs/t-impossiblebugs-unenumerated-effects-fn-arrow-refactor-worker.md`](t-impossiblebugs-unenumerated-effects-fn-arrow-refactor-worker.md)** — pre-prereq refactor brief. Lands `SurfaceArrow` (type-position arrow-shape carrier) + `FnSignature` (declaration-position arrow-shape carrier) + `NamedArrowInput` (declaration-position input atom). **This brief depends on that refactor having landed**; pre-flight check + STOP if it hasn't.
- **`SurfaceArrow` + `FnSignature` in `parse_surface.dag` (post-refactor)** — both arrow-shape sub-carriers. Today (pre-refactor) carries no `declared_effects` field on either. New surface-syntax + carrier extension lands on **both** sub-carriers per the refactor brief's carrier-distinction rationale (req 3): both are type-signature shapes per `feedback_no_annotations`, so effects must appear on each as a co-invariant — declaration-position effects (on `FnSignature`) and type-position effects (on `SurfaceArrow`) are real co-invariants, not bookkeeping duplication.
- **[`src/v3/compiler/parse_parser_body.txt`](../../src/v3/compiler/parse_parser_body.txt)** — parse-body algorithm authority. The function-type / function-item parsing paths produce the `Arrow` / `Fn` surface shapes; new declared-effects syntax lands here. Worker picks syntax (recommend post-arrow-output suffix: `fn foo() -> T effects [Read, Write]` or similar; surface choice in PR description).
- **[`src/v3/compiler/src/parse_generated.rs`](../../src/v3/compiler/src/parse_generated.rs)** — auto-generated from `parse_surface.dag` + `parse_parser_body.txt`. Per `feedback_no_generated_code_on_disk`, edits flow through `.dag` + `.txt` authority then regen. **No hand edits to `parse_generated.rs`**.
- **[`src/v3/std/substrate.dag:154-163`](../../src/v3/std/substrate.dag)** — `Declaration` and `Arrow` connectives have **zero** effect slots today; adding `declared_effects: List<OperationEffect>` to `Arrow` is purely additive on the substrate side.
- **[`src/v3/std/effects.dag` lines 262-506](../../src/v3/std/effects.dag)** — current `OperationEffect` taxonomy (Read/Upsert/Create/Append/Delete). The declared-effects carrier reuses this enum (no new effect classes; sibling substrate sub-lane's req 4 anchors on existing taxonomy).
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **`feedback_no_annotations`** + **`feedback_no_validation_passes`**.

## Frame

The substrate `declared_effects` carrier cannot land standalone — the parser must first emit declared-effects syntactically for the lowerer to populate the field. Without parser support, every user function would have `declared_effects = []` and the substrate side's lens (sibling brief) would fire `EffectLeakageError` on every effectful function in the codebase the moment it's enabled — exactly the scaffold-without-consumer pattern `feedback_parallel_representation_debt` warns against.

This sub-lane lands the parser side. Output:

1. New `declared_effects: List<SurfaceType>` (or equivalent — element shape worker's call) on **both** `SurfaceArrow` (type position) and `FnSignature` (declaration position) — per refactor-brief carrier-distinction rationale (req 3), effects are a co-invariant on both arrow-shape sub-carriers, not a single placement choice.
2. Surface syntax: post-arrow-output suffix expressing the declared effect set (recommended `fn foo() -> T effects [Read, Write]` or worker-equivalent; surface in PR description).
3. Lowering: surface `declared_effects` lowers to `Declaration.connective` (or sibling `Arrow` substrate field) carrying the resolved `List<OperationEffect>`.
4. Regen `parse_generated.rs` from authority.
5. Exhaustive-match audit: every `match SurfaceType` / `match SurfaceItem` site that today destructures `Arrow` / `Fn` updated to handle the new field (with sensible default for sites that don't care).

Today the parser silently accepts no effect declaration; after this PR, the surface accepts an explicit effects clause and lowers it into a substrate fact. The substrate side (sibling brief) consumes that fact via the lens.

**Function type signatures only** for this PR. Effects on closures, lambdas, method declarations, etc. — out of scope unless the same parse path is structurally reused; worker's call.

## Six consumer-side requirements

1. **`declared_effects` field on both `SurfaceArrow` and `FnSignature`** (post-refactor sub-carriers; not a worker placement choice — co-invariant per refactor-brief req 3). Element shape: `List<SurfaceType>` where each entry resolves to an `OperationEffect` declaration (Read/Upsert/Create/Append/Delete from `effects.dag`). No new variants (purely additive field on two existing carriers); structural-carrier rationale required in PR body explaining why this isn't `feedback_parallel_representation_debt` (answer: each sub-carrier names a different position — declaration vs. type — both genuinely require their own effect set).
2. **Surface syntax** for the declared-effects clause. Worker picks (recommend post-arrow-output suffix); document choice + reasoning in PR description. The syntax is part of the function type signature, not an annotation (per `feedback_no_annotations`).
3. **`looks_like_effects_clause` lookahead** at the relevant parse site (function type / item parsing); routes into a new `parse_effects_clause` body. Disambiguation must not break existing function syntax.
4. **`parse_effects_clause` body parser** producing `List<SurfaceType>` (or equivalent) with the parsed effect-type references.
5. **Lowerer extension** producing the post-parser substrate carrier on `Arrow` (or `Fn`-level) declarations. Resolves each surface effect-type to its `OperationEffect` declaration.
6. **Exhaustive-match audit + updates.** Every `match` site over `SurfaceArrow` / `FnSignature` updated for the new `declared_effects` field. **No wildcard `_` swallowing**; per `feedback_missing_checks_review_heuristic`.

## Slice — parser extension

1. **Pre-flight check**: confirm the Fn→Arrow refactor brief has merged and `parse_surface.dag` carries `SurfaceArrow` + `FnSignature` + `NamedArrowInput`. STOP if not — sequencing error.
2. Add `declared_effects` field to **both** `SurfaceArrow` and `FnSignature` (per req 1) in `parse_surface.dag`. Add structural-carrier rationale (no new coproduct variants; field-level addition on two existing carriers).
3. Edit `parse_parser_body.txt` (per reqs 2 + 3 + 4): add `looks_like_effects_clause` + `parse_effects_clause`; route from the function-type/item parse entry.
4. Add lowerer extension (per req 5) producing the post-parser substrate carrier; resolve each surface effect to its `OperationEffect` declaration.
5. Regen `parse_generated.rs` (per `feedback_no_generated_code_on_disk`).
6. Exhaustive-match audit + updates (per req 6).
7. Smoke tests at the right stage boundaries (parser-output vs. lowerer-resolved — facts-flow-forward discipline):
   - **Parser smoke**: parser accepts `fn read_user(id: String) -> User effects [Read]` and produces a function declaration whose `signature: FnSignature` carries `declared_effects: List<SurfaceType>` with one entry — a `SurfaceType::Named { name: "Read", ... }` reference. Assert the parsed surface shape only; do NOT assert resolution to `OperationEffect`.
   - **Lowerer smoke**: same source through the lowerer produces the resolved post-parser substrate carrier with `[ReadEffect]` (or whatever variant `derive_op_effect` / req 5's resolution produces). The boundary between the two tests is the parser→lowerer authority split.
   - **NOTE**: this PR does NOT consume the field via lens (sibling substrate sub-lane does that); after this PR the field is populated and resolved but unread by any lens. That's expected; the sibling sub-lane closes that path.

## Acceptance

- [ ] All 6 consumer-side requirements satisfied + documented in PR body.
- [ ] `declared_effects` field on **both** `SurfaceArrow` and `FnSignature` (post-refactor sub-carriers); structural-carrier rationale documented in PR body.
- [ ] `looks_like_effects_clause` + `parse_effects_clause` in `parse_parser_body.txt`.
- [ ] Lowerer resolves surface effects to `OperationEffect` declarations.
- [ ] `parse_generated.rs` regenerated; no hand edits.
- [ ] All `SurfaceType` / `SurfaceItem` exhaustive-match sites updated.
- [ ] Parser smoke test asserts surface shape only (`List<SurfaceType>` references); lowerer smoke test asserts resolution to `OperationEffect`. Stage boundary preserved.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas: regen-output updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director.

- **Surface-syntax design contention** (e.g., post-arrow-output suffix conflicts with existing rules; effects-clause-first vs effects-clause-last design call) — STOP. Surface Manager / parser-owner sign-off needed.
- **`Arrow` vs `Fn`-level placement** — if execution surfaces a strong reason to put the field on `Fn` instead of `Arrow` (e.g., lambdas don't have explicit type signatures; effect-as-type-component breaks), STOP. Director call.
- **`parse_parser_body.txt` edits cascade beyond function-parsing** — STOP. Cross-cutting parser changes need Surface Manager coordination.
- **Exhaustive-match audit reveals consumer using wildcard `_`** — STOP. Surface a fix decision (this PR vs follow-up).
- **DB-11 / refinement interaction surfaces** — if the declared-effects-as-type-signature shape interacts with DB-11's refinement strip (`infer.rs:3693-3703`), STOP. May need explicit handling.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not adding the lens.** That's the sibling substrate sub-lane.
- **Not consuming `declared_effects`** beyond storage; the lens consumes it.
- **Not changing existing function syntax.** Functions without effects clauses continue to parse with empty `declared_effects`.
- **Not extending `OperationEffect` taxonomy.** New effect classes (e.g., `Logging`) are out of scope; lane anchors on existing Read/Upsert/Create/Append/Delete.
- **Not extending to closures/lambdas/method-decls** unless the parse path naturally reuses.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs parser-extension — declared_effects on FnSignature + SurfaceArrow (prereq for unenumerated-effects substrate sub-lane)`.
- PR body cites this brief + addresses each of the 6 reqs + documents surface-syntax choice + `FnSignature` / `SurfaceArrow` co-invariant carrier rationale (placement is not a worker choice — fixed by req 1 + refactor-brief req 3).
- On merge: signal Director; Director signals sibling substrate sub-lane (`t-impossiblebugs-unenumerated-effects-worker.md`) is now dispatchable.

## Cross-manager note

- **Surface Manager**: heads-up at dispatch. This PR is Surface Manager territory by overlap (parser surface-syntax authority); coordinate at landing.
- **Zero-Floor Manager**: heads-up if `parse_surface.dag` changes touch substrate.dag-adjacent territory. Coproduct receipt requirement (req 1) is the substrate-discipline anchor.
- **Grounding Manager**: no current overlap.
