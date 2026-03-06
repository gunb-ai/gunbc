# Design: Self-Describing Mock Responses via `@mock_response` Pipeline

**Status**: Draft
**Date**: 2026-02-27
**Related**: `service-codegen.md` §10.3, `modeling/protocol-stack-layering.md` §Step 5
**Motivation**: 5 cleanup/refactor opportunities discovered during SDLC execution gap fixes

## Problem

DryRun execution of services with non-trivial response shapes (LLM, GCP Secret Manager,
GitHub PR API) produces empty/default values because the auto-mock system doesn't know
what the response body should look like. Today this is papered over by a "kitchen sink"
`default_rest_response()` that grows a new blob of fields for every service type. This
violates single-source-of-truth: the DSL already declares response shapes via `from`
paths on output fields, but the mock system ignores them.

Five related issues compound the problem:

| # | Issue | Symptom | Root Cause |
|---|-------|---------|------------|
| 1 | `@mock_response` dead scaffolding | Annotation type exists in AST, parser never populates it | Parser, lowerer pipeline incomplete |
| 2 | Kitchen sink `default_rest_response()` | One static JSON blob for all REST services | No per-service mock shape |
| 3 | `from` path format split (`.` vs `/`) | Two conventions, fragile `navigate_json_path` | No canonical path format |
| 4 | `IdentityCallableOp` overloading | Debug output doesn't distinguish node intents | One op for 3 unrelated roles |
| 5 | Pessimistic probe ordering | REST services waste 2 trial executions per parse node | `[Shell, File, REST]` ordering |

These are not independent — they're all consequences of **mock responses not being part
of the service contract**. When `@mock_response` is wired end-to-end, issues 1-3 resolve
structurally and issues 4-5 become trivial.

## Design

### Principle: Mock data is a layer in the protocol stack

From `protocol-stack-layering.md`:

```
Service layer:  @mock_response(status: 200, body: { "content": [...] })
  REST layer:   default 200/4xx/5xx mock responses (derived from status policy)
    HTTP layer: status code classification (2xx=success, 4xx=client, 5xx=server)
      TCP layer: connect/timeout/refuse mock scenarios
```

Each layer provides baseline mock data; higher layers override with domain-specific
content. The DSL annotation `@mock_response` is the service-layer override — it provides
the one piece of information no protocol layer can derive: **what does this specific API
return on success?**

### Phase 1: Wire `@mock_response` parser → lowerer → RestOperationSpec

**Parser** (`daglang-syntax/src/parser.rs`):
- In `parse_operation_inner()`, after parsing `transport`, check for `@mock_response`
  annotation and populate the existing `mock_response: Vec<MockResponseDef>` field
- Parse syntax: `@mock_response(status: N, body: { ... })` where body is an `Expr`

**Lowerer** (`daglang-lower/src/lib.rs`):
- Add `mock_response: Option<MockResponseValue>` to `RestOperationSpec`
- `MockResponseValue { status: u16, body: serde_json::Value }` — evaluated at lower time
- In `extract_rest_spec()`, read `operation.mock_response.first()` and evaluate the body
  expression to JSON via `expr_to_json_literal()`

**Outcome**: `RestOperationSpec` carries mock data through the compiler pipeline.

### Phase 2: Auto-mock consumes `@mock_response` from spec

**`mock_defaults.rs`**:
- In `default_value_for_slot()`, when the slot is `TransportResponse` and the downstream
  consumer is a `GenericRestParseOp`, check if the parse op's spec has a `mock_response`
- If present: build `RestResponse::new(spec.mock_response.status, spec.mock_response.body)`
- If absent: fall back to the synthesized response (Phase 3)

**`resolve_service.rs`**:
- `GenericRestParseOp` already carries `spec: RestOperationSpec`
- Expose the `mock_response` field so the auto-mock can read it via the resolved DAG

**Outcome**: Services with `@mock_response` get accurate DryRun data. Kitchen sink
response is only used for services without the annotation.

### Phase 3: Synthesize mock responses from output field paths

For services **without** `@mock_response`, the auto-mock can still do better than
a static blob. The output field specs (`OutputFieldSpec.json_path`) describe exactly
what paths the parse node will navigate. Synthesize a minimal response body that
satisfies all output paths:

```rust
fn synthesize_mock_body(output_fields: &[OutputFieldSpec]) -> serde_json::Value {
    let mut root = serde_json::Map::new();
    for field in output_fields {
        // Navigate/create path segments, inserting objects and arrays as needed.
        // Numeric segments create arrays; string segments create objects.
        insert_at_path(&mut root, &field.json_path, mock_value_for_type(&field.type_id));
    }
    serde_json::Value::Object(root)
}
```

Given `content: String from "content/0/text"`, this builds:
```json
{ "content": [{ "text": "mock" }] }
```

**Outcome**: The kitchen sink `default_rest_response()` shrinks to OAuth/token fields
only. LLM, GCP, and any future REST service gets correct mock shapes automatically.
No more editing `mock_defaults.rs` when adding services.

### Phase 4: Normalize `from` path format

The `from` syntax in DSL output fields uses two conventions today:
- `.` separator: `head.sha`, `base.ref` (GitHub services) — 3 occurrences
- `/` separator: `content/0/text`, `usage/input_tokens` (LLM, GCP) — 12+ occurrences

**Decision**: Normalize to `/` (JSON Pointer-like).

Rationale:
- `/` is the majority convention (4:1 ratio)
- `/` naturally supports array indices (`content/0/text` vs ambiguous `content.0.text`)
- Aligns with RFC 6901 JSON Pointer (familiar to API developers)
- No conflict with DSL field access syntax (which uses `.`)

**Migration**:
1. Update 3 GitHub `from` paths: `head.sha` → `head/sha`, `base.ref` → `base/ref`
2. Simplify `navigate_json_path` to split on `/` only (remove `.` fallback)
3. Add compiler warning for `from` paths containing `.` (migration aid)

### Phase 5: Disambiguate overloaded ops

Replace overloaded `IdentityCallableOp` with intent-specific ops:

| Current | Replacement | Purpose |
|---------|-------------|---------|
| `IdentityCallableOp` (CallParamSource) | `CallParamSourceOp` | **Done** (this PR) |
| `IdentityCallableOp` (ContentUpsertOutputPath) | `MetadataPassthroughOp` | Metadata-only node, no-op |
| `IdentityCallableOp` (DSL callable passthrough) | `DeclaredOutputCallableOp` | DSL fn/func passthrough |

Each op has a descriptive `Debug` impl for tracing. Execution behavior is identical
(identity/passthrough) but intent is unambiguous.

### Phase 6: Optimize response probe ordering

Reorder `probe_best_response` candidates from `[Shell, File, REST]` to `[REST, Shell, File]`.

REST services are the majority of transport nodes. Trying REST first skips 2 wasted
trial executions per REST parse node. Shell-first ordering made sense historically when
most services were shell-based; the REST-heavy SDLC workflow inverts the ratio.

## Dependency Graph

```
Phase 1 (parser + lowerer)
  ↓
Phase 2 (auto-mock consumes spec)
  ↓
Phase 3 (synthesize from output paths) ← can start in parallel with Phase 2
  ↓
Phase 4 (normalize from paths) ← independent, can start anytime
Phase 5 (disambiguate ops) ← independent, can start anytime
Phase 6 (probe ordering) ← independent, trivial
```

Phases 4, 5, 6 are independent leaf tasks with no prerequisites. Phases 1→2→3 are
the critical path.

## Verification

### Per-phase acceptance criteria

| Phase | Test | Gate |
|-------|------|------|
| 1 | Parser round-trip: `@mock_response` survives parse→emit | `daglang-syntax` unit test |
| 1 | Lowerer: `RestOperationSpec.mock_response` populated for LLM services | `daglang-lower` unit test |
| 2 | DryRun LLM parse nodes produce "Mock design: ..." content | `gunbc-app` integration test |
| 3 | Service without `@mock_response` still gets correct mock shape | `mock_defaults` unit test |
| 3 | Kitchen sink `default_rest_response()` has no LLM/GCP fields | Code review |
| 4 | `grep 'from ".*\\..*"' dsl/` returns 0 results | CI lint |
| 5 | `grep IdentityCallableOp gunbc-app/src/resolve.rs` returns ≤1 result | Code review |
| 6 | `probe_best_response` tries REST first | Unit test |

### End-to-end acceptance

```bash
gunbc-sdlc --profile unit_test --dry-run
```

Output shows stage transitions (idea→design→...→done) with non-empty LLM content in
design/review stages. No `"(unresolved)"`, no empty strings where content is expected.

## Relationship to Protocol Stack Layering

This design is **Phase 3** of `protocol-stack-layering.md` (testgen derives baseline
obligations from protocol stack). Once the protocol stack model splits HTTP/REST/TCP
into layered system models, each layer provides baseline mock data:

- **TCP**: `{connected: true}` / `{error: "connection_refused"}`
- **HTTP**: `{status: 200, body: ""}` / `{status: 500, body: "Internal Server Error"}`
- **REST**: `{status: 200, body: {}}` / `{status: 401, body: {message: "Unauthorized"}}`
- **Service**: `@mock_response(status: 200, body: { ... })` — domain-specific

The service `@mock_response` overrides the REST-layer default. For services without the
annotation, Phase 3's synthesized response fills the gap. The kitchen sink response is
eliminated entirely — every mock is either derived from the protocol stack or declared
by the service.

## Cost Estimate

| Phase | Files | Lines | Risk |
|-------|-------|-------|------|
| 1 | 3 (parser, lowerer, syntax) | ~60 | Medium — parser adjacency issues (see MEMORY.md annotation adjacency) |
| 2 | 2 (mock_defaults, resolve_service) | ~30 | Low — plumbing only |
| 3 | 1 (mock_defaults) | ~40 | Low — pure function, well-tested |
| 4 | 4 (3 .dag files, navigate_json_path) | ~10 | Low — mechanical rename |
| 5 | 1 (resolve.rs) | ~20 | Low — extract existing code |
| 6 | 1 (mock_defaults) | ~5 | Low — reorder array |
| **Total** | **~10 files** | **~165 lines** | |
