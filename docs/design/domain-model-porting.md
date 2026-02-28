# Domain Model Porting Reference

Source material for **Lane 4: Domain Model Foundation**. All content ported from
sibling repos (`the-gunbai`, `gunb.ai`) so Lane 4 workers need no access to those repos.

## How to Use This Doc

Each section below contains raw behavioral data from the sibling repos. Lane 4
tasks reference specific sections. The workflow is:

1. Read the relevant section for your DM-* task
2. Translate the behavioral data into `.dag` types and data declarations
3. Use the vocabulary types from DM-1:5 (`std/behavioral.dag`, etc.) to structure
   the behavioral property data
4. Follow the extdeps modeling pattern established by `dsl/extdeps/clippy.dag` and
   `dsl/extdeps/github_actions.dag`

---

## §1 Understanding Structure (from `the-gunbai`)

Every "understanding" in the-gunbai follows this structure. Lane 4 task DM-1
(`std/behavioral.dag`) should define DSL types that capture all of these dimensions.

```
Understanding:
  ID:           unique identifier
  Kind:         category (CLI Tool | REST API | SDK/Library | Secret Provider)
  Version:      date-stamped
  Behaviors:    list of operations, each with:
    - Invocation:  how to call it (HTTP method+path, CLI command, function call)
    - Inputs:      typed parameters (name, type, required/optional)
    - Outputs:     typed return values
    - Properties:
        - Side effects:    Read-only | Writes to external state
        - Idempotency:     Idempotent [with keys: ...] | Not idempotent
        - Determinism:     Deterministic output | Non-deterministic
        - Failure modes:   Fails when <condition> (<error code>)
        - Edge cases:      <description>
        - Confidence:      Documented | HighConfidence | MediumConfidence
  Dependencies:  external requirements (secrets, tools, artifacts)
  Assumptions:   what must be true for behaviors to work
  Unknowns:      things not yet validated
```

### Behavioral dimensions → DSL type mapping

| Dimension | DSL representation |
|-----------|-------------------|
| Side effects | `SideEffects` sum type (ReadOnly \| WritesState \| WritesExternal) |
| Idempotency | `idempotent: Bool` + `idempotency_keys: List<String>?` on `OperationBehavior` |
| Determinism | `Determinism` sum type (Deterministic \| NonDeterministic \| EventuallyConsistent) |
| Failure modes | `List<FailureMode>` where `FailureMode { name, condition, http_status?, recoverable, retry_safe }` |
| Edge cases | `List<EdgeCase>` where `EdgeCase { description, trigger?, severity }` |
| Confidence | `Confidence` sum type (Documented \| HighConfidence \| MediumConfidence \| LowConfidence \| Assumed) |
| Dependencies | maps to `uses` declarations and `CapabilityRequirement` (DM-5) |
| Assumptions | `List<String>` on `OperationBehavior` |
| Unknowns | `List<String>` on `OperationBehavior` |

---

## §2 Secret Providers

### §2.1 GCP Secret Manager (DM-7)

Source: `the-gunbai/docs/understandings/gcp-secret-manager.md`

**8 operations:**

| Operation | Method | Path | Side Effects | Idempotent | Failure Modes | Edge Cases |
|-----------|--------|------|-------------|------------|---------------|------------|
| access_secret_version | GET | `/v1/projects/{project}/secrets/{secret}/versions/{version}:access` | ReadOnly | Yes | NOT_FOUND (secret/version), PERMISSION_DENIED, version disabled/destroyed | version can be number or "latest" |
| list_secrets | GET | `/v1/projects/{project}/secrets` | ReadOnly | Yes | — | Paginated (page_size, page_token, filter) |
| list_secret_versions | GET | `/v1/projects/{project}/secrets/{secret}/versions` | ReadOnly | Yes | — | Paginated, filterable |
| create_secret | POST | `/v1/projects/{project}/secrets` | WritesState | Yes (keys: project, secret_id) | — | Creates metadata only, no value — must call add_secret_version separately |
| add_secret_version | POST | `/v1/projects/{project}/secrets/{secret}:addVersion` | WritesState | No | NOT_FOUND (secret) | New version state is ENABLED |
| disable_secret_version | POST | `...versions/{version}:disable` | WritesState | Yes | — | Retains data, prevents access |
| destroy_secret_version | POST | `...versions/{version}:destroy` | WritesState | Yes | — | **Irreversible** — data cannot be recovered |
| delete_secret | DELETE | `/v1/projects/{project}/secrets/{secret}` | WritesState | Yes | — | **Unlike AWS, deletion is immediate** — no recovery window. Optional etag for CAS. |

**Dependencies:** `GOOGLE_APPLICATION_CREDENTIALS` (GCP ADC)
**Assumptions:** GCP credentials via ADC, project ID known, IAM role includes `secretmanager.versions.access`
**Unknowns:** Exact propagation delay for IAM policy changes, behavior during regional outages with automatic replication

### §2.2 GitHub Secrets (DM-8)

Source: `the-gunbai/docs/understandings/github-secrets.md`

**8 operations:**

| Operation | Method | Path | Side Effects | Idempotent | Failure Modes | Edge Cases |
|-----------|--------|------|-------------|------------|---------------|------------|
| list_repo_secrets | GET | `/repos/{owner}/{repo}/actions/secrets` | ReadOnly | Yes | — | Returns names only, **not values** — values are write-only |
| get_repo_secret | GET | `/repos/{owner}/{repo}/actions/secrets/{secret_name}` | ReadOnly | Yes | 404 (not found) | Returns metadata only (name, created_at, updated_at) |
| get_repo_public_key | GET | `/repos/{owner}/{repo}/actions/secrets/public-key` | ReadOnly | Yes | — | Key rotates periodically — **always fetch fresh before encrypting** |
| create_or_update_repo_secret | PUT | `/repos/{owner}/{repo}/actions/secrets/{secret_name}` | WritesState | Yes (keys: owner, repo, secret_name) | Invalid characters in name | **No versioning** — update overwrites. Name must match `^[a-zA-Z_][a-zA-Z0-9_]*$`. Requires `encrypted_value` (libsodium sealed box) + `key_id`. |
| delete_repo_secret | DELETE | `/repos/{owner}/{repo}/actions/secrets/{secret_name}` | WritesState | Yes | — | — |
| list_environment_secrets | GET | `/repositories/{repository_id}/environments/{environment_name}/secrets` | ReadOnly | Yes | — | Scoped to deployment environment |
| get_oidc_token | GET | (Actions runtime) | ReadOnly | NonDeterministic | — | Requires `id-token: write` permission. Token contains claims: sub, aud, iss, ref, sha, repository. |
| access_secret_in_workflow | (YAML env) | `${{ secrets.SECRET_NAME }}` | ReadOnly | Yes | — | Values masked in logs. **Empty string if secret doesn't exist** (no error). |

**Dependencies:** `GITHUB_TOKEN` with appropriate permissions
**Assumptions:** Token has permissions, public key available for encryption
**Unknowns:** Exact propagation delay when updating secrets, rate limits for secrets API

### §2.3 .env Files (DM-9)

Source: `the-gunbai/docs/understandings/env-file.md`

**6 operations:**

| Operation | Invocation | Side Effects | Idempotent | Edge Cases |
|-----------|-----------|-------------|------------|------------|
| load | `dotenv::dotenv()` | WritesState (process env) | Yes | Missing file is usually silent (not error). `override_existing` flag controls behavior. |
| get | `std::env::var()` | ReadOnly | Yes | Fails when variable not set |
| set | `write_to_file()` | WritesState | Yes (keys: path, key) | — |
| list | `parse_file()` | ReadOnly | Yes | — |
| generate_example | `generate_template()` | WritesState | Yes | Must identify secrets vs config values |
| validate | `validate_against_template()` | ReadOnly | Yes | Returns missing/extra variable lists |

**Dependencies:** `.gitignore` must include `.env`
**Assumptions:** File exists in project root, properly gitignored

### §2.4 HashiCorp Vault (DM-10)

Source: `the-gunbai/docs/understandings/hashicorp-vault.md`

**10 operations:**

| Operation | Method | Path | Side Effects | Idempotent | Failure Modes | Edge Cases |
|-----------|--------|------|-------------|------------|---------------|------------|
| kv_read | GET | `/v1/{mount}/data/{path}` | ReadOnly | Yes | 404 (not found), 403 (denied), version deleted/destroyed | `version=0` means latest |
| kv_list | GET (LIST) | `/v1/{mount}/metadata/{path}` | ReadOnly | Yes | — | Uses HTTP LIST method. Folders end with `/`. |
| kv_write | POST | `/v1/{mount}/data/{path}` | WritesState | Yes (keys: path, cas) | CAS mismatch | `cas=0` means create-only, `cas=N` means update-if-at-version-N |
| kv_delete | DELETE | `/v1/{mount}/data/{path}` | WritesState | Yes | — | **Soft delete** — can be undeleted |
| kv_destroy | POST | `/v1/{mount}/destroy/{path}` | WritesState | Yes | — | **Permanent** — data cannot be recovered. Takes version list. |
| auth_token_lookup | GET | `/v1/auth/token/lookup-self` | ReadOnly | Yes | — | Returns: accessor, policies, ttl (remaining seconds), renewable |
| auth_approle | POST | `/v1/auth/approle/login` | WritesState | NonDeterministic | Invalid role_id or secret_id | Returns: client_token, policies, lease_duration |
| auth_oidc | POST | `/v1/auth/jwt/login` | WritesState | NonDeterministic | — | JWT must not be expired, must match bound claims |
| database_creds | GET | `/v1/database/creds/{role}` | WritesState | NonDeterministic | — | **Credentials auto-revoke after TTL** unless renewed |
| lease_renew | POST | `/v1/sys/leases/renew` | WritesState | Yes | Lease expired/revoked, renewal exceeds max_ttl | — |

**Dependencies:** `VAULT_TOKEN` (token auth), `VAULT_ROLE_ID` + `VAULT_SECRET_ID` (AppRole)
**Assumptions:** Vault server at `VAULT_ADDR`, KV v2 at `secret/` by default
**Unknowns:** Performance under high load, behavior during HA leader election

---

## §3 Coordination Stores

### §3.1 GCS as Coordination Store (DM-12)

Source: `the-gunbai/docs/understandings/gcs.md`

**6 operations:**

| Operation | Method | Path | Side Effects | Idempotent | CAS Mechanism | Edge Cases |
|-----------|--------|------|-------------|------------|---------------|------------|
| objects/get | GET | `/storage/v1/b/{bucket}/o/{object}?alt=media` | ReadOnly | Yes (Deterministic) | — | `generation` is i64, monotonically increasing. `metageneration` for metadata. `custom_time` for TTL tracking. |
| objects/put_versioned | PUT | `/upload/storage/v1/b/{bucket}/o?uploadType=media&name={object}&ifGenerationMatch={generation}` | WritesState | Yes (keys: bucket, object, expected_generation, data) | `ifGenerationMatch=0` → object must NOT exist. `ifGenerationMatch=N` → update if at generation N. Also `ifGenerationNotMatch`, `ifMetagenerationMatch`. | Returns new_generation on success. 412 on precondition failure. |
| objects/list | GET | `/storage/v1/b/{bucket}/o?prefix={prefix}` | ReadOnly | NonDeterministic | — | Paginated (nextPageToken). **Eventually consistent for recent writes.** Delimiter `/` gives directory-like prefixes. |
| objects/delete | DELETE | `/storage/v1/b/{bucket}/o/{object}?ifGenerationMatch={generation}` | WritesState | Yes | Optional generation precondition | 412 on precondition failure |
| lifecycle/configure | PATCH | `/storage/v1/b/{bucket}` | WritesState | — | — | **Changes take up to 24h to take effect.** Use `customTimeBefore` for TTL. Rules evaluated once per day per object. |
| notifications/create | POST | `/storage/v1/b/{bucket}/notificationConfigs` | WritesState | — | — | Requires PubSub topic to exist. **Notifications are best-effort hint channel.** |

**Dependencies:** `GOOGLE_APPLICATION_CREDENTIALS`
**Unknowns:** Exact latency for generation precondition checks, behavior under high concurrent CAS, whether 412 errors count against rate limits, regional vs multi-regional consistency guarantees

### §3.2 PostgreSQL as Coordination Store (DM-13)

Source: `the-gunbai/docs/understandings/postgres.md`

**7 operations:**

| Operation | SQL Pattern | Side Effects | Idempotent | Mechanism | Edge Cases |
|-----------|------------|-------------|------------|-----------|------------|
| kv/get | `SELECT value, version, expires_at FROM kv WHERE key = $1` | ReadOnly | Yes | — | Client must check `expires_at` for soft TTL |
| kv/put_versioned | `BEGIN; SELECT FOR UPDATE; UPDATE; COMMIT` with row count check | WritesState | Yes (keys: key, expected_version, value) | SELECT FOR UPDATE locks row. INSERT ON CONFLICT for conditional create. RETURNING gives new version atomically. | Returns current_value + current_version on conflict. |
| queue/claim | `SELECT FOR UPDATE SKIP LOCKED` + `UPDATE claimed_by` | WritesState | NonDeterministic | **SKIP LOCKED** skips rows locked by others. ORDER BY priority, created_at for fair scheduling. | Unclaimed items have `claim_expires_at < now()`. |
| fencing/next_token | `INSERT ON CONFLICT DO UPDATE` with RETURNING | WritesState | NonDeterministic | INSERT ON CONFLICT for atomic increment | Monotonically increasing within scope |
| notify/send | `NOTIFY channel, payload` | WritesState | — | **Transactional** — sent on COMMIT | Payload limit 8000 bytes. Use as hint channel only, not for correctness. |
| notify/listen | `LISTEN channel; poll` | ReadOnly | NonDeterministic | — | Requires persistent connection. Notifications buffered server-side until polled. |
| ttl/cleanup | `DELETE FROM kv WHERE expires_at < now()` | WritesState | Yes | — | Consider pg_cron or application-level periodic execution. |

**Dependencies:** `DATABASE_URL`
**Assumptions:** PostgreSQL 12+, sqlx crate with postgres feature
**Unknowns:** Optimal pool size, advisory lock vs row-level lock tradeoffs, pg_cron availability on managed services

### §3.3 SQLite as Coordination Store (DM-14)

Source: `the-gunbai/docs/understandings/sqlite.md`

**5 operations:**

| Operation | SQL Pattern | Side Effects | Idempotent | Edge Cases |
|-----------|------------|-------------|------------|------------|
| kv/get | `SELECT value, version, expires_at FROM kv WHERE key = ?` | ReadOnly | Yes (Deterministic) | Client must check expires_at for soft TTL |
| kv/put_versioned | `INSERT OR REPLACE with WHERE version = ?` | WritesState | Yes (keys: key, expected_version, value) | Version is auto-incremented. Atomic via single UPDATE + row count. Conditional create via INSERT OR IGNORE + row count. |
| fencing/next_token | `INSERT OR REPLACE with RETURNING` | WritesState | NonDeterministic | INSERT ON CONFLICT DO UPDATE for atomic increment. Scopes are independent. |
| ttl/cleanup | `DELETE FROM kv WHERE expires_at < datetime('now')` | WritesState | Yes | Run periodically (e.g., every minute) |
| kv/list | `SELECT key FROM kv WHERE key LIKE ? || '%'` | ReadOnly | Yes | Requires index on key column. Must escape `%` and `_` in prefix. |

**Constraints:** WAL mode requires write access to -wal and -shm files. **Single-writer** — SQLite locks entire database for writes.
**Assumptions:** SQLite 3.35+ (RETURNING clause), WAL mode, rusqlite with bundled-full
**Unknowns:** Performance under high write load, behavior on NFS/SMB

---

## §4 Tool Lifecycle

### §4.1 Rust Toolchain (DM-15)

Source: `the-gunbai/docs/understandings/tool-rust.md`

**6 operations:**

| Operation | Invocation | Side Effects | Idempotent | Edge Cases |
|-----------|-----------|-------------|------------|------------|
| verify | `rustc --version` | ReadOnly | Yes (Deterministic) | — |
| install_script | `curl ... \| sh -s -- -y` | WritesState | Yes | Requires curl + network. Requires shell restart or `source $HOME/.cargo/env`. |
| install_winget | `winget install Rustlang.Rustup` | WritesState | Yes | Windows only |
| cargo_build | `cargo build [--release] [--workspace]` | WritesState | Yes | — |
| cargo_test | `cargo test [--workspace]` | ReadOnly | Yes | — |
| cargo_install | `cargo install <crate>` | WritesState | Yes | — |

**Assumptions:** curl available, network access to rustup.rs, write access to `$HOME/.cargo`

### §4.2 GitHub CLI (DM-16)

Source: `the-gunbai/docs/understandings/tool-gh.md`

**6 operations:**

| Operation | Invocation | Side Effects | Idempotent | Edge Cases |
|-----------|-----------|-------------|------------|------------|
| verify | `gh --version` | ReadOnly | Yes (Deterministic) | — |
| install_apt | `apt-get install -y gh` | WritesState | Yes | May need GitHub apt repository first |
| install_brew | `brew install gh` | WritesState | Yes | — |
| install_choco | `choco install -y gh` | WritesState | Yes | — |
| auth_status | `gh auth status` | ReadOnly | — | Requires network |
| release_download | `gh release download <tag> --repo <owner/repo> --pattern <pattern>` | WritesState | — | Fails when release or asset not found |

**Assumptions:** GitHub auth configured, network access
**Unknowns:** Token scope requirements vary by operation

### §4.3 Package Managers (DM-17)

Source: `the-gunbai/docs/understandings/package_manager.md`

**7 operations:**

| Operation | Invocation | Platform | Side Effects | Idempotent | Edge Cases |
|-----------|-----------|----------|-------------|------------|------------|
| apt_install | `apt-get install -y <pkgs>` | Debian/Ubuntu | WritesState | Yes | Requires sudo/root. Fails when package not found or no network. `update_first` flag. |
| brew_install | `brew install <pkgs>` | macOS/Linux | WritesState | Yes | Requires Homebrew. `cask` flag for GUI apps. |
| choco_install | `choco install -y <pkgs>` | Windows | WritesState | Yes | Requires Administrator shell. Optional `version` pin. |
| winget_install | `winget install --id <pkg>` | Windows | WritesState | Yes | Uses `--accept-source-agreements --accept-package-agreements`. |
| cargo_install | `cargo install --locked <crate>` | Cross-platform | WritesState | Yes (keys: crate, version) | Requires Rust toolchain. |
| github_release_download | `curl -L <url> -o <output>` | Cross-platform | WritesState | Yes (keys: repo, version) (MediumConfidence) | Requires curl. Fails when release not found. |
| script_run | `sh -c <command>` | Cross-platform | WritesState | **No** (MediumConfidence) | Arbitrary execution. |

**Unknowns:** Behavior when disk space insufficient, race conditions with concurrent managers, corporate proxy behavior, partial installation recovery

---

## §5 LLM Provider Detail

### §5.1 Anthropic Models (DM-19)

Source: `gunb.ai/tools/extdeps/anthropic_api.go`

| Model | ID | Input $/MTok | Output $/MTok | Context | MaxOutput | ExtThinking | Cache |
|-------|-----|-------------|---------------|---------|-----------|-------------|-------|
| claude-haiku-4-5 | claude-haiku-4-5-20251001 | $1.00 | $5.00 | 200K | 64K | yes | yes |
| claude-sonnet-4-5 | claude-sonnet-4-5-20250929 | $3.00 | $15.00 | 200K | 64K | yes | yes |
| claude-opus-4-5 | claude-opus-4-5-20251101 | $5.00 | $25.00 | 200K | 64K | yes | yes |
| claude-sonnet-4 | claude-sonnet-4-20250514 | $3.00 | $15.00 | 200K | 64K | yes | yes (deprecated) |
| claude-opus-4 | claude-opus-4-20250514 | $15.00 | $75.00 | 200K | 32K | yes | yes (deprecated) |
| claude-3-5-haiku | claude-3-5-haiku-20241022 | $1.00 | $5.00 | 200K | 8K | no | yes (deprecated) |

**Caching:** Explicit `cache_control` markers, max 4 breakpoints, 90% read discount, 5min default / 1hr extended TTL.

### §5.2 OpenAI Models (DM-19)

Source: `gunb.ai/tools/extdeps/openai_api.go`

| Model | Input $/MTok | Cached $/MTok | Output $/MTok | Context | MaxOutput |
|-------|-------------|---------------|---------------|---------|-----------|
| gpt-5.1-codex-mini | $0.25 | $0.025 | $2.00 | 128K | 16K |
| gpt-5.1-codex | $1.25 | $0.125 | $10.00 | 128K | 16K |
| gpt-5.2 | $1.75 | $0.175 | $14.00 | 128K | 16K |
| gpt-5.1 | $1.25 | $0.125 | $10.00 | 128K | 16K |
| gpt-5 | $1.25 | $0.125 | $10.00 | 128K | 16K |
| gpt-5-mini | $0.25 | $0.025 | $2.00 | 128K | 16K |
| gpt-5-nano | $0.05 | $0.005 | $0.40 | 128K | 16K |
| o3 | $2.00 | $0.50 | $8.00 | 128K | 32K |
| o3-mini | $1.10 | $0.55 | $4.40 | 128K | 32K |
| o3-pro | $20.00 | — | $80.00 | 128K | 32K |
| o4-mini | $1.10 | $0.275 | $4.40 | 128K | 32K |
| gpt-4.1 | $2.00 | $0.50 | $8.00 | 128K | 16K |
| gpt-4.1-mini | $0.40 | $0.10 | $1.60 | 128K | 16K |
| gpt-4.1-nano | $0.10 | $0.025 | $0.40 | 128K | 16K |

**Caching:** Automatic prefix matching, 128-token increments, 50% default read discount, 5min default / 1hr extended TTL.

### §5.3 Gemini Models (DM-19)

Source: `gunb.ai/tools/extdeps/gemini_api.go`

| Model | Input $/MTok | Output $/MTok | Context | MaxOutput | StructuredOutput | Grounding |
|-------|-------------|---------------|---------|-----------|-----------------|-----------|
| gemini-3-pro-preview | $1.25 (est) | $5.00 (est) | 1M | 65K | yes | yes |
| gemini-3-flash-preview | $0.075 (est) | $0.30 (est) | 1M | 65K | yes | yes |
| gemini-2.5-pro | $1.25 | $5.00 | 1M | 65K | yes | yes |
| gemini-2.5-flash | $0.075 | $0.30 | 1M | 65K | yes | yes |
| gemini-2.5-flash-lite | $0.01875 | $0.075 | 1M | 65K | yes | no |

**Access:** Via Vertex AI (OAuth/ADC), not API key.

### §5.4 LLM API Behaviors (DM-19)

Source: `the-gunbai/docs/understandings/llm_api.md`

| Operation | Method | Side Effects | Idempotent | Failure Modes | Edge Cases |
|-----------|--------|-------------|------------|---------------|------------|
| chat_completion | POST /v1/chat/completions | WritesState | No (NonDeterministic) | Rate limit exceeded, context length exceeded | Some providers return different model than requested. Temperature 0-100 scaled to 0.0-2.0. |
| count_tokens | (local) | ReadOnly | Yes (Deterministic) | — | Count may vary between tokenizer versions |
| stream_completion | POST /v1/chat/completions (stream) | WritesState | No (NonDeterministic) | — | Stream may disconnect mid-response |
| list_models | GET /v1/models | ReadOnly | Yes | — | Available models depend on account permissions |

**Dependencies:** `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`
**Unknowns:** Rate limit behavior varies by account tier, token counting accuracy varies, response latency varies significantly

---

## §6 Infrastructure Scope Model

Source: `gunb.ai/tools/extdeps/proto/cloud_extdeps.proto`

### §6.1 Abstract Scope Types (DM-5)

```
InfraScopeType:
  SECRET       — Secret/credential storage
  IDENTITY     — IAM/identity management
  API          — API access/enablement
  STORAGE      — Object/blob storage
  COMPUTE      — Compute instances/containers
  NETWORK      — VPC/firewall/DNS
  DATABASE     — Database access
  QUEUE        — Message queue/pubsub
  FEDERATION   — Workload identity federation

InfraAccessLevel:
  READ   — Read-only access
  WRITE  — Read + write access
  ADMIN  — Full administrative access
```

### §6.2 Service Access Requirements (DM-5)

Source: `gunb.ai/tools/extdeps/infra_services.go` — 18 services modeled:

| Service | Scopes | Secrets | Dependencies |
|---------|--------|---------|-------------|
| infra-secrets | Secret:READ | — | — |
| infra-wif | Federation:READ | — | — |
| infra-gar | Identity:WRITE | — | — |
| infra-compute | Compute:WRITE | — | — |
| infra-storage | Storage:READ | — | — |
| buildbuddy | — | buildbuddy-api-key | infra-secrets |
| github-actions | — | github-token (opt) | infra-wif, infra-secrets |
| google-workspace | — | workspace-sa-key, workspace-admin-email, workspace-domain | infra-secrets |
| openai | — | openai-api-key | infra-secrets |
| anthropic | — | anthropic-api-key (opt) | infra-secrets |
| gemini | — | — (ADC) | — |
| cursor | — | cursor-api-key (opt) | infra-secrets |
| engine | — | — | openai, anthropic, cursor, infra-storage |
| planner | — | — | openai, anthropic |
| runner | — | — | engine, google-workspace |
| ci-runners | — | github-runner-token | infra-compute, github-actions, buildbuddy |
| ci-build | — | — | github-actions, buildbuddy, infra-gar |
| ci-test | — | iap-oauth-client-id, iap-oauth-client-secret | ci-build, openai, anthropic, google-workspace |

---

## §7 Rate Limits & Retry Configs

### §7.1 GitHub Rate Limits (DM-20)

Source: `gunb.ai/tools/extdeps/github_api.go`

| Scope | Requests | Window | Notes |
|-------|----------|--------|-------|
| core | 5,000 | per hour | Most REST endpoints |
| search | 30 | per minute | **Must serialize search requests** |
| graphql | 5,000 | per hour | — |

**API versioning:** `2022-11-28`, header `X-GitHub-Api-Version`, supported until `2026-11-28`.
**Polling:** 60s min wait before first poll, 30s interval, 30min max duration.
**App auth:** JWT expiry 9min, clock skew tolerance 60s.
**Content limits:** 1MB API truncation, 10MB git clone threshold, 100MB LFS threshold.
**Gist limits:** 300 files max, 50MB recommended max payload.

### §7.2 GCP Rate Limits & Retry (DM-21)

Source: `gunb.ai/tools/extdeps/gcloud_api.go`

| Service | Read Rate | Write Rate | Max Concurrent |
|---------|-----------|------------|----------------|
| Secret Manager | 60,000/min | 6,000/min | 10 |
| IAM | — | — | 5 |
| Storage | — | — | 10 |
| Compute | — | — | 10 |
| Cloud Run | — | — | 5 |
| Cloud Functions | — | — | 3 |
| Artifact Registry | — | — | 10 |

**Retry configs:**

| Config | Max Retries | Initial Delay | Backoff Multiplier | Max Delay |
|--------|-------------|---------------|-------------------|-----------|
| default | 5 | 30s | 1.5x | 120s |
| api_enablement | 4 | (default) | (default) | (default) |
| iam_propagation | 5 | 60s | 1.5x | 180s |

**Propagation delays:**
- IAM policy changes: ~60s typical, up to 7 minutes worst case
- DNS changes: ~300s typical
- API enablement: ~30s typical

**GCP label constraints:** 63 char max key/value, 64 labels max per resource.
**IAM roles modeled:** SecretAccessor, StorageObjectViewer, StorageObjectCreator, ArtifactRegistryReader, ArtifactRegistryWriter.
**Org policy constraints:** `constraints/iam.allowedPolicyMemberDomains`, `constraints/storage.publicAccessPrevention`.

### §7.3 GitHub Product Tiers (DM-20)

Source: `gunb.ai/tools/extdeps/proto/github_api.proto`

```
GitHubProductTier:
  FREE_PRO_TEAM      — api.github.com
  ENTERPRISE_CLOUD   — api.github.com (different features)
  ENTERPRISE_SERVER  — {hostname}/api/v3 (version-dependent)
```

Tier from env: `GITHUB_PRODUCT_TIER`, `GITHUB_API_BASE`, `GITHUB_ENTERPRISE_VERSION`.

---

## §8 Git CLI (reference for DM-15/DM-16 patterns)

Source: `the-gunbai/docs/understandings/git.md`

**13 operations** (full behavioral detail):

| Operation | Invocation | Side Effects | Idempotent | Deterministic | Edge Cases |
|-----------|-----------|-------------|------------|---------------|------------|
| status | `git status --porcelain=v1` | ReadOnly | Yes | Yes | Use `--porcelain=v1` for stable machine parsing. Format: `XY PATH`. |
| diff | `git diff [--staged]` | ReadOnly | Yes | Yes | Empty diff returns empty string, not error |
| log | `git log -n <count> --format=<format>` | ReadOnly | Yes | Yes | — |
| current_branch | `git rev-parse --abbrev-ref HEAD` | ReadOnly | Yes | Yes | Returns `HEAD` in detached HEAD state |
| ls_files | `git ls-files [<path>]` | ReadOnly | Yes | Yes | Only committed or staged files |
| check_ignore | `git check-ignore <paths>` | ReadOnly | Yes | Yes | Evaluates all .gitignore files in hierarchy |
| add | `git add <paths>` | WritesState | Yes | — | Adding a deleted file stages the deletion |
| commit | `git commit -m <msg>` | WritesState | Yes (keys: message) (MediumConfidence) | — | Fails when nothing staged. Pre-commit hooks can modify staged content or abort. |
| checkout | `git checkout <ref>` | WritesState | Yes | — | Fails when uncommitted changes conflict. Detached HEAD when checking out commit. |
| create_branch | `git branch <name>` / `checkout -b <name>` | WritesState | Yes (keys: name) (MediumConfidence) | — | Fails when branch already exists |
| push | `git push [-u] [<remote>] [<branch>]` | WritesState | Yes | — | Fails when remote has commits not in local. **Force push can lose commits.** |
| pull | `git pull` | WritesState | **No** | **No** | Can create merge conflicts. May create merge commit for fast-forward. |
| fetch | `git fetch [<remote>]` | WritesState | Yes | — | — |

**Dependencies:** `git` CLI installed, `.gitignore`
**Assumptions:** Git 2.20+, repo initialized, credentials configured
**Unknowns:** `git gc` under concurrent ops, performance for >1GB repos, LFS interaction

---

## §9 Devcontainers (DM-18)

Source: `gunb.ai/tools/extdeps/proto/devcontainer.proto`

### Lifecycle Hooks

| Hook | Execution Context | Runs As | Timing |
|------|-------------------|---------|--------|
| initializeCommand | Host machine | Host user | Before container creation |
| onCreateCommand | Inside container | Root or remoteUser | After container creation, first time only |
| postCreateCommand | Inside container | remoteUser | After onCreateCommand, first time only |
| postStartCommand | Inside container | remoteUser | Every container start |
| postAttachCommand | Inside container | remoteUser | Every VS Code attach |

### Environment Variables (injected by devcontainer runtime)

Key env vars: `CODESPACES`, `GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN`,
`CODESPACE_NAME`, `GITHUB_TOKEN`, `GITHUB_USER`, `GITHUB_REPOSITORY`.

### Quirks

- postCreateCommand runs **before** extensions are installed
- Port forwarding may not be ready during lifecycle hooks
- Extensions listed in devcontainer.json are installed asynchronously
- Git credentials are forwarded from host but may expire during long sessions
