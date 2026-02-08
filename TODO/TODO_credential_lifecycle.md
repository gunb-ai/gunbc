# Credential/Auth Convergence Tracker

Status date: 2026-02-08
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
- [ ] Review graph emits explicit required scopes in its auth contract node.
- [ ] LLM graph emits explicit required scopes in its auth contract node.
- [ ] Transport shell auth contract is standardized (explicit credential materialization semantics for shell flows).
- [ ] CI guardrails prevent reintroduction of env-var credential nodes in tool graphs.

## Strict Invariants (Must Hold)

- [ ] No production tool graph may acquire credentials directly from env-var-only auth nodes.
- [ ] Every credentialed action/request type exposes a scope contract.
- [ ] Every `execute` node that requires auth must consume `res:credential`.
- [ ] Missing or invalid scope contract fails deterministically before transport I/O.

## Migration Checklist

### A. Interface Contracts

- [x] Add shared contract types in `core/ir/src/transport/scope.rs`.
- [x] Add gist scope contract in `core/ir/src/transport/gist.rs`.
- [ ] Add review scope contract type(s).
- [ ] Add llm provider scope contract type(s).
- [ ] Add compile-time tests for each contract type.

### B. Graph Wiring

- [x] Migrate `lib/tools/gist/src/graph.rs` to cloud credential chain.
- [ ] Migrate any remaining auth-required tool graphs that still use direct env var providers.
- [ ] Ensure auth contract nodes emit `service/scheme/header_name/required_scopes`.
- [ ] Ensure required scopes are visible in mock specs for every auth flow.

### C. Tests and Mocks

- [x] Update gist mock spec/integration mocks for cloud lifecycle nodes.
- [ ] Regenerate testgen artifacts after auth graph migrations.
- [ ] Add regression tests that assert no `credential_env` nodes in production tool graphs.
- [ ] Add tests that fail when contract scope list is empty.

### D. Enforcement

- [ ] Add CI audit step: fail if auth-required graphs skip canonical credential chain.
- [ ] Add CI audit step: fail if new interface uses credentials without `ScopeContract`.
- [ ] Add runtime preflight error contract for missing scope declarations.

## Acceptance Criteria

- [ ] `make gist-recent` no longer depends on `GITHUB_TOKEN` env wiring in graph modeling.
- [ ] Auth modeling for gist/review/llm follows one canonical chain shape.
- [ ] Auth contract declarations are typed and colocated with action/request interfaces.
- [ ] No fallback-only auth paths remain undocumented.
- [ ] CI has at least one hard guard preventing credential modeling drift.
