# End-to-End Service Codegen from DSL

Status: Draft
Owner: `gunbc` codegen/daglang
Scope: Generating all service transport infrastructure from `.dag` service definitions
Upstream: `dsl/services/**/*.dag` (source of truth)
Downstream: `gunbc-dag/src/resolve.rs`, `core/ir/src/transport/`, `lib/*-ops/`

## 0. Document Contract

1. This document is the normative source for the service codegen pipeline design.
2. It supersedes no existing design docs — it covers a new capability.
3. Existing `.dag` service definitions are the source of truth for service contracts.
   Generated code must be mechanically equivalent to what those files declare.

## 1. Problem Statement

The `.dag` service definitions already declare everything needed to build and parse
REST/shell transport requests: endpoint URL, HTTP method, path template, input fields
with types, output fields with JSON mappings, auth scheme, permissions, and mock
responses.

Despite this, every service operation today requires hand-written Rust across three
locations:

1. **IR transport types** (`core/ir/src/transport/{service}.rs`): Request/response
   structs, `to_rest_request()`, `credential_intent()`, response parsers. ~270 lines
   per service (gist), ~2800 lines for LLM (multi-provider).

2. **Resolver adapters** (`gunbc-dag/src/resolve.rs`): A `ServiceXxxPrepareOp` and
   `ServiceXxxParseOp` struct per operation, each implementing `Executable`. Currently
   ~800 lines of boilerplate spanning 12 service operations.

3. **Ops library SubDag builders** (`lib/{service}-ops/`): Credential chain wiring,
   transport triplet composition, composite op enums. ~500 lines of structural
   boilerplate per service.

### 1.1 Current State

The daglang compiler *already* does two things:

- **Parses** `service` blocks in `.dag` files (syntax, typecheck, resolve)
- **Lowers** service calls in `func` bodies to transport triplets in the GraphIR:
  `prepare_transport_{suffix}` → `execute_transport_{suffix}` → `parse_transport_{suffix}`

What it does *not* do is generate the Rust `Executable` implementations that back those
triplet nodes. Instead, `resolve.rs` hand-matches each
`"service_transport::prepare::{Service}::{Op}"` string to a hand-written struct.

### 1.2 Consequences

| Consequence | Impact |
|-------------|--------|
| Adding a new REST service requires ~10 files and ~1500 lines of Rust | Adoption friction |
| The `.dag` file says "Replaces: ..." but doesn't actually replace | Semantic drift |
| IR types duplicate what `.dag` already declares | Maintenance burden |
| Each prepare/parse adapter follows identical patterns | Wasted engineering time |
| New services (LLM, Dropbox, etc.) must reimplement the same scaffold | Not scalable |

### 1.3 Goal

After this work, adding a new REST service (e.g., Dropbox file upload) requires:

1. One `.dag` service definition (~30 lines)
2. One `.dag` tool definition (~60 lines)
3. Domain-specific pure functions in Rust (only if non-trivial logic exists)
4. `make install` (codegen generates everything else)

The hand-written Rust-per-service drops from ~1500 lines to ~0 for standard REST
services, and to domain-logic-only for complex ones (LLM multi-provider dispatch).

## 2. Architecture

### 2.1 Current Pipeline (DSL → Execution)

```
.dag service definition
  │
  ├──[daglang-syntax]──→ ServiceDef AST node
  │
  ├──[daglang-typecheck]──→ validated types
  │
  ├──[daglang-lower]──→ transport triplets in GraphIR (LoweredOp::Callable)
  │                       prepare: "service_transport::prepare::github.Gist::Create"
  │                       execute: "service_transport::execute::github.Gist::Create"
  │                       parse:   "service_transport::parse::github.Gist::Create"
  │
  ├──[resolve.rs]──→ DynOp (hand-written ServiceGistCreatePrepareOp, etc.)
  │                   ▲▲▲ THIS IS THE GAP ▲▲▲
  │
  └──[exec]──→ runtime execution
```

### 2.2 Target Pipeline

```
.dag service definition
  │
  ├──[daglang-syntax]──→ ServiceDef AST node
  │
  ├──[daglang-typecheck]──→ validated types
  │
  ├──[daglang-lower]──→ transport triplets in GraphIR
  │
  ├──[daglang-emit / service-codegen]──→ generated Rust for prepare/parse ops
  │     NEW: reads ServiceDef metadata, emits Executable impls
  │
  ├──[resolve.rs]──→ DynOp via generated lookup table (no hand-written adapters)
  │
  └──[exec]──→ runtime execution
```

### 2.3 What Gets Generated

For each `service` block with each `operation`:

**Generated prepare adapter** (from `@rest(METHOD, "/path/{param}")` + `input { ... }`):
```rust
// AUTO-GENERATED from dsl/services/github/gist.dag — do not edit
struct PrepareGithubGistCreate;

impl Executable for PrepareGithubGistCreate {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let description = inputs.get("description").and_then(Value::as_str).unwrap_or("");
        let files = /* extract Map<String,String> from inputs */;
        let public = inputs.get("public").and_then(Value::as_bool).unwrap_or(false);

        let body = serde_json::json!({
            "description": description,
            "files": files_to_json(files),
            "public": public,
        });

        OutputMap::new()
            .request("request", TransportRequest::Rest(
                RestRequest::post("https://api.github.com/gists").json(body)
            ))
            .ok()
    }
}
```

**Generated parse adapter** (from `output { url: Url @json("html_url"), id: GistId }`):
```rust
struct ParseGithubGistCreate;

impl Executable for ParseGithubGistCreate {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Rest(rest))) => {
                if !rest.is_success() {
                    return Err(ExecError::new(format!("github.Gist.Create failed ({})", rest.status)));
                }
                let url = rest.body.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
                let id = rest.body.get("id").and_then(|v| v.as_str()).unwrap_or("");
                OutputMap::new().str("url", url).str("id", id).ok()
            }
            Some(Value::Skipped) | None => OutputMap::new().str("url", "").str("id", "").ok(),
            Some(other) => Err(ExecError::new(format!(
                "expected REST response for github.Gist.Create, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}
```

**Generated resolver entry** (replaces hand-written match arms in `resolve_service_transport`):
```rust
fn resolve_service_transport_generated(name: &str) -> Option<DynOp> {
    match name {
        "service_transport::prepare::github.Gist::Create" => Some(DynOp::new(PrepareGithubGistCreate)),
        "service_transport::parse::github.Gist::Create" => Some(DynOp::new(ParseGithubGistCreate)),
        // ... all other services ...
        _ => None,
    }
}
```

## 3. Service Transport Classes

The codegen must handle all transport classes declared in `.dag` files:

### 3.1 REST Services (`@rest`)

The simplest and highest-value target. Input declared via `@rest(METHOD, "/path")`.

**Codegen contract:**

| DSL Annotation | Generated Rust |
|----------------|----------------|
| `@endpoint("https://api.example.com")` | Base URL constant |
| `@rest(POST, "/gists")` | `RestRequest::post(url)` |
| `@rest(GET, "/v1/projects/{project}/secrets/{secret}/...")` | URL interpolation from inputs |
| `input { field: Type }` | Extract field from `inputs` HashMap |
| `input { field: Type = default }` | Extract with fallback to default |
| `output { field: Type @json("json_key") }` | Extract from `rest.body.get("json_key")` |
| `output { field: Type }` | Extract from `rest.body.get("field")` (name = json key) |
| `output { field: Secret }` | Wrap in `SecretString` |
| `@auth(BearerToken)` | Auth metadata for credential chain |
| `@auth(AwsSigV4)` | AWS SigV4 signing metadata |
| `@mock_response(status: N, body: {...})` | Mock spec generation |

**Path interpolation rule:** `{param}` in path is replaced by the value of the input
field named `param`. The field must be declared in `input { ... }`.

**Body construction rule:** For POST/PUT/PATCH, all non-path input fields become JSON
body fields. For GET/DELETE, non-path fields become query parameters.

### 3.2 Shell Services (`@shell`)

Input is a CLI command. Slightly more varied than REST.

**Codegen contract:**

| DSL Pattern | Generated Rust |
|-------------|----------------|
| CLI binary + subcommand | `ShellRequest::new(binary).args(subcommand)` |
| Input fields → CLI args | Positional or flag-based (needs annotation) |
| Output parsing | Stdout line splitting, exit code check |

Shell services have more variation in how inputs map to CLI arguments. Two strategies:

1. **Annotated mapping** (preferred): DSL declares `@cli_arg("--flag")` per input field
2. **Convention-based**: For simple services (find, which), codegen uses positional args

### 3.3 File Services (`@file`)

Direct filesystem read/write. Already handled by infrastructure resolution in
`resolve.rs` (the `content_upsert::` pattern). No new codegen needed — these are
cross-module patterns, not per-service.

## 4. Handling Service Complexity Tiers

Not all services are equal. The codegen must handle a spectrum from trivial to complex.

### 4.1 Tier 1: Pure REST (fully generatable)

Services where the `.dag` declaration is the complete specification.

**Examples:** github.Gist, gcp.SecretManager, gcp.IAM, gcp.STS

**What codegen produces:** Complete prepare + parse adapters. Zero hand-written Rust.

**Characteristics:**
- Single endpoint per operation
- JSON body from input fields
- JSON response parsed to output fields
- Standard auth (Bearer or header-based)

### 4.2 Tier 2: REST with Custom Body Shape

Services where the body construction doesn't follow the simple "all inputs → JSON
fields" rule.

**Examples:** gcp.STS.Exchange (OAuth2 grant_type fields), AWS services (X-Amz-Target
header + JSON protocol)

**What codegen produces:** Prepare adapter with a body template. Parse adapter generated.

**DSL extension needed:** Body template annotation:
```
operation Exchange {
    input { subject_token: Secret, audience: String }
    output { access_token: Secret, expires_in: Int }
    @rest(POST, "/v1/token")
    @body_template({
        "audience": audience,
        "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",
        "requested_token_type": "urn:ietf:params:oauth:token-type:access_token",
        "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
        "subject_token": subject_token,
    })
}
```

This lets the DSL declare literal constants alongside input field references. Codegen
emits the JSON construction with the right mix of constants and extracted values.

### 4.3 Tier 3: Multi-Provider Dispatch (partially generatable)

Services that dispatch to different endpoints/formats based on a runtime input.

**Examples:** LLM (OpenAI vs Anthropic — different endpoints, request shapes, response
shapes, auth schemes)

**What codegen produces:** Per-provider prepare/parse adapters. Hand-written dispatch
function.

**DSL extension needed:** Provider variants:
```
service llm.ChatCompletion {
    variant openai {
        @endpoint("https://api.openai.com")
        @auth(BearerToken)
        // OpenAI-specific request/response shape
    }
    variant anthropic {
        @endpoint("https://api.anthropic.com")
        @auth(Header("x-api-key"))
        // Anthropic-specific request/response shape
    }
}
```

Each variant generates its own prepare/parse pair. The dispatch function can be either:
- Generated (if the variant selector is a simple input field)
- Hand-written (if dispatch logic is complex, e.g., model name → provider ID)

### 4.4 Tier 4: Domain-Specific Logic (codegen + escape hatch)

Services where prepare or parse involves non-trivial business logic beyond
field extraction/construction.

**Examples:**
- Gist: filename generation with branch sanitization, timestamp formatting
- LLM: content block assembly, thinking config, cache hints
- Review: structured finding extraction from LLM response, stable hashing

**What codegen produces:** The transport scaffold (request building, response field
extraction). Domain logic stays in hand-written Rust, invoked via a hook.

**Pattern: `@prepare_hook` / `@parse_hook`:**
```
operation Create {
    input { description: String, files: Map<String, String>, public: Bool = false }
    output { url: Url @json("html_url"), id: GistId }
    @rest(POST, "/gists")
    @prepare_hook("gist_ops::prepare_gist_request")
}
```

When a hook is declared, codegen generates:
1. The standard adapter skeleton (input extraction, request building, output mapping)
2. A call to the hook function at the appropriate point
3. The hook function signature is derived from input/output types

This keeps the scaffold generated while allowing arbitrary Rust logic where needed.

## 5. Credential Chain Generation

The credential chain is the most repetitive cross-service boilerplate. Today,
`build_gist_upload_subdag()` is ~280 lines of SubDag wiring for a pattern that's
identical across services except for:

- `secret_name` (e.g., "github-token" vs "openai-api-key")
- `service` identifier (e.g., "github" vs "openai")
- `scheme` (e.g., "bearer" vs "header")
- `header_name` (empty for bearer, "x-api-key" for Anthropic)
- `required_scopes` (e.g., ["gist:write"] vs ["llm:chat_completion"])

### 5.1 Current State

The `credential_chain()` DSL pattern in `dsl/std/patterns.dag` already parameterizes
all of this. But the *Rust-side* SubDag builders (`build_gist_upload_subdag`,
`build_chat_completion_graph`) still hand-wire the credential chain in Rust.

### 5.2 Target

Services that use `credential_chain()` in their `.dag` tool definition get the
credential SubDag generated from the pattern expansion. The DSL already declares:

```
credential_chain(
    runtime: detect_runtime(),
    audience: "sigstore",
    project: "gunbc",
    secret_name: "github-token",
    source_id: "gist",
    required_scopes: ["gist"],
    scheme: Bearer,
)
```

This should generate the full SubDag: resolve_auth → bind_secret → cloud_credential →
scope_preflight, with all edges wired from the declared parameters.

### 5.3 Scope Contract Generation

From the `.dag` service definition:
```
service github.Gist {
    @auth(BearerToken)
    operation Create { @permissions(["gist"]) }
}
```

Codegen can emit:
```rust
fn gist_create_credential_intent() -> CredentialIntent {
    CredentialIntent::new("github", "github", "bearer")
        .with_required_scopes(["gist"])
        .with_interactive_allowed(true)
}
```

This replaces `GistScopeContract`, `GistScope` enum, `GITHUB_SECRET_ID` constant,
and the entire `impl ScopeContract` block — all derivable from the `.dag` annotations.

## 6. Upcoming Service Cases

### 6.1 LLM Services (In Progress)

The LLM integration is the most complex service in the codebase (~8400 lines across
IR, ops, and graph). It exemplifies Tier 3/4:

**What's generatable:**
- Per-provider REST request building (endpoint, method, headers)
- Per-provider response parsing (field extraction)
- Scope contracts (per provider)
- Credential chain SubDag wiring
- Mock specs from `@mock_response` annotations

**What stays hand-written:**
- `ChatRequest`/`ChatResponse` unified types (cross-provider abstraction)
- Content block assembly (text, cache hints, thinking blocks)
- Multi-turn thinking block preservation
- Provider dispatch logic (model → provider ID mapping)
- Extended thinking budget configuration

**DSL representation:**
```
service llm.ChatCompletion {
    variant openai {
        @endpoint("https://api.openai.com")
        @auth(BearerToken)
        operation Complete {
            input { model: String, messages: List<Message>, temperature: Float? }
            output { content: String, model: String, finish_reason: String }
            @rest(POST, "/v1/chat/completions")
        }
    }
    variant anthropic {
        @endpoint("https://api.anthropic.com")
        @auth(Header("x-api-key"))
        operation Complete {
            input { model: String, messages: List<Message>, max_tokens: Int }
            output { content: String, model: String, stop_reason: String }
            @rest(POST, "/v1/messages")
            @headers({ "anthropic-version": "2023-06-01" })
        }
    }
}
```

**Migration plan:** Incrementally replace IR types with generated code:
1. Generate per-provider request builders (replaces `openai.rs::build_openai_request`,
   `anthropic.rs::build_anthropic_request`)
2. Generate per-provider response parsers (replaces `openai.rs::parse_openai_response`,
   `anthropic.rs::parse_anthropic_response`)
3. Generate scope contracts (replaces `LlmScopeContract`, `LlmScope` enum)
4. Keep unified `ChatRequest`/`ChatResponse` as hand-written domain types
5. Keep provider dispatch as hand-written (uses domain types)

### 6.2 File Storage Services (Future)

If switching from gist to Dropbox or S3 for file uploads:

**Dropbox example:**
```
service dropbox.Files {
    @endpoint("https://content.dropboxapi.com")
    @auth(BearerToken)

    operation Upload {
        input {
            path: String
            content: Bytes
            mode: String = "add"
        }
        output {
            id: String @json("id")
            path_display: String @json("path_display")
            rev: String @json("rev")
        }
        @rest(POST, "/2/files/upload")
        @headers({ "Dropbox-API-Arg": "{\"path\": \"{path}\", \"mode\": \"{mode}\"}" })
        @body_raw(content)
        @permissions(["files.content.write"])
    }
}
```

With e2e codegen, adding Dropbox requires:
1. This `.dag` definition (~25 lines)
2. A `.dag` tool definition wiring `dropbox.Files.Upload` to content acquisition (~30 lines)
3. Zero Rust (unless domain-specific logic is needed)

### 6.3 Infrastructure Services (AWS, Azure, GCP)

The `dsl/infra/` directory already defines ~20 infrastructure services (S3, Lambda,
SecretManager, KeyVault, etc.). These are currently DSL-only with no Rust backing.

With e2e codegen, all of these become immediately executable — the codegen generates
prepare/parse adapters from their `.dag` definitions.

## 7. Implementation Plan

### Phase 1: Service Metadata Extraction

**Goal:** Extract structured service metadata from the daglang compiler's existing AST.

The compiler already parses `ServiceDef` nodes and derives `ServiceCallMetadata`
(transport class, permissions, idempotent/readonly flags). Extend this to also extract:

- Endpoint URL (from `@endpoint`)
- HTTP method and path template (from `@rest`)
- Input field names, types, and defaults
- Output field names, types, and JSON mappings (from `@json`)
- Auth scheme (from `@auth`)
- Mock response (from `@mock_response`)
- Body template (from `@body_template`, new annotation)
- Custom headers (from `@headers`, new annotation)
- Prepare/parse hooks (from `@prepare_hook`/`@parse_hook`, new annotations)

**Output:** `ServiceOperationSpec` struct capturing all of the above, available after
typecheck.

### Phase 2: Prepare/Parse Codegen

**Goal:** Generate `Executable` impls for each service operation.

Two emission strategies:

**Strategy A — Static code generation (preferred for resolver):**
- A new codegen pass reads all `ServiceOperationSpec` from the compiled project
- Emits a Rust source file (`target/codegen/service_transport.rs`) with:
  - One prepare struct + `impl Executable` per operation
  - One parse struct + `impl Executable` per operation
  - A dispatch function `resolve_service_transport_generated(name: &str) -> Option<DynOp>`
- `resolve.rs` calls the generated dispatch function before falling through to
  hand-written adapters (gradual migration)

**Strategy B — Dynamic dispatch (preferred for DSL-native execution):**
- The lowered `LoweredOp` carries the full `ServiceOperationSpec`
- A generic `ServicePrepareOp` and `ServiceParseOp` take the spec at construction time
- No per-service Rust structs needed — one generic impl handles all Tier 1/2 services
- The spec is the configuration; the impl is the interpreter

**Recommendation:** Start with Strategy B (simpler, no codegen step) and optionally
add Strategy A later for performance if needed (static dispatch vs dynamic).

### Phase 3: Credential Chain Codegen

**Goal:** Generate credential SubDag wiring from `credential_chain()` pattern calls.

The DSL `credential_chain()` call in `.dag` files already declares all parameters.
When the pattern expander resolves this call, it should produce a SubDag in the IR
that the resolver can execute without hand-written Rust.

This requires:
1. The pattern expander emits concrete SubDag nodes (it already partially does this)
2. The resolver handles pattern-expanded nodes generically (auth resolve, bind secret,
   cloud credential, scope preflight)
3. The auth resolver reads the declared `@auth` + `@permissions` from the service spec
   instead of hardcoding `GistScopeContract`

### Phase 4: IR Type Elimination

**Goal:** Remove hand-written IR transport types for services that are fully covered.

For each service that's 100% generated:
1. Delete `core/ir/src/transport/{service}.rs`
2. Delete the corresponding `ScopeContract` impl
3. Delete the hand-written `resolve_service_transport` match arms
4. The `.dag` file becomes the single source of truth with no Rust shadow

**Ordering (by risk):**
1. `gcp.STS.Exchange` — simplest REST, already in resolve.rs
2. `gcp.SecretManager.AccessVersion` — simple REST with path interpolation
3. `shell.Find.ListDirs` — simple shell service
4. `shell.Codegen.{Check,Run}` — shell services with fixed commands
5. `cargo.Build.{Build,Test,Clippy}` — shell services with input-dependent args
6. `github.Gist.Create` — REST with `@body_template` needed
7. LLM providers — Tier 3, partial generation only

### Phase 5: DSL Extensions

New annotations needed (in order of implementation):

1. **`@body_template({...})`** — Declare literal + interpolated JSON body (Phase 1)
2. **`@headers({...})`** — Declare extra HTTP headers (Phase 1)
3. **`@body_raw(field)`** — Send input field as raw body, not JSON (Phase 2)
4. **`@prepare_hook("module::function")`** — Delegate to hand-written Rust (Phase 2)
5. **`@parse_hook("module::function")`** — Delegate to hand-written Rust (Phase 2)
6. **`variant name { ... }`** — Multi-provider service blocks (Phase 3)
7. **`@cli_arg("--flag")`** — Shell service input-to-arg mapping (Phase 3)

### Phase 6: Mock Spec Generation

**Goal:** Generate mock specs from `@mock_response` annotations.

The `.dag` files already declare mock responses:
```
@mock_response(status: 201, body: { "html_url": "https://...", "id": "..." })
```

Codegen can emit the `MockSpec` boundary mock for each service operation,
eliminating the hand-written `graph_mock.rs` for service transport boundaries.

## 8. Migration Strategy

### 8.1 Gradual Cutover

The generated dispatch is tried *before* the hand-written fallback:

```rust
fn resolve_service_transport(node_id: &str, module: &str, name: &str) -> Result<DynOp, ResolveError> {
    // Try generated adapters first
    if let Some(op) = resolve_service_transport_generated(name) {
        return Ok(op);
    }
    // Fall through to hand-written adapters (shrinks over time)
    // ...existing match arms...
}
```

This means:
- New services are immediately generated (no Rust needed)
- Existing services can be migrated one at a time
- Each migration is verifiable: run the tool, compare output

### 8.2 Verification

For each migrated service operation:

1. **Structural test:** Generated prepare output matches hand-written output for
   identical inputs (property test across input space)
2. **Integration test:** `make {tool} --dry-run` produces identical mock execution log
3. **Live test:** If live test infrastructure exists, run against real API

### 8.3 Deletion Criteria

A hand-written adapter can be deleted when:
1. The generated adapter passes all existing tests
2. The dry-run output is byte-identical
3. A live smoke test passes (if applicable)
4. The `.dag` file's `// Replaces:` comment is honored

## 9. Cross-Cutting Concerns

### 9.1 Error Messages

Generated parse adapters must produce error messages that include:
- Service name and operation
- HTTP status code (for REST)
- Response body excerpt (for debugging)
- The source `.dag` file and line number (via `#[track_caller]` or embedded metadata)

### 9.2 Secret Handling

Input fields typed as `Secret` must:
- Be extracted via `SecretString::new()` wrapping
- Never appear in error messages or logs
- Use `Value::Secret` in the output map, not `Value::Str`

### 9.3 Auth Header Injection

For REST services, the `execute` node (TransportOps::Execute) already handles
credential injection. The prepare adapter only needs to indicate *which* auth scheme
is expected — it doesn't attach the token itself. This is already correct.

### 9.4 Backward Compatibility

During migration, the same tool must work regardless of whether the generated or
hand-written adapter is active. The verification tests (Section 8.2) enforce this.

## 10. Impact Assessment

### 10.1 Lines of Code

| Component | Current (hand-written) | After codegen | Delta |
|-----------|----------------------|---------------|-------|
| `resolve.rs` service adapters | ~800 | ~50 (dispatch stub) | -750 |
| `core/ir/src/transport/gist.rs` | ~270 | 0 (deleted) | -270 |
| `core/ir/src/transport/llm/` | ~2800 | ~1200 (domain types only) | -1600 |
| `lib/gist-ops/` SubDag builder | ~280 | 0 (generated) | -280 |
| `lib/llm-ops/` SubDag builder | ~267 | 0 (generated) | -267 |
| New: codegen pass | 0 | ~400 | +400 |
| New: DSL annotations | 0 | ~100 | +100 |
| **Net** | **~4415** | **~1750** | **-2665** |

### 10.2 Developer Experience

| Scenario | Before | After |
|----------|--------|-------|
| Add Tier 1 REST service | ~10 files, ~1500 lines Rust | 1 `.dag` file, ~30 lines |
| Add Tier 3 multi-provider service | ~15 files, ~3000 lines | 1 `.dag` file + domain types |
| Change endpoint URL | Edit Rust, recompile | Edit `.dag`, `make install` |
| Add field to existing operation | Edit 3 Rust files | Edit `.dag` definition |
| Add new auth scheme | Implement trait, wire everywhere | Add `@auth(NewScheme)` support |

### 10.3 Compile Time

Strategy B (dynamic dispatch) adds zero to compile time — the `ServiceOperationSpec`
is a runtime struct, not generated code. Strategy A adds one generated Rust file
per codegen pass, which is fast to compile.

## 11. Relationship to Existing Work

### 11.1 Workflow Minimization (Sprint 5/5b)

The service codegen is *orthogonal* to workflow minimization but enables it:
- WF16 (gist base workflow): The credential SubDag codegen (Phase 3) directly
  supports the "minimize credentialing" capability requirement
- WF14 (compilation): Pre-built binary dispatch is separate from service codegen
- WF15 (codegen freshness): The codegen pass itself becomes a keyed unit in the
  minimization ledger

### 11.2 Unified Registration (docs/design/unified-registration.md)

Service codegen feeds into the tool registry. Generated adapters are discovered
the same way as hand-written ones — via `resolve.rs` dispatch.

### 11.3 Test Generation (docs/design/testgen.md)

Generated mock specs from `@mock_response` annotations integrate with the existing
proof obligation model. Each generated adapter gets Bucket A (execution semantics)
tests automatically.

## 12. Open Questions

1. **Strategy A vs B:** Should we generate static Rust code or use a generic interpreter?
   Strategy B is simpler but may have performance implications for hot-path services.
   For the current service count (~12 operations), this is negligible.

2. **Shell service input mapping:** Shell services have more varied input-to-arg patterns
   than REST services. Do we need `@cli_arg` annotations, or can we use a convention
   (e.g., inputs become positional args in declaration order)?

3. **AWS SigV4:** This auth scheme requires request signing, not just header injection.
   Does the `execute` boundary handle this, or does the prepare adapter need to sign?

4. **Body template vs body_raw:** For services like Dropbox where the body is raw bytes
   (not JSON), we need `@body_raw`. Is this sufficient, or do we need a more general
   content-type system?
