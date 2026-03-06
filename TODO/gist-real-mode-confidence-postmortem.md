# Postmortem: `make gist` Real Mode Failed Despite Green Compile/Test Signals

Canonical rolling tracker: `TODO/rolling-postmortem.md`

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

## Architectural Clarification

This incident exposed a deeper design problem than one broken tool:

1. `profile` leaked domain/configuration decisions into user-facing tooling and compiler/runtime surfaces.
2. Credential selection was modeled as a workflow concern instead of a domain-model concern.
3. The repo mixed architectural layers:
   - abstract credential requirement in workflow/interface space
   - profile binding in compiler/runtime space
   - concrete local/cloud fallback logic inside a single tool path

That layering is not where we want to end up.

### Target Direction

The intended model is:

1. Workflows declare abstract credentialing/capability requirements.
2. Domain modeling resolves the concrete credential path from caller setup and modeled environment.
3. Local development, CI, and cloud execution are domain facts, not `--profile` flags.
4. Workflow-facing helpers should not be forced to contain fallback trees like "try env, otherwise gcloud" to compensate for missing domain modeling.
5. Rust should not carry repo-specific `gcloud` acquisition/install residue for this path; the concrete operational model belongs in `.dag`.

### Current State, Fairly Stated

As of this branch:

1. The specific `CredentialProvider.acquire requires --profile` failure was avoided by moving gist credential resolution out of the old profile-bound interface path.
2. That is an immediate reliability improvement for `make gist`.
3. It is still transitional, not the desired end state, because gist still depends on a shared concrete credential helper that encodes fallback behavior outside the modeled provider path.

---

## User Expectation vs Actual Behavior

### Expected

If the repo compiles and tests pass, running `make gist` should have high probability of success, or fail earlier with a compile/test-time signal.

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

## Incident Root Cause

At the time of the failure:

1. `make gist` passed `--profile profiles.gist.local`.
2. The generated CLI accepted that flag.
3. Runtime still reached an unresolved `CredentialProvider` interface-stub execute path.
4. Real mode failed fast on that unresolved stub, while DryRun auto-mocked it.

So the immediate bug was real, but the larger lesson is that the system made user success depend on profile plumbing that should not be first-class to begin with.

### Why This Happened

1. Credential realization depended on active profile binding.
2. The compile/test path did not prove that the real command path was concretely realized.
3. DryRun and Real mode diverged in exactly the place where unresolved profile/interface binding mattered.

The failure mode was specific, but the architectural problem was broader: user-facing success depended on a profile concept that should eventually disappear.

---

## Why DryRun Did Not Catch It

DryRun and Real mode are not equivalent for interface-stub nodes:

1. DryRun auto-mocks stub execute path.
2. Real mode errors immediately on stub execute.

So DryRun success is not currently sufficient evidence for Real mode success when profile-bound interfaces are in the graph.

---

## Contributing Gaps

1. **Architecture gap**: profile selection remained a required runtime/compiler concern for a user tool path.
2. **Contract gap**: tests did not prove the concrete command path used by `make gist`.
3. **Coverage gap**: several gist end-to-end tests were ignored or exercised a different graph shape than the real command path.
4. **Signal gap**: runtime error said “no active profile bindings” even though the user did pass a profile, which obscured the real failure mode.
5. **CI noise gap**: unrelated failing tests (e.g., stale `dsl/pipelines/ci.dag` path in daglang-cli tests) dilute confidence in “all green means safe to run”.

---

## Reliability Goal (Repo-Level)

For tool entrypoints (`make <tool>` / generated CLIs), enforce:

1. **Compile + tests green => high confidence command success** for normal local prerequisites.
2. **If not satisfiable**, fail during compile/test with targeted diagnostics, not at runtime deep in execution.

---

## Proposed Repo-Level Fixes

### P0: Add command-contract tests for generated CLIs

Add tests that execute generated binaries with:

1. the same invocation shape users actually run
2. real-mode graph build + lowered-node contract checks (without external network side effects)

Focus is parity with actual user command invocation, not just library-level graph construction.

### P0: Keep concrete credential/runtime modeling in `.dag`, not Rust residue

Clean out repo-specific Rust-side `gcloud` acquisition/install residue for this path.

If a concrete `gcloud`-based local-dev path still exists, it should be modeled in `.dag` as part of the domain/runtime model, not as ad hoc Rust-side tool acquisition policy.

### P1: Remove profile as a user-facing/runtime binding concept

Move from:

1. workflow asks for abstract credential
2. user/tool passes `--profile`
3. compiler/runtime selects concrete implementation

To:

1. workflow asks for abstract credential
2. domain model resolves concrete provider from modeled caller/runtime context
3. command execution uses that realization directly

### P1: Eliminate workflow-local credential fallback trees

The desired end state is not "better fallback logic." The desired end state is "no fallback logic in the workflow."

If local development uses `gcloud`, that should emerge from the modeled local-dev credential path, not from helper-level branches like:

1. try `GITHUB_TOKEN`
2. otherwise call `gcloud`

### P1: Strengthen diagnostics while profile machinery still exists

Until profile surfaces are removed, stub-execution errors should at least print:

1. selected profile
2. discovered bindings
3. interface lookup key
4. why resolution still produced a stub

### P1: Unignore or replace currently ignored gist e2e tests

Ignored tests hide active regressions in the exact tool where confidence is required.
Convert to deterministic contract tests where possible (transport backend injection).

### P1: CI stability cleanup of known stale test paths

Fix known stale references like `dsl/pipelines/ci.dag` vs `dsl/workflows/ci.dag` so global test status is trustworthy and does not mask signal quality issues.

---

## Immediate Action Plan

1. Document clearly that the old failure was caused by profile-coupled credential realization.
2. Record that the current gist fix is tactical, not the long-term credential architecture.
3. Remove Rust-side `gcloud` residue that should live in `.dag`.
4. Follow up by removing remaining profile plumbing from tool/compiler/runtime surfaces.
5. Fix stale CI pipeline test path references so command reliability signals are trustworthy.

---

## Acceptance Criteria for Closure

This incident is closed only when:

1. `make gist` path is covered by automated tests that mirror real user invocation.
2. A regression in credential realization fails tests before user-facing runtime.
3. Credential/runtime realization is modeled in `.dag`, not split between `.dag`, profile flags, and Rust residue.
4. No ignored tests remain for core gist command contracts.
5. Profile is no longer required to make user-facing tool paths work.

---

## Meta: Confidence Contract

The repo should treat this as a product-level quality contract:

> A user running a first-class command (`make gist`, `make ci`, etc.) should not discover a credential-realization failure that our compile/test gates could have detected.

This is not only a gist bug; it is a command-reliability systems issue.
