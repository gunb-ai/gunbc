# T-ImpossibleBugs unenumerated effects — `Fn`→`Arrow` signature refactor `(M, R2 pre-prereq)`

> **Director ad-hoc dispatch.** Sibling pre-requisite to the
> [`t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)
> parser sub-lane, which is itself pre-requisite to the
> [`t-impossiblebugs-unenumerated-effects-worker.md`](t-impossiblebugs-unenumerated-effects-worker.md)
> substrate sub-lane. Authored 2026-04-25 post-`sunny-otter-128`
> STOP-AND-ESCALATE on the parser-prereq brief — worker verified that
> the parser brief's recommended `declared_effects on SurfaceType.Arrow`
> placement does not load-bear on `SurfaceItem.Fn`, because `Fn` today
> carries `params: List<SurfaceParam>` + `return_type: SurfaceType` as
> two separate fields, not as an `Arrow`-shaped signature. Per the
> parser brief's STOP #2 (*"if execution surfaces a strong reason to
> put the field on `Fn` instead of `Arrow` (e.g., effect-as-type-component
> breaks), STOP. Director call."*), Director picked **(b)(2): structural
> refactor first**.
>
> This brief reshapes `SurfaceItem.Fn` to carry an `Arrow`-shaped
> signature so that the downstream parser sub-lane can land
> `declared_effects` on `Arrow` once and have it apply uniformly to
> top-level functions and higher-order function types. Reports back to
> Director (`zesty-bear-812`); Surface Manager territory overlap
> (parser surface-syntax authority).
>
> **Precedent**: this split mirrors the parser-prereq pattern from
> PR #797 ([`t-substrate-valuebody-map-parser-worker.md`](t-substrate-valuebody-map-parser-worker.md))
> and PR #799 ([`t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)).
> The chain: this brief → parser-effects brief → substrate-effects brief.

## Read first

- **[`docs/briefs/t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)** — sibling parser sub-lane brief; blocks on this PR landing. The parser brief assumes `SurfaceType.Arrow` is the load-bearing function-type-signature carrier; this brief makes that assumption true on `Fn`.
- **[`src/v3/std/parse_surface.dag:60-75`](../../src/v3/std/parse_surface.dag)** — `SurfaceType.Arrow` declaration; today carries `inputs: List<SurfaceType>` (pure types, no names, no refinements) + `output: SurfaceType` + `span: SourceSpan`. Higher-order types written `(A, B) -> C` flow through this.
- **[`src/v3/std/parse_surface.dag:33-37`](../../src/v3/std/parse_surface.dag)** — `SurfaceParam = { name: String, ty: SurfaceType, refinement: SurfaceExpr? }`. Carries the param-binding info that today lives parallel to `SurfaceType` on `Fn`.
- **[`src/v3/std/parse_surface.dag:179-200`](../../src/v3/std/parse_surface.dag)** — `SurfaceItem.Fn` (and `FnExternalBody`); today: `name`, `type_params`, `params: List<SurfaceParam>`, `return_type: SurfaceType`, `body`, `span`. The `params` + `return_type` split is the vestige this brief dissolves.
- **[`src/v3/compiler/parse_parser_body.txt`](../../src/v3/compiler/parse_parser_body.txt)** — parse-body algorithm authority. The function-item path constructs `Fn { params, return_type, ... }`; this brief routes that construction through an `Arrow` signature.
- **[`src/v3/compiler/src/parse_generated.rs`](../../src/v3/compiler/src/parse_generated.rs)** — auto-generated. Per `feedback_no_generated_code_on_disk`, edits flow through `.dag` + `.txt` authority then regen. **No hand edits to `parse_generated.rs`**.
- **[`src/v3/compiler/src/lower.rs`](../../src/v3/compiler/src/lower.rs)** — primary `Fn` consumer; lowers params + return_type to substrate connectives. Worker re-confirms exhaustive-match site count via `grep -n "SurfaceItem::Fn\b\|match.*SurfaceItem\b" src/v3/compiler/src/lower.rs` at dispatch (snapshot grep: ~20 hits across `src/v3/compiler/src/`; not authority).
- **[`src/v3/std/substrate.dag:154-163`](../../src/v3/std/substrate.dag)** — substrate `Declaration` + `Arrow` connectives. The substrate-side function-arrow shape is a separate concern from this surface-side refactor; this brief does NOT touch substrate `Arrow`.
- **`feedback_construction_over_ratchets`** + **`feedback_parallel_representation_debt`** + **`feedback_coproduct_dissolution`** + **`feedback_missing_checks_review_heuristic`** + **`feedback_no_generated_code_on_disk`** + **`feedback_no_annotations`** + **MODELING.md M9** + **INVARIANTS.md**.

## Frame

`SurfaceItem.Fn` today carries `params: List<SurfaceParam>` + `return_type: SurfaceType` as two fields. There is no synthesized `Arrow` on the function declaration — `SurfaceType.Arrow` only appears when a higher-order function type is *written out as a type annotation* (e.g., a parameter typed `(A, B) -> C`).

This split is a vestige. Function declarations *are* arrows: their structural type signature is `(input₁, …, inputₙ) -> output`. The parallel encoding (separate `params` + `return_type` fields on `Fn`, separate `Arrow.inputs` + `Arrow.output` on `SurfaceType`) is `feedback_parallel_representation_debt` waiting to be dissolved — the same shape encoded twice.

The downstream parser-effects sub-lane needs to add a `declared_effects` field that is *part of the function type signature* (per `feedback_no_annotations` — first-class language feature, not an annotation). With the current split, that field would either:
- Have to live on `SurfaceItem.Fn` only (effects can't appear in higher-order function types — discipline anchor weakens), OR
- Have to live on both `SurfaceType.Arrow` and `SurfaceItem.Fn` (parallel representation; two carriers for the same concept).

Both are dead-ends. The constructive fix is to dissolve the split: `SurfaceType.Arrow` carries the structural function-type signature uniformly, and `SurfaceItem.Fn` carries an `Arrow`-shaped signature alongside the bind-site param-name + refinement metadata.

This sub-lane reshapes `SurfaceType.Arrow` to carry `List<ArrowInput>` (where `ArrowInput` carries optional name + refinement alongside the type) and reshapes `SurfaceItem.Fn` to use that `Arrow` shape directly. Effects do NOT land in this PR — that's the next sub-lane in the chain.

**Top-level functions + higher-order function types** for this PR. Closures / lambdas (which today use `SurfaceExpr::Lambda` with no explicit type signature) — out of scope; `Lambda` continues to carry `params: List<String>` + `body` as today.

## Six consumer-side requirements

1. **New `ArrowInput` coproduct in `parse_surface.dag`.** Two variants — illegal states unrepresentable per `feedback_state_space_vs_behavioral_invariants` and modeling-discipline practice 2:

   ```
   type ArrowInput
     = Anonymous { ty: SurfaceType }
     | Named { name: String, ty: SurfaceType, refinement: SurfaceExpr? }
   ```

   Higher-order types written `fn(A, B) -> C` produce `ArrowInput::Anonymous { ty: A }` entries (no name, no refinement — both structurally absent, not "optional but None"). Named function-declaration params produce `ArrowInput::Named { name: "x", ty: A, refinement: ... }`. The previous shape (`name: String?` + `refinement: SurfaceExpr?` as independent optionals) is rejected: `name: None, refinement: Some` is a meaningless state (refinement without a binder) and would be representable-but-illegal. **Coproduct dissolution receipt** mandatory per `feedback_coproduct_dissolution` and the `LoopBound` precedent at `docs/design-mutual-recursion-lowering.md:117-134` — worker authors the receipt in `parse_surface.dag` adjacent to the type. Receipt should also address why this isn't `feedback_parallel_representation_debt` against `SurfaceParam`: `SurfaceParam` is the *data of a param at a binding site* (lowerer-binding view); `ArrowInput::Named` is the *type-signature view*. Once the refactor lands, `SurfaceParam` dissolves into `ArrowInput::Named` (req 3).
2. **`SurfaceType.Arrow` refactored to wrap a typed `SurfaceArrow` sub-carrier.** Introduce a new top-level struct `SurfaceArrow { inputs: List<ArrowInput>, output: SurfaceType, span: SourceSpan }`, then change `SurfaceType.Arrow(SurfaceArrow)` (single positional payload, not record fields). This makes the `Arrow`-shape a typed entity that consumers can hold directly without destructuring through `SurfaceType` first — load-bearing for req 3. **No silent compatibility shim**; existing callers updated per req 6.
3. **`SurfaceItem.Fn` + `SurfaceItem.FnExternalBody` refactored to carry `SurfaceArrow` directly.** Drop `params: List<SurfaceParam>` + `return_type: SurfaceType` fields; replace with `signature: SurfaceArrow` (the typed sub-carrier from req 2 — **not** `SurfaceType`). This makes `Fn { signature: SurfaceType::Bool, ... }` and similar illegal states **structurally unrepresentable** per `feedback_state_space_vs_behavioral_invariants` (rejecting the earlier draft's behavioral-enforcement-via-fail-closed-lowerer compromise that codex / Director both flagged). The bound-name + refinement information that today lives on `SurfaceParam` flows through `signature.inputs[i]` as `ArrowInput::Named { name, ty, refinement }`. **`SurfaceParam` is fully retired** by this PR — no consumers reference it post-refactor. Worker should grep-survey + cite count in PR description.
4. **`parse_parser_body.txt` updates.** The function-item parse path (where `SurfaceItem::Fn` is constructed) now routes its parsed params + return-type through `ArrowInput` construction → `SurfaceType::Arrow` → `SurfaceItem::Fn { signature, ... }`. The higher-order type parse path (where `SurfaceType::Arrow` is constructed for type annotations) constructs `ArrowInput { name: None, refinement: None, ty }` entries. **No new lookahead** — this is a pure construction-site refactor; the surface syntax doesn't change.
5. **Lowerer extension.** Every consumer that today reads `Fn.params` + `Fn.return_type` is updated to read `Fn.signature.inputs` + `Fn.signature.output`. Per-input destructuring branches on `ArrowInput::Anonymous` vs `ArrowInput::Named` — the lowerer's bind-name + refinement consumption only fires on `Named`, and `Anonymous` in a function-declaration position is a parser-side invariant violation that the lowerer surfaces as a structured Diagnostic (per `feedback_fail_closed_discipline` + C-8). This is surface-side reshaping; substrate-side `Declaration` + `Arrow` untouched. **No wildcard arms** that swallow the `Anonymous` case silently.
6. **Exhaustive-match audit + updates.** Every `match` site over `SurfaceType::Arrow` (snapshot grep: ~5 hits) and `SurfaceItem::Fn` / `FnExternalBody` (snapshot grep: ~20 hits) updated for the new shapes. **No wildcard `_` swallowing** per `feedback_missing_checks_review_heuristic`. Where a consumer today destructures `Fn { params, return_type, ... }`, it now destructures `Fn { signature: SurfaceArrow { inputs, output, .. }, ... }`. Where a consumer today destructures `SurfaceType::Arrow { inputs, .. }` over `List<SurfaceType>`, it now destructures `SurfaceType::Arrow(arrow)` and reads `arrow.inputs: List<ArrowInput>`, branching per-entry on `Anonymous { ty }` vs `Named { name, ty, refinement }`.

## Slice — `Fn`→`Arrow` refactor

1. Add `ArrowInput` coproduct (per req 1) to `parse_surface.dag` with coproduct dissolution receipt.
2. Add `SurfaceArrow` typed sub-carrier (per req 2); refactor `SurfaceType.Arrow` to wrap it as a single positional payload.
3. Refactor `SurfaceItem.Fn` + `FnExternalBody` (per req 3): drop `params` + `return_type`, add `signature: SurfaceArrow` (typed; not `SurfaceType`). Retire `SurfaceParam`.
4. Edit `parse_parser_body.txt` (per req 4): update function-item + higher-order-type construction sites to flow through the new shape.
5. Regen `parse_generated.rs` (per `feedback_no_generated_code_on_disk`).
6. Lowerer extension (per req 5): update every `Fn` / `Arrow` consumer in `src/v3/compiler/src/lower.rs` + sibling consumer files. Branch per-input on `ArrowInput::Anonymous` vs `Named`; structured Diagnostic for `Anonymous` in function-declaration position.
7. Exhaustive-match audit + updates (per req 6) across every consumer.
8. Smoke + regression tests (note: v3 surface requires `fn(...)` prefix for higher-order types per `parse_atom_type`'s `TokenKind::KwFn` gate — examples below use that form):
   - Parser accepts `fn foo(x: Int, y: Bool) -> String { ... }` and produces `Fn { signature: SurfaceArrow { inputs: [ArrowInput::Named { name: "x", ty: Int, refinement: None }, ArrowInput::Named { name: "y", ty: Bool, refinement: None }], output: String, .. }, .. }`.
   - Parser accepts `fn higher_order(f: fn(Int, Bool) -> String) -> Int { ... }` and produces an `ArrowInput::Named` for `f` whose `ty` is `SurfaceType::Arrow(SurfaceArrow { inputs: [ArrowInput::Anonymous { ty: Int }, ArrowInput::Anonymous { ty: Bool }], output: String, .. })`.
   - Existing v3 compiler tests pass unchanged (refactor is structurally equivalent on the surface; no surface-syntax change).

## Acceptance

- [ ] All 6 consumer-side requirements satisfied + documented in PR body.
- [ ] `ArrowInput` coproduct (`Anonymous` | `Named`) in `parse_surface.dag` with coproduct dissolution receipt.
- [ ] `SurfaceArrow` typed sub-carrier in `parse_surface.dag`; `SurfaceType.Arrow(SurfaceArrow)` wraps it.
- [ ] `SurfaceItem.Fn` + `FnExternalBody` carry `signature: SurfaceArrow` (typed; not `SurfaceType`; illegal-state-unrepresentable).
- [ ] `SurfaceParam` retired (zero references post-refactor).
- [ ] Post-refactor exhaustive-match-site count for `SurfaceType::Arrow` + `SurfaceItem::Fn` / `FnExternalBody` recorded in PR body.
- [ ] `parse_parser_body.txt` construction-site updates landed.
- [ ] `parse_generated.rs` regenerated; no hand edits.
- [ ] All `SurfaceType::Arrow` + `SurfaceItem::Fn`/`FnExternalBody` exhaustive-match sites updated; no new wildcard `_` arms.
- [ ] Lowerer fail-closed on non-Arrow `signature`.
- [ ] Smoke tests for top-level fn + higher-order-type both produce the new shape.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas: regen-output updates land in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Director.

- **Higher-order-type surface syntax accepts named params today** — if `(x: A, y: B) -> C` is already a valid surface form (i.e., `SurfaceType::Arrow.inputs` already carries names somewhere the snapshot grep missed), STOP. The refactor is then a *renaming* not a *promotion*; the structural-carrier rationale needs to reflect that.
- **`SurfaceParam` has consumers beyond `Fn` / `FnExternalBody`** — req 3 retires `SurfaceParam`. If grep surfaces consumers in (e.g.) `Lambda`, `Match`, or other carriers, STOP. Either keep `SurfaceParam` as a live carrier (and have `ArrowInput` be a sibling) or scope a coupled retirement.
- **Higher-order types appear in positions that today silently drop refinements** — if existing `Arrow.inputs: List<SurfaceType>` consumers somewhere assume the type carries no refinement (and the new `ArrowInput.refinement: Some` would surface a previously-impossible state), STOP. Fail-closed at the point of inspection, not silent fall-through.
- **`parse_parser_body.txt` edits cascade beyond function-item / higher-order-type construction** — STOP. Cross-cutting parser changes need Surface Manager coordination.
- **Exhaustive-match audit reveals consumer using wildcard `_`** — STOP. Surface a fix decision (this PR vs follow-up).
- **Substrate-side `Arrow` shape needs a parallel refactor** — if the lowerer can't translate `Fn.signature: SurfaceArrow` into the substrate `Declaration` / `Arrow` shape without also reshaping the substrate side, STOP. That's a separate sub-lane.
- **`SurfaceArrow` introduction cascades into substrate authority** — req 2 introduces `SurfaceArrow` as a top-level `parse_surface.dag` carrier wrapped by `SurfaceType.Arrow`. If grep surfaces consumers that expect `SurfaceType::Arrow { inputs, output, span }` as record-style fields (rather than `Arrow(SurfaceArrow)` positional payload) and updating them cascades beyond `lower.rs` + immediate parse consumers, STOP. May indicate a smaller-blast-radius shape (e.g., keeping `Arrow` as record-style and instead defining `SurfaceArrow` as a type alias / typed view rather than a wrapping payload) is preferable.
- **DB-11 / refinement-strip interaction surfaces** (`infer.rs:3693-3703`) — if refinements on `ArrowInput` interact with DB-11's refinement strip in a non-obvious way, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not adding `declared_effects`.** That's the sibling parser-effects sub-lane; this PR makes `Arrow` the load-bearing site so that the *next* PR can add the field once and have it apply uniformly.
- **Not refactoring `SurfaceExpr::Lambda`.** Lambdas don't have explicit type signatures today; they're out of scope.
- **Not refactoring substrate-side `Declaration` / `Arrow`.** Surface-side only.
- **Not changing surface syntax.** The user-visible function-declaration syntax is unchanged; this is a pure construction-site refactor.
- **Not changing higher-order type syntax.** `(A, B) -> C` continues to parse as today; the only structural change is that `Arrow.inputs` now wraps each entry in `ArrowInput { name: None, refinement: None, ty }`.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs Fn→Arrow refactor — SurfaceItem.Fn carries Arrow-shaped signature (pre-prereq for unenumerated-effects parser sub-lane)`.
- PR body cites this brief + addresses each of the 6 reqs + documents structural-carrier rationale for `ArrowInput`.
- On merge: signal Director; Director signals sibling parser sub-lane (`t-impossiblebugs-unenumerated-effects-parser-worker.md`) is now dispatchable. The parser sub-lane brief itself does NOT need editing — it already assumes `Arrow` is the load-bearing carrier; this PR makes that assumption true.

## Cross-manager note

- **Surface Manager**: heads-up at dispatch. This PR is Surface Manager territory by overlap (parser surface-syntax authority — though no syntax change, parse-body construction sites move); coordinate at landing.
- **Zero-Floor Manager**: heads-up — `parse_surface.dag` shape changes are substrate-discipline-adjacent. Structural-carrier rationale (req 1) is the discipline anchor.
- **Grounding Manager**: no current overlap.
