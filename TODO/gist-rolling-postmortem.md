# Rolling Postmortem: Gist Command Reliability

> Canonical tracker for `make gist` reliability incidents.
> Goal: if compile + tests are green, command execution should have high success probability.

## Reliability Contract

For first-class commands (`make gist`, `make ci`, etc.):

1. Regressions should fail in compile/test stages, not first appear in Real mode runtime.
2. DryRun coverage must not mask Real mode profile/binding failures.
3. Profile-bound interface calls must be concretely resolved (or fail with precise diagnostics) before user execution.

## Incident Timeline

### Incident A (2026-02-27): 401 and silent credential loss

- Canonical doc: `TODO/gist-auth-postmortem.md`
- Symptom: gist request went unauthenticated; GitHub returned 401.
- Core issue: DSL auth intent existed, but transport credential wiring and shell error semantics were not enforced end-to-end.
- Status: partially resolved at infrastructure level; broader modeling/testgen hardening still open.

### Incident B (2026-03-05): Real mode profile-stub failure

- Canonical doc: `TODO/gist-real-mode-confidence-postmortem.md`
- Symptom: `make gist` failed in Real mode with:
  `interface stub CredentialProvider.acquire requires --profile: no active profile bindings`
- Core issue: command path depended on interface-stub execution semantics; DryRun and existing tests did not mirror Real mode profile resolution strongly enough.
- Status: acute path fixed in this worktree (details below); repo-level confidence gates still open.

### Incident C (2026-03-05): `test-all` perceived as hung/failing due one exhaustive test

- Symptom: `make test-all` appeared stuck for a long period with no output.
- Root cause: `gunbc_dag::testgen_dag::dag_test_discovery::comprehensive_auto_testgen_pipeline_validation` runs full auto-testgen compile/generation across 79 modules and took about 1398s in debug test mode.
- Resolution: marked this test `#[ignore]` by default as slow integration validation; it remains runnable explicitly when needed.

## Relationship Between Incidents

These incidents are related. Both are command-confidence failures caused by a gap between DSL intent and enforced end-to-end runtime invariants:

1. Declared auth/profile intent existed.
2. Compile/test signals stayed green.
3. Real mode execution exposed unresolved wiring/coverage gaps.

## Acute Remediation Applied (2026-03-05)

### Runtime path fix

- Updated `dsl/tools/gist.dag` to resolve the GitHub token via concrete service calls (`shell.Env.Get` with fallback path) instead of relying on `CredentialProvider` interface acquisition in the gist command path.
- This removes the immediate interface-stub/profile failure mode for `make gist`.

### Test fixes and coverage alignment

- Updated gist regression assertions in `gunbc-dag/tests/gist_recent_regressions.rs` to validate token-resolution nodes now used by gist.
- Fixed stale CI DAG path assumptions in `core/daglang/daglang-cli/tests/compile_commands.rs` (`dsl/pipelines/ci.dag` -> `dsl/workflows/ci.dag`) and aligned assertions with current output shape.

## Current Verification State

Green in this worktree:

1. `make gist`
2. `cargo test -p daglang-cli --test compile_commands`
3. `cargo test -p gunbc-dag gist`
4. `cargo test -p gunbc-dag --test shell_exit_enforcement_proof`
5. `make test-all`

Slow exhaustive validation remains available explicitly:

1. `cargo test -p gunbc-dag comprehensive_auto_testgen_pipeline_validation -- --ignored --nocapture --test-threads=1`

## Open Repo-Level Actions

P0:

1. Add profile-realization contract tests for profile-bound tools (start with gist): fail if Real mode path still contains unresolved interface-stub execute nodes.
2. Add command-contract tests that mirror generated CLI invocation (`--profile`, mode, graph contract checks) with deterministic mocked backends.
3. Improve interface-stub runtime diagnostics to print selected profile, discovered bindings, and lookup key context.

P1:

1. Unignore or replace gist e2e tests with deterministic equivalents.
2. Expand testgen obligations to include REST status/error cases and shell non-zero exit behavior.
3. Track and remove stale path/test assumptions that degrade trust in global green status.

## Closure Criteria

This rolling postmortem is closed when:

1. `make gist` and other profile-bound commands have pre-runtime contract tests that fail on unresolved bindings.
2. DryRun/Real mode confidence boundaries are explicit and tested.
3. Full repo gate (`make test-all`) is green with these protections in place.
