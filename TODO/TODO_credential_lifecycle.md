# Credential/Auth Convergence Tracker

Status date: 2026-02-09
Owner: runtime/auth modeling

## Goal

Make credentialed actions uniform across the codebase:

1. Every credentialed interface declares required scopes.
2. Runtime derives credential acquisition from that contract.
3. Graph shape is consistent across local/dev/test/prod.
4. Missing contract is a hard failure, not a fallback path.

## Canonical Pattern

For any credentialed workflow:

`resolve_auth_contract -> cloud_env -> bind_secret -> cloud_credential -> execute(res:credential)`

Environment can change provider implementation, but the graph pattern must stay the same.

## Current Convergence State

- [x] Shared scope-contract primitives exist in `core/ir` (`ScopeContract`, `CredentialIntent`).
- [x] Gist interface declares a typed scope contract (`GistScopeContract`) and intent.
- [x] Gist graph uses cloud credential lifecycle chain.
- [x] Review graph declares typed scope contract (`ReviewScopeContract`) wrapping LLM contract.
- [x] LLM graph declares typed scope contract (`LlmScopeContract`) with provider-specific auth.
- [ ] Transport shell auth contract is standardized (explicit credential materialization semantics for shell flows).
- [x] CI guardrails prevent reintroduction of env-var credential nodes in tool graphs.

## Strict Invariants (Must Hold)

- [x] No production tool graph may acquire credentials directly from env-var-only auth nodes.
- [x] Every credentialed action/request type exposes a scope contract.
- [x] Every `execute` node that requires auth must consume `res:credential`.
- [ ] Missing or invalid scope contract fails deterministically before transport I/O.

## Migration Checklist

### A. Interface Contracts

- [x] Add shared contract types in `core/ir/src/transport/scope.rs`.
- [x] Add gist scope contract in `core/ir/src/transport/gist.rs`.
- [x] Add review scope contract in `core/ir/src/transport/review.rs`.
- [x] Add LLM provider scope contract in `core/ir/src/transport/llm/mod.rs`.
- [x] Add compile-time tests for each contract type (unit tests in each module + `credential_chain.rs` regression tests).

### B. Graph Wiring

- [x] Migrate `lib/tools/gist/src/graph.rs` to cloud credential chain.
- [x] Migrate all auth-required tool graphs — all use canonical chain (gist, review, llm, github-credential).
- [x] Auth contract nodes emit `service/scheme/header_name/required_scopes` in all credentialed graphs.
- [x] Required scopes are visible in mock specs for every auth flow.
- [x] `build_cloud_credential_graph_for_runtime()` dispatches on both provider AND runtime (not just runtime).
- [x] Local dev auth refactored into `local_auth_upsert` sub-DAG with formal guard on create phase.

### C. Tests and Mocks

- [x] Update gist mock spec/integration mocks for cloud lifecycle nodes.
- [x] Regenerate testgen artifacts after auth graph migrations.
- [x] Add regression tests that assert no `credential_env` nodes in production tool graphs (`credential_chain.rs`).
- [x] Add tests that fail when contract scope list is empty (`credential_chain.rs`).
- [x] Update `gcp_local_mock_spec()` for `local_auth_upsert` sub-DAG boundaries.
- [x] Fix credential_lifecycle mock config to use `LocalDev` runtime for DryRun mode.
- [x] Fix `should_panic` expected message for `test_transport_mock_coverage_required`.

### D. Enforcement

- [x] Add CI audit step: fail if auth-required graphs skip canonical credential chain (`credential_chain.rs` regression tests run in `cargo test --workspace`).
- [x] Add CI audit step: fail if new interface uses credentials without `ScopeContract` (scope contract validation tests in `credential_chain.rs`).
- [ ] Add runtime preflight error contract for missing scope declarations.

### E. Cloud Env Layering

- [x] Model cloud selectors layer (`CLOUD_PROVIDER`, `CLOUD_RUNTIME`) explicitly.
- [x] Model provider/runtime required vars via `CloudEnvRequirements`.
- [x] Model developer identity as required-any-of groups (for local GCP: `GCP_SECRETS_SA | GCP_SECRETS_IMPERSONATE_SA`).
- [x] Make `cloud_env` fail with aggregated missing requirements (not first-missing only).
- [x] Include first-class diagnosis path in errors (`make gist-auth-doctor RUNTIME=...`).
- [x] Add a provider-neutral `cloud-auth-doctor` tool (`lib/cloud-ops/src/auth_doctor.rs`); gist doctor delegates to it.
- [ ] Define/implement local developer profile source of truth and precedence (shell env vs managed profile).

## Acceptance Criteria

- [x] `make gist-recent` no longer depends on `GITHUB_TOKEN` env wiring in graph modeling.
- [x] Auth modeling for gist/review/llm follows one canonical chain shape.
- [x] Auth contract declarations are typed and colocated with action/request interfaces.
- [ ] No fallback-only auth paths remain undocumented.
- [x] CI has at least one hard guard preventing credential modeling drift.

## Remaining Open Items

- [ ] Transport shell auth contract standardization (explicit credential materialization semantics for shell flows).
- [ ] Runtime preflight error contract for missing scope declarations.
- [ ] Local developer profile source of truth and precedence (shell env vs managed profile).
- [ ] Document remaining fallback-only auth paths (if any exist).
