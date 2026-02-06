# Credential Lifecycle Management

**Status**: Design
**Date**: 2026-02-04
**Depends on**: TODO_consolidate_di.md Item 3

## Ownership
- [x] Taken by Codex (2026-02-05)

Auth should be invisible to business logic. Processes either receive a
resolved token or the result of an authenticated call. All credentials
should be short-lived with a managed lifecycle: acquire → use → expire.
No manual rotation.

---

## Problem Statement

Three independent auth models existed historically (now consolidated):

| Type | Location | What it models |
|------|----------|----------------|
| `Credential` | `core/ir/src/transport/credential.rs` | How to attach auth to HTTP |
| `GitHubEnvVarProvider` | `lib/transport/src/credential.rs` | Where GitHub token comes from |
| `AuthScheme` + `api_key_env` | `core/ir/src/transport/llm/provider.rs` | LLM provider auth scheme + env var name |

These conflate three concerns:
1. **Source** — where the secret comes from (env var, OAuth exchange, vault)
2. **Value** — the opaque token string and its lifecycle
3. **Scheme** — how to attach it to a request (Bearer, custom header, Basic)

Previously, env-var auth (e.g., `OPENAI_API_KEY`) encoded all three in one variant.
There's no lifecycle, no expiry, no uniform way to test or mock auth.

---

## Design

### Layer 0: Secret

The value itself — an opaque string with lifecycle metadata.

```rust
/// An opaque secret value with lifecycle tracking.
///
/// Debug and Display redact the value (prints `***`).
/// Clone is intentionally derived — secrets flow through DAG edges.
struct Secret {
    /// The actual secret value. Never logged.
    value: String,
    /// When this secret expires. None = valid for the session.
    expires_at: Option<SystemTime>,
    /// Where this secret came from (for audit/debug, not logic).
    source: SecretSource,
}

/// How a secret was obtained. For logging/audit only.
enum SecretSource {
    /// Read from an environment variable.
    EnvVar(String),
    /// Exchanged via a token grant (OAuth, GitHub App, etc.).
    Exchange { provider: String },
    /// Injected directly (tests, static config).
    Static,
}
```

`Secret` implements redacting `Debug`/`Display` (same pattern as
`Credential` today). `CiContext::mask()` can mask the value in CI output.

### Layer 1: AuthScheme

How to attach a secret to an HTTP request. This is a property of the
*API*, not the credential — Anthropic always uses `x-api-key`, OpenAI
always uses Bearer. This never changes per-token.

```rust
/// How a credential is attached to an HTTP request.
///
/// This is a property of the target API, not the credential itself.
enum AuthScheme {
    /// Authorization: Bearer <secret>
    Bearer,
    /// <header_name>: <secret>  (e.g., Anthropic's x-api-key)
    Header { name: String },
    /// Authorization: Basic base64(<username>:<secret>)
    Basic { username: String },
}
```

This replaces the "how to use it" aspect that was previously spread across
legacy auth variants and provider-specific auth handling.

### Layer 2: Credential

Secret + Scheme, ready to attach to a request. This is what flows
through the DAG and what `RestRequest` holds.

```rust
/// A resolved credential ready to attach to a request.
///
/// Contains a secret value and knows how to apply it to HTTP.
/// Flows through DAG edges as a Value.
struct Credential {
    secret: Secret,
    scheme: AuthScheme,
}

impl Credential {
    /// Apply this credential to an HTTP request.
    ///
    /// Sets the appropriate header based on the scheme.
    fn apply(&self, request: &mut HttpRequest) {
        match &self.scheme {
            AuthScheme::Bearer => {
                request.headers.insert(
                    "Authorization".into(),
                    format!("Bearer {}", self.secret.value),
                );
            }
            AuthScheme::Header { name } => {
                request.headers.insert(name.clone(), self.secret.value.clone());
            }
            AuthScheme::Basic { username } => {
                let encoded = base64(&format!("{}:{}", username, self.secret.value));
                request.headers.insert("Authorization".into(), format!("Basic {}", encoded));
            }
        }
    }

    /// Whether this credential is still valid.
    fn is_valid(&self) -> bool {
        self.secret.expires_at
            .map(|exp| SystemTime::now() < exp)
            .unwrap_or(true)
    }
}
```

### Layer 3: CredentialProvider

How credentials are acquired. One implementation per source.

```rust
/// Acquires credentials from a specific source.
///
/// Implementations handle the mechanics of obtaining a secret
/// (reading env, exchanging tokens, calling APIs). The caller
/// receives a ready-to-use Credential.
trait CredentialProvider: Send + Sync {
    /// Identifier for this provider (e.g., "github", "openai").
    fn service_id(&self) -> &str;

    /// Acquire a fresh credential.
    fn acquire(&self) -> Result<Credential, CredentialError>;
}
```

### Layer 4: Provider-specific auth (scopes, grants)

Each service has its own scope/permission model. This lives inside
the provider, not in the credential that flows through the DAG.
Business logic never sees this — it's an implementation detail of
how `acquire()` works.

#### GitHub

```rust
/// GitHub App credential provider.
///
/// Exchanges a GitHub App private key for a short-lived installation
/// token (~1hr). The system manages the lifecycle: acquire on DAG start,
/// the token expires naturally, no manual rotation needed.
struct GitHubAppProvider {
    app_id: u64,
    private_key: Secret,       // The App's RSA private key
    installation_id: u64,
    permissions: HashMap<String, GitHubPermission>,
}

enum GitHubPermission { Read, Write }

impl CredentialProvider for GitHubAppProvider {
    fn service_id(&self) -> &str { "github" }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        // 1. Generate JWT from app_id + private_key (10 min expiry)
        // 2. POST /app/installations/{id}/access_tokens with JWT
        // 3. Receive installation token (1hr expiry)
        // 4. Wrap in Credential { secret, scheme: Bearer }
        todo!()
    }
}

/// GitHub PAT/env var provider (bridge from current world).
struct GitHubEnvVarProvider {
    env_var: String,  // "GITHUB_TOKEN"
}

impl CredentialProvider for GitHubEnvVarProvider {
    fn service_id(&self) -> &str { "github" }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        let token = std::env::var(&self.env_var)
            .map_err(|_| CredentialError::missing_env(&self.env_var))?;

        Ok(Credential {
            secret: Secret {
                value: token,
                expires_at: None,  // PATs don't expire (this is the problem)
                source: SecretSource::EnvVar(self.env_var.clone()),
            },
            scheme: AuthScheme::Bearer,
        })
    }
}
```

#### OpenAI / Anthropic

```rust
/// LLM API key provider (env var based).
///
/// Reads the API key from an environment variable and wraps it
/// with the correct auth scheme for the provider.
struct LlmEnvVarProvider {
    service_id: String,       // "openai", "anthropic"
    env_var: String,          // "OPENAI_API_KEY", "ANTHROPIC_API_KEY"
    scheme: AuthScheme,       // Bearer (OpenAI), Header("x-api-key") (Anthropic)
}

impl CredentialProvider for LlmEnvVarProvider {
    fn service_id(&self) -> &str { &self.service_id }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        let key = std::env::var(&self.env_var)
            .map_err(|_| CredentialError::missing_env(&self.env_var))?;

        Ok(Credential {
            secret: Secret {
                value: key,
                expires_at: None,
                source: SecretSource::EnvVar(self.env_var.clone()),
            },
            scheme: self.scheme.clone(),
        })
    }
}
```

---

## DAG Integration

### CredentialOp (boundary node)

Same pattern as `EnvOp` for tools:

```rust
struct CredentialOp {
    providers: Vec<Box<dyn CredentialProvider>>,
}

impl Executable for CredentialOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut outputs = HashMap::new();
        for provider in &self.providers {
            let credential = provider.acquire()
                .map_err(|e| ExecError::new(format!(
                    "Failed to acquire credential for '{}': {}",
                    provider.service_id(), e
                )))?;

            // Mask the secret in CI output
            // (caller can pass CiContext to mask())

            let port = format!("credential:{}", provider.service_id());
            outputs.insert(port, credential.into());  // Value::Credential(...)
        }
        Ok(outputs)
    }
}
```

### Request flow after migration

```
CredentialOp (boundary)        Prepare (pure)              Execute (pure pipe)
───────────────────────        ──────────────              ───────────────────
providers: [                   receives Credential         receives RestRequest with
  GitHubAppProvider{...},      calls req.authenticate()    resolved auth
  LlmEnvVarProvider{...},     → sets concrete header      just sends it
]                              no env vars, no scheme
                               logic, no auth concern
emits:
  credential:github            no env-var bearer auth
  credential:openai            no env-var header auth
```

### Legacy Auth Mapping

```rust
// After full migration:
// RestRequest.auth becomes Option<Credential>
// Env-var auth variants are removed (resolved at boundary)
// Bearer/API key/Basic become AuthScheme variants
// No-auth becomes Option::None
```

### What gets replaced

| Current type | Replaced by | Notes |
|-------------|-------------|-------|
| Env-var bearer auth | `CredentialProvider::acquire()` at boundary | Env var read moves to provider |
| Env-var header auth | `CredentialProvider::acquire()` at boundary | Same |
| Bearer token | `Credential { scheme: Bearer, .. }` | |
| API key header | `Credential { scheme: Header{..}, .. }` | |
| Basic auth | `Credential { scheme: Basic{..}, .. }` | |
| No auth | `Option::<Credential>::None` | |
| `GitHubAuth` (removed) | `GitHubAppProvider` / `GitHubEnvVarProvider` | Enum → trait impls |
| `LlmAuthStyle` (removed) | `AuthScheme` | Unified |
| `api_key_env: String` | `LlmEnvVarProvider.env_var` | Field on provider |

---

## Testing Strategy

### What testgen generates today (no changes needed)

Once the credential DAG exists with `CredentialOp`, `Prepare*`, transport
`Execute`, and `Parse*` nodes, testgen automatically generates:

**Bucket A — Execution semantics:**
- DryRun completion: full credential DAG runs with mocked transports
- Transport interception: every transport executor node is interceptable
  (the `acquire_token` HTTP call, the `use_api` HTTP call, etc.)

**Bucket B — Contract obligations:**
- Edge predicate entailment across credential → prepare → execute edges
- Cardinality coverage for credential ports (One credential vs None)
- Node contract compliance for each pure node

**Bucket C — Scenario coverage (N+1):**
- All transports succeed (acquire + use both work)
- Single transport failure: acquire fails → verify error propagation
- Single transport failure: use fails (token rejected) → verify handling

**Bucket D — Resource hygiene:**
- `resource_lease("credential:github", 3600_000)` → generates
  `test_resource_credential_github_acquire` and
  `test_resource_credential_github_timeout`
- Resource conflict detection between parallel credential consumers

**Node I/O examples** (from MockSpec):
- Per-node unit tests for `AcquireCredential`, `ValidateToken`, etc.
- Each with `OutputMatcher` assertions (exact values, type checks, non-empty)

### What testgen CANNOT generate today — gaps

Three capabilities are missing for full lifecycle test generation:

| Gap | Description | Blocking testgen phase |
|-----|-------------|----------------------|
| **Stateful mocking** | Mocks are static per-node. Can't return token A on first call, token B after refresh. Need predicate-based or call-count-aware mock responses. | New capability — `ConditionalMock` or stateful `MockSequence` |
| **Multi-phase flows** | No way to test "run nodes 1-3, assert intermediate state, then run 4-5." The full acquire → use → refresh → use → destroy sequence is a choreography that DryRun runs in one shot with static mocks. | **Phase 5: Window-based testing** (`testgen-improvements.md`) |
| **Credential resource type** | Resource sim has Lock/Lease but not acquire/refresh/revoke semantics. Lease timeout tests TTL but can't test "refresh extends the lease" or "revoke invalidates." | Extension to Bucket D — add `ResourceType::Credential` with refresh/revoke behaviors |

**Dependency chain:**

```
Phase 5 (windowed testing)
    → enables multi-phase credential flow tests
    → testgen can enumerate windows: [acquire..use], [use..refresh], [refresh..revoke]
    → inject intermediate values at window boundaries
    → test each phase independently AND in sequence

Bucket D extension (credential resource type)
    → ResourceType::Credential { expiry_ms, refreshable }
    → ResourceBehavior::RefreshSucceeds { new_ttl_ms }
    → ResourceBehavior::RefreshFails { error }
    → ResourceBehavior::RevokeSucceeds
    → generates: acquire, use-while-valid, refresh, use-after-refresh,
                 expire, revoke lifecycle test sequence

Stateful mocking (new)
    → MockSequence: ordered list of responses per transport node
    → First call to acquire_token → returns token A
    → Second call (after refresh) → returns token B
    → OR: ConditionalMock with predicate on input values
```

### Manual baseline tests (written now, absorbed by testgen later)

These are the tests we write by hand for Phases 1-3. They become
absorption targets per `testgen-improvements.md` Phase 8 when the
gaps above are filled. Mark each with `// TESTGEN-ABSORB: <gap>` so
they're easy to find.

**Test 1: Credential expiry lifecycle (unit test, no network)**

```
Setup:    Create Credential with expires_at = now - 1 second
Verify:   credential.is_valid() == false
Setup:    Create Credential with expires_at = now + 1 hour
Verify:   credential.is_valid() == true
Setup:    Create Credential with expires_at = None
Verify:   credential.is_valid() == true (session-scoped)
```

Testgen coverage: Bucket B node contract tests will cover this once
`Credential.is_valid()` is a node output. No gap — this is expressible
today via `NodeExample` with `OutputMatcher::exact(Value::Bool(true))`.

**Test 2: Env var provider acquire/fail cycle**

```
Setup:    GITHUB_TOKEN env var set (in test harness, not real env)
Acquire:  EnvVarProvider reads it, wraps as Credential
Verify:   credential.secret.value == expected token
Verify:   credential.scheme == AuthScheme::Bearer
Teardown: Clear env var
Verify:   Re-acquire fails with CredentialError::missing_env
```

Testgen coverage: Per-node I/O example covers the success case. The
clear-then-fail case requires **stateful mocking** (env changes between
calls). Mark: `// TESTGEN-ABSORB: stateful-mocking`.

**Test 3: GitHub App — full lifecycle with real App**

```
Setup:    GitHub App configured (app_id, private_key, installation_id)
Acquire:  Exchange JWT for installation token
Use:      GET /repos/{owner}/{repo} — benign, read-only
Verify:   Response is 200, token worked
Destroy:  DELETE /installation/token — explicitly revoke
Verify:   Same GET now returns 401 — token is dead
```

Testgen coverage: Requires **multi-phase flows** (Phase 5 windowed
testing) AND **stateful mocking** (revoke changes subsequent responses).
Mark: `// TESTGEN-ABSORB: phase-5-windows, stateful-mocking`.

### Testgen improvements needed (cross-reference)

These are additions to `testgen-improvements.md` that credential
lifecycle testing depends on. They are NOT blockers for Phases 1-3
of this doc — we write manual tests now and absorb later.

**Existing phases that help:**

- **Phase 5 (Window-based testing)** — enables testing credential flow
  sub-segments: `[acquire..use]`, `[use..refresh]`, `[refresh..revoke]`.
  Currently pending (TODO 5.1–5.5).

- **Phase 8 (Manual test absorption)** — the manual baseline tests
  above are absorption targets. TODO 8.1–8.4 define the assertion types
  testgen needs to replace hand-written tests.

**New items for testgen-improvements.md (document, don't block on):**

- `TODO 10.1`: Add `ResourceType::Credential` with refresh/revoke
  behaviors to resource simulation (Bucket D extension)
- `TODO 10.2`: Add `MockSequence` or `ConditionalMock` to MockSpec
  for stateful transport mocking
- `TODO 10.3`: Generate credential lifecycle test suites when
  `CredentialOp` nodes are detected in a DAG (Bucket C extension)

---

## Implementation Plan

### Phase 1: Base abstractions (additive, no breaking changes)

- [ ] Add `Secret` type to `core/ir/src/transport/` with redacting Debug/Display
- [ ] Add `AuthScheme` enum to `core/ir/src/transport/`
- [ ] Add `Credential` type with `apply()` and `is_valid()` methods
- [ ] Add `CredentialProvider` trait
- [ ] Add `CredentialError` type
- [ ] Unit tests: Secret redaction, Credential.is_valid(), AuthScheme.apply()
      (manual — Test 1. Expressible as `NodeExample` once wired into DAG)

### Phase 2: Concrete providers — GitHub + LLM (additive)

- [ ] Add `GitHubEnvVarProvider` (reads GITHUB_TOKEN, emits Credential)
- [ ] Add `LlmEnvVarProvider` (reads OPENAI_API_KEY / ANTHROPIC_API_KEY)
- [ ] Add `CredentialOp` boundary node (mirrors EnvOp pattern)
- [ ] Add mock credential provider for DryRun
- [ ] Add `MockSpec` for credential DAG (DryRun, scenarios, I/O examples auto-generated by testgen)
- [ ] Manual test: env var acquire/fail cycle (Test 2 — `// TESTGEN-ABSORB: stateful-mocking`)

### Phase 3: GitHub App provider (first real lifecycle)

- [ ] Add `GitHubAppProvider` (JWT exchange → installation token)
- [ ] JWT generation from App private key (RS256)
- [ ] Installation token acquisition (POST /app/installations/{id}/access_tokens)
- [ ] Token revocation (DELETE /installation/token)
- [ ] Manual test: full lifecycle with real GitHub App (Test 3 — `// TESTGEN-ABSORB: phase-5-windows, stateful-mocking`)

### Phase 4: Migration (breaking changes, crate by crate)

- [x] Update `RestRequest.auth` to `Option<Credential>`
- [ ] Update `execute_rest()` to call `credential.apply()` instead of match arms
- [ ] Migrate `github_rest_request()` to accept `Credential` parameter
- [ ] Migrate `build_openai_request()` to accept `Credential` parameter
- [ ] Migrate `build_anthropic_request()` to accept `Credential` parameter
- [x] Remove env-var auth variants (resolved at boundary)
- [x] Remove `GitHubAuth` enum (replaced by provider impls)
- [x] Remove `LlmAuthStyle` (replaced by AuthScheme in provider)
- [x] Remove env-var wrapper type (replace with `api_key_env: String`)
- [ ] Update all DAG graphs to include CredentialOp boundary node

### Phase 5: Testgen absorption (after testgen-improvements Phase 5 + 8)

- [x] TODO 10.1: Add `ResourceType::Credential` to resource simulation
- [x] TODO 10.2: Add `MockSequence`/`ConditionalMock` to MockSpec
- [x] TODO 10.3: Generate credential lifecycle suites for DAGs with `CredentialOp`
- [ ] Absorb manual Tests 2, 3 into testgen-generated tests
- [ ] Delete hand-written tests marked `// TESTGEN-ABSORB`

---

## Design Principles

- **Processes don't care about auth.** They receive a `Credential` or a
  result. No env var names, no scheme logic in business code.
- **Short-lived tokens, managed lifecycle.** Every credential has an
  `expires_at`. `EnvVarProvider` is the bridge (session-scoped, no expiry),
  `GitHubAppProvider` is the target (1hr, auto-refresh).
- **Uniform modeling.** One `Credential` type for all services. Provider
  differences are implementation details of `CredentialProvider::acquire()`.
- **Follows existing patterns.** `CredentialOp` mirrors `EnvOp`.
  `Credential` mirrors `ToolHandle`. Secret redaction mirrors previous auth redaction behavior.
- **Incremental migration.** Phases 1-3 are additive. Phase 4 is the
  breaking migration, done per-crate.
