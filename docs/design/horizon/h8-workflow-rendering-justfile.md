# H8 Design: Render Workflows as DAGs to Justfile

## Problem

Workflow rendering currently has one concrete output (Makefile). Without a second renderer, registry abstractions are weakly validated.

## Decision

Adopt `Justfile` as the second renderer to validate workflow-model portability.

## Proposed Rendering Contract

Input model: `WorkflowSpec` graph with targets, deps, environment, and command steps.

Output mapping:

- `WorkflowTarget.id` -> `just` recipe name.
- DAG dependencies -> recipe dependencies.
- target variables -> recipe parameters / shell env export.
- dry-run metadata -> optional `@echo` preview recipes.

## Invariants

- Target graph must stay acyclic before rendering.
- Rendered ordering must be deterministic.
- Makefile and Justfile renderers must agree on target set and dependency edges.

## Migration Plan

1. Define renderer-neutral workflow model contract.
2. Implement `Justfile` renderer.
3. Add parity test comparing Makefile vs Justfile topology.
4. Add CLI flag to emit one or both formats.

## Follow-up Implementation Tasks

- `H8.1` Freeze `WorkflowSpec` renderer contract.
- `H8.2` Implement Justfile renderer module.
- `H8.3` Add parity tests for target/dependency graph equivalence.
- `H8.4` Add `makegen` output mode selection (`make|just|both`).
- `H8.5` Add golden snapshots for Justfile output.
