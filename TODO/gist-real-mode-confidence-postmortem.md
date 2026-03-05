# Postmortem: `make gist` Real Mode Failed Despite Green Compile/Test Signals

Canonical rolling tracker: `TODO/gist-rolling-postmortem.md`

> **Date**: 2026-03-05  
> **Severity**: High (trust regression)  
> **Impact**: `make gist` failed in Real mode with a profile-stub error after successful compile and broad test pass signals.

---

## Summary

`make gist` failed in Real mode with:

`interface stub CredentialProvider.acquire requires --profile: no active profile bindings`

This happened even though:

1. The make target passed `--profile profiles.gist.local`.
2. Codegen and compile completed successfully.
3. Gist-focused tests passed.

The result is a trust-model violation: green compile/tests did not provide high confidence that the user-facing command would succeed.

---

## User Expectation vs Actual Behavior

### Expected

If the repo compiles and tests pass, running `make gist` should have high probability of success (or fail earlier with a compile/test-time signal).

### Actual

`make gist` failed at runtime in Real mode due to interface-stub execution, while DryRun and existing gist tests did not reliably surface the same path.

---

## Reproduction

From workspace root:

```bash
make gist
```

Observed failure:

```text
interface stub `CredentialProvider.acquire` requires --profile: no active profile bindings
```

Dry-run path succeeds:

```bash
make gist-dry
```

This mismatch creates false confidence for command-level reliability.

---

## Technical Root Cause

### 1. Profile flag is present, so failure is not a CLI parsing issue

- `make gist` passes `--profile profiles.gist.local`.
- Generated `gunbc-gist` CLI accepts and validates that profile.

So the failure is downstream of argument parsing.

### 2. Credential provider operation was treated as interface-stub transport in the failing runtime path

Lowering classifies service operations with no explicit `transport { ... }` as `InterfaceStub` when the service implements an interface.

`LocalCredentialProvider` implements `CredentialProvider`, and `acquire` lacks explicit transport binding at operation level in current modeling style.

Runtime resolver behavior for `InterfaceStubExecuteOp` is fail-fast in Real mode by design:

- Real mode: always errors with `requires --profile`.
- DryRun mode: auto-mocked, so execution appears healthy.

This design is useful for unresolved interfaces, but it also means a bad/ambiguous lowering path can escape compile-time validation and fail only in live command execution.

### 3. Existing gist test suite did not exercise the same command contract

Current gist tests mostly build graphs with `profile=None`, and several end-to-end tests are ignored.  
This means the tested graph shape is not equivalent to `make gist` real invocation shape.

---

## Why DryRun Did Not Catch It

DryRun and Real mode are not equivalent for interface-stub nodes:

1. DryRun auto-mocks stub execute path.
2. Real mode errors immediately on stub execute.

So DryRun success is not currently sufficient evidence for Real mode success when profile-bound interfaces are in the graph.

---

## Contributing Gaps

1. **Contract gap**: No enforced invariant that profile-selected tools must not contain unresolved interface-stub execute nodes in Real mode paths.
2. **Coverage gap**: Gist tests primarily validate `profile=None` graphs and structural properties, not command-level real-profile execution contracts.
3. **Signal gap**: Runtime error message says “no active profile bindings”, but in this incident a profile was passed; the message does not expose selected profile + resolved bindings + why stub remained.
4. **CI noise gap**: unrelated failing tests (e.g., stale `dsl/pipelines/ci.dag` path in daglang-cli tests) dilute confidence in “all green means safe to run”.

---

## Reliability Goal (Repo-Level)

For tool entrypoints (`make <tool>` / generated CLIs), enforce:

1. **Compile + tests green => high confidence command success** for normal local prerequisites.
2. **If not satisfiable**, fail during compile/test with targeted diagnostics, not at runtime deep in execution.

---

## Proposed Repo-Level Fixes

### P0: Add profile-realization invariant test for each profile-bound tool

For each tool/profile pair used by make targets (starting with gist):

1. Build graph with the same profile as make target.
2. Assert no unresolved interface-stub execute nodes exist for required interface calls.
3. Assert expected concrete provider nodes are present.

This catches profile-binding regressions before runtime.

### P0: Add command-contract tests for generated CLIs

Add tests that execute generated binaries with:

1. `--profile ... --print-inputs json` parsing checks.
2. Real-mode graph build + lowered-node contract checks (without external network side effects).

Focus is parity with actual user command invocation, not just library-level graph construction.

### P0: Strengthen diagnostics for stub execution errors

When Real mode hits interface-stub execute, include:

1. Selected profile value.
2. Whether profile bindings were discovered.
3. Interface key looked up and candidate bindings.
4. Suggested fix path (missing bind vs unresolved implementation vs wrong profile name).

This reduces debugging time and avoids misleading “no active profile bindings” ambiguity.

### P1: Explicit policy for DryRun/Real mode confidence boundary

Document and enforce:

1. DryRun is structural and mock-path validation.
2. DryRun alone is not sufficient for profile-bound interface resolution correctness.
3. Each profile-bound command must have at least one non-ignored real-mode contract test with controlled backend/mocks.

### P1: Unignore or replace currently ignored gist e2e tests

Ignored tests hide active regressions in the exact tool where confidence is required.  
Convert to deterministic contract tests where possible (transport backend injection).

### P1: CI stability cleanup of known stale test paths

Fix known stale references like `dsl/pipelines/ci.dag` vs `dsl/workflows/ci.dag` so global test status is trustworthy and does not mask signal quality issues.

---

## Immediate Action Plan

1. Add `gist` profile-realization contract test:
   - Build `tools/gist.dag` with `profiles.gist.local`.
   - Fail if execute nodes include unresolved interface-stub path for `CredentialProvider.acquire`.
2. Fix stale CI pipeline test path references in `daglang-cli` tests.
3. Improve interface-stub Real-mode error to print active profile resolution context.
4. Track all profile-bound tool targets and add the same invariant test template.

---

## Acceptance Criteria for Closure

This incident is closed only when:

1. `make gist` path is covered by automated tests that mirror real invocation profile selection.
2. A regression in profile resolution fails tests before user-facing runtime.
3. Error diagnostics clearly identify profile-selection and binding-resolution state.
4. No ignored tests remain for core gist command contracts.

---

## Meta: Confidence Contract

The repo should treat this as a product-level quality contract:

> A user running a first-class command (`make gist`, `make ci`, etc.) should not discover a profile-resolution failure that our compile/test gates could have detected.

This is not only a gist bug; it is a command-reliability systems issue.
