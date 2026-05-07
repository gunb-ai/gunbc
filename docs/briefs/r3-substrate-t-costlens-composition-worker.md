# Worker brief — Substrate T-CostLens-Composition (γ ratified)

**Sub-issue**: gunbc#2141 (parented under #1939 Substrate Mgr lane; supersedes #1957 per surgical-recreate at gunbc#828 #issuecomment-4397693699).
**Authority**: Director ratification of **option γ** at gunbc#828 #issuecomment-4395691775 (2026-05-07); supersedes the canvas at `docs/briefs/r3-substrate-t-costlens-composition-canvas.md` (canvas may be deleted after this brief lands per single-authority discipline).
**Closure predicate**: §1.8 gates #37 `cost_lens_reads_target_realization`, #38 `coercion_cost_equals_complexity_by_construction`, #39 `no_coercion_cost_dimension`, #40 `symbolic_cost_expr_equals_executable`, plus #70 `cost_lens_demonstration`.

## Important framing — this is NOT a substrate-fact-introduction

`src/v3/lenses/cost.dag` ALREADY EXISTS at HEAD as a `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` lens (per the file's status header). T-CostLens-Composition is **behavioral-completion + target-realization-wiring**, NOT P1 carrier introduction. Substrate primitives are all already-substrate:

- `Lens<C>` generic at `src/v3/std/lens.dag:70-77` — **untouched** per Director ratification (any refactor is separate scope, STOP-and-PING)
- `SymbolicCost` 7-variant + `Semiring<SymbolicCost>` at `src/v3/std/algebra.dag:12+`
- `Lookup<SymbolicCost>` + `MissingCost` lens-boundary-fallback at `src/v3/std/lookup.dag:48-60`
- `lenses/cost.dag` lens-instance scaffolding (existing behavioral PROXY)

Worker should **read the existing `lenses/cost.dag`** before doing anything else; the work is to advance it from PROXY → BEHAVIORALLY COMPLETE via target-realization composition, NOT to manufacture parallel substrate.

## Scope (binding per Director γ ratification)

Wire target-realization-cost into the existing `lenses/cost.dag` `Lens<SymbolicCost>` instance via `Lookup<SymbolicCost>` composition, satisfying all 4 §1.8 gates by construction:

- **#37 `cost_lens_reads_target_realization`** — lens output reads target-realization cost from `Lookup<SymbolicCost>` keyed on per-primitive identity (target-spec-derived). `MissingCost` lens-boundary-fallback already substrate at `lookup.dag:48-60`.
- **#38 `coercion_cost_equals_complexity_by_construction`** — coercion-cost composes into the same `SymbolicCost` algebra as algebra-cost via `Semiring::sum` / `::product`; the equation is structural, not derived.
- **#39 `no_coercion_cost_dimension`** — single `SymbolicCost` algebra; no parallel cost dimension. The existing lens is already structurally aligned (uses `std.algebra::sequential`/`iterate`/`max_path`); worker confirms no parallel dimension introduced.
- **#40 `symbolic_cost_expr_equals_executable`** — runtime-executable `SymbolicCostExprEquals` predicate consuming the lens output. Test_runner / cementing-test harness wiring per existing infrastructure.

Plus **#70 `cost_lens_demonstration`** — fixture program with ≥2 algebra-instances composed + ≥1 recursive call + observable cost-bound output. Same-slice acceptance.

## Out-of-scope (deferred per Director ratification)

- `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>` — Director ratified DEFER to separate Dimensions sub-lane (per §10.3 Q-CostLens-Dimensions framing if/when authored). T-CostLens proceeds without it; consumes the dimension as structural fact only when it lands.
- `Lens<C>` generic carrier-shape refactor — STOP-and-PING the Mgr if implementation reveals a need; that's separate Director-tier scope question, NOT T-CostLens absorption.

## Acceptance gates (same-slice, all must pass)

1. `lenses/cost.dag` advanced from `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` → `BEHAVIORALLY COMPLETE` per `docs/v3-lens-capability-register.md` audit; status header updated in the file itself.
2. **#37 satisfied**: lens output documented to read target-realization cost via `Lookup<SymbolicCost>`. Code-level cite in PR description.
3. **#38 + #39 satisfied by construction**: lens output is `SymbolicCost`-typed; no parallel cost dimension visible in the diff. Verified via grep at acceptance time.
4. **#40 satisfied**: `SymbolicCostExprEquals` predicate executable in test_runner; runtime predicate evaluates against representative input (NOT NotYetImplemented shell).
5. **#70 satisfied**: `cost_lens_demonstration` fixture program lands at `src/v3/compiler/tests/integration/` (or appropriate fixture location); ≥2 algebra-instances + ≥1 recursive call + observable cost-bound output verified by demonstration test.
6. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
7. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.
8. **§10.3 row text refresh**: `docs/r3-program-plan.md` §10.3 T-CostLens-Composition row (currently "(TBD from Substrate canvas)") updated to cite γ-disposition + this PR's # as the receipt.

## STOP / PING criteria

- **STOP** if implementation reveals `Lens<C>` generic at `lens.dag:70-77` needs carrier-shape refactor (e.g., `read: fn(Dag, Behavior) -> Witness<C>` needs target-context threading) — that's separate Director-tier scope per Director's confirmed ask #2; do NOT absorb into T-CostLens.
- **STOP** if `Lookup<SymbolicCost>` composition is structurally insufficient for end-to-end realization-cost reading (e.g., target-realization-cost requires more context than per-primitive lookup can carry) — surface to Mgr; canvas option β (composed-witness) may need re-ratification.
- **STOP** if `cost_lens_demonstration` fixture authoring requires `data symbolic_cost_dimension` to be present — that contradicts Director's defer disposition; surface to Mgr to resolve cross-lane dependency.
- **PING** Verification Mgr (#2075 / `wise-bear-525`) at PR-open time so they can advance Pattern-A executable gate #40 `symbolic_cost_expr_equals_executable` per §1.6 NYI → executable transition discipline.

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: PING at PR-open per gate #40 transition; ratchet authoring is Verification's standing concern.
- **Grounding Mgr**: target-realization-cost reading depends on target language spec (per `r3-design-schedule:72` cost-lens-as-discriminator). If target-spec data isn't yet populated for the demonstration fixture's targets, surface to Grounding Mgr (#1944 / current active session) for cross-lane data population.
- **R4-carve note**: T-LBP option (b) RATIFIED 2026-05-06 means T-LBP R3 scope = complexity + cost lenses only. T-CostLens-Composition closing simultaneously with `cost_lens_behaviorally_complete` (gate #80) is the critical-path-fastest sequencing per Q-LBP option (b).

## Worker pin (Mgr disposition)

Substrate-fact-introduction precedent owners — **valiant-ibex-312** (delivered IntPlatform/UIntPlatform via PR #1933, S5 candidate) OR **smart-ram-167**. Lens-behavioral work also has precedent owners; final pin at dispatch.

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch on this brief until auto-spawn fix lands per L-sized substrate-behavioral-completion threshold, OR Mgr-direct authoring if scope-cohering small-enough (this is L-sized; Mgr-direct is unlikely fit unless Director ratifies surgical-recreate path for cascade unblock).

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 post-#2112 merge per Director γ-ratification at gunbc#828 #issuecomment-4395691775.
