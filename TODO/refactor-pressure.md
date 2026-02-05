# Refactor Pressure: Missing Structural Patterns

**Status**: Draft
**Date**: 2026-02-05

## Goal

Reduce refactor churn by identifying the recurring structural gaps that
force rework and by adding lightweight guardrails that prevent regressions.

This doc is **adjacent to** `TODO/architecture-debt.md`:
- Architecture debt explains *what* is broken and why.
- This doc explains *how we keep re‑creating similar debt* and the
  project-level rules to stop it.

## Observed Patterns (Root Causes)

1. **Multiple sources of truth**
   - Same concept defined in more than one place (registries, DAG targets,
     meta-target deps, string builder names).
   - Leads to rename drift and silent breakage.

2. **Environment access outside the DAG**
   - IO or resource acquisition happens inside nodes rather than via
     explicit resource/env nodes.
   - Breaks testability, DryRun, and dependency reasoning.

3. **Split semantics (dual encoding)**
   - Cardinality lives both in `type_id` and on `Port`.
   - Coercion analysis diverges from runtime behavior.
   - Type semantics drift and require repeated fixes.

4. **IR is syntactically complete but idiomatically incomplete**
   - Generated code is valid but not idiomatic → lint failures →
     IR extensions after the fact.

5. **Performance fast paths are added late**
   - Correctness-first is good, but missing fast paths cause
     post‑merge refactors (e.g., hashing, freshness checks).

6. **Ambient global state / hidden inputs**
   - Env vars or implicit dependencies leak behavior outside the model.

7. **Tests partially detached from DAG structure**
   - Mock/spec coverage lags real DAG structure, creating constant
     testgen and mock refactors.

## Guardrails (Process Rules)

1. **Single Source of Truth Gate**
   - Every new concept must answer: *Where is the single source of truth?*
   - Derived artifacts must be generated from it, not hand‑maintained.

2. **No new stringly‑typed references**
   - Builder names, target deps, and registry IDs must be typed
     or generated from symbols in one place.

3. **Every environment touchpoint is a resource**
   - IO happens only in env/resource nodes; nodes consume capabilities
     via ports.

4. **IR completeness checklist**
   - If a lint fires on generated code, first fix the IR (not the output).

5. **Fast path required for checks**
   - Any new freshness or validation check defines a fast path up front.

6. **No ambient globals**
   - Execution mode, toolchain info, and other context must be explicit
     inputs (resource or executor context).

## Tasks

- [ ] Add a short “Refactor‑Pressure Checklist” to `AGENT.md` and/or `SPEC.md`
      (single source of truth, no stringly refs, env access via resources).
- [ ] Add CI guardrails:
      - Fail if new `#[allow(clippy::disallowed_methods)]` appears in `core/ir`
      - Fail if new generated code lints appear (regenerate + clippy)
- [ ] Replace string‑based DAG builder registry with a typed registry
      (DagSpec or equivalent single source of truth).
- [ ] Complete resource acquisition Phases 4–5
      (sub‑DAG delegation + derived `ResourceAccess`).
- [ ] Merge type/cardinality semantics into a single source of truth
      (TypeContract‑based coercion and validation).
- [ ] Define explicit fast paths for resource freshness checks
      (mtime + per‑file cache, optional git‑aware).

## Notes

This doc should be revisited quarterly to validate that new work
is not re‑introducing these structural gaps.
