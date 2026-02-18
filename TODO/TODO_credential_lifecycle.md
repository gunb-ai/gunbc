# Credential Lifecycle Architecture (Reset)

**Status**: Draft (architecture/spec; migration not complete)
Status date: 2026-02-12  
Owner: runtime/auth modeling  
Supersedes: checklist-style convergence tracker version
**DSL Alignment**: Credential/service lifecycle modeling for future DSL service consumers
**Track**: E — Domain Parity

## Why This Rewrite Exists

The prior document tracked migration tasks, but it mixed three different concerns:

1. What is implemented now.
2. What abstraction boundaries we want.
3. What debt remains tactical vs architectural.

This rewrite resets around one rule:

`credentialed workflows request capability intent, not provider mechanics`

## Problem Statement

Current flows can work when environment assumptions happen to match local setup, but abstraction leakage makes auth brittle:

- defaults and provider details leak into graph construction callsites,
- impersonation behavior is not selected through an explicit strategy layer,
- profile and secret source-of-truth are fragmented,
- lifecycle actions (acquire/create/rotate/verify) are not represented as one policy-driven pipeline.

## Current State (Verified)

### Solid and worth keeping

- Canonical chain shape is enforced by tests:
  `resolve_auth -> cloud_env -> bind_secret -> cloud_credential -> execute(res:credential)`  
  (`gunbc-dag/tests/credential_chain.rs`)
- Typed contract primitives already exist: `ScopeContract`, `CredentialIntent`  
  (`core/ir/src/transport/scope.rs`)
- Credentialed interfaces are already contract-based (gist/review/llm transport modules).
- Discovery DAG exists for generating cloud config from live GCP state:  
  `lib/gcp-ops/src/discovery_graph.rs`, `lib/gcp-ops/src/discovery_ops.rs`

### Half-implemented or mismatched abstractions

- `CloudConfigResource::create()` does not yet execute discovery; it currently records manifest key/outputs only.  
  (`lib/cloud-ops/src/config_resource.rs`)
- Context/profile resolution now centralizes in `graph_cloud_config()` with deterministic source precedence, but compatibility fallback defaults still exist when no source is configured and strict mode is off.
- Profile/config precedence is specified for JSON/TOML/env sources, but repo-file loading and policy-file binding are still pending.

### E1.0 baseline diagnostics (completed)

- `make gist-recent` was executed with strict diagnostic settings (`RUSTUP_TOOLCHAIN=nightly`, `GUNBC_FRESHNESS_ACTIVE=1`) and traced through the credential chain.
- Baseline execution path and failure point were documented in `docs/design/v4/gist-recent-credential-diagnostics.md`.
- Hidden defaults identified and recorded:
  - implicit `default_local_dev_config()` fallback when config is not required,
  - implicit ADC file path fallback (`GOOGLE_APPLICATION_CREDENTIALS` → `$HOME/.config/gcloud/...` → `/root/...`),
  - legacy audience default to `"local-dev"` when `GCP_WIF_PROVIDER` is absent.

## Target Architecture

Credential lifecycle is modeled as five strict layers.

1. Intent layer (capability request)
- Input: `CredentialIntent` from interface contract.
- Output: normalized intent id + required scopes.
- Rule: callers say what they need, never how to fetch it.

2. Context layer (environment understanding)
- Input: runtime signals + profile selection.
- Output: `CredentialContext` (runtime kind, namespace, actor identity, candidate providers).
- Rule: env detection happens once here; downstream layers do not inspect env directly.

3. Policy layer (spec-driven decision)
- Input: intent + context.
- Output: `CredentialPolicy` (secret refs, scopes, impersonation rule, rotation policy, status).
- Rule: all intent-to-secret mapping lives in policy spec, not callsites.

4. Provider strategy layer (concrete auth plan)
- Input: policy + context.
- Output: provider-specific execution plan (`gcp_wif_secret`, `adc_direct`, etc.).
- Rule: branching decisions such as impersonation happen here.

5. Execution layer (DAG apply)
- Input: provider plan.
- Output: credential lease + expiry + structured diagnostics.
- Rule: lifecycle actions (acquire/create/rotate/verify) run as explicit DAG steps.

## Extraction from `the-gunbai`

The most reusable parts from `the-gunbai` are architecture patterns, not code copy:

1. Credential flow algebra (typed auth strategy)
- `CredentialFlow` is an explicit enum of acquisition strategies:
  `Stored`, `PlatformInjected`, `WorkloadIdentity`, `InteractiveAuth`, `Derived`, `Chained`.  
  (`../the-gunbai/crates/gunbai-types/src/credential.rs`)
- This cleanly separates "what flow class is this?" from provider details.

2. Strict runtime/provisioning split
- Runtime secret reads are read-only; provisioning/upsert uses separate identity and command path.  
  (`../the-gunbai/docs/design/ci-secrets-minimal-invariants.md`)
- No implicit runtime upsert fallback.

3. Policy-driven auth config
- `secret-sources.toml` binds provider config + auth flow + per-secret mapping in one declarative model.  
  (`../the-gunbai/config/secret-sources.toml`)
- CI code resolves that model into concrete GCP OIDC -> STS -> optional impersonation execution.  
  (`../the-gunbai/crates/gunbai-ci/src/secrets.rs`)

4. First-class behavior patterns
- Authentication should be modeled as a reusable pattern (like `pattern/upsert`), not duplicated workflow glue.  
  (`../the-gunbai/docs/design/behavior-patterns.md`)

5. Requirements as generic I/O contracts
- Secrets are treated as requirements within a generic prerequisite model (not a bespoke side subsystem).  
  (`../the-gunbai/docs/design/requirements-and-io.md`)

6. Scope rigor by behavior
- Secret requirements merge scopes across understandings and support per-behavior scope lookup.  
  (`../the-gunbai/crates/gunbai-integrations-contracts/src/secret_validation.rs`)

## Base `authenticate` Pattern (New Requirement for Gunbc)

To make auth robust across the full codebase, define one foundational pattern:

`pattern/authenticate`

Every credentialed flow (gist/review/llm/cloud ops/future providers) must consume this pattern, not invent local auth logic.

### Pattern contract

Input:
- `CredentialIntent` (capability + scopes + scheme)
- `AuthContext` (runtime/profile/provider candidates)
- `AuthPolicy` (flow constraints and lifecycle policy)

Output:
- `CredentialLease` (materialized credential + expiry + source metadata)
- `AuthDiagnostics` (structured path, phase failures, remediations)

### Required phases

1. `ResolveContext`
- detect runtime and profile using deterministic precedence.

2. `SelectFlow`
- choose typed flow class:
  `PlatformInjected | WorkloadIdentity | InteractiveAuth | Stored | Derived | Chained`.

3. `AcquireBaseIdentity`
- gather initial token/credential seed for selected flow.

4. `ExchangeOrDerive`
- run token exchange or derivation (STS, OAuth refresh, app token, etc.).

5. `Impersonate` (conditional)
- executed only when strategy/policy requires it.

6. `VerifyScopes`
- preflight required scopes/capabilities before downstream execute transport.

7. `MaterializeLease`
- return uniform credential object with expiry and provenance.

### Pattern invariants

- No provider-specific branching in tool graphs.
- No implicit fallback auth path.
- Runtime auth read path is non-mutating.
- Provisioning/rotation paths are explicit, separate operations.
- Missing required scopes fail before external business transport I/O.

## Spec Model (Gunb.ai Pattern, Gunbc Adaptation)

Use two specs with a clean boundary:

1. Discovered config (generated)
- Source: infra discovery DAG.
- Purpose: describe cloud topology/resources.
- Artifact: `CloudConfigSpec` (TOML/JSON, generated).

2. Credential policy (authored)
- Source: repo-authored policy file(s).
- Purpose: describe auth behavior per intent and context.
- Fields include:
  - required scopes,
  - secret binding,
  - provider preference,
  - impersonation policy,
  - rotation handler/max age,
  - status (`active`/`deleted`) for reconcile/prune.

This preserves gunbc’s DAG runtime while adopting spec-driven apply principles proven in `gunb.ai`.

## Canonical Runtime Contract

Graph shape stays canonical, but semantics tighten:

`resolve_auth_contract -> resolve_context -> bind_policy -> cloud_credential(provider_plan) -> execute(res:credential)`

Mapping from current node names:

- `resolve_auth`: stays intent-focused.
- `cloud_env`: evolves into context/profile resolution.
- `bind_secret`: evolves into policy binding (not only naming composition).
- `cloud_credential`: executes strategy-selected provider plan.

`cloud_credential` becomes the implementation of `pattern/authenticate` inside the canonical chain.

## Reconciliation with Existing Gunbc DAGs

This section maps the target architecture to the DAGs already in the repo so migration is incremental, not a rewrite.

### Existing flow inventory (today)

- Tool entry DAGs:
  - `lib/tools/gist/src/graph.rs` (`build_gist_graph_with_config`)
  - `lib/llm-ops/src/graph.rs` (`build_chat_completion_graph_with_config`)
  - `lib/review/src/graph.rs` (`build_review_phase_graph_with_config`)
  - `lib/cloud-ops/src/github_credential_graph.rs` (`build_github_credential_graph`)
- Provider-neutral cloud DAG:
  - `lib/cloud-ops/src/graph.rs` (`build_cloud_secret_manager_credential_graph_from_config`)
- GCP credential DAG:
  - `lib/gcp-ops/src/graph.rs` (`build_gcp_secret_manager_credential_graph`)
- GCP upsert DAG:
  - `lib/gcp-ops/src/graph.rs` (`build_gcp_secret_manager_upsert_graph`)
- Local auth sub-DAG (shared):
  - `lib/gcp-ops/src/graph.rs` (`build_local_auth_upsert_dag`)

### Phase mapping (`pattern/authenticate` -> current nodes)

1. `ResolveContext`
- Current implementation:
  - Tool graphs use `cloud_env` via `CloudOps::ConstCloudConfig`.
  - Cloud DAG normalizes config via `resolve_config` + `map_gcp_inputs`.
- Coverage: medium.
- Update (2026-02-18):
  - `ResolveContext` now implements deterministic precedence:
    `explicit > env > profile file > default`.
  - Added file-backed profile loading from `.gunbc/config-<profile>.toml`
    (resolved when `GUNBC_CLOUD_PROFILE` / `GUNBC_CLOUD_NAMESPACE` is set).
  - Added precedence regression tests in `config_loader`.
- Update (2026-02-18):
  - Added policy loader + binding helper:
    `lib/cloud-ops/src/credential_policy.rs`
    (`bind_credential_intent_policy`).
  - Supports env-backed policy sources:
    - `GUNBC_CREDENTIAL_POLICY_JSON`
    - `GUNBC_CREDENTIAL_POLICY_PATH`
    - `GUNBC_CREDENTIAL_POLICY_PROFILE`
  - Gist + LLM resolve-auth paths now apply policy-bound secret/scopes.

2. `SelectFlow`
- Current implementation:
  - Provider/runtime dispatch in `build_cloud_secret_manager_credential_graph_from_config`.
  - Runtime guard in `CloudOps::MapToGcpInputs`.
- Coverage: partial.
- Gap: selection is runtime/provider based, not policy + typed flow class.

3. `AcquireBaseIdentity`
- Current implementation:
  - GitHub runtime: `prepare_github_oidc -> execute -> parse_github_oidc`.
  - Metadata runtime: `prepare_metadata_oidc -> execute -> parse_metadata_oidc`.
  - Local runtime: `local_auth_upsert` sub-DAG (`check + ADC/OAuth refresh` path).
- Coverage: strong.

4. `ExchangeOrDerive`
- Current implementation:
  - `prepare_sts -> execute_sts -> parse_sts`.
- Coverage: strong.

5. `Impersonate` (conditional)
- Current implementation:
  - `prepare_impersonate -> execute_impersonate -> parse_impersonate`.
- Coverage: medium.
- Update (2026-02-18):
  - `ShouldImpersonate` is now explicitly policy-gated via
    `allow_impersonation` (derived from credential policy `impersonation`
    mode and threaded through gist/llm/cloud/gcp flows).
- Remaining gap:
  - policy-driven service-account selection + provider-side scope checks
    still pending.

6. `VerifyScopes`
- Current implementation:
  - Contract typing and validation exists in `core/ir`.
  - Gist/LLM/review/GitHub resolve auth nodes emit `required_scopes`.
  - `required_scopes` now threads through `cloud_credential` inputs across credentialed tool flows.
- Dedicated `scope_preflight` nodes now gate business transport execute paths.
- Coverage: medium.
- Update (2026-02-18):
  - Scope preflight now fails fast when scope declarations are missing or empty
    (before outbound business transport calls).
- Remaining gap:
  - Scope preflight still does not verify provider-granted effective scope sets.

7. `MaterializeLease`
- Current implementation:
  - `build_credential` constructs `Credential`.
  - `expires_in` is surfaced by credential sub-DAG.
- Coverage: strong for materialization, partial for structured provenance diagnostics.

### Reconciliation by workflow

- Gist flow (`lib/tools/gist/src/graph.rs`):
  - Best aligned today.
  - Has `required_scopes`, `lifetime_seconds`, and `expires_in` wiring.
  - Has explicit `scope_preflight` before gist execute.
  - Gap: preflight validates declared scope IDs, not provider-granted effective scopes.

- LLM flow (`lib/llm-ops/src/graph.rs` + `lib/llm-ops/src/lib.rs`):
  - Canonical chain shape is correct.
  - `LlmOps::ResolveAuth` emits `required_scopes` and passes them through `cloud_credential`.
  - Has explicit `scope_preflight` before execute.
  - Gap: preflight validates declared scope IDs, not provider-granted effective scopes.

- Review flow (`lib/review/src/graph.rs`):
  - Canonical chain shape is correct.
  - Uses `ReviewOps::ResolveAuthContract` (backed by `ReviewScopeContract`) and carries review scopes.
  - Has explicit `scope_preflight` before execute.
  - Gap: preflight validates declared scope IDs, not provider-granted effective scopes.

- GitHub credential validation flow (`lib/cloud-ops/src/github_credential_graph.rs`):
  - Canonical chain shape is correct.
  - `resolve_auth` now emits and threads `required_scopes`.
  - Has explicit `scope_preflight` before execute.
  - Gap: validation endpoint remains auth-presence focused only.

- Cloud config resource path (`lib/cloud-ops/src/config_resource.rs`):
  - Modeled as managed resource.
  - Gap: `create()` currently does not invoke discovery DAG, so config lifecycle orchestration is incomplete.

### Behavior-pattern alignment already present

- Upsert pattern exists concretely for secret provisioning:
  - `prepare_secret_get -> parse_secret_get -> prepare_secret_create -> prepare_secret_add_version`.
- Local auth sub-DAG now fails fast with explicit `gcloud auth application-default login` remediation when ADC is missing (no late impersonation 401 fallback path).
- Canonical tool-chain invariant is already enforced in tests:
  - `gunbc-dag/tests/credential_chain.rs`.

### Reconciliation decisions (authoritative)

- Keep the external canonical chain node names in tool graphs.
- Implement `pattern/authenticate` inside `cloud_credential` internals first.
- Add `required_scopes` as required resolve_auth output in all credentialed flows.
- Add explicit `scope_preflight` node in credential chain before business `execute`.
- Wire conditional impersonation (`ShouldImpersonate`) in GCP credential and upsert DAGs.
- (Done) Replace direct `default_local_dev_config()` callsites with centralized context/profile resolver.

## Keep / Refactor / Replace

Keep:

- existing `CredentialIntent` / `ScopeContract` types,
- canonical chain tests and guardrails,
- reusable GCP local auth and discovery ops.

Refactor:

- config/profile precedence and loading,
- `cloud_credential` internals to strategy + diagnostics,
- `bind_secret` semantics from “string binding” to policy binding.

Replace/Add:

- credential-policy spec + resolver,
- `pattern/authenticate` core module and tests,
- provider strategy interface,
- secret reconcile and rotation DAGs,
- explicit runtime preflight for missing/invalid required scopes.

## Proposed Interfaces (Rust Sketch)

```rust
trait CredentialResolver {
    fn resolve(&self, req: CredentialRequest) -> Result<CredentialLease, CredentialError>;
}

struct CredentialRequest {
    intent: CredentialIntent,
    interactive_allowed: bool,
    ttl_seconds: Option<i64>,
}

trait ContextResolver {
    fn resolve(&self) -> Result<CredentialContext, CredentialError>;
}

trait PolicyResolver {
    fn resolve(
        &self,
        intent: &CredentialIntent,
        context: &CredentialContext,
    ) -> Result<CredentialPolicy, CredentialError>;
}

trait ProviderStrategy {
    fn plan(
        &self,
        policy: &CredentialPolicy,
        context: &CredentialContext,
    ) -> Result<CredentialPlan, CredentialError>;
}
```

## Migration Plan

Phase 0: Baseline diagnostics

- Improve impersonation failures to include status and parsed error summary.
- Keep current graph shape unchanged while improving observability.

Phase 1: Context/profile precedence

- Implement deterministic precedence:
  1) explicit `--cloud-config` path,
  2) repo config (e.g. `.gunbc/config-<env>.toml`),
  3) env overrides,
  4) fallback defaults (dev-only with explicit warning).
- Remove hidden hardcoded config from graph constructors.

Phase 1.5: Introduce `pattern/authenticate`

- Add a provider-neutral authenticate contract module in `core/ir`.
- Rewire existing `cloud_credential` internals to call pattern phases.
- Keep external graph shape unchanged while internals migrate.

Update (2026-02-18):
- Added `core/ir/src/patterns/authenticate.rs` with canonical phase model:
  - `AuthenticatePhase`
  - `canonical_authenticate_chain(...)`
  - `validate_authenticate_chain(...)`
- Gist and LLM credential flows now consume the pattern contract via canonical
  binding validation:
  - `lib/gist-ops::build_gist_upload_subdag(...)`
  - `lib/llm-ops::build_chat_completion_graph_with_config(...)`
  both validate phase bindings with `validate_authenticate_bindings(...)`.

Phase 2: Policy binding

- ✅ Introduced `credential-policy` schema + loader/binder path.
- ✅ Gist/LLM intent->secret/scopes mapping now supports policy overrides.
- ⏳ Remaining: expand `bind_secret` semantics beyond current
  resolve-auth policy application (next lifecycle phases).

Phase 3: Strategy execution

- Add provider strategy selection inside `cloud_credential`.
- Wire conditional impersonation.
- Support local flows that do not require impersonation when policy allows.

Phase 4: Secret lifecycle apply loops

- Add reconcile DAG: check/create/bind/upsert.
- Add rotation DAG: age check -> rotate handler -> version add -> verify.
- Add prune path for policy entries marked `deleted`.

Update (2026-02-18):
- Added foundational rotation primitives in `lib/cloud-ops/src/secret_rotation.rs`:
  - `rotate_secret(...)` handlers (`Manual`, `GitHubPat`, `None` + SA key fallback)
  - `check_secret_age(...)` with `max_age_days` evaluation.
- `SecretSpec` now carries `max_age_days` for rotation recommendations.
- Added `build_secrets_provision_dag(...)` in
  `lib/cloud-ops/src/secret_provision_graph.rs` to provision all active
  secrets from spec via composed upsert sub-DAGs.
- Added secret fetch/export integration primitives:
  - `render_direnv_exports(...)` (`secret_exports`)
  - `SecretValueCache` + `plan_secret_fetch(...)` (`secret_cache`)
  for TTL-aware cache reuse and partial fetch planning.

Phase 5: Hardening and cutover

- Fail preflight before transport I/O if required scopes are absent/invalid.
- Add regression tests for precedence and policy resolution.
- Deprecate and then remove fallback-only behavior behind explicit compatibility gate.

## Definition of Done

- Credential behavior can be explained for every workflow as:  
  `intent -> context -> policy -> strategy -> execute`
- `make gist-recent` no longer depends on hidden hardcoded project/service-account defaults.
- Impersonation is conditional and diagnosable.
- Rotation is policy-driven, not per-workflow custom logic.
- Missing scope declarations fail before outbound network calls.
- Generated discovery config and authored credential policy are both first-class inputs.

## Open Decisions

- policy layout: single root file vs domain-split includes,
- namespace model: `local/dev/prod` flat vs inherited hierarchy,
- fallback compatibility window duration for existing env-driven setups.
