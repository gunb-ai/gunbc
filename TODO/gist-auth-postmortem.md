# Postmortem: `make gist` 401 — Systemic Modeling and Testgen Gaps

Canonical rolling tracker: `TODO/rolling-postmortem.md`

> **Date**: 2026-02-27
> **Severity**: High — silent credential loss across all service operations
> **Scope**: Not gist-specific; every DSL-defined service with `config { auth }` was affected

---

## Timeline

1. `@headers({ "Authorization": "Bearer {auth_token}" })` annotation was manually wiring credentials into REST requests
2. Annotation deleted during service DSL migration; `config { auth: BearerToken }` added as replacement
3. No lowerer logic existed to wire `auth_token` → `res:credential` on execute nodes
4. `make gist` sends unauthenticated request → GitHub returns 401
5. `gcloud secrets versions access` (fetching the GitHub token) uses shell transport — returns `exit 1` when gcloud session expires
6. `GenericShellParseOp::TrimStdout` does not check exit codes — "succeeds" with empty token
7. Empty token propagates through the chain silently

**Root cause**: The DSL compiler pipeline had no structural enforcement that declared auth actually reached the transport layer. Five independent gaps compounded to produce a silent failure.

---

## The Five Compounding Gaps

### Gap 1: No credential wiring from DSL config to execute node

**What**: `config { auth: BearerToken }` declared intent but the lowerer never wired the `auth_token` input to `res:credential` on the execute node. The credential went into the request body like any other field.

**Where**: `core/daglang/daglang-lower/src/lib.rs` — `add_service_transport_triplets()` and `add_service_call_edges()`

**Fix applied (RT1)**: Added `auth_input` to `ServiceConfig`, lowerer now reads it, excludes the field from prepare inputs, and wires it to `res:credential`.

### Gap 2: Execute node silent fallthrough on missing credential

**What**: `TransportOps::Execute` in `lib/transport/src/ops.rs:65-94` checks for `res:credential`, falls back to `r.auth.take()`, and if both are `None` sends the request unauthenticated. No error.

**Where**: `lib/transport/src/ops.rs`

**Fix applied (RT2)**: `requires_auth: bool` on `RestRequest`, set by prepare when auth scheme declared, fail-closed in execute.

### Gap 3: Shell parse swallows non-zero exit codes

**What**: `GenericShellParseOp::TrimStdout` (`gunbc-dag/src/resolve_service.rs:521-528`) always succeeds by trimming stdout. When `gcloud` returns exit 1 (expired session), stdout is empty, and the parse "succeeds" with an empty string that becomes an empty Secret.

**Where**: `gunbc-dag/src/resolve_service.rs:521`

**Partial fix on separate branch**: Only fail on non-zero exit when output field is `Secret`. This is too narrow — see analysis below.

### Gap 4: Misleading diagnostic

**What**: `decorate_service_failure` shows `key: ""` and `source: static` when no credential exists. "static" implies a hardcoded credential when really there is no credential at all.

**Where**: Error reporting layer (separate concern, not analyzed here)

### Gap 5: Mock backend always returns 200/201

**What**: `GistRecentBackend` in `gunbc-dag/tests/gist_recent_regressions.rs:18-83` always returns success. The test verifies the happy path but never tests what happens when GitHub returns 401, or when gcloud returns exit 1.

**Where**: `gunbc-dag/tests/gist_recent_regressions.rs`

---

## Why This Is Not a Gist Bug — It's a Systemic Modeling Failure

The gist tool is the canary, not the disease. Every service in `dsl/services/` that declares `config { auth }` is potentially affected by the same class of gaps. The fundamental issue is that the DSL's compositional modeling promises are not enforced end-to-end.

### What the DSL promises (from `docs/handbook.md`)

> Every external system is modeled as a **composition of layered concerns** (TCP → TLS → HTTP → REST → provider → operation), where each layer imposes invariants on the generated code.

### What actually happens

1. **Auth layer is not compositional** — `config { auth: BearerToken }` is metadata that the lowerer ignores unless `auth_input` is also set (our fix). Before RT1, the auth declaration had no structural consequence.

2. **Transport error semantics are not modeled** — Shell operations return exit codes. REST operations return status codes. Neither the DSL nor the resolver expresses what constitutes "success" vs "failure" for a given operation. `printenv` returning exit 1 means "variable not set" (valid). `gcloud` returning exit 1 means "session expired" (fatal for credential fetch). The resolver treats both identically.

3. **The credential chain is implicit** — The path from GCP Secret Manager → GitHub auth token involves:
   - `shell.GCloud.SecretManagerAccessVersion` → shell transport → TrimStdout parse → `secret_value: Secret`
   - DSL field access: `secret.secret_value`
   - Passed as `auth_token: secret.secret_value` to `github.Gist.Create`
   - Lowerer wires to `res:credential` (after RT1 fix)
   - Execute node applies credential to request

   At no point does the compiler verify that the full chain is connected. Each hop is independently typed, but the end-to-end invariant ("this REST call requires a valid credential and it must arrive") is never asserted.

### Services affected

Every service with `config { auth }` in `dsl/services/`:

| Service | File | Auth Config | Shell or REST? |
|---------|------|-------------|----------------|
| `github.Gist` | `dsl/services/github/gist.dag` | `auth: BearerToken, auth_input: auth_token` | REST |
| `gcp.IAM` | `dsl/services/gcp/iam.dag` | `auth: BearerToken, auth_input: access_token` | REST |
| `gcp.ResourceManager` | `dsl/services/gcp/iam.dag` | `auth: BearerToken` | REST (profile-bound) |
| `gcp.SecretManager` | `dsl/services/gcp/secret_manager.dag` | `auth: BearerToken` | REST |
| `oauth2.Google` | `dsl/services/shell.dag` | `endpoint` only | REST |
| `shell.GCloud.*` | `dsl/services/shell.dag` | none declared | Shell |

The shell-based GCloud operations (`AuthPrintAccessToken`, `SecretManagerAccessVersion`) have no `config { auth }` because they use the local gcloud session. But their outputs are credentials that feed into REST services — and this cross-service credential chain is not modeled or tested.

---

## Auth Should Be Application-Managed, Not User-Managed

### Current state: manual `gcloud auth login`

The user must manually run `gcloud auth login` before `make gist` works. If the session expires, gist silently fails (or now, with the exit-code fix, loudly fails).

### Previous state: `ensures` pattern (broken at some point)

The DSL already has the building blocks for application-managed auth:

```dag
// dsl/services/shell.dag:13-30
service gcloud.Auth {
  operation Login {
    input { update_adc: Bool = true }
    output { ok: Bool }
    transport shell { argv: ["gcloud", "auth", "login", "--update-adc"] }
  }
  operation ReadADC {
    input { path: FilePath = "~/.config/gcloud/application_default_credentials.json" }
    output { client_id: String, client_secret: Secret, refresh_token: Secret }
    transport file { op: READ, path: "\{path\}" }
  }
}
```

And the oauth2 refresh service exists:

```dag
// dsl/services/shell.dag:32-47
service oauth2.Google {
  config { endpoint: "https://oauth2.googleapis.com" }
  operation Refresh {
    input { client_id: String, client_secret: Secret, refresh_token: Secret }
    output { access_token: Secret from "access_token", expires_in: Int from "expires_in" }
    transport rest { method: POST, path: "/token" }
  }
}
```

### What should happen

The DSL **already has the solution**. `dsl/std/patterns.dag:236-283` defines `credential_chain` — a 5-step OAuth2 pattern (OIDC → STS → optional impersonation → Secret Manager → AccessToken). And `dsl/std/patterns.dag:392-411` defines `local_auth()` — ADC check → OAuth2 refresh → fallback to `gcloud auth print-access-token`.

The gist tool should use `credential_chain` (or at minimum `local_auth()`) instead of raw `shell.GCloud.SecretManagerAccessVersion`:

```dag
// Current (broken when gcloud session expires):
secret = shell.GCloud.SecretManagerAccessVersion(
  project_id: "gunbai-secrets",
  secret_name: "github-token"
)
gist_result = github.Gist.Create(auth_token: secret.secret_value)

// Should be (self-healing via credential_chain):
cred = credential_chain(
  runtime: LocalDev,
  audience: "sigstore",
  secret_name: "github-token",
  project_id: "gunbai-secrets",
  scheme: Bearer,
  source_id: "gist",
  required_scopes: ["gist"]
)
gist_result = github.Gist.Create(auth_token: cred.token.token)
```

The `credential_chain` pattern internally calls `local_auth()` for `LocalDev` runtime, which implements the ensure pattern:

```
1. Check: GOOGLE_APPLICATION_CREDENTIALS env → read ADC file
2. Refresh: oauth2.Google.Refresh(refresh_token) → fresh access_token
3. Fallback: gcloud auth print-access-token (if ADC unavailable)
4. Exchange: gcp.STS.Exchange(subject_token) → federated token
5. Access: gcp.SecretManager.AccessVersion(REST) → GitHub token
```

This flow is entirely REST-based (except the fallback), self-healing, and the DSL pattern already `provides auth: AuthContext` — making the credential a first-class resource.

**The fact that this pattern exists but gist doesn't use it is the core regression.** At some point the gist tool was simplified to use raw gcloud CLI, bypassing the entire credential lifecycle.

This is the **resource acquire pattern** already defined in `dsl/infra/gcp/resources.dag:383` for GCS buckets:

```dag
acquire {
  check = gcp.Storage.GetBucket(name: config.name)
  create [when !check.exists] = gcp.Storage.CreateBucket(...)
  resolve = gcp.Storage.GetBucket(...) [after create]
}
```

The same pattern should apply to credentials: check → acquire → use. The fact that it worked before and stopped working suggests a regression during DSL migration, or the ensures pattern was in the Rust substrate and was never ported to DSL.

### Recommendation: Use REST via `credential_chain`, not gcloud CLI

The current flow uses `shell.GCloud.SecretManagerAccessVersion` (gcloud CLI) to fetch the GitHub token. This has problems:

1. **Requires gcloud installed** — unnecessary dependency when ADC + REST exists
2. **Exit code semantics are ambiguous** — is exit 1 "not logged in" or "secret not found"?
3. **Shell transport is less testable** — no structured response, just stdout
4. **No self-healing** — expired session requires manual `gcloud auth login`

The `credential_chain` pattern (`dsl/std/patterns.dag:236-283`) already solves all of this:
- Uses `local_auth()` for local dev (ADC → OAuth2 refresh → gcloud fallback)
- Uses `gcp.STS.Exchange` (REST) for federated token exchange
- Uses `gcp.SecretManager.AccessVersion` (REST, `dsl/services/gcp/secret_manager.dag:10-20`) for secret access
- Every step is REST with structured responses, typed error codes, and mockable at every layer
- Declares `provides auth: AuthContext` — making credentials a resource the compiler can reason about

---

## Testgen Analysis: Why Automated Tests Didn't Catch This

### What testgen generates today

From `docs/design/testgen.md` — Bucket C (Scenario Coverage):

| Obligation | Generated? | Status |
|-----------|------------|--------|
| All transports succeed | Yes | N+1 scenarios with mock responses |
| Single transport failure | Yes | Per-transport `Value::Skipped` injection |
| Skip-path propagation | Yes | Downstream skip verification |
| Guard branch coverage | Yes | Per-Bool-guard two-scenario |

### What testgen does NOT generate (and should)

#### 1. Error status fuzzing for REST operations

`default_rest_response()` in `gunbc-dag/src/mock_defaults.rs:164` always returns status 200 with a stock JSON body. There is no testgen obligation that injects 401, 403, 404, 500, or 503 responses.

**Testgen design says (Bucket C)**: "Single transport failure — N per transport executor." But the "failure" is `Value::Skipped`, not a realistic error response. A 401 is not a skip — it's a response that the parse node must handle.

**What's needed**: For every REST operation, testgen should generate N status-code scenarios:
- 200 (success) — already covered
- 401 (auth failure) — missing
- 403 (forbidden) — missing
- 404 (not found) — missing
- 500 (server error) — missing
- Timeout/network error — missing

This is directly derivable from the `@rest` annotation. Every `@rest(POST, "/path")` implies HTTP semantics. The compiler knows the status code space. `@mock_response` annotations (currently unused in real services) could supply per-operation error shapes.

**Code reference**: `core/codegen/src/testgen/obligation.rs` — `Obligation::SingleTransportFailure { node_id }` only generates skip-injection, not status-code variation.

#### 2. Shell exit code testing

`default_shell_response()` in `gunbc-dag/src/mock_defaults.rs:145` returns `ShellResponse::ok(String::new())` — always exit 0 with empty stdout.

**What's needed**: For every shell operation, testgen should generate:
- Exit 0 with valid output (success) — partially covered
- Exit 1 (command failure) — missing
- Exit 0 with empty stdout (edge case for credential fetches) — missing
- Exit 0 with malformed output — missing

**Code reference**: `ShellResponse::ok("")` in `mock_defaults.rs:146`. The `ShellResponse` struct has `exit_code`, `stdout`, `stderr` fields — the infrastructure exists, the obligations don't.

#### 3. Cross-service credential chain verification

No testgen obligation verifies that a credential output from service A actually reaches the auth input of service B. This is the gap that RT1 filled manually — but testgen should generate this class of test automatically.

**What's needed**: A new obligation type: `CredentialChainIntegrity { producer_node, consumer_node, credential_port }`. For every service with `config { auth }`, trace backwards through the DAG to find the credential source node and assert the edge exists.

**Code reference**: The `res:credential` port on execute nodes (`core/daglang/daglang-lower/src/lib.rs:5731`) is structurally identifiable. The edge assertion is trivially derivable.

#### 4. Parse node output validation under error conditions

Parse nodes (GenericRestParseOp, GenericShellParseOp) are only tested with the default success response. No test verifies what a parse node produces when given a 401 response or exit-code-1 shell output.

**What's needed**: For every parse node, testgen should exercise it with each mock response variant (success + each error status) and verify it either produces valid typed output or propagates an error.

**Code reference**: `core/codegen/src/testgen/codegen.rs` — `SeedMatrix` controls which values are injected per node. Currently only success seeds are populated.

### Testgen infrastructure that exists but is unused

| Infrastructure | File | Status |
|---------------|------|--------|
| `MockSpec::error_cases()` | `core/test/src/mockable.rs` | Trait method exists, returns `Vec<(String, Value)>` — never populated for services |
| `@mock_response` annotation | DSL language spec | Parsed by compiler, but not a single real service uses it |
| `FidelityLevel::VirtualIo` | `core/test/src/fidelity.rs` | Defined but blocked — no virtual HTTP server impl |
| AC-14 (per-node corpus execution) | `core/codegen/src/testgen/codegen.rs` | Stub only; `corpus_tests: false` |
| AC-15 (adjacent-pair window tests) | `core/codegen/src/testgen/codegen.rs` | Stub only; blocked on `__deps MixedInput` |

### The fundamental testgen modeling gap

The testgen design doc (`docs/design/testgen.md`) says:

> **Bucket C — Scenario Coverage (N+1, not 2^N)**
> | Obligation | Count |
> | All transports succeed | 1 |
> | Single transport failure | N (per transport executor) |

The "N+1" model assumes two states per transport: success or skip. But real transport has a much richer failure space:

- REST: 200, 201, 204, 301, 400, 401, 403, 404, 409, 429, 500, 502, 503, timeout
- Shell: exit 0-255, stdout empty/malformed, stderr messages
- File: exists/not-exists, permission denied, corrupt content

The testgen should generate **N × K** scenarios where K is the number of meaningful response classes per transport type, not just N+1. The `@mock_response` annotation is designed for exactly this — each annotation provides a response fixture for a specific scenario. But no service uses it.

---

## Compositional Modeling Gaps to Harden

### Current layer model (from `docs/handbook.md`)

```
TCP → TLS → HTTP → REST → Provider → Operation
```

### Where the model breaks

| Layer | Promise | Reality |
|-------|---------|---------|
| **Auth** | `config { auth: BearerToken }` declares auth scheme | Before RT1/RT2, declaration had no structural effect |
| **Transport error** | Layers define error semantics | Shell exit codes and REST status codes are not modeled in DSL |
| **Credential lifecycle** | Credentials are acquired and refreshed | Manual `gcloud auth login`; no ensure/upsert in workflow |
| **Cross-service wiring** | Compiler verifies all edges | Credential chain (shell output → rest input) not verified end-to-end |
| **Parse fidelity** | Parse nodes handle all response shapes | Parse only tested with success response |

### What needs to happen

1. **Transport error types in DSL**: Operations should declare their error space. Example:
   ```dag
   operation Create {
     // ... existing ...
     errors {
       Unauthorized { status: 401, retry: false }
       RateLimited { status: 429, retry: true, backoff: exponential(1000) }
       ServerError { status: 500, retry: true, max: 3 }
     }
   }
   ```
   This is derivable from `@rest` (HTTP semantics) + `@retry` (retry policy).

2. **Credential resource type**: Credentials should be modeled as resources with acquire/refresh/expire lifecycle, not as raw Secret values passed through field access.

3. **`@mock_response` adoption**: Every service operation should have at least:
   - `@mock_response(status: 200, body: { ... })` — success
   - `@mock_response(status: 401, body: { ... })` — auth failure
   These feed directly into testgen Bucket C scenarios.

4. **Shell exit code semantics**: Shell operations should declare expected exit codes:
   ```dag
   operation SecretManagerAccessVersion {
     // ... existing ...
     transport shell {
       argv: [...]
       exit_codes { 0: success, 1: auth_expired, _: unknown_error }
     }
   }
   ```

---

## Affected Code References

### Testgen pipeline

| File | Purpose | Line |
|------|---------|------|
| `core/codegen/src/testgen/obligation.rs` | Obligation IR — needs error-status obligations | — |
| `core/codegen/src/testgen/codegen.rs` | SeedMatrix — needs error response seeds | — |
| `gunbc-dag/src/mock_defaults.rs:145-180` | Default mock responses — always success | `default_shell_response()`, `default_rest_response()` |
| `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` | Auto-discovery — could derive error fixtures | — |
| `core/test/src/mockable.rs` | `error_cases()` trait — exists but unused for services | — |

### Credential wiring

| File | Purpose | Line |
|------|---------|------|
| `core/daglang/daglang-lower/src/lib.rs` | `auth_input` wiring to `res:credential` | ~5731, ~6002 |
| `gunbc-dag/src/resolve_service.rs` | `GenericRestPrepareOp` — sets `requires_auth` | ~60-94 |
| `lib/transport/src/ops.rs` | Credential application (3 paths) | 65-94 |
| `lib/transport/src/executor.rs` | Standalone `execute_rest()` credential path | 55 |

### Service definitions needing `@mock_response`

| File | Service | Operations |
|------|---------|------------|
| `dsl/services/github/gist.dag` | `github.Gist` | Create |
| `dsl/services/gcp/iam.dag` | `gcp.IAM` | GenerateAccessToken |
| `dsl/services/gcp/iam.dag` | `gcp.ResourceManager` | GetIamPolicy, SetIamPolicy |
| `dsl/services/gcp/secret_manager.dag` | `gcp.SecretManager` | AccessVersion, CreateSecret, AddVersion |
| `dsl/services/shell.dag` | `oauth2.Google` | Refresh |
| `dsl/services/shell.dag` | `shell.GCloud` | AuthPrintAccessToken, SecretManagerAccessVersion |

### Shell parse (exit code handling)

| File | Purpose | Line |
|------|---------|------|
| `gunbc-dag/src/resolve_service.rs:521-528` | `TrimStdout` — no exit code check | |
| `gunbc-dag/src/resolve_service.rs:488` | `SuccessStdoutStderr` — uses `shell.success()` | |
| `gunbc-dag/src/resolve_service.rs:501` | `ExitCodeBool` — uses `shell.success()` | |

Only `SuccessStdoutStderr` and `ExitCodeBool` check exit codes. `TrimStdout` and `SplitLines` do not. This means any shell operation using these parse modes silently succeeds on error.

---

## Recommended Red Team Tasks

### Analysis tasks (study before implementing)

- **RT-A1**: Audit all shell operations for exit-code-dependent semantics. Which operations use TrimStdout/SplitLines? Which can legitimately return non-zero? Build a decision matrix for exit code handling per operation.

- **RT-A2**: Audit `@mock_response` adoption gap. Every service operation should declare at least success + auth-failure mock responses. Count the gap: how many operations × how many missing mock responses = total obligation deficit.

- **RT-A3**: Trace the credential chain for every service with `config { auth }`. Map producer → consumer edges. Identify which chains are verified by tests and which are not.

- **RT-A4**: Study why gist bypasses `credential_chain`. The pattern exists at `dsl/std/patterns.dag:236-283` with `local_auth()` at lines 392-411. The gist tool uses raw `shell.GCloud.SecretManagerAccessVersion` instead. Was `credential_chain` ever wired into gist? What broke? What's the migration path to wire `credential_chain(runtime: LocalDev, ...)` into all three gist entrypoints?

- **RT-A5**: Study the credential-as-resource model. `credential_chain` declares `provides auth: AuthContext` (`dsl/std/resources.dag:87-98`). The `AuthContext` resource has `expires: true`. How should the compiler enforce that tools consuming `AuthContext` handle expiry? Should testgen generate expiry-scenario tests?

### Implementation tasks (after analysis)

- **RT-I1**: Add `@mock_response` annotations to all service operations (success + error variants)
- **RT-I2**: Extend testgen Bucket C: inject error status codes, not just `Value::Skipped`
- **RT-I3**: Add `CredentialChainIntegrity` obligation to testgen
- **RT-I4**: Add shell exit code check to `TrimStdout` and `SplitLines` (with semantic opt-out)
- **RT-I5**: Wire `credential_chain(runtime: LocalDev, ...)` into gist tool, replacing raw `shell.GCloud.SecretManagerAccessVersion` calls in all three entrypoints
- **RT-I6**: Verify `credential_chain` pattern compiles and runs end-to-end (it references `gcp.STS.Exchange`, `gcp.SecretManager.AccessVersion`, `local_auth()` — all must lower correctly with RT1/RT2 fixes)

---

## Key Takeaway

The DSL's compositional modeling is sound in design but has enforcement gaps at every layer boundary. The compiler verifies types and edges but does not verify:

1. Auth declarations produce credential wiring (fixed by RT1/RT2)
2. Transport errors are handled, not swallowed
3. Cross-service credential chains are connected
4. Parse nodes handle error responses
5. Credentials are acquired/refreshed automatically

Testgen is the right mechanism to close these gaps — it already has the obligation IR and generation pipeline. The missing piece is **error-variant mock responses** and **cross-service chain obligations**. The `@mock_response` annotation exists in the DSL spec but is unused on real services. Adopting it across all operations would mechanically generate the test coverage that would have caught this bug.
