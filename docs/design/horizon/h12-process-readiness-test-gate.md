# H12 Design: Process Readiness Test Gate

## Problem

Current execution paths can start side-effecting processes even when fast safety checks are red or unknown. This creates avoidable runtime failures and weakens confidence in process-level reliability.

## Decision

Introduce a process readiness gate where a process must satisfy a declared "small test" dependency before execution. This is a planner/runtime policy, not an ad-hoc preflight hook in each binary.

## Scope

- Gate only fast checks (unit/contract/smoke), not full integration suites.
- Attach readiness requirements to process units in workflow metadata.
- Fail closed for required readiness checks in real mode.

## Invariants

- A process with `requires_readiness=true` cannot execute until readiness is green.
- Readiness evidence is keyed and cached independently from process output cache.
- Stale or missing readiness evidence is a typed miss reason (not silent fallback).
- Dry-run can bypass execution while still reporting readiness state.

## Migration Plan

1. Extend process unit metadata with readiness policy (`none`, `small`, future tiers).
2. Add a readiness ledger keyed by test command + inputs + policy version.
3. Teach planner to require readiness nodes before gated process nodes.
4. Surface readiness status/miss reason in `--plan` and execution reports.
5. Roll out to side-effecting processes first (`network:*` writers).

## Follow-up Implementation Tasks

- `H12.1` Add readiness policy fields to process unit schema/registry.
- `H12.2` Define readiness key model + ledger format.
- `H12.3` Planner integration: readiness dependency edges + miss reasons.
- `H12.4` Executor integration: fail-closed admission in real mode.
- `H12.5` Report/CLI UX for readiness state.
- `H12.6` Migrate existing tools from ad-hoc preflight to readiness policy.
