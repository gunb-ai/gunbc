# Design: Protocol Stack Layering

**Status**: Draft
**Date**: 2026-02-23
**Related tasks**: M16 (SystemModel/TransportBehavior unification), M8 (metadata separation)
**Reference**: `docs/handbook.md` § "Compositional Modeling Philosophy"

## Problem

The transport layer currently models HTTP and REST as **sibling behaviors in a flat
system model** (`transport.http_rest` with 8 behaviors: http_get, http_post, rest_get,
rest_post, etc.). This misses the compositional structure: REST is a policy layer
**on top of** HTTP, which runs **on top of** TCP/TLS. The system model doesn't capture
this dependency, so:

1. REST behaviors don't inherit HTTP status semantics — they redeclare them
2. HTTP behaviors don't express their TLS/TCP prerequisites
3. Adding a new protocol layer (gRPC, WebSocket) requires modifying the flat enum
4. Credential chains, error classification, and mock responses can't be derived
   from the protocol stack — they're hand-wired per service
5. Test obligations (retry on 503, fail on 401, success on 304) are per-service
   instead of per-protocol-layer

## Current State

### Transport types (closed enum)

```rust
// core/ir/src/transport/mod.rs
pub enum TransportRequest {
    Rest(RestRequest),   // JSON body, auth, query params
    Http(HttpRequest),   // Raw string body, no auth
    File(FileRequest),   // Path + mode + content
    Tcp(TcpRequest),     // Host + port + data
    Shell(ShellRequest), // Command + args + env
}
```

### Execution delegation (implicit layering)

```rust
// lib/transport/src/executor.rs
fn execute_rest(req: &RestRequest) -> Result<RestResponse> {
    // 1. Apply credential to headers (REST-layer concern)
    // 2. Convert RestRequest → HttpRequest (REST→HTTP delegation)
    // 3. execute_http(http_req)  (HTTP-layer execution)
    // 4. Parse JSON body (REST-layer concern)
}

fn execute_http(req: &HttpRequest) -> Result<HttpResponse> {
    // Uses ureq library (handles TLS/TCP internally)
}
```

The layering **exists in the executor** but is **invisible to the type system and
system model**. The DSL can't reason about it.

### System model (flat)

```rust
// lib/transport/src/system_models.rs
SystemModel {
    id: "transport.http_rest",
    kind: SystemKind::RestApi,
    behaviors: [
        // HTTP and REST are siblings, not layers:
        http_get, http_post, http_put, http_delete,
        rest_get, rest_post, rest_put, rest_delete,
    ],
}
```

## Design

### Principle: Model the stack, don't refactor the runtime

The transport executor works fine — ureq handles TLS/TCP, REST delegates to HTTP.
We don't need to refactor execution. We need to **model the existing layering** so
the type system and DSL can reason about it.

### Step 1: Split system models by protocol layer

Replace `transport.http_rest` with three models that declare dependencies:

```
transport.tcp
  kind: Convention
  behaviors: [connect, send, receive]
  properties: [WritesWorld]
  depends_on: []

transport.http
  kind: Protocol
  behaviors: [get, post, put, delete, head, options]
  properties: per-method (get=ReadOnly, post=WritesWorld, put=Idempotent, delete=Idempotent)
  depends_on: [transport.tcp]  // Declares: HTTP requires TCP connectivity
  status_semantics: RFC 9110 classification (2xx=success, 4xx=client error, 5xx=server error)

transport.rest
  kind: Protocol
  behaviors: [get, post, put, patch, delete]
  properties: inherits from transport.http + adds JSON content-type invariant
  depends_on: [transport.http]  // Declares: REST layers on HTTP
  status_overrides: [304 → success]  // REST-specific policy
```

Each model is registered via `submit_system_model!()` independently. The
`depends_on` field is the composition mechanism — it makes the layering explicit
and traversable.

### Step 2: DSL protocol type declarations

Add protocol types to `dsl/std/types.dag`:

```dag
// Protocol layer types (structural, not runtime values)
type TcpEndpoint { host: String, port: Port }
type HttpEndpoint { url: Url, method: HttpMethod }
type RestEndpoint { url: Url, method: HttpMethod, content_type: String = "application/json" }

// Status semantics (composable — REST overrides HTTP where needed)
type HttpStatusPolicy {
  success: List<Int>        // [200, 201, 204, ...]
  client_error: List<Int>   // [400, 401, 403, 404, ...]
  server_error: List<Int>   // [500, 502, 503, ...]
  retryable: List<Int>      // [429, 503, 502]
}

type RestStatusPolicy extends HttpStatusPolicy {
  overrides: List<StatusOverride>  // [{code: 304, outcome: "success"}]
}
```

### Step 3: Service declarations compose protocol layers

When a DSL service declares `@rest(POST, "/gists")` + `@endpoint(...)` + `@auth(BearerToken)`:

```dag
service github.Gist {
  @endpoint("https://api.github.com")  // → RestEndpoint + HttpEndpoint
  @auth(BearerToken)                   // → Credential layer (applied at REST level)

  operation Create {
    @rest(POST, "/gists")              // → REST layer: method + path + JSON body
    @permissions(["gist"])             // → Auth layer: required scopes
  }
}
```

The compiler resolves the protocol stack:

```
Gist.Create
  → REST layer:  POST /gists, JSON body, 304=success
    → HTTP layer:  POST https://api.github.com/gists, status classification
      → TCP layer:  connect api.github.com:443
        → TLS:      (handled by transport executor / ureq)
```

Each layer's invariants flow into:
- **Transport code**: correct URL construction, header application, body encoding
- **Error classification**: status code → success/client_error/server_error/retryable
- **Mock responses**: testgen generates mock responses per status category
- **Credential chain**: auth applied at REST layer, scopes validated before execution

### Step 4: Behavioral property inheritance

Properties compose across protocol layers via dependency resolution:

```
transport.rest.post:
  properties = transport.http.post.properties ∪ {JsonContentType}
  status_semantics = transport.http.status_semantics ⊕ rest_overrides

transport.http.post:
  properties = {WritesWorld} ∪ transport.tcp.connect.properties
  status_semantics = RFC 9110

transport.tcp.connect:
  properties = {WritesWorld, NonDeterministic}
```

The `∪` and `⊕` operations are well-defined:
- Property union: `ReadOnly ∪ WritesWorld = WritesWorld` (WritesWorld dominates)
- Status override: REST overrides take precedence over HTTP defaults for specific codes

### Step 5: Test obligation derivation from protocol stack

Given a service operation's protocol stack, testgen derives obligations:

| Layer | Derived test obligation |
|-------|------------------------|
| TCP | Connection failure → transport error (not silent) |
| HTTP | 4xx → client error assertion; 5xx → server error assertion |
| REST | 304 → success (not error); JSON parse failure → explicit error |
| Auth | 401 → credential refresh or fail; missing token → fail before request |
| Service | Operation-specific mock response; field extraction assertions |

Currently these are hand-written per service MockSpec. With the protocol stack
model, testgen can derive the HTTP/REST/auth-layer obligations automatically —
only the service-specific layer needs manual mock data (`@mock_response`).

## Scope & Non-Goals

### In scope
- Split `transport.http_rest` into layered system models with `depends_on`
- Add protocol type declarations to DSL
- Make behavioral property inheritance traversable
- Derive baseline test obligations from protocol stack

### Not in scope (future work)
- Refactoring the transport executor (ureq handles TCP/TLS fine)
- Runtime middleware/interceptor framework (logging, retry, circuit breaking)
- Connection pooling or resource acquisition modeling for TCP/TLS
- gRPC or WebSocket protocol layer (no current use case)

## Relationship to Other Tasks

- **M16** (SystemModel/TransportBehavior unification): This design is a prerequisite.
  M16 unifies the invocation contract model — this design provides the layered
  structure that the unified model represents.
- **M8** (metadata separation): Protocol properties (status semantics, retryable codes)
  should use `TypeOp::Meta`, not `Validate`. M8 enables clean property encoding.
- **M13** (contract tests): Protocol-layer obligations are contract tests. M13 provides
  the harness; this design provides the obligations to test.

## Migration Path

1. **Phase 0**: Split `transport.http_rest` into `transport.http` and `transport.rest`
   with explicit `depends_on`. No runtime changes. System model consumers updated.
2. **Phase 1**: Add protocol types to DSL. Compiler recognizes `@rest` as implying
   HTTP dependency. Status policies defined per layer.
3. **Phase 2**: Behavioral property inheritance implemented. `transport.rest.post`
   automatically includes `transport.http.post` properties plus JSON invariant.
4. **Phase 3**: Testgen derives baseline obligations from protocol stack. Per-service
   MockSpec only provides service-specific mock data; protocol-layer obligations are
   automatic.

## Examples

### Before (current state)

Adding a new REST service requires:
1. Write `dsl/services/stripe/payments.dag` with `@rest`, `@endpoint`, `@auth`
2. Hand-write `graph_mock.rs` with mock responses for 200, 401, 500
3. Hand-write credential chain wiring in `graph.rs`
4. Hand-write testgen target registration

### After (target state)

Adding a new REST service requires:
1. Write `dsl/services/stripe/payments.dag` with `@rest`, `@endpoint`, `@auth`
2. Provide `@mock_response` annotation (service-specific data only)
3. Protocol stack derives: credential chain, error classification, HTTP status tests,
   auth failure tests, JSON parse tests, retry behavior for 429/503

The workflow author writes `stripe.Payments.CreateCharge(amount: 100)` and the
compiler generates all transport code, mock specs, and test obligations from the
composed protocol stack.
