# Refactor Pressure: Structural Gaps That Drive Churn

**Status**: Draft
**Date**: 2026-02-05

## Goal

Reduce refactor churn by identifying the recurring structural gaps that
force rework and by adding lightweight guardrails that prevent regressions.

This doc is **adjacent to** `TODO/TODONE/architecture-debt.md` (DONE):
- Architecture debt explained what was broken and why (all phases complete).
- This doc explains how we keep re-creating similar debt and the
  project-level rules to stop it.

## One-sentence root cause

> We keep refactoring because the system still allows key behavior and
> meaning to exist outside the model (DAG/resources/types/IR), and the
> resulting duplicate sources of truth drift until they force a
> structural cleanup.

## Root Causes

### A) Model is not closed (behavior exists outside the DAG/declared inputs)

Any time code reaches out to the environment implicitly (env vars, clock,
platform detection, filesystem handles), the DAG cannot see or control
that input. This breaks DryRun interception, testability, and dependency
reasoning, which then forces late refactors to re-model the behavior.

Explains: resource acquisition not fully structural, hidden inputs/global
context, performance model bolted on, and part of the "tests detached
from DAGs" churn.

Current state: Resource acquisition Phases 1-3 are complete; remaining
gaps are sub-DAG delegation and resource accounting/auto-derive
`ResourceAccess` (Phases 4-5).

### B) Invariants are not enforced by construction (policy exists, but the system allows escape hatches)

We have strong invariants (no backdoors, no fallbacks, no warnings), but
churn happens when the codebase can violate them and only finds out later
(review, lint, runtime). Enforcement must be structural, not social.

Explains: lint fights, crate-boundary debt, fast paths added late, and
IR/modeling gaps that surface as generated-code lints.

### C) Semantics are duplicated across layers (two sources of truth)

When the same meaning lives in two places, they drift. Examples include
cardinality split between ports and type contracts, duplicated hash
logic, and registry/dependency definitions across multiple lists.

Explains: multiple sources of truth, split semantics (dual encoding),
string-based references, and rename drift.

Proven pattern to copy: Makefile/gitignore generation already uses a
single source of truth via `ToolRegistry` + `BuildConfig`. Apply that
pattern to DagSpec/registry dispatch/meta-target deps.

### D) Cross-cutting concerns appear before they have a home

Shared concerns (hashing/manifest, registry metadata, build artifact
policy, resource dependency rules) get wedged into a convenient crate
until the third duplication forces a refactor. The rule is to create a
proper crate/module boundary early, not after drift.

## Decision Rules (PR Gate)

- Single source of truth: if a PR introduces the same concept in two
  places, refactor before merge or add a scoped exception with a linked
  TODO.
- No new stringly references: any new string that names a node/target/
  resource/registry must be replaced with a typed ref or a macro that
  derives the string from a symbol (renames must fail at compile time).
- No hidden env or IO: any `std::env::var`, `SystemTime::now`, platform
  detection, or `FilesystemHandle` creation outside env/resource nodes
  is a hard fail (unless listed in Exceptions).
- Fast path declaration: any new freshness/check logic must state its
  fast path and slow path in the PR description and encode the contract
  in code.
- Generated code linting: if a generated file triggers a lint, fix the
  IR or clippy config. Never add `#[allow]` in generated output.
- No ambient globals: exec mode, toolchain info, and policy flags must
  be explicit inputs (resource or executor context), not global state.
- Sub-DAGs receive only delegated resources: no implicit inheritance of
  the parent environment.

## Signals and Metrics

- Count of stringly builder/registry references (target: trend to zero).
- Count of manual registries or duplicate lists for the same concept.
- Count of `std::env::var`/`SystemTime::now` outside env/resource nodes.
- Count of generated-code lints in CI (target: zero).
- Count of `#[allow(clippy::disallowed_methods)]` in runtime crates
  outside explicit exceptions (target: zero).
- Freshness-check p95 when inputs unchanged (target: <10ms).
- Percent of DAGs with typed MockSpecs and node examples (target: 100%).

## Exceptions (Explicit and Scoped)

Exceptions are allowed only when they are crate-scoped and documented in
one place (not sprinkled `#[allow]` at call sites). Approved categories:

- Bootstrap/codegen/testgen (the generators themselves).
- Dev-only introspection tools (e.g., DAG visualizers).
- One-off migration scripts.

## Tasks (Acceptance Criteria)

- [x] Add a short "Refactor-Pressure Checklist" to `AGENT.md` and/or
      `SPEC.md`. ✅ Both files contain the checklist (AGENT.md, SPEC.md).
- [ ] CI guardrail: generated code linting. Done when CI runs
      `make codegen && make testgen && cargo clippy --all-targets -- -D warnings`
      and fails on any lint in generated output.
- [ ] CI guardrail: boundary erosion. Done when CI fails if new
      `#[allow(clippy::disallowed_methods)]` appears in runtime crates
      outside the explicit exceptions list.
- [x] **CI guardrail: generated artifact drift.** ✅ `make verify` target
      runs `--check` on makegen, bootstrap, and testgen. `make test`
      includes `verify` in its dependency chain. Each `--check` flag
      does dry-run + `verify_drift()` comparison against disk.
      Remaining: cigen uses FileWriter, not DAG transport; migrate later.
- [ ] DagSpec / typed registry. Done when testgen, makegen, and CI
      generation all consume DagSpec, and registry dispatch is no longer
      string-based. (Note: `GraphBuilderId` enum already eliminates
      string-based builder dispatch.)
- [ ] Resource acquisition completion. Done when Phases 4-5 are landed:
      sub-DAG delegation + resource accounting/auto-derive `ResourceAccess`.
      (Note: Phases 1-3 complete per TODONE/design-resource-acquisition.md.)
- [ ] Type/cardinality unification. Done when cardinality has a single
      source of truth (TypeContract or equivalent) and port cardinality
      is derived or validated with no fallback semantics.
      **Decision needed:** interval model (property-based) vs current enum.
      See `TODO_hacks` "Cardinality constants are flat" and type coercion
      design doc. Deciding this unblocks the remaining dual-encoding cleanup.
- [x] Fast path for freshness checks. ✅ mtime fast path in
      `core/infra/src/freshness.rs`. `ManifestEntry.input_file_count` tracks
      file count for fast invalidation.
- [x] Remove `GUNBC_EXEC_MODE` global. ✅ exec_mode threaded via DAG edges.

## Quick Scans (rg)

- `rg 'graph_builder:\s*String'`
- `rg '"build_.*_graph"'`
- `rg 'extra_deps|fix_deps|PrepLevel'`
- `rg 'std::env::var\(|env::var\('`
- `rg 'SystemTime::now\('`
- `rg 'Platform::detect\('`
- `rg 'FilesystemHandle::'`
- `rg '#\[allow\(clippy::disallowed_methods\)\]'`
- `rg 'TypeId\(|type_id:|Cardinality::'`

## Notes

Review quarterly to confirm new work is not re-introducing these
structural gaps.
