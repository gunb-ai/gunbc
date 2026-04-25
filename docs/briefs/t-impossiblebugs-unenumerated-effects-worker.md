# T-ImpossibleBugs — unenumerated effects `(S, R2; SPLIT — parser sub-lane lands first)`

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 3 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes — any worker can dispatch
> in parallel. Reports to Director (`zesty-bear-812`).
>
> **🔄 SPLIT 2026-04-25 (post-`sunny-otter-128` STOP-AND-ESCALATE).**
> Worker correctly identified that brief req 2 (declared-effect carrier
> as part of function type signature) requires surface-syntax authoring:
> `SurfaceType.Arrow` and `SurfaceItem.Fn` (`src/v3/std/parse_surface.dag:71-75`,
> `:185-199`) have **zero** effect slots today. Without parser surface,
> every user function would have `declared_effects = []` while inference
> returns non-empty — lens fires `EffectLeakageError` everywhere it's
> enabled. Worker recommended sibling parser sub-lane (mirror of
> `t-substrate-valuebody-map-parser-worker.md` precedent from #797).
> **Director picked split.** Parser side now lives in
> [`t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)
> as a sibling pre-requisite sub-lane. This brief is now scoped to
> **post-parser-extension state**: assumes `declared_effects` field is
> already on `SurfaceType.Arrow` (or `SurfaceItem.Fn`) by the parser;
> this PR consumes that via the lens + diagnostic.

## Read first

- **[`THESIS.md` §"Enumerable impossible-bug classes" lines 345-347](../../THESIS.md)** — class definition: *"actual effect set must match declared; silent effect leakage rejected."* THESIS.md:347: *"Gated on deeper effect-system work beyond R1's Sub-A scope."*
- **[`docs/r2-structure.md` §"Goal 4"](../r2-structure.md)** — sub-lane scoping; `[R2+]` per ROADMAP T-Demo row.
- **[`src/v3/std/effects.dag` lines 262-506, 421-423, 501-505, 722-755](../../src/v3/std/effects.dag)** — current effect-system state. **Operation-level effects modeled** (ReadEffect / UpsertEffect / CreateEffect / AppendEffect / DeleteEffect; composition for idempotency at `:501-505`); effects derived from HTTP transport (`derive_op_effect` at `:722-755`). **Declaration-level effect enumeration NOT modeled** — no `effects: [Read, Write]` declaration syntax checked against function-body inference.
- **[`src/v3/lenses/idempotency.dag`](../../src/v3/lenses/idempotency.dag)** — closest existing precedent for walking effect lists. Walks declared `OperationEffect` lists at the workflow level; does not walk function-body computation DAGs.
- **[`src/v3/lenses/cost.dag`](../../src/v3/lenses/cost.dag)** + **[`src/v3/lenses/complexity.dag`](../../src/v3/lenses/complexity.dag)** — **strongest precedent**. Lane E (complexity lens) infers from structure → matches against declared. The exact pattern this brief generalizes: walk function body DAG → collect effects → check declared ⊇ inferred.
- **[`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)** — current `DiagnosticKind` (extend or sibling for `EffectLeakageError`).
- **[`MODELING.md`](../../MODELING.md)** — M9 + closed-system framing.
- **[`INVARIANTS.md`](../../INVARIANTS.md)** — `feedback_construction_over_ratchets` + `feedback_no_textual_enforcement_bridges`.

## Frame

The current effect system models operation-level effect shapes (HTTP method → effect class) and validates composition for idempotency. It does **not** validate that a function's declared effect set matches its actual body behavior. Today: a function declaring `effects: []` (pure) can call `log(...)` and `network_request(...)` with no compile-time error — silent effect leakage.

The structural fix follows the **complexity-lens precedent** exactly:
- Add an effect-inference pass that walks function bodies + collects actual effects (calls to side-effecting functions, I/O, mutation).
- Store inferred effects as a structural fact on each function declaration.
- At declaration site, type-check `declared_effects ⊇ inferred_effects` (declared covers everything actual).
- On mismatch: emit `EffectLeakageError { declared, inferred, leaking_ops }`.

Sub-lane scope: enough effect-substrate to close declared-vs-actual leakage end-to-end for at least one effect class **using the existing `OperationEffect` taxonomy**. Other effect classes follow the same pattern.

## Three consumer-side requirements

1. **Effect-inference pass — direct-call scope only (no inter-procedural call-graph walk).** Walks function body DAG; collects effects from **direct calls to known-effectful primitives** (e.g., calls that already have an `OperationEffect` derived via `derive_op_effect` at `effects.dag:722-755`). Does NOT follow transitive call chains (A calls B which calls C). Lives as a **lens** at `src/v3/lenses/effect_enumeration.dag`, parallel to `cost.dag` precedent (per `feedback_no_validation_passes` + cost/complexity-lens precedent: lens-shape, not validation-pass-shape; not a lowering-phase fact). The function-declaration *carries* a declared-effect carrier and a derived inferred-effect carrier as facts; mismatch is a structural Diagnostic, not a "check pass."
2. **Declared-effect carrier on function declarations as part of the function type signature — assumed already landed via parser sub-lane.** Per the split: the parser sub-lane ([`t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)) lands the surface-syntax extension to `SurfaceType.Arrow` (or `SurfaceItem.Fn`) and the lowered `declared_effects` field on the function-arrow connective. **STOP-AND-ESCALATE if dispatching against this brief and the parser sub-lane has NOT landed** — sequencing error. This substrate brief consumes the post-parser carrier; does NOT author it.
3. **Structural carrier mismatch surfaces a Diagnostic (no validation pass).** At declaration site, the carrier facts (declared + inferred) are present; if declared ⊇ inferred fails (set-cover violation), the lens emits `EffectLeakageError` naming declared set + inferred set + specific operations that leak. Per `feedback_no_validation_passes` + C-8 fail-closed: this is structural mismatch on always-present carriers, not a pass that "checks" a validity property. Smoke + integration test: a function declares no effects but body directly calls `service.upsert(...)` (an `UpsertEffect`-emitting op per `derive_op_effect`) → diagnostic; same function with `UpsertEffect` declared → accepted.

## Slice — effect inference + declared-effect carrier + leakage diagnostic

1. **Pre-flight check (NOT a parser-extension step)**: confirm the parser sub-lane PR has merged and `SurfaceType.Arrow` (or `SurfaceItem.Fn`) carries the `declared_effects` field on `main`. If not, STOP per req 2 — that's a sequencing error.
2. Implement effect-inference lens (per req 1) at `src/v3/lenses/effect_enumeration.dag`, parallel to cost.dag precedent.
3. Structural-mismatch Diagnostic (per req 3). New `EffectLeakageError` variant.
4. Demo using existing `OperationEffect` taxonomy: pick a function that calls a known-effectful primitive directly (e.g., a service method invoking `derive_op_effect` paths to produce `UpsertEffect` or `CreateEffect`). Do NOT pick `Logging` — it's not in the existing taxonomy and would force a coupled enum extension.
5. Smoke + integration tests per req 3.

## Acceptance

- [ ] All 3 consumer-side requirements satisfied + documented in PR body.
- [ ] Declared-effect carrier on function declarations; round-trips through DB-8.
- [ ] Effect-inference pass walks function bodies + collects actual effects.
- [ ] Declared-vs-inferred comparison at type-check; structured diagnostic on mismatch.
- [ ] End-to-end demo using existing `OperationEffect` taxonomy: silent leakage on a function declaring no effects but calling a known-effectful primitive (e.g., service `upsert`) → diagnostic; same function with explicit declaration → accepted.
- [ ] Existing operation-level effect modeling (`derive_op_effect`, idempotency lens) regression-free.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas as needed.

## STOP-AND-ESCALATE

Surface to Director.

- **Inference scope creeps beyond direct calls** — req 1 explicitly scopes to direct-call-only effect collection. If execution surfaces that the demo requires inter-procedural call-chain following (A→B→effect), STOP. That's a re-scoping decision; sub-lane may need to bundle a one-hop inter-procedural read or stay direct-call-only with the demo narrowed.
- **Existing `OperationEffect` taxonomy can't cover the demo** — req 4 anchors on the existing taxonomy. If the chosen demo function's effect doesn't already have a `derive_op_effect`-driven `OperationEffect` variant, STOP. Either pick a different demo function, or escalate the enum-extension decision to Director (which would couple this lane to a taxonomy-extension PR).
- **Declared-effect surface syntax requires non-trivial parser/grammar work** — req 2 anchors on extending the function-type-signature shape. If parser-side authoring requires invasive `parse_parser_body.txt` changes or a new `SurfaceExpr` variant, STOP. Coordinate with parser owners on a coupled PR or sibling parser sub-lane.
- **Cost / complexity lens precedent doesn't generalize** — if reusing the lens-pattern for effect inference reveals shape mismatches, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not closing inter-procedural effect inference** — single-function-body scope only for this demo.
- **Not lifting all std/ functions to declared-effect form** — only the demo path.
- **Not implementing the other two T-ImpossibleBugs classes** — independent briefs.
- **Not changing existing operation-level effect modeling** beyond what's needed for the new declared-effect facts to coexist.
- **Not building a full effect-polymorphism / effect-row system** — scoped to declared-vs-inferred match.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs — declared-vs-inferred effect check (closes unenumerated-effects class; demo via existing OperationEffect taxonomy)`.
- PR body cites this brief + addresses the 3 reqs + documents the chosen demo effect class + the inference scope choice (lens vs lowering-phase).
- On merge: signal Director; inter-procedural inference + bulk std/ annotation is post-cascade work.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if substrate.dag-adjacent.
- **Grounding Manager**: no current overlap.
