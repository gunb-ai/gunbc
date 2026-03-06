# End-to-End Service Codegen from DSL

Status: Draft
Owner: `gunbc` codegen/daglang
Scope: Bottom-up transport interface modeling so all services are pure DSL

## 0. Document Contract

1. This document is the normative source for service interface modeling and codegen.
2. `.dag` service definitions are the sole source of truth for service contracts.
3. No hand-written Rust per-service. The Rust layer implements *protocol interfaces*,
   not individual services.

## 1. Problem Statement

The DSL already declares 7 service files with 19 operations across 3 transport classes.
Every operation specifies its endpoint, method, path, input/output types, auth scheme,
permissions, and mock responses. The daglang compiler already parses these and lowers
service calls to `prepare → execute → parse` transport triplets in the GraphIR.

Despite this, `resolve.rs` contains 12 hand-written `Executable` adapter structs — one
per service operation — that do mechanically identical work. Each REST prepare adapter:
extracts inputs, interpolates URL, builds JSON body, returns `TransportRequest::Rest`.
Each REST parse adapter: checks status, extracts JSON fields, returns outputs.

The problem is that these adapters are written *per service* when they should be written
*per transport class*. There are exactly 3 protocol interfaces (REST, Shell, File), and
every service operation is an instance of one of them.

### 1.1 What Exists Today

**Compiler pipeline (works):**
```
.dag service definition
  → [syntax] ServiceDef AST
  → [typecheck] validated types
  → [lower] transport triplets (prepare/execute/parse nodes in GraphIR)
```

**Resolution gap (the problem):**
```
resolve_service_transport() matches on service name strings:
  "service_transport::prepare::github.Gist::Create"  → ServiceGistCreatePrepareOp
  "service_transport::prepare::gcp.STS::Exchange"     → ServiceGcpStsExchangePrepareOp
  "service_transport::parse::gcp.STS::Exchange"       → ServiceGcpStsExchangeParseOp
  ... (12 structs, ~800 lines)
```

Each of these structs does the same thing parameterized by different data that's already
in the `.dag` file.

### 1.2 Goal

After this work:

1. The resolver has **3 generic executables** (one per transport class), not N per-service
   executables. Adding a service means adding a `.dag` file — nothing else.
2. All existing services (`resolve.rs` adapters, `core/ir/src/transport/gist.rs`,
   scope contracts, etc.) are deleted and replaced by DSL definitions.
3. New services (LLM, Dropbox, Slack, any REST API) are defined entirely in DSL.

## 2. Interface Hierarchy

The core insight: model bottom-up from transport primitive to application service.

### Layer 0: Transport Primitives (exists today)

```
TransportRequest  = Shell(ShellRequest) | Rest(RestRequest) | File(FileRequest)
TransportResponse = Shell(ShellResponse) | Rest(RestResponse) | File(FileResponse)
TransportOps::Execute  — the boundary that sends a request and receives a response
```

These are correct and don't change.

### Layer 1: Protocol Interfaces (new — this is what we build)

A protocol interface defines **how to prepare a request** and **how to parse a
response** for a class of transport. The DSL annotations map directly to protocol
interface parameters.

#### REST Protocol

Every REST operation follows one pattern:

```
prepare(inputs, spec) → TransportRequest::Rest:
  1. Interpolate path template:  "/v1/projects/{project}/secrets/{secret}"
     with input field values:    project="my-proj", secret="my-secret"
  2. Build URL:                  endpoint + interpolated_path
  3. Build body (POST/PUT/PATCH): JSON from non-path input fields
     or query params (GET/DELETE): from non-path input fields
  4. Return:                     RestRequest::method(url).json(body)

parse(response, spec) → outputs:
  1. Check status:               response.is_success()
  2. Extract fields:             response.body.get(json_path) for each output field
  3. Wrap secrets:               SecretString for Secret-typed outputs
  4. Return:                     OutputMap with extracted values
```

DSL annotations that parameterize this:

| Annotation | Maps to |
|---|---|
| `@endpoint("https://...")` | Base URL |
| `@rest(METHOD, "/path/{param}")` | HTTP method + path template |
| `input { field: Type = default }` | Fields to extract from inputs |
| `output { field: Type @json("key") }` | Fields to extract from response |
| `@auth(BearerToken \| Header("name"))` | Auth metadata (for execute boundary) |
| `@body_template({...})` | Explicit body shape (when not all-inputs-as-JSON) |
| `@headers({...})` | Extra HTTP headers |
| `@mock_response(...)` | Test mock |

**One Rust struct handles all REST operations:**

```rust
struct RestPrepareOp {
    spec: RestOperationSpec,  // endpoint, method, path_template, input_fields, body_template, headers
}

impl Executable for RestPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let url = self.spec.interpolate_url(&inputs)?;
        let body = self.spec.build_body(&inputs)?;
        let request = match self.spec.method {
            Method::GET    => RestRequest::get(url),
            Method::POST   => RestRequest::post(url).json(body),
            Method::PUT    => RestRequest::put(url).json(body),
            Method::DELETE => RestRequest::delete(url),
            // ...
        };
        let request = self.spec.apply_headers(request);
        OutputMap::new().request("request", TransportRequest::Rest(request)).ok()
    }
}
```

```rust
struct RestParseOp {
    spec: RestOutputSpec,  // output_fields: Vec<{name, json_path, type, is_secret}>
    service_label: String, // for error messages: "github.Gist.Create"
}

impl Executable for RestParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Rest(rest))) => {
                if !rest.is_success() {
                    return Err(ExecError::new(format!(
                        "{} failed (status {})", self.service_label, rest.status
                    )));
                }
                let mut out = OutputMap::new();
                for field in &self.spec.fields {
                    let value = rest.body.pointer(&field.json_path)
                        .and_then(|v| field.extract(v));
                    out = out.value(&field.name, value.unwrap_or(field.default_value()));
                }
                out.ok()
            }
            Some(Value::Skipped) | None => {
                // Produce typed defaults for all output fields
                self.spec.default_outputs()
            }
            Some(other) => Err(ExecError::new(format!(
                "expected REST response for {}, got {:?}",
                self.service_label, std::mem::discriminant(other)
            ))),
        }
    }
}
```

#### Shell Protocol

Every shell operation follows one pattern:

```
prepare(inputs, spec) → TransportRequest::Shell:
  1. Start with argv template:   ["cargo", "clippy", "--all-targets"]
  2. Interpolate {param}:        replace template variables with input values
  3. Append conditional args:    e.g., "--all-targets" only if all_targets=true
  4. Return:                     ShellRequest::new(binary).args(argv)

parse(response, spec) → outputs:
  1. Extract exit code:          response.success()
  2. Parse stdout:               split by newline (List), trim (String), or raw
  3. Return:                     OutputMap with (success, stdout, stderr) or custom fields
```

DSL annotations:

| Annotation | Maps to |
|---|---|
| `@shell(["binary", "arg1", "{param}", ...])` | Argv template with interpolation |
| `@hermetic` | No side effects outside process |
| `@readonly` | Read-only access |
| `input { field: Type }` | Values interpolated into argv |
| `output { success: Bool, stdout: String, ... }` | Parsing rules |

**One Rust struct handles all Shell operations:**

```rust
struct ShellPrepareOp {
    spec: ShellOperationSpec,  // argv_template, conditional_args
}

struct ShellParseOp {
    spec: ShellOutputSpec,  // output_fields with parsing rules (trim, split, raw)
    service_label: String,
}
```

#### File Protocol

Already handled by `content_upsert::` infrastructure patterns and `Filesystem`
resource capabilities in `dsl/std/resources.dag`. No per-service adapters needed.
The `@file(READ|WRITE, "{path}")` annotation on resource capabilities is resolved
by existing infrastructure resolution. No changes needed here.

### Layer 2: Application Services (existing .dag files — no changes needed)

Application services are just data that parameterizes Layer 1:

```
// This .dag file IS the complete service definition.
// No Rust backing needed — the REST protocol interface handles it.
service github.Gist {
    @endpoint("https://api.github.com")
    @auth(BearerToken)

    operation Create {
        input { description: String, files: Map<String, String>, public: Bool = false }
        output { url: Url @json("html_url"), id: GistId }
        @rest(POST, "/gists")
        @permissions(["gist"])
        @mock_response(status: 201, body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" })
    }
}
```

Every existing `.dag` service file already has exactly this shape. Nothing changes
in the DSL — what changes is that the resolver now interprets these specs generically
instead of matching them to hand-written Rust.

## 3. Spec Extraction: Compiler → Resolver

### 3.1 `ServiceOperationSpec`

The daglang lowerer already creates `ServiceCallMetadata` (transport class, permissions,
idempotent/readonly). We extend this to carry the full protocol spec:

```rust
/// Complete specification for a service operation, extracted from the .dag AST.
/// Carries all information needed for the generic protocol interpreters.
#[derive(Debug, Clone)]
pub enum ServiceOperationSpec {
    Rest(RestOperationSpec),
    Shell(ShellOperationSpec),
    File(FileOperationSpec),
}

#[derive(Debug, Clone)]
pub struct RestOperationSpec {
    pub endpoint: String,                    // from @endpoint
    pub method: HttpMethod,                  // from @rest(METHOD, ...)
    pub path_template: String,               // from @rest(..., "/path/{param}")
    pub input_fields: Vec<FieldSpec>,         // from input { ... }
    pub output_fields: Vec<OutputFieldSpec>,  // from output { ... @json(...) }
    pub body_template: Option<BodyTemplate>, // from @body_template({...}), if present
    pub headers: Vec<(String, String)>,       // from @headers({...})
    pub auth_scheme: AuthScheme,             // from @auth(...)
    pub permissions: Vec<String>,            // from @permissions([...])
    pub mock_response: Option<MockSpec>,     // from @mock_response(...)
}

#[derive(Debug, Clone)]
pub struct ShellOperationSpec {
    pub argv_template: Vec<ArgvSegment>,     // from @shell([...])
    pub input_fields: Vec<FieldSpec>,
    pub output_fields: Vec<OutputFieldSpec>,
    pub output_parsing: ShellOutputParsing,  // Trim, SplitLines, SuccessStdoutStderr
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: String,
    pub type_id: String,
    pub default: Option<DefaultValue>,
    pub is_secret: bool,        // type is Secret
    pub is_path_param: bool,    // appears in {param} in path/argv template
}

#[derive(Debug, Clone)]
pub struct OutputFieldSpec {
    pub name: String,
    pub type_id: String,
    pub json_path: String,      // from @json("key") or defaults to field name
    pub is_secret: bool,
}

/// Body template for REST operations with non-default body shapes.
/// Mixes literal JSON values with input field references.
#[derive(Debug, Clone)]
pub struct BodyTemplate {
    pub entries: Vec<BodyEntry>,
}

#[derive(Debug, Clone)]
pub enum BodyEntry {
    Literal(String, serde_json::Value),      // "grant_type": "urn:ietf:..."
    InputRef(String, String),                 // "audience": audience  (json_key, input_field)
}

#[derive(Debug, Clone)]
pub enum ArgvSegment {
    Literal(String),                          // "cargo", "--all-targets"
    InputRef(String),                         // "{package}"
    Conditional(String, String, String),       // if input_field, then flag, else nothing
}

#[derive(Debug, Clone)]
pub enum ShellOutputParsing {
    /// Single string: trim(stdout)
    TrimStdout,
    /// List of strings: split(trim(stdout), "\n")
    SplitLines,
    /// Standard triple: (success: Bool, stdout: String, stderr: String)
    SuccessStdoutStderr,
    /// Bool from exit code: success = exit_code == 0
    ExitCodeBool,
    /// Custom field extraction
    Custom(Vec<OutputFieldSpec>),
}
```

### 3.2 How the Spec Travels

```
[daglang-lower]
  ├── reads ServiceDef AST node
  ├── extracts ServiceOperationSpec from annotations + fields
  └── attaches spec to LoweredOp::Callable.service_metadata
        (currently ServiceCallMetadata; extend to include full spec)

[resolve.rs]
  ├── receives LoweredOp with attached spec
  └── constructs RestPrepareOp(spec) or ShellPrepareOp(spec)
        (no per-service matching — dispatch on transport class only)
```

### 3.3 Resolver Changes

The entire `resolve_service_transport()` function (currently ~80 lines of per-service
match arms) becomes:

```rust
fn resolve_service_transport(
    node_id: &str,
    _module: &str,
    name: &str,
    spec: &ServiceOperationSpec,
) -> Result<DynOp, ResolveError> {
    if name.starts_with("service_transport::execute::") {
        return Ok(DynOp::new(TransportOps::Execute));
    }

    let is_prepare = name.starts_with("service_transport::prepare::");
    let is_parse = name.starts_with("service_transport::parse::");

    match spec {
        ServiceOperationSpec::Rest(rest_spec) => {
            if is_prepare {
                Ok(DynOp::new(RestPrepareOp { spec: rest_spec.clone() }))
            } else if is_parse {
                Ok(DynOp::new(RestParseOp {
                    spec: rest_spec.output_spec(),
                    service_label: extract_service_label(name),
                }))
            } else {
                Err(unknown_callable(node_id, _module, name))
            }
        }
        ServiceOperationSpec::Shell(shell_spec) => {
            if is_prepare {
                Ok(DynOp::new(ShellPrepareOp { spec: shell_spec.clone() }))
            } else if is_parse {
                Ok(DynOp::new(ShellParseOp {
                    spec: shell_spec.output_spec(),
                    service_label: extract_service_label(name),
                }))
            } else {
                Err(unknown_callable(node_id, _module, name))
            }
        }
        ServiceOperationSpec::File(_) => {
            // File ops handled by infrastructure resolution (content_upsert pattern)
            Err(unknown_callable(node_id, _module, name))
        }
    }
}
```

That's it. ~30 lines replaces ~800 lines. No per-service structs. No per-service
match arms. Transport class dispatch only.

## 4. All Existing Services as Examples

Every existing hand-written adapter maps to a spec instance. Here's the proof that
the generic interpreters cover all 19 operations:

### 4.1 REST Operations (10 operations)

| Service.Operation | Method | Path Template | Body Shape | Output Shape |
|---|---|---|---|---|
| `github.Gist.Create` | POST | `/gists` | all inputs | `@json` fields |
| `gcp.SecretManager.AccessVersion` | GET | `/v1/projects/{project}/secrets/{secret}/versions/{version}:access` | none | nested `payload.data` (base64) |
| `gcp.SecretManager.CreateSecret` | POST | `/v1/projects/{project}/secrets` | partial | `@json` fields |
| `gcp.SecretManager.AddVersion` | POST | `/v1/{secret_name}:addVersion` | partial | `@json` fields |
| `gcp.IAM.GenerateAccessToken` | POST | `/v1/projects/-/serviceAccounts/{target_sa}:generateAccessToken` | partial | `@json` fields with camelCase |
| `gcp.ResourceManager.GetIamPolicy` | POST | `/v1/projects/{project}:getIamPolicy` | empty | `@json` fields |
| `gcp.ResourceManager.SetIamPolicy` | POST | `/v1/projects/{project}:setIamPolicy` | all inputs | bool |
| `gcp.STS.Exchange` | POST | `/v1/token` | `@body_template` (OAuth2 constants) | `@json` fields |
| `github.OIDC.GetToken` | GET | `?audience={audience}` | none | `@json` fields |
| `oauth2.Google.Refresh` | POST | `/token` | `@body_template` (OAuth2 grant) | `@json` fields |

**Special cases handled by spec:**
- `gcp.SecretManager.AccessVersion`: nested JSON path (`payload.data`) + base64 decode
  → `OutputFieldSpec.json_path = "payload/data"` + `type_id = "Bytes"` triggers decode
- `gcp.STS.Exchange`: non-default body shape → `@body_template` with literal constants
- `gcp.Metadata.GetIdentityToken`: raw body (not JSON) → response body is the token
  → `OutputFieldSpec.json_path = ""` (empty = raw body)

### 4.2 Shell Operations (9 operations)

| Service.Operation | Argv Template | Output Parsing |
|---|---|---|
| `git.Core.CurrentBranch` | `["git", "rev-parse", "--abbrev-ref", "HEAD"]` | TrimStdout |
| `git.Core.RemoteBranches` | `["git", "branch", "-r", "--format=%(refname:short)"]` | SplitLines |
| `git.Core.LsFiles` | `["git", "ls-files"]` | SplitLines |
| `git.Core.Diff` | `["git", "diff", "{base}...{head}"]` | TrimStdout |
| `git.Core.RevList` | `["git", "rev-list", "--since={since}", "HEAD"]` | SplitLines |
| `git.Core.Show` | `["git", "show", "{ref}:{path}"]` | TrimStdout |
| `shell.Find.ListDirs` | `["find", "{path}", "-maxdepth", "{max_depth}", ...]` | SplitLines |
| `shell.Codegen.Check` | `["test", "-f", "target/codegen/.stamp"]` | ExitCodeBool |
| `shell.Codegen.Run` | `["cargo", "run", "-p", "gunbc-app", ...]` | SuccessStdoutStderr |

Plus cargo operations (`cargo.Build.Build`, `cargo.Build.Test`, `cargo.Build.Clippy`,
etc.) which all use `SuccessStdoutStderr` parsing with conditional `--all-targets` flag.

### 4.3 File Operations

Handled entirely by `dsl/std/resources.dag` capabilities + `content_upsert` pattern.
No per-service infrastructure needed.

## 5. LLM Service Design

The LLM integration is the most complex upcoming case. The bottom-up approach handles
it cleanly by recognizing that LLM providers are just REST services with different
request/response shapes.

### 5.1 Each Provider Is a Separate REST Service

```
service llm.OpenAI {
    @endpoint("https://api.openai.com")
    @auth(BearerToken)

    operation ChatCompletion {
        input {
            model: String
            messages: Json           // pre-built message array
            temperature: Float?
            max_tokens: Int?
        }
        output {
            content: String @json("choices/0/message/content")
            model: String @json("model")
            finish_reason: String @json("choices/0/finish_reason")
            prompt_tokens: Int @json("usage/prompt_tokens")
            completion_tokens: Int @json("usage/completion_tokens")
        }
        @rest(POST, "/v1/chat/completions")
        @permissions(["chat_completion"])
    }
}

service llm.Anthropic {
    @endpoint("https://api.anthropic.com")
    @auth(Header("x-api-key"))

    operation Messages {
        input {
            model: String
            messages: Json
            max_tokens: Int = 4096
        }
        output {
            content: String @json("content/0/text")
            model: String @json("model")
            stop_reason: String @json("stop_reason")
            input_tokens: Int @json("usage/input_tokens")
            output_tokens: Int @json("usage/output_tokens")
        }
        @rest(POST, "/v1/messages")
        @headers({ "anthropic-version": "2023-06-01" })
        @permissions(["messages"])
    }
}
```

Each is a standard REST service. The generic `RestPrepareOp` / `RestParseOp` handles
both — no special LLM code needed at the transport layer.

### 5.2 Unified Chat Interface Is a DSL Pattern

The cross-provider abstraction (unified `ChatRequest` → provider-specific request →
provider-specific response → unified `ChatResponse`) is a **DSL-level composition**,
not a transport-level concern:

```
// dsl/shared/llm.dag

import services.llm.openai { llm.OpenAI }
import services.llm.anthropic { llm.Anthropic }

/// Unified chat interface — dispatches to provider at runtime.
func chat_completion(
    provider: String,
    model: String,
    messages: Json,
    max_tokens: Int = 4096,
    temperature: Float?
) -> { content: String, model: String, tokens_used: Int } {
    // Provider dispatch is a conditional branch in the DAG
    openai_result = llm.OpenAI.ChatCompletion(
        model: model, messages: messages, temperature: temperature, max_tokens: max_tokens
    ) [when provider == "openai"]

    anthropic_result = llm.Anthropic.Messages(
        model: model, messages: messages, max_tokens: max_tokens
    ) [when provider == "anthropic"]

    // Merge results (only one branch executes)
    return {
        content: openai_result.content ?? anthropic_result.content,
        model: openai_result.model ?? anthropic_result.model,
        tokens_used: (openai_result.prompt_tokens + openai_result.completion_tokens)
                  ?? (anthropic_result.input_tokens + anthropic_result.output_tokens),
    }
}
```

This keeps:
- **Transport layer**: pure protocol (REST prepare/parse) — no LLM knowledge
- **Service layer**: per-provider REST definitions — no cross-provider knowledge
- **Composition layer**: DSL pattern for unified dispatch — no Rust

### 5.3 Message Building

The `messages: Json` input is built by the caller (the tool `.dag` file), not by
the transport layer. Message formatting (system prompts, user content, tool calls,
thinking blocks) is a pure function in DSL or Rust — it produces a JSON value that
the REST prepare op sends as-is.

For advanced LLM features (extended thinking, cache hints, content blocks), the
message builder is a domain-specific pure function (`fn` in DSL or Rust) that produces
the `messages` JSON. The transport layer doesn't need to know about these features —
it just sends the JSON body.

## 6. Credential Scope Contracts

### 6.1 Current Problem

Hand-written per-service: `GistScopeContract`, `LlmScopeContract`, each implementing
`ScopeContract` with hardcoded provider/scheme/scopes. ~50-80 lines per service.

### 6.2 Solution: Derive from Service Spec

The `@auth` and `@permissions` annotations already declare everything needed:

```
service github.Gist {
    @auth(BearerToken)
    operation Create { @permissions(["gist"]) }
}
```

The `RestOperationSpec` carries `auth_scheme` and `permissions`. The credential chain
pattern reads these from the spec at runtime — no per-service Rust trait impl needed.

```rust
fn credential_intent_from_spec(spec: &RestOperationSpec, service_id: &str) -> CredentialIntent {
    let scheme = match &spec.auth_scheme {
        AuthScheme::Bearer => "bearer",
        AuthScheme::Header { name } => "header",
    };
    let mut intent = CredentialIntent::new(service_id, service_id, scheme)
        .with_required_scopes(spec.permissions.clone())
        .with_interactive_allowed(true);
    if let AuthScheme::Header { name } = &spec.auth_scheme {
        intent = intent.with_header_name(name);
    }
    intent
}
```

One function, all services. Delete `GistScopeContract`, `GistScope`, `LlmScopeContract`,
`LlmScope`, `GITHUB_SECRET_ID`, and every other per-service credential boilerplate.

## 7. What Gets Deleted

This is not gradual migration. Every item below is replaced by the generic protocol
interpreters + DSL specs:

### 7.1 Resolver Adapters (resolve.rs)

**Delete all of these structs:**

| Struct | Lines | Replaced by |
|---|---|---|
| `ServiceGcpStsExchangePrepareOp` | ~20 | `RestPrepareOp` |
| `ServiceGcpStsExchangeParseOp` | ~30 | `RestParseOp` |
| `ServiceGcpSecretManagerAccessVersionPrepareOp` | ~20 | `RestPrepareOp` |
| `ServiceGcpSecretManagerAccessVersionParseOp` | ~40 | `RestParseOp` |
| `ServiceCargoBuildPrepareOp` | ~15 | `ShellPrepareOp` |
| `ServiceCargoTestPrepareOp` | ~12 | `ShellPrepareOp` |
| `ServiceCargoClippyPrepareOp` | ~18 | `ShellPrepareOp` |
| `ServiceCargoParseOp` | ~18 | `ShellParseOp` |
| `ServiceShellFindListDirsPrepareOp` | ~20 | `ShellPrepareOp` |
| `ServiceShellFindListDirsParseOp` | ~25 | `ShellParseOp` |
| `ServiceShellCodegenCheckPrepareOp` | ~12 | `ShellPrepareOp` |
| `ServiceShellCodegenCheckParseOp` | ~15 | `ShellParseOp` |
| `ServiceShellCodegenRunPrepareOp` | ~20 | `ShellPrepareOp` |
| `ServiceShellCodegenRunParseOp` | ~18 | `ShellParseOp` |
| `resolve_service_transport()` | ~80 | ~30 lines (transport class dispatch) |
| **Total** | **~363** | **~200** (3 generic structs + dispatch) |

### 7.2 IR Transport Types

**Delete entirely:**

| File | Lines | Reason |
|---|---|---|
| `core/ir/src/transport/gist.rs` | ~270 | `dsl/services/github/gist.dag` is source of truth |
| `GistRequest`, `GistFile`, `GistScope`, `GistScopeContract` | — | All derivable from spec |
| `GITHUB_SECRET_ID` constant | — | Inline in DSL |

**LLM — delete transport boilerplate, keep domain types:**

| Delete | Keep |
|---|---|
| `build_chat_request()` dispatcher | `ChatMessage` builder helpers (pure fn) |
| `parse_chat_response()` dispatcher | Content block types (if needed by callers) |
| `LlmScopeContract`, `LlmScope` | — |
| Per-provider `build_*_request()` | — |
| Per-provider `parse_*_response()` | — |

### 7.3 Ops Library SubDag Builders

**Delete entirely** once credential chain and transport triplet composition are
DSL-resolved:

| Component | Lines | Replaced by |
|---|---|---|
| `lib/gist-ops/` SubDag builder | ~280 | DSL tool definition + `credential_chain` pattern |
| `lib/llm-ops/` SubDag builder | ~267 | DSL tool definition + per-provider service defs |

## 8. Implementation Plan

No phased migration. Just build the interface, switch everything over, delete the old code.

### Step 1: `ServiceOperationSpec` in the Lowerer

Extend `daglang-lower` to extract the full spec from `ServiceDef` AST nodes and attach
it to the `LoweredOp::Callable` that represents prepare/parse triplet nodes.

**What changes:**
- `ServiceCallMetadata` gains a `spec: Option<ServiceOperationSpec>` field
- `derive_service_call_metadata()` populates it from annotations + input/output fields
- The spec is available to the resolver via `LoweredOp::Callable.service_metadata`

**Verify:** `daglang expand <service>.dag` shows spec data on prepare/parse nodes.

### Step 2: Generic Protocol Interpreters

Implement `RestPrepareOp`, `RestParseOp`, `ShellPrepareOp`, `ShellParseOp` as
generic `Executable` impls that take a spec struct.

**What changes:**
- New file `gunbc-app/src/resolve_service.rs` with the 4 generic structs
- Each struct's `execute()` interprets the spec (URL interpolation, body building,
  response field extraction, etc.)
- Comprehensive unit tests: one test per existing hand-written adapter, verifying
  identical output for identical inputs

**Verify:** For every existing adapter, a test shows `generic.execute(inputs) ==
handwritten.execute(inputs)`.

### Step 3: Switch the Resolver

Replace `resolve_service_transport()` with transport-class dispatch.

**What changes:**
- `resolve_service_transport()` dispatches on `spec.transport_class()`, not service name
- All per-service match arms deleted
- All per-service structs deleted

**Verify:** `cargo test --workspace` passes. `make gist --dry-run`, `make bootstrap
--dry-run`, `make build --dry-run` all produce identical output.

### Step 4: Delete Redundant Rust

Delete `core/ir/src/transport/gist.rs`, scope contracts, SubDag builders, and any
other per-service Rust that's now fully covered by DSL + generic interpreter.

**Verify:** `cargo test --workspace` passes. The `.dag` `// Replaces:` comments
are honored.

### Step 5: LLM as DSL Services

Write `dsl/services/llm/openai.dag` and `dsl/services/llm/anthropic.dag` as standard
REST service definitions. Write `dsl/shared/llm.dag` for unified dispatch. Delete
LLM transport boilerplate from `core/ir/src/transport/llm/`.

**Verify:** `gunbc review --provider anthropic --dry-run` uses DSL-defined service.

### Step 6: Smoke Test — New Service from Scratch

Define a test-only REST service entirely in `.dag` (e.g., `httpbin.Anything`) and
verify end-to-end execution with zero Rust.

**Verify:** Service defined, tool wired, `--dry-run` works, no service-specific Rust.

## 9. Special Cases

### 9.1 `@body_template` (Non-Default Body Shape)

Some REST operations don't follow the "all non-path inputs become JSON body fields"
convention. `gcp.STS.Exchange` mixes input fields with OAuth2 literal constants.

The `@body_template` annotation declares the exact body shape:

```
@body_template({
    "audience": audience,                                    // input field ref
    "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",  // literal
    "requested_token_type": "urn:ietf:params:oauth:token-type:access_token",
    "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
    "subject_token": subject_token,                          // input field ref
})
```

The `RestPrepareOp` checks: if `spec.body_template.is_some()`, use the template;
otherwise, build body from all non-path input fields.

### 9.2 Nested JSON Paths in Responses

`gcp.SecretManager.AccessVersion` returns `payload.data` (nested). Use JSON pointer
syntax in `@json`:

```
output { payload: Bytes @json("payload/data") }
```

The `RestParseOp` uses `response.body.pointer("/payload/data")` instead of
`response.body.get("payload")`.

### 9.3 Raw Body Responses

`gcp.Metadata.GetIdentityToken` returns the token as a raw string body (not JSON).
Handle with:

```
output { subject_token: Secret @raw_body }
```

The `RestParseOp` checks: if the output field has `@raw_body`, use the response body
as a string directly instead of JSON-parsing it.

### 9.4 Base64 Encoded Fields

`gcp.SecretManager.AccessVersion` returns base64-encoded payload. The type system
handles this: `Bytes` type in the output spec triggers base64 decoding.

### 9.5 Dynamic Endpoint URLs

`github.OIDC.GetToken` uses an environment variable as the endpoint:

```
@endpoint("$ACTIONS_ID_TOKEN_REQUEST_URL")
```

The `$` prefix tells the spec extractor to resolve the endpoint from an environment
variable at runtime, not hardcode a URL.

### 9.6 Conditional Argv Segments

`cargo.Build.Build` appends `--all-targets` only when `all_targets=true`:

```
@shell(["cargo", "build", "--all-targets" if all_targets])
```

The `ArgvSegment::Conditional` variant handles this.

## 10. Cross-Cutting Concerns

### 10.1 Secret Handling

- Input fields typed `Secret` → extracted via `Value::as_secret()`, never logged
- Output fields typed `Secret` → wrapped in `SecretString::new()` in parse op
- Error messages never include secret field values

### 10.2 Error Messages

All parse ops include the service label (e.g., "gcp.STS.Exchange") in error messages.
The label is derived from the lowered node name at resolve time — no manual annotation
needed.

### 10.3 Mock Specs

`@mock_response` annotations generate `MockSpec` entries for dry-run execution. The
generic parse op handles mock responses identically to real responses — no special
mock path.

### 10.4 Auth Injection

The `execute` boundary (`TransportOps::Execute`) handles credential injection. The
prepare op doesn't attach tokens — it just builds the request. Auth metadata (scheme,
header name) travels via the spec to the credential chain, not through the prepare op.

## 11. Multi-Language Emission

The protocol interfaces aren't Rust-specific. The daglang compiler already emits Go, C,
and MIPS (with more backends planned). The `ServiceOperationSpec` must be available to
all emission targets, not just the Rust exec-runtime resolver.

### 11.1 Current Multi-Language Pipeline

```
.dag file
  → [lower] GraphIR (Dag<LoweredOp>)
  → [emit] target language code
       ├── rust_exec_runtime.rs  (Layer 1: Rust via gunbc-exec)
       ├── lower_rust.rs         (Layer 2: standalone Rust)
       ├── lower_go.rs           (Go)
       ├── lower_c.rs            (C)
       └── lower_mips.rs         (MIPS assembly)
```

Today, service transport triplets in the GraphIR are opaque `LoweredOp::Callable`
nodes. The emission backends skip them or emit stub functions. With the spec attached,
each backend can generate native protocol implementations.

### 11.2 Per-Language Protocol Interface Generation

The `ServiceOperationSpec` is language-agnostic data. Each emission backend interprets
it to generate native code:

**Rust (exec-runtime):** Generic `Executable` impls as described above. This is the
runtime interpreter path — the spec is interpreted at execution time.

**Rust (standalone / Layer 2):** Generate static Rust functions:
```rust
fn prepare_github_gist_create(description: &str, files: &HashMap<String, String>, public: bool) -> RestRequest {
    RestRequest::post("https://api.github.com/gists").json(json!({
        "description": description,
        "files": files_to_json(files),
        "public": public,
    }))
}
```

**Go:** Generate Go functions with `net/http`:
```go
func prepareGithubGistCreate(description string, files map[string]string, public bool) *http.Request {
    body := map[string]interface{}{
        "description": description,
        "files":       filesToJSON(files),
        "public":      public,
    }
    // ...
}
```

**C:** Generate C functions with libcurl or raw sockets:
```c
struct rest_request prepare_github_gist_create(const char *description, /* ... */) {
    // JSON body construction via string builder
    // URL construction from template
}
```

**MIPS:** Generate MIPS syscall sequences for HTTP (via runtime library or libc shim).

### 11.3 Spec in the IR

The `ServiceOperationSpec` becomes part of the `LoweredOp` that the emission backends
read. This is the key modeling change — the spec isn't just for the Rust resolver, it's
IR-level data that travels through all emission paths:

```
LoweredOp::Callable {
    module: "services.github.gist",
    kind: CallableKind::Pattern,
    name: "service_transport::prepare::github.Gist::Create",
    service_metadata: Some(ServiceCallMetadata {
        transport: ServiceTransportClass::Rest,
        spec: Some(ServiceOperationSpec::Rest(RestOperationSpec { ... })),
        // ...
    }),
    // ...
}
```

Each emission backend pattern-matches on the spec:
- **rust_exec_runtime**: constructs `RestPrepareOp(spec)` (runtime interpreter)
- **lower_rust**: emits a static function with inlined spec values
- **lower_go**: emits a Go function with the same logic
- **lower_c**: emits a C function
- **lower_mips**: emits MIPS assembly

### 11.4 Shared AbstractIR for Service Transport

The emission pipeline already uses `AbstractIR` as an intermediate between GraphIR and
target-language code. Service transport operations can be lowered to AbstractIR nodes
that each renderer then emits:

```
AbstractIR::ServicePrepare {
    spec: RestOperationSpec,
    inputs: Vec<AbstractIR::Binding>,
}
→ Rust renderer: emits RestRequest construction
→ Go renderer: emits http.NewRequest construction
→ C renderer: emits curl_easy_setopt calls
→ MIPS renderer: emits syscall sequence
```

This keeps the pattern uniform: one `ServiceOperationSpec` in the IR, each backend
renders it natively.

### 11.5 Protocol Interface as Shared Library per Language

For each target language, there's a thin runtime library that implements the protocol
interface:

| Language | REST Library | Shell Library | File Library |
|---|---|---|---|
| Rust | `RestRequest::post(url).json(body)` | `ShellRequest::new(cmd).args(args)` | `FileRequest::write(path, content)` |
| Go | `http.NewRequest("POST", url, body)` | `exec.Command(cmd, args...)` | `os.WriteFile(path, content)` |
| C | `curl_easy_setopt(...)` | `posix_spawn(...)` | `fopen/fwrite/fclose` |
| MIPS | `syscall(SYS_socket, ...)` | `syscall(SYS_execve, ...)` | `syscall(SYS_write, ...)` |

The emitter generates code that *calls* these libraries with spec-derived parameters.
The libraries themselves are small, per-language, and shared across all services.

## 12. Summary

**Before:** N per-service adapter structs, per language. Adding a service = writing
adapters in every target language.
**After:** 3 per-protocol-class interfaces, implemented once per language. Adding a
service = writing DSL.

The interface hierarchy is:
```
Transport Primitives (Layer 0)  — per-language: http client, process exec, file I/O
  ↓ parameterized by
Protocol Interfaces (Layer 1)   — per-language: RestPrepare, RestParse, ShellPrepare, ShellParse
  ↓ parameterized by
Service Specs (Layer 2)         — .dag files: endpoint, method, path, fields, auth (language-agnostic)
```

Layer 0 is implemented once per language (thin runtime library).
Layer 1 is generated once per language by the emission backend.
Layer 2 is written once in DSL and shared across all languages.
