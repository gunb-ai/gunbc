# T-ImpossibleBugs — unenumerated effects `(S, R2)`

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 3 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes — any worker can dispatch
> in parallel. Reports to Director (`zesty-bear-812`).

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

Sub-lane scope: enough effect-substrate to close declared-vs-actual leakage end-to-end for at least one effect class (likely `Logging` as the demo). Other effect classes follow the same pattern.

## Three consumer-side requirements

1. **Effect-inference pass.** Walks function body DAG; collects effects from called functions / built-in side-effect ops. Lives as a new lens (`src/v3/lenses/effect_enumeration.dag`, parallel to cost.dag) or as a lowering-phase fact (worker decides; surface in PR description).
2. **Declared-effect carrier on function declarations.** Substrate-level field on function declarations carrying the declared effect set (e.g., `Set<EffectClass>` or `List<EffectClass>`). Parser must accept the syntax (likely `fn foo() effects: [...] { ... }` or annotation-style — worker decides surface syntax with explicit reasoning).
3. **Type-check + diagnostic.** At declaration site, compare `declared_effects` against inferred-from-body. If declared ⊇ inferred (declared covers everything actual), accept. Otherwise emit `EffectLeakageError` naming the declared set, the inferred set, and specific operations that leak. Smoke + integration test: function declares `effects: []` but body calls `log(...)` → diagnostic; function declares `effects: [Logging]` and body calls `log(...)` → accepted.

## Slice — effect inference + declared-effect carrier + leakage diagnostic

1. Add declared-effect carrier (per req 2) to function declarations. Parser/lowerer extension.
2. Implement effect-inference (per req 1) — new lens or lowering-phase fact.
3. Type-check + diagnostic (per req 3). New `EffectLeakageError` variant.
4. Annotate one demo effect class (`Logging` suggested) on the relevant std/ functions (e.g., `log`, `info`, etc.) so the inference has something to discover.
5. Smoke + integration tests per req 3.

## Acceptance

- [ ] All 3 consumer-side requirements satisfied + documented in PR body.
- [ ] Declared-effect carrier on function declarations; round-trips through DB-8.
- [ ] Effect-inference pass walks function bodies + collects actual effects.
- [ ] Declared-vs-inferred comparison at type-check; structured diagnostic on mismatch.
- [ ] End-to-end demo: silent `log` leakage → diagnostic; explicit `effects: [Logging]` → accepted.
- [ ] Existing operation-level effect modeling (`derive_op_effect`, idempotency lens) regression-free.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas as needed.

## STOP-AND-ESCALATE

Surface to Director.

- **Effect-inference walks beyond function-body scope** — if inference needs to follow inter-procedural call chains (e.g., A calls B which calls log; A must declare Logging), STOP. Inter-procedural inference is its own scope; sub-lane may need re-scoping.
- **Effect-class enumeration** — if the existing `OperationEffect` enum doesn't cover the effect classes the demo needs (e.g., `Logging` isn't in the existing taxonomy), STOP. Director-call on whether to extend the enum or use a generalized effect-class.
- **Declared-effect surface syntax conflicts** — if the chosen syntax (`fn foo() effects: [...]`) conflicts with parser / surface-lang authority, STOP. Coordinate with parser owners.
- **Cost / complexity lens precedent doesn't generalize** — if reusing the lens-pattern for effect inference reveals shape mismatches, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not closing inter-procedural effect inference** — single-function-body scope only for this demo.
- **Not lifting all std/ functions to declared-effect form** — only the demo path.
- **Not implementing the other two T-ImpossibleBugs classes** — independent briefs.
- **Not changing existing operation-level effect modeling** beyond what's needed for the new declared-effect facts to coexist.
- **Not building a full effect-polymorphism / effect-row system** — scoped to declared-vs-inferred match.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs — declared-vs-inferred effect check (closes unenumerated-effects class via Logging demo)`.
- PR body cites this brief + addresses the 3 reqs + documents the chosen demo effect class + the inference scope choice (lens vs lowering-phase).
- On merge: signal Director; inter-procedural inference + bulk std/ annotation is post-cascade work.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if substrate.dag-adjacent.
- **Grounding Manager**: no current overlap.
