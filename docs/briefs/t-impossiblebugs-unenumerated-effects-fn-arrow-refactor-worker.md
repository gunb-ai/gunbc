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
> `declared_effects` on the two arrow-shape sub-carriers (`FnSignature`
> for declaration position; `SurfaceArrow` for type position) as a
> co-invariant — both genuinely require their own effect set per the
> carrier-distinction rationale (req 3). Reports back to
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

This sub-lane introduces two arrow-shape sub-carriers — `SurfaceArrow` (type position, `inputs: List<SurfaceType>` to match today's parser surface) wrapped by `SurfaceType.Arrow(SurfaceArrow)`, and `FnSignature` (declaration position, `inputs: List<NamedArrowInput>` so binderless params are structurally unrepresentable) carried directly by `SurfaceItem.Fn`. Effects do NOT land in this PR — that's the next sub-lane in the chain.

**Top-level functions + higher-order function types** for this PR. Closures / lambdas (which today use `SurfaceExpr::Lambda` with no explicit type signature) — out of scope; `Lambda` continues to carry `params: List<String>` + `body` as today.

## Six consumer-side requirements

1. **New `NamedArrowInput` carrier in `parse_surface.dag`.** Single new carrier — declaration-position-only, since the parser today only accepts anonymous inputs in type position (verified at HEAD: `src/v3/compiler/parse_parser_body.txt:864-877` `parse_atom_type` reads `inputs` via `parse_type_expr_list_until`, which parses pure type expressions with no name-binding):

   ```
   type NamedArrowInput {
     name: String
     ty: SurfaceType
     refinement: SurfaceExpr?
   }
   ```

   `NamedArrowInput` is the load-bearing atom for declaration position (function-decl params always have a name, so the binder is structurally mandatory; refinement remains optional because not every param is refined). Refinement-without-binder is structurally unrepresentable. Type-position inputs continue to be raw `SurfaceType` entries (no binder, no refinement — the parser doesn't surface either today, so there's nothing to model). **No `ArrowInput` coproduct** — earlier drafts proposed `ArrowInput = Anonymous { ty } | Named(NamedArrowInput)` for a hypothetical `fn(x: A) -> B` type-position syntax that the parser doesn't accept today; pre-resolving the simplification per Claude-API exploratory observation. If a future PR adds named binders in type position, that PR introduces the coproduct (with its own dissolution receipt at that time), not this one. **Structural-carrier rationale** (mandatory in PR description) addressing why `NamedArrowInput` isn't `feedback_parallel_representation_debt` against `SurfaceParam`: `SurfaceParam` is the *data of a param at a binding site* (lowerer-binding view, today); `NamedArrowInput` is the *type-signature view at declaration position*. Once the refactor lands, `SurfaceParam` dissolves into `NamedArrowInput` (req 3).
2. **`SurfaceType.Arrow` refactored to wrap a typed `SurfaceArrow` sub-carrier.** Introduce a new top-level struct `SurfaceArrow { inputs: List<SurfaceType>, output: SurfaceType, span: SourceSpan }` (anonymous inputs, type position — matches today's parser surface), then change `SurfaceType.Arrow(SurfaceArrow)` (single positional payload, not record fields). The wrapper variant carries **no separate span** — `SurfaceArrow.span` is the source-of-truth span for the arrow; consumers reading `SurfaceType::Arrow(arrow)` use `arrow.span`. This makes the type-position arrow-shape a typed entity that consumers can hold directly without destructuring through `SurfaceType` first, and avoids the two-spans-per-arrow risk per Claude-API exploratory observation. **No silent compatibility shim**; existing callers updated per req 6.
3. **`SurfaceItem.Fn` + `SurfaceItem.FnExternalBody` refactored to carry a typed `FnSignature` sub-carrier with restricted inputs.** Drop `params: List<SurfaceParam>` + `return_type: SurfaceType` fields; replace with `signature: FnSignature` where:

   ```
   type FnSignature {
     inputs: List<NamedArrowInput>
     output: SurfaceType
     span: SourceSpan
   }
   ```

   `inputs: List<NamedArrowInput>` (not `List<SurfaceType>`) makes binderless params **structurally unrepresentable in declaration position** per `feedback_state_space_vs_behavioral_invariants` — rejecting an earlier draft's behavioral fall-back where the lowerer would have surfaced a Diagnostic for an anonymous-shape inside `Fn.signature`. The parser cannot construct `Fn { signature: FnSignature { inputs: [<anonymous-shape>] } }` because the input element type is `NamedArrowInput` directly. The bound-name + refinement information that today lives on `SurfaceParam` flows through `signature.inputs[i]: NamedArrowInput`. **`SurfaceParam` is fully retired** by this PR — no consumers reference it post-refactor. Worker should grep-survey + cite count in PR description. `FnSignature` carries its own `span: SourceSpan`; the wrapping context (`SurfaceItem.Fn`) consumes it directly without authoring a parallel span.

   **Carrier-distinction rationale (mandatory in PR description).** `FnSignature` and `SurfaceArrow` are both arrow-shape sub-carriers but encode different invariants: `FnSignature.inputs: List<NamedArrowInput>` (declaration must bind names so the body can reference them); `SurfaceArrow.inputs: List<SurfaceType>` (type position; the parser doesn't accept binders in this position today). This is concept distinction, not parallel-representation debt — same justification as `SurfaceParam` vs `NamedArrowInput` carrying overlapping fields but different concepts. The downstream parser-effects sub-lane lands `declared_effects` on **both** `FnSignature` and `SurfaceArrow` — both are type-signature shapes per `feedback_no_annotations`, so the discipline anchor is satisfied per-carrier; this is co-invariant duplication, not bookkeeping duplication.
4. **`parse_parser_body.txt` updates.** The function-item parse path (where `SurfaceItem::Fn` is constructed) routes its parsed params directly through `NamedArrowInput` construction → `FnSignature` → `SurfaceItem::Fn { signature: FnSignature, ... }`. The higher-order type parse path (where `SurfaceType::Arrow(SurfaceArrow)` is constructed for type annotations) constructs `SurfaceArrow { inputs: List<SurfaceType>, output, span }` directly — no per-input variant branching, since type-position inputs are anonymous-only today. **No new lookahead** — this is a pure construction-site refactor; the surface syntax doesn't change.
5. **Lowerer extension.** Every consumer that today reads `Fn.params` + `Fn.return_type` is updated to read `Fn.signature.inputs: List<NamedArrowInput>` + `Fn.signature.output`. Per-input destructuring is uniform — every entry is a `NamedArrowInput` carrying `name` + `ty` + `refinement?`. Type-position consumers (those reading `SurfaceArrow.inputs: List<SurfaceType>`) read each input as a raw `SurfaceType`. This is surface-side reshaping; substrate-side `Declaration` + `Arrow` untouched. **No wildcard arms** anywhere.
6. **Exhaustive-match audit + updates.** Every `match` site over `SurfaceType::Arrow` (snapshot grep: ~5 hits) and `SurfaceItem::Fn` / `FnExternalBody` (snapshot grep: ~20 hits) updated for the new shapes. **No wildcard `_` swallowing** per `feedback_missing_checks_review_heuristic`. Where a consumer today destructures `Fn { params, return_type, ... }`, it now destructures `Fn { signature: FnSignature { inputs, output, .. }, ... }` and reads each input as a `NamedArrowInput { name, ty, refinement }`. Where a consumer today destructures `SurfaceType::Arrow { inputs, .. }` over `List<SurfaceType>`, it now destructures `SurfaceType::Arrow(arrow)` and reads `arrow.inputs: List<SurfaceType>` — same element shape as today, just routed through the typed sub-carrier.

## Slice — `Fn`→`Arrow` refactor

1. Add `NamedArrowInput` + `ArrowInput` (per req 1) to `parse_surface.dag` with coproduct dissolution receipt for `ArrowInput`.
2. Add `SurfaceArrow` typed sub-carrier (per req 2; type position; `inputs: List<ArrowInput>`); refactor `SurfaceType.Arrow` to wrap it as a single positional payload.
3. Add `FnSignature` typed sub-carrier (per req 3; declaration position; `inputs: List<NamedArrowInput>`); refactor `SurfaceItem.Fn` + `FnExternalBody`: drop `params` + `return_type`, add `signature: FnSignature`. Retire `SurfaceParam`.
4. Edit `parse_parser_body.txt` (per req 4): update function-item construction (→ `NamedArrowInput` → `FnSignature`) + higher-order-type construction (→ `ArrowInput::Anonymous` or `Named` → `SurfaceArrow`).
5. Regen `parse_generated.rs` (per `feedback_no_generated_code_on_disk`).
6. Lowerer extension (per req 5): update every `Fn` / `Arrow` consumer in `src/v3/compiler/src/lower.rs` + sibling consumer files. Declaration-position consumers read `NamedArrowInput` directly (no variant branching). Type-position consumers branch on `Anonymous` vs `Named`.
7. Exhaustive-match audit + updates (per req 6) across every consumer.
8. Smoke + regression tests (note: v3 surface requires `fn(...)` prefix for higher-order types per `parse_atom_type`'s `TokenKind::KwFn` gate — examples below use that form):
   - Parser accepts `fn foo(x: Int, y: Bool) -> String { ... }` and produces `Fn { signature: FnSignature { inputs: [NamedArrowInput { name: "x", ty: Int, refinement: None }, NamedArrowInput { name: "y", ty: Bool, refinement: None }], output: String, .. }, .. }`.
   - Parser accepts `fn higher_order(f: fn(Int, Bool) -> String) -> Int { ... }` and produces a `NamedArrowInput` for `f` whose `ty` is `SurfaceType::Arrow(SurfaceArrow { inputs: [ArrowInput::Anonymous { ty: Int }, ArrowInput::Anonymous { ty: Bool }], output: String, .. })`.
   - Existing v3 compiler tests pass unchanged (refactor is structurally equivalent on the surface; no surface-syntax change).

## Acceptance

- [ ] All 6 consumer-side requirements satisfied + documented in PR body.
- [ ] `NamedArrowInput` + `ArrowInput` (coproduct: `Anonymous` | `Named(NamedArrowInput)`) in `parse_surface.dag` with coproduct dissolution receipt for `ArrowInput`.
- [ ] `SurfaceArrow` typed sub-carrier (`inputs: List<ArrowInput>`) in `parse_surface.dag`; `SurfaceType.Arrow(SurfaceArrow)` wraps it.
- [ ] `FnSignature` typed sub-carrier (`inputs: List<NamedArrowInput>`) in `parse_surface.dag`; `SurfaceItem.Fn` + `FnExternalBody` carry `signature: FnSignature` (binderless params structurally unrepresentable in declaration position).
- [ ] PR description includes carrier-distinction rationale for `FnSignature` vs `SurfaceArrow` (both arrow-shape; different invariants per req 3).
- [ ] `SurfaceParam` retired (zero references post-refactor).
- [ ] Post-refactor exhaustive-match-site count for `SurfaceType::Arrow` + `SurfaceItem::Fn` / `FnExternalBody` recorded in PR body.
- [ ] `parse_parser_body.txt` construction-site updates landed.
- [ ] `parse_generated.rs` regenerated; no hand edits.
- [ ] All `SurfaceType::Arrow` + `SurfaceItem::Fn`/`FnExternalBody` exhaustive-match sites updated; no new wildcard `_` arms.
- [ ] Lowerer destructures `FnSignature` exhaustively without wildcard arms (no runtime non-Arrow check needed — typed sub-carrier eliminates the case structurally per req 3).
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
- **Substrate-side `Arrow` shape needs a parallel refactor** — if the lowerer can't translate `Fn.signature: FnSignature` into the substrate `Declaration` / `Arrow` shape without also reshaping the substrate side, STOP. That's a separate sub-lane.
- **`SurfaceArrow` / `FnSignature` introduction cascades into substrate authority** — reqs 2 + 3 introduce two top-level `parse_surface.dag` carriers. If grep surfaces consumers that expect `SurfaceType::Arrow { inputs, output, span }` as record-style fields (rather than `Arrow(SurfaceArrow)` positional payload) and updating them cascades beyond `lower.rs` + immediate parse consumers, STOP. May indicate a smaller-blast-radius shape (e.g., keeping `Arrow` as record-style and instead defining the sub-carriers as type aliases / typed views rather than wrapping payloads) is preferable.
- **Type-position higher-order syntax accepts named binders today** (`fn(x: A) -> B`) — req 1 / `ArrowInput::Named(NamedArrowInput)` is the path. If the parser doesn't currently accept that form (i.e., type-position is anonymous-only on the surface today), worker may simplify `ArrowInput` to a single non-coproduct shape (just `Anonymous { ty }`-equivalent) and surface that simplification with rationale. STOP if the simplification would conflict with downstream effect-on-Arrow consumer expectations.
- **DB-11 / refinement-strip interaction surfaces** (`infer.rs:3693-3703`) — if refinements on `ArrowInput` interact with DB-11's refinement strip in a non-obvious way, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not adding `declared_effects`.** That's the sibling parser-effects sub-lane; this PR establishes the two arrow-shape sub-carriers (`FnSignature` + `SurfaceArrow`) so the next PR can land `declared_effects` on each as a co-invariant per req 3's carrier-distinction rationale.
- **Not refactoring `SurfaceExpr::Lambda`.** Lambdas don't have explicit type signatures today; they're out of scope.
- **Not refactoring substrate-side `Declaration` / `Arrow`.** Surface-side only.
- **Not changing surface syntax.** The user-visible function-declaration syntax is unchanged; this is a pure construction-site refactor.
- **Not changing higher-order type syntax.** `fn(A, B) -> C` continues to parse as today; the only structural change is that `Arrow.inputs` now wraps each entry in `ArrowInput::Anonymous { ty }`.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs Fn→Arrow refactor — SurfaceItem.Fn carries Arrow-shaped signature (pre-prereq for unenumerated-effects parser sub-lane)`.
- PR body cites this brief + addresses each of the 6 reqs + documents structural-carrier rationale for `ArrowInput`.
- On merge: signal Director; Director signals sibling parser sub-lane (`t-impossiblebugs-unenumerated-effects-parser-worker.md`) is now dispatchable. The parser sub-lane brief was updated alongside this brief (same PR) to point at `FnSignature` + `SurfaceArrow` (post-refactor) and to mandate co-invariant placement of `declared_effects` on both carriers per req 3's carrier-distinction rationale.

## Cross-manager note

- **Surface Manager**: heads-up at dispatch. This PR is Surface Manager territory by overlap (parser surface-syntax authority — though no syntax change, parse-body construction sites move); coordinate at landing.
- **Zero-Floor Manager**: heads-up — `parse_surface.dag` shape changes are substrate-discipline-adjacent. Structural-carrier rationale (req 1) is the discipline anchor.
- **Grounding Manager**: no current overlap.
