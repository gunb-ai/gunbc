# Provider Response Contracts: Modeling External API Behavior

> **Status**: Design
> **Author**: Red team
> **Date**: 2026-02-27
> **Depends on**: RT-I1, RT-I2, RT-A2 (findings)
> **Supersedes**: RT-I1 (scope expanded from "add @mock_response" to "model provider contracts")

---

## Problem

Every service in `dsl/services/` declares **only success outputs**. A GitHub
Gist Create operation declares `output { id: GistId, html_url: Url }` — period.
No 401 Unauthorized, no 404 Not Found, no 422 Validation Failed, no 429 Rate
Limited, no 500 Internal Server Error. The service model is half a model.

This is not just a testing gap. It's a **modeling gap**. The `.dag` files are
supposed to be our model of external dependencies — what they do, what they
return, under what conditions. A model that only represents the happy path is
a model that guarantees we only handle the happy path.

### Current state

| Metric | Value |
|--------|-------|
| REST operations across all services | 29 |
| Operations with error response declarations | 0 |
| Operations with `@mock_response` annotations | 0 |
| `@mock_response` parser implementation | Stub (returns `Vec::new()`) |
| `error_cases()` trait implementations | 0 (all return default empty) |
| Testgen error-case obligations | 0 |
| Status code checks in `GenericRestParseOp` | 0 |

**Consequence**: The gist 401 bug lived in production because no test ever
ran the flow with a non-200 response. Five compounding failures (see
`TODO/gist-auth-postmortem.md`) were all downstream of the fact that the
service model didn't declare that 401 is a thing that can happen.

---

## Principle: Services Model Provider Contracts

A service definition in `.dag` is a **model of an external dependency**. It
models what the provider documents: endpoints, methods, request shapes,
response shapes, authentication requirements, error codes, rate limits,
idempotency guarantees.

This model will always have some lag/skew relative to the real API — we are
not the provider, we are modeling their docs. But the model should be
**structurally complete**: if the provider documents a response code, we
declare it. If they document an error body shape, we type it.

The model then creates **mandatory obligations**:
- If a service declares `@response(401, ...)`, testgen **must** generate a
  test that exercises code handling a 401.
- If an interface declares a capability, every implementor inherits the
  response contract obligations.
- If a workflow calls an operation, it inherits the obligation to handle
  every declared response.

This moves testing from "optional coverage we might add" to "structural
requirement imposed by the model." You can't use an API without handling
its documented failure modes, and the compiler enforces this.

---

## Design

### 1. `@response` annotation on operations

New annotation on service operations. Not `@mock_response` — that's a test
utility. `@response` is a **declaration about what the provider returns**.

```dag
service github.Gist {
  config { endpoint: "https://api.github.com", auth: BearerToken, auth_input: auth_token }

  operation Create {
    input {
      description: String
      content: String
      public: Bool = false
      auth_token: Secret
    }
    output {
      id: GistId
      html_url: Url
    }
    transport rest { method: POST, path: "/gists" }

    // Provider contract: what GitHub actually returns
    @response(201, output)  // success — maps to the output type
    @response(401, Unauthorized { message: String })
    @response(403, Forbidden { message: String })
    @response(404, NotFound { message: String })
    @response(422, ValidationFailed { message: String, errors: List<ValidationError>? })
    @response(429, RateLimited { message: String, retry_after: Int? })
    @response(500, ServerError { message: String })

    // Optional: link to provider documentation
    @doc("https://docs.github.com/en/rest/gists/gists#create-a-gist")
  }
}
```

**Semantics**:
- `@response(STATUS, TYPE)` declares "this operation can return HTTP status
  STATUS with a body matching TYPE."
- `@response(STATUS, output)` (keyword) means "this status code produces the
  declared output type" — i.e., the success case.
- Every operation with `transport rest {}` MUST have at least one success
  `@response` and one error `@response`. The compiler errors if missing.
- Response types can be inline records or references to named types.

**For shell operations**: The equivalent is `@exit(CODE, TYPE)`:
```dag
operation Build {
  @exit(0, output)  // success
  @exit(1, BuildFailed { stderr: String })
  @exit(101, CompilerIce { stderr: String })
}
```

### 2. Standard error types

Define common error shapes in `dsl/std/errors.dag` so services don't
repeat themselves:

```dag
module std.errors

// Common REST API error shapes
type HttpError { message: String, documentation_url: String? }
type ValidationError { resource: String?, field: String?, code: String? }
type RateLimitInfo { message: String, retry_after: Int? }

// GitHub-specific (inherits from HttpError pattern)
type GitHubError { message: String, documentation_url: String? }
type GitHubValidationError : HttpError {
  errors: List<ValidationError>?
}

// GCP-specific
type GcpError { error: GcpErrorDetail }
type GcpErrorDetail { code: Int, message: String, status: String, errors: List<GcpErrorItem>? }
type GcpErrorItem { reason: String, domain: String?, message: String? }
```

### 3. Response contract on interfaces

Interfaces declare response contracts that all implementors must handle:

```dag
interface IssueProvider {
  capability get(id: NonEmptyStr) -> { issue: TrackedIssue, found: Bool }

  // Provider-agnostic contract: these responses are possible
  @response get(200, output)
  @response get(404, { found: false })
  @response get(401, Unauthorized)

  // Behavioral contracts (existing)
  @contract: get(id) after create(title, body, labels) => { found: true }
}
```

When `github.Issues : IssueProvider`, the compiler checks that github.Issues
handles (or at least declares) the response codes from the interface.

### 4. New obligation: `ProviderResponseContract`

Add to Bucket C in `obligation.rs`:

```rust
/// Provider response contract: service declares this status code is possible,
/// so testgen must exercise the code path that handles it.
ProviderResponseContract {
    node_id: NodeId,
    operation: String,
    status_code: u16,
    response_type: TypeId,
    /// Whether this is the success response or an error response
    is_error: bool,
},
```

**Generation rule**: For every transport execute node whose service operation
declares `@response` annotations, emit one `ProviderResponseContract`
obligation per declared response. These are `RuntimeOnly` — can never be
statically discharged because they test real behavior.

**Test shape**: For each error `@response`, the generated test:
1. Sets up the workflow with valid happy-path inputs
2. Mocks the transport execute node to return `rest_response(STATUS, BODY)`
   where BODY matches the declared response type
3. Asserts the workflow handles it (doesn't panic, produces appropriate outputs)

This replaces `SingleTransportFailure`'s blunt `"<TRANSPORT_FAILURE>"` sentinel
with **realistic, provider-documented error responses**.

### 5. `GenericRestParseOp` status code checking

Currently `GenericRestParseOp` ignores the HTTP status code and just tries
to extract fields from the body. Fix:

```rust
// In GenericRestParseOp::execute():
let status = response.status;
if status < 200 || status >= 300 {
    // Check if service declares a handler for this status
    if let Some(error_type) = self.spec.error_responses.get(&status) {
        // Parse error body according to declared type
        return parse_error_response(status, &response.body, error_type);
    }
    // Undeclared error status → hard failure with diagnostic
    return Err(TransportError::UnexpectedStatus {
        operation: self.spec.operation.clone(),
        status,
        body: response.body.to_string(),
    });
}
```

### 6. `@mock_response` becomes derivable

Once `@response` annotations exist, `@mock_response` becomes **derivable**:
the compiler can generate mock responses from the declared response types.
A `@response(401, Unauthorized { message: String })` automatically produces
a mock `rest_response(401, { "message": "Unauthorized" })` with
type-derived example values.

This means RT-I1 ("add @mock_response on all service operations") is
**superseded** — we add `@response` annotations instead, and mock data
is derived.

### 7. Documentation references

Each service should reference the provider's API documentation:

```dag
service github.Gist {
  @doc("https://docs.github.com/en/rest/gists/gists")

  operation Create {
    @doc("https://docs.github.com/en/rest/gists/gists#create-a-gist")
    // ...
  }
}
```

This is metadata, not enforced by the compiler, but it creates a traceable
link from our model to the source of truth. When GitHub changes their API,
we can trace which `.dag` files need updating.

---

## Implementation Sequence

This is a vertical slice through parser → lowerer → obligation → codegen.

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | PC-1 | **`@response` annotation parsing.** Add to `daglang-syntax` parser. Response type can be inline record, named type reference, or `output` keyword. Store as `Vec<ResponseDecl>` on `OperationDef`. | M | — |
| 2 | PC-2 | **Standard error types.** `dsl/std/errors.dag` with common HTTP, GitHub, GCP error shapes. | S | — |
| 3 | PC-3 | **`@response` on all REST services.** Add response declarations to all 29 REST operations in `dsl/services/`. Include `@doc` references to provider API docs. | L | PC-1, PC-2 |
| 4 | PC-4 | **`@exit` on all shell services.** Same for shell operations (exit code → output type mapping). | M | PC-1 |
| 5 | PC-5 | **Lowerer propagation.** `@response` declarations flow to `ServiceOperationSpec` in the lowered IR. Parse nodes get access to declared error types. | M | PC-1 |
| 6 | PC-6 | **`GenericRestParseOp` status checking.** Check HTTP status before field extraction. Route to error type if declared, hard-fail if undeclared non-2xx. | M | PC-5 |
| 7 | PC-7 | **`ProviderResponseContract` obligation.** New obligation kind in Bucket C. Collector emits one per declared `@response` per transport node. | M | PC-5 |
| 8 | PC-8 | **Testgen codegen for response contracts.** Generate per-status-code tests from `ProviderResponseContract` obligations. Mock body derived from response type. | L | PC-7 |
| 9 | PC-9 | **Interface response contract inheritance.** Interfaces declare response contracts; implementors inherit obligations. Compiler warns on unhandled responses. | M | PC-7 |
| 10 | PC-10 | **Completeness enforcement.** Compiler requires at least one success `@response` and one error `@response` on every operation with `transport rest {}`. | S | PC-1 |

**Total estimated effort**: ~L (3-5 days for the full vertical slice)

### Relationship to existing tasks

| Existing | Disposition |
|----------|------------|
| RT-I1 (`@mock_response` on all ops) | **Superseded** by PC-3. We add `@response` instead; mock data is derived. |
| RT-I2 (testgen error-status obligations) | **Subsumed** by PC-7 + PC-8. Same goal, better mechanism. |
| RT-A2 findings (parser not implemented) | **Input** to PC-1. We implement a new annotation rather than fixing the old one. |
| RT9 (Virtual I/O DSL types) | **Orthogonal**. VirtualBackend is the runtime mechanism; `@response` is the declaration. |

---

## Design Decisions

### Why `@response` not `@mock_response`?

`@mock_response` conflates two concerns:
1. **What the provider does** (contract) — this is a fact about the external system
2. **What to use in tests** (fixture) — this is a test-engineering choice

Separating them means:
- `@response(401, Unauthorized { message: String })` = "GitHub returns this"
- The compiler derives mock data from the response type automatically
- You can still write manual `mock ... -> rest_response(401, ...)` in test blocks for specific scenarios

### Why not `Result<T, E>` return types?

Sum type returns (`type GistResult = Ok { id: GistId } | Err { message: String }`)
would work for pure DSL functions but don't fit transport operations:
- Transport operations have a structural success/error split at the HTTP level
- The parse node needs to route on status code BEFORE constructing the output
- Error handling is at the transport layer, not the type layer

`@response` keeps the split at the transport annotation layer where it belongs.

### Why mandatory?

If `@response` declarations are optional, they won't get written. The gist 401
bug happened precisely because error handling was optional. Making the compiler
require at least one error `@response` on every REST operation means you can't
define a service without thinking about what happens when it fails.

---

## Example: GitHub Issues after migration

```dag
service github.Issues : IssueProvider {

  operation get {
    input { id: NonEmptyStr }
    output {
      issue: TrackedIssue from "."
      found: Bool from "status == 200"
    }
    transport rest { method: GET, path: "/repos/{owner}/{repo}/issues/{id}" }

    @response(200, output)
    @response(301, MovedPermanently { url: Url })
    @response(404, GitHubError)
    @response(410, GitHubError)  // gone (transferred)
    @response(401, GitHubError)
    @response(403, GitHubError)
    @response(429, RateLimitInfo)
    @response(500, GitHubError)

    @doc("https://docs.github.com/en/rest/issues/issues#get-an-issue")
  }
}
```

**What testgen produces from this** (8 tests for this one operation):
1. `test_get_status_200` — happy path, extract TrackedIssue
2. `test_get_status_301` — redirect, verify handling
3. `test_get_status_404` — not found, verify `found: false`
4. `test_get_status_410` — gone, verify handling
5. `test_get_status_401` — unauthorized, verify error propagation
6. `test_get_status_403` — forbidden, verify error propagation
7. `test_get_status_429` — rate limited, verify retry or error
8. `test_get_status_500` — server error, verify error propagation

Each test mocks the transport execute node with a response matching the
declared type and verifies the workflow handles it correctly.

---

## Open Questions

1. **Granularity of enforcement**: Should every declared `@response` require
   explicit handling in the consuming workflow, or is "propagate error" sufficient?
   (Recommendation: propagate is sufficient for v1; explicit handling is a v2 feature.)

2. **Rate limit handling**: Should `@response(429, ...)` compose with `@retry`
   to auto-generate retry-after logic? (Recommendation: yes, but v2.)

3. **Versioning**: Should `@response` declarations be version-scoped
   (`@response(404, NotFound) @since("2022-11-28")`)? (Recommendation: defer
   until we have a concrete use case for version-aware modeling.)
