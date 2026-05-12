# Worker brief — T-WAD FULL R3 ci.dag scaffold first-draft

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-2; operator FULL elevation 2026-05-12.
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698) — this is forward-looking scaffold; final ratification + emitter integration in Slice 4-5.
**Closure predicate**: scaffold authored at `dsl/extdeps/github/ci.dag` + Mgr-ratified shape; downstream Slice 4 YamlStatic emitter consumes; downstream Slice 8 deletes hand-authored `.github/workflows/ci.yml`.

## Output

`dsl/extdeps/github/ci.dag` — new file, captures current `.github/workflows/ci.yml` content as `Workflow<Trigger, Steps, Resources>` `.dag` data.

## Scope

Translate current `.github/workflows/ci.yml` into a `.dag` data instance using existing carriers at `dsl/extdeps/github/actions.dag`:

1. **Trigger**: capture `on:` shape (push / PR / schedule) as `WorkflowTrigger` instance (per `dsl/extdeps/github/actions.dag:40`)
2. **Jobs**: each `job` in ci.yml becomes a `Step` (or composition of `Step`s) per `dsl/extdeps/github/actions.dag:103+`
3. **Dependencies**: `needs:` graph captured as Step dependency edges
4. **Matrix**: any `strategy.matrix:` captured as `MatrixStrategy` instance per `dsl/extdeps/github/actions.dag:66+`
5. **Runner selection**: `runs-on:` captured as `RunnerSpec` + `RunnerLabel` per `:88+` (use `gunbc-quick` / `gunbc-v3` / generic pools as appropriate)
6. **Layer 1 skip** (changes job): capture as `Step` with conditional output binding
7. **Layer 2 cluster skip** (Director template per PR #2721 — NOT YET WIRED): SKIP — capture as `[TODO]` marker; this is what Slice 7 affected-set integration will replace
8. **Slow-test ratchet**: SKIP `scripts/slow-test-exemptions.txt` reference for now (Slice 6 will replace with Cost dimension on test nodes)
9. **emission_target placeholder**: include comment-marker `// emission_target: YamlStatic` placeholder field (final shape ratified by emitter-dispatch canvas in WI-1; this scaffold can be revised when canvas ratifies)

## Coverage scope

This is a FIRST DRAFT — does NOT need to cover 100% of ci.yml. Coverage acceptance:

- **MUST**: all `jobs:` represented as Step nodes
- **MUST**: `needs:` graph faithful
- **MUST**: trigger conditions captured (push/PR/schedule trio)
- **MUST**: runner pool selection per job correct
- **SHOULD**: matrix strategies for jobs that use them
- **SHOULD**: env / secrets references (Slice 1 substrate `WorkflowSecret<Name>` is HELD; use placeholder comment-markers for now)
- **NICE-TO-HAVE**: complete `with:` / `uses:` action references for each Step
- **DEFERRED**: Layer 2 cluster `if:` regex (TODO marker); slow-test-exemptions (TODO marker); affected-set wiring (TODO marker, Slice 7)

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope
- `docs/briefs/r3-t-wad-full-r3-emitter-dispatch-canvas-worker.md` — sibling brief; WI-1 emitter-dispatch canvas (this scaffold's emitter consumer)
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 1 substrate (`WorkflowSecret<Name>` not yet landed — placeholder)
- `dsl/extdeps/github/actions.dag` — existing carriers (read this first; reuse, don't duplicate)
- `.github/workflows/ci.yml` — source content to translate
- `docs/design-emission-model.md` — emission architecture

## Acceptance gates

1. `dsl/extdeps/github/ci.dag` exists; valid `.dag` syntax per existing v3 parser
2. All ci.yml `jobs:` represented as Step nodes (count matches)
3. `needs:` graph faithfully captured
4. Trigger conditions captured (push / PR / schedule trio)
5. Runner pool selection per job correct (`gunbc-quick` for fast jobs, `gunbc-v3` for v3 work, generic otherwise)
6. TODO markers explicit for: Layer 2 cluster `if:` regex; slow-test-exemptions reference; affected-set wiring; `WorkflowSecret<Name>` references (Slice 1 HELD)
7. `cargo test --workspace` green (no breakage of existing tests)
8. `cargo clippy --all-targets -- -D warnings` clean
9. `cargo fmt --all --check` clean

## STOP / PING criteria

- **STOP** if existing carriers at `dsl/extdeps/github/actions.dag` are insufficient (e.g., need new step-shape, new trigger-shape) — surface to Mgr; substrate-shape additions belong to Substrate Mgr canvas (per `feedback_substrate_shape_belongs_in_mgr_canvas`)
- **STOP** if Slice 1 substrate (`WorkflowSecret<Name>`) is genuinely required for scaffold coverage (e.g., secrets used in critical jobs that can't be TODO-markered) — surface; may need to wait on Slice 1 dispatch
- **STOP** if `ci.dag` requires `emission_target` field on existing `Workflow<>` carrier and that field isn't yet declared — surface; either add as placeholder comment OR wait for WI-1 canvas ratification
- **PING** PM (deep-wolf-155) on PR-open for review-routing
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification + Slice 4 emitter-implementation dispatch routing

## Sequencing

- Dispatch-ready NOW (independent of Slice 1 substrate HOLD — uses existing carriers; TODOs marked for held substrate)
- Output is a scaffold/placeholder; final shape ratified during Slice 4 YamlStatic emitter implementation
- Slice 4 YamlStatic emitter consumes this `ci.dag` and emits `.github/workflows/ci.yml`-equivalent; equivalence is the Slice 4 acceptance gate

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive.
