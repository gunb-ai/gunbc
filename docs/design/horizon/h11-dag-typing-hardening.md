# H11 Design: DAG Typing Hardening

## Problem

Node I/O still has weakly typed edges in some paths, and semantic carrier handling can silently degrade to structural compatibility.

## Decision

Introduce typed node I/O wrapper APIs at DAG boundaries and enforce fail-closed semantic carrier refinement.

## Proposed API Direction

- Typed wrappers:
  - `TypedInput<T>`
  - `TypedOutput<T>`
  - `TypedPort<T>`
- Builder helpers require typed ports for new code paths.
- Legacy stringly APIs remain temporarily with explicit deprecation gates.

## Semantic Carrier Policy

- Unknown semantic carriers are rejected.
- Carrier refinements are explicit and validated in registry.
- No fallback from semantic to structural compatibility in strict mode.

## Invariants

- Every edge has both structural type compatibility and semantic carrier compatibility.
- Carrier mismatches fail during build/validation, not execution.
- Typed wrappers map to existing `TypeId`/`Cardinality` without ambiguity.

## Migration Plan

1. Introduce typed wrappers and adapter methods.
2. Migrate key builders and codegen entrypoints.
3. Enforce strict semantic carrier checks by default in new paths.
4. Remove legacy fallback behavior after migration window.

## Follow-up Implementation Tasks

- `H11.1` Add typed wrapper structs and conversion helpers.
- `H11.2` Add typed builder APIs for node ports and edges.
- `H11.3` Enforce semantic carrier compatibility in validator.
- `H11.4` Migrate high-traffic builders to typed APIs.
- `H11.5` Add deprecation warnings for legacy untyped APIs.
