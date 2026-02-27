# Provider Response Contracts: Modeling External API Behavior

> **Status**: Design
> **Author**: Red team
> **Date**: 2026-02-27
> **Depends on**: RT-A2 (findings), `annotation-to-dag-modeling.md` Phase 2
> **Supersedes**: RT-I1, RT-I2

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
| `MockResponseDef` parser implementation | Stub (returns `Vec::new()`) |
| `error_cases()` trait implementations | 0 (all return default empty) |
| Testgen error-case obligations | 0 |
| Status code checks in `GenericRestParseOp` | 0 |
| Error mapping infrastructure in lowered IR | None (deleted in RT36) |

**Consequence**: The gist 401 bug lived in production because no test ever
ran the flow with a non-200 response. Five compounding failures (see
`TODO/gist-auth-postmortem.md`) were all downstream of the fact that the
service model didn't declare that 401 is a thing that can happen.

### Prior infrastructure (deleted)

The lowerer previously had `error_mappings: Vec<ErrorMapping>` on
`ServiceOperationSpec` and an `ErrorMapping` struct, but these were always
empty (`vec![]`) with a TODO that was never implemented. They were deleted
in RT36 as unused scaffolding. The `response` block design (below) replaces
this with a properly designed mechanism.

---

## Principle: Services Model Provider Contracts

A service definition in `.dag` is a **model of an external dependency**. It
models what the provider documents: endpoints, methods, request shapes,
response shapes, authentication requirements, error codes, rate limits,
idempotency guarantees.

The model creates **mandatory obligations**:
- If a service declares an error response, testgen **must** generate a
  test that exercises code handling it.
- If an interface declares a capability, every implementor inherits the
  response contract obligations.
- If a workflow calls an operation, it inherits the obligation to handle
  every declared response.

This is not metadata — it's **structure in the DAG**. Per the compositional
modeling philosophy and `annotation-to-dag-modeling.md` Phase 2, error
classification is a node in the transport DAG, not an inert annotation.

---

## Design

### 1. `response` block on operations

A structural block inside the operation — the same pattern as `transport`,
`input`, and `output`. Not an `@` annotation.

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
    response {
      201 => output                                          // success case
      401 => Unauthorized { message: String }
      403 => Forbidden { message: String }
      404 => NotFound { message: String }
      422 => ValidationFailed { message: String, errors: List<ValidationError>? }
      429 => RateLimited { message: String, retry_after: Int? }
      500 => ServerError { message: String }
    }

    // Optional: link to provider documentation
    doc "https://docs.github.com/en/rest/gists/gists#create-a-gist"
  }
}
```

**Semantics**:
- `response { STATUS => TYPE }` declares "this operation can return HTTP
  status STATUS with a body matching TYPE."
- `STATUS => output` (keyword) means "this status code produces the declared
  output type" — the success case.
- The `response` block is **mandatory** on every operation with
  `transport rest { ... }`. At least one success and one error entry
  are required. The compiler errors if missing.
- Response types can be inline records or references to named types.

**For shell operations**: The equivalent is an `exit` block:
```dag
operation Build {
  exit {
    0 => output
    1 => BuildFailed { stderr: String }
    101 => CompilerIce { stderr: String }
  }
}
```

**Parser representation** (replaces the unused `MockResponseDef`):

```rust
pub struct OperationDef {
    pub name: String,
    pub inputs: Vec<Field>,
    pub outputs: Vec<Field>,
    pub idempotent: bool,
    pub readonly: bool,
    pub hermetic: bool,
    pub permissions: Vec<String>,
    pub transport: Option<TransportBinding>,
    pub response_map: Vec<ResponseEntry>,  // NEW — replaces mock_response
}

pub struct ResponseEntry {
    pub status: u16,        // HTTP status code or exit code
    pub is_output: bool,    // true if `=> output` (success case)
    pub response_type: Option<TypeRef>,  // None if is_output=true
}
```

### 2. How it composes into the transport DAG

Per `annotation-to-dag-modeling.md` Phase 2, the lowerer generates an
**error classification node** from the `response` block:

```
prepare → execute → classify_response → parse_output
                         │
                    (201 → route to parse_output)
                    (401 → route to error: Unauthorized)
                    (404 → route to error: NotFound)
                    (412 → route to error: PreconditionFailed)
                    (5xx → route to error: ServerError)  ← protocol stack default
```

The classification node is a real node in the DAG. It:
1. Reads the HTTP status code from the execute node's output
2. Routes success statuses to the existing parse node
3. Routes error statuses to typed error outputs
4. Hard-fails on undeclared status codes (no silent swallowing)

This composes with protocol stack defaults (M16): the HTTP layer
contributes default 4xx/5xx classification, and the service-level
`response` block overrides specific codes. This is the same layered
composition as `transport rest { ... }` composing with `config { auth: ... }`.

### 3. Error mapping in the lowered IR

The prior `ErrorMapping` struct and `error_mappings` field were deleted in
RT36 (unused scaffolding). When implementing PC-5, introduce a new
`ResponseMapping` struct on `RestOperationSpec` populated from the parsed
`response` block:

```rust
// daglang-lower/src/spec.rs — new field on RestOperationSpec
pub response_mappings: Vec<ResponseMapping>,

pub struct ResponseMapping {
    pub status: u16,
    pub is_success: bool,
    pub response_type: Option<String>,
}
```

The lowerer populates this from `op.response_map` entries during
`derive_rest_spec()`.

### 4. Standard error types

Define common error shapes in `dsl/std/errors.dag`:

```dag
module std.errors

// Common REST API error shapes
type HttpError { message: String, documentation_url: String? }
type ValidationError { resource: String?, field: String?, code: String? }
type RateLimitInfo { message: String, retry_after: Int? }

// GitHub-specific
type GitHubError { message: String, documentation_url: String? }
type GitHubValidationError {
  message: String
  documentation_url: String?
  errors: List<ValidationError>?
}

// GCP-specific
type GcpError { error: GcpErrorDetail }
type GcpErrorDetail { code: Int, message: String, status: String, errors: List<GcpErrorItem>? }
type GcpErrorItem { reason: String, domain: String?, message: String? }
```

### 5. Response contracts on interfaces

Interfaces declare response contracts. All implementors inherit them.

```dag
interface IssueProvider {
  capability get(id: NonEmptyStr) -> { issue: TrackedIssue, found: Bool }
  capability create(title: String, body: String) -> { issue: TrackedIssue }

  // Response contract — implementors must handle these
  response get {
    200 => output
    404 => { found: false }
    401 => HttpError
  }

  // Behavioral contracts (existing)
  contract get(id) after create(title, body, labels) => { found: true }
}
```

When `github.Issues : IssueProvider`, the compiler checks that
`github.Issues.get` declares at least the response codes from the interface.
Provider-specific codes (301, 410) can be added on top.

### 6. New obligation: `ProviderResponseContract`

Add to Bucket C in `obligation.rs`:

```rust
/// Provider response contract: service declares this status code is possible,
/// so testgen must exercise the code path that handles it.
ProviderResponseContract {
    node_id: NodeId,
    operation: String,
    status_code: u16,
    response_type: TypeId,
    is_error: bool,
},
```

**Generation rule**: For every transport execute node whose service operation
has a `response` block, emit one `ProviderResponseContract` obligation per
entry. These are `RuntimeOnly` — can never be statically discharged.

**Test shape**: For each error entry, the generated test:
1. Sets up the workflow with valid happy-path inputs
2. Mocks the transport execute node to return `rest_response(STATUS, BODY)`
   where BODY matches the declared response type
3. Asserts the workflow handles it (doesn't panic, produces appropriate outputs)

This replaces `SingleTransportFailure`'s blunt `"<TRANSPORT_FAILURE>"` sentinel
with **realistic, provider-documented error responses**.

### 7. `GenericRestParseOp` status code checking

Currently `GenericRestParseOp` ignores the HTTP status code and just tries
to extract fields from the body. Fix:

```rust
// In GenericRestParseOp::execute():
let status = response.status;
if status < 200 || status >= 300 {
    if let Some(mapping) = self.spec.response_mappings.iter().find(|m| m.status == status) {
        return parse_error_response(status, &response.body, mapping);
    }
    // Undeclared error status → hard failure with diagnostic
    return Err(TransportError::UnexpectedStatus {
        operation: self.spec.operation.clone(),
        status,
        body: response.body.to_string(),
    });
}
```

### 8. Mock responses are derived

Once `response` blocks exist, mock responses are **derived** from the
declared response types. A `401 => Unauthorized { message: String }` entry
automatically produces mock data `rest_response(401, { "message": "..." })`
with type-derived example values. No manual `@mock_response` authoring.

This means RT-I1 ("add @mock_response on all service operations") is
**superseded** — we add `response` blocks instead, and mock data is derived.

---

## Implementation Sequence

Vertical slice through parser → lowerer → obligation → codegen.

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | PC-1 | **`response` block parsing.** Add to `daglang-syntax` parser. Store as `Vec<ResponseEntry>` on `OperationDef` (replaces `mock_response`). | M | — |
| 2 | PC-2 | **Standard error types.** `dsl/std/errors.dag` with common HTTP, GitHub, GCP error shapes. | S | — |
| 3 | PC-3 | **`response` blocks on all REST services.** Add to all 29 REST operations. Include `doc` references. | L | PC-1, PC-2 |
| 4 | PC-4 | **`exit` blocks on all shell services.** Exit code → output type mapping. | M | PC-1 |
| 5 | PC-5 | **Lowerer: populate response mappings.** Add `ResponseMapping` struct and `response_mappings` field on `RestOperationSpec`. Wire `response` block entries from parsed AST. Generate classify_response node in transport DAG. | M | PC-1 |
| 6 | PC-6 | **`GenericRestParseOp` status checking.** Route on status code before field extraction. Hard-fail on undeclared non-2xx. | M | PC-5 |
| 7 | PC-7 | **`ProviderResponseContract` obligation.** New Bucket C obligation, one per `response` entry. | M | PC-5 |
| 8 | PC-8 | **Testgen codegen for response contracts.** Per-status-code tests, mock body derived from response type. | L | PC-7 |
| 9 | PC-9 | **Interface response contract inheritance.** Implementors inherit obligations from interface `response` declarations. | M | PC-7 |
| 10 | PC-10 | **Completeness enforcement.** Compiler requires ≥1 success + ≥1 error entry in `response` block on every `transport rest {}` operation. | S | PC-1 |

### Relationship to existing tasks

| Existing | Disposition |
|----------|------------|
| RT-I1 (`@mock_response` on all ops) | **Superseded** by PC-3. Mock data is derived from `response` blocks. |
| RT-I2 (testgen error-status obligations) | **Subsumed** by PC-7 + PC-8. Same goal, better mechanism. |
| RT-A2 findings (parser not implemented) | **Input** to PC-1. We implement a structural block, not an annotation. |
| `annotation-to-dag-modeling.md` Phase 2 | **Aligned**. This design doc specifies the DSL surface for what Phase 2 describes architecturally. |
| M16 (protocol stack) | **Composes**. Protocol stack provides default HTTP error classification; `response` block overrides at service level. |

---

## Design Decisions

### Why a structural block, not an annotation?

Per `docs/design/modeling/annotation-to-dag-modeling.md`, annotations like
`@error_map(404 => NotFound)` are **Category 2: declared intent with no
enforcement**. The annotation-to-DAG migration explicitly calls for these
to become structural concerns that compose into the transport DAG.

The DSL already models transport as a block (`transport rest { method: POST,
path: "/gists" }`), auth as a config field (`config { auth: BearerToken }`),
and behavioral properties as keywords (`readonly`, `idempotent`). None of
these are `@` annotations. `response` follows the same pattern.

The `@` annotation syntax in CLAUDE.md's reference was **stale documentation**
that didn't match actual DSL syntax. Annotations exist only for type refinement
(`@pattern`, `@range`, `@non_empty`, `@brand`) and a few test config markers.
Everything structural is a block or keyword.

### Why not `Result<T, E>` return types?

Sum type returns would work for pure DSL functions but don't fit transport
operations. The response routing happens at the HTTP protocol layer — the
classify_response node routes on status code **before** constructing typed
output. This is a transport concern, not a type-system concern.

### Why mandatory?

If `response` blocks are optional, they won't get written. The gist 401
bug happened precisely because error handling was optional. Making the
compiler require them means you can't define a REST service without
thinking about what happens when it fails.

---

## Example: GCS ClaimStore after migration

The GCS claim store already handles 412 in tests, but the mapping is
implicit. With `response` blocks it becomes explicit:

```dag
service gcs.ClaimStore : ClaimStore {
  config { bucket: NonEmptyStr, project_id: NonEmptyStr }

  operation acquire {
    input {
      issue_id: NonEmptyStr
      stage: NonEmptyStr
      owner: NonEmptyStr
      lease_ttl_ms: Int
      expected_generation: Int = 0
    }
    output {
      acquired: Bool
      conflict: Bool
      lease_generation: Int
    }

    response {
      200 => output
      412 => GcpError    // precondition failed → conflict: true
      401 => GcpError
      403 => GcpError
      500 => GcpError
    }
  }
}
```

The lowerer generates a classify_response node that routes 412 to the
error path. The parse node for the 412 case sets `conflict: true`,
`acquired: false`. Testgen generates tests for 200, 412, 401, 403, 500
automatically.

---

## Open Questions

1. **Status-to-output-field mapping**: For operations like `acquire` where
   412 maps to `conflict: true` in the *output type* (not a separate error
   type), should the `response` block express this mapping explicitly?
   E.g., `412 => output { acquired: false, conflict: true }`. Or should
   the parse node handle this implicitly via the `from` path extraction?

2. **Protocol stack defaults**: Should the HTTP layer contribute default
   `response` entries (5xx → ServerError) that the service layer can
   override? This avoids repeating `500 => ServerError` on every operation.

3. **Rate limit composition**: Should `429 => RateLimited` entries compose
   with a `retry` block (when implemented per Phase 3 of
   `annotation-to-dag-modeling.md`) to auto-generate retry-after logic?
