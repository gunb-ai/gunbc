# Design: Annotation-to-DAG Modeling Migration

**Status**: Draft
**Date**: 2026-02-23
**Related tasks**: M22, M8 (metadata separation), M10 (resource declarations), M16 (transport unification)
**Reference**: `docs/handbook.md` § "Compositional Modeling Philosophy"

## Problem

The DSL has **43 unique annotations** with ~2,630 usages. Only ~26 are consumed by the
compiler. The remaining ~17 are parsed but silently ignored — the compiler's forward
compatibility strategy accepts unknown annotations without error:

```rust
// daglang-typecheck/src/lib.rs
_ => {
    // Unknown annotations are silently accepted for forward compatibility
}
```

This creates three problems:

1. **Silent breakage**: An annotation like `@error_map(404 => NotFound)` looks like it
   does something, but the compiler never generates error classification code from it.
   The author believes their error handling is modeled; it's actually documentation.

2. **Parallel truth**: `@contract` has 43 usages on interfaces. The compiler generates
   `verify_contract_*` nodes, but these are scaffolding — no actual proof obligations
   are enforced. The annotation declares a contract; the system doesn't verify it.

3. **Compositional model gap**: `@error_map`, `@retry`, `@requires`, `@credential` all
   describe real transport/resource concerns that should **compose into generated code**
   per the compositional modeling philosophy. Instead, they're inert metadata that the
   protocol stack can't reason about.

## Annotation Census

### Category 1: Working infrastructure (keep as annotations)

These are load-bearing and correctly consumed:

| Annotation | Count | Consumer | What it generates |
|------------|-------|----------|-------------------|
| `@rest`, `@shell`, `@file` | ~50 | lower | Transport class selection |
| `@endpoint`, `@auth`, `@headers` | ~30 | lower | Request construction |
| `@permissions` | ~15 | lower | Scope validation |
| `@body_template`, `@parse`, `@json` | ~20 | lower | Body encoding/decoding |
| `@idempotent`, `@readonly` | ~10 | lower | Behavioral properties |
| `@content`, `@brand`, `@non_empty` | ~20 | typecheck | Type refinement predicates |
| `@pattern`, `@range`, `@file_types` | ~10 | typecheck | Validation predicates |
| `@tier`, `@hermetic`, `@mock`, etc. | ~20 | emit | Test harness config |
| `@interactive` | ~5 | lower | Callable flag |
| `@outputs` | ~5 | lower | Output path tracking |

**No change needed** — these work. Some of these (transport annotations) will
eventually be subsumed by the protocol stack model (M16), but they're correct today.

### Category 2: Declared intent, no enforcement (model as DAG concerns)

These describe real system properties that the compiler should enforce:

| Annotation | Count | What it declares | What it should generate |
|------------|-------|------------------|------------------------|
| `@contract` | 43 | Behavioral spec on interface capability | Compile-time proof obligations; generated contract tests per implementation |
| `@error_map` | 4 | Status/body → error classification | Error classifier nodes in transport DAG; testgen generates per-status-code tests |
| `@retry` | 1 | Retry policy (backoff, budget) | Retry wrapper node in transport DAG; testgen generates retry/exhaust tests |
| `@requires` | 5 | Dependency/prerequisite declarations | Resource/capability edges in DAG; M10 resource admission checks |
| `@testgen_skip` | 3 | Exclude from test generation | Consumed by testgen emit; skip listed targets |

### Category 3: Metadata noise (delete or migrate to `TypeOp::Meta`)

These are either documentation tags or leftover scaffolding:

| Annotation | Count | Disposition |
|------------|-------|-------------|
| `@tool` | 14 | **Migrate to Meta**: access mode metadata on operations, useful for M14 (inventory) |
| `@format` | 2 | **Migrate to Validate**: `@format(uuid)` → `Predicate::Matches(UUID_PATTERN)` |
| `@pred` | 4 | **Already consumed** (type predicates — actually Category 1) |
| `@invariant` | 2 | **Delete or migrate to contract**: if it's a real invariant, it's a contract |
| `@mode` | 3 | **Migrate to Meta**: service mode metadata |
| `@network` | 2 | **Delete**: redundant with transport class (`@rest` implies network) |
| `@credential` | 1 | **Delete**: redundant with `@auth` |
| `@external` | 1 | **Delete**: redundant with transport class |
| `@derived_from` | 1 | **Delete**: source tracking, not a system property |
| `@ledger` | 2 | **Delete**: interface metadata, not a system property |
| `@test_hermetic` | 1 | **Merge into @hermetic**: duplicate of existing `@hermetic` |
| `@test_integration` | 1 | **Merge into @tier(Integration)**: duplicate of existing `@tier` |

## Design: Category 2 Migration

### Phase 1: `@contract` → Compile-Time Proof Obligations

**Current state**: `@contract` generates `verify_contract_*` DAG nodes, but these are
scaffolding stubs that don't enforce anything.

**Target state**: Each `@contract` on an interface capability becomes a **proof obligation**
that every implementation must satisfy at compile time.

```dag
interface ClaimStore {
    @contract("acquire returns claim with valid lease_id and expiry > now")
    capability acquire(issue_id: String, stage: String) -> Claim

    @contract("heartbeat extends expiry; fails if claim expired or not held")
    capability heartbeat(claim: Claim) -> Claim

    @contract("release is idempotent; succeeds even if claim already released")
    capability release(claim: Claim) -> Bool
}
```

Each contract generates:
1. A **typed test obligation** in the testgen output for every implementation
2. A **verification node** in the implementation's DAG that asserts the postcondition
3. A **CI gate** that fails if any implementation lacks contract test coverage

This connects to M12 (coercion proof nodes) — contracts are proof obligations on
interface boundaries, just as coercion proofs are proof obligations on type boundaries.

### Phase 2: `@error_map` → Error Classification Layer

**Current state**: `@error_map` is parsed but generates nothing.

**Target state**: `@error_map` composes into the transport DAG as an error classification
node between the transport execute node and the caller.

```dag
service github.Gist {
    @rest(POST, "/gists")
    @error_map({
        404 => NotFound,
        422 => ValidationError,
        401 => AuthError,
    })
    operation Create { ... }
}
```

Generates a classification node in the transport DAG:

```
execute_transport → classify_error → caller
                         │
                    (404 → NotFound)
                    (422 → ValidationError)
                    (401 → AuthError)
                    (5xx → ServerError)    ← from protocol stack default
```

The protocol stack (M16) provides default error classification per layer
(HTTP 4xx/5xx). `@error_map` overrides specific codes at the service layer.
This is compositional: REST layer defaults + service-specific overrides.

Testgen derives per-status-code test obligations from the error map:
- Mock 404 response → assert NotFound error
- Mock 422 response → assert ValidationError
- Mock 200 response → assert success

### Phase 3: `@retry` → Retry Policy Layer

**Current state**: `@retry` appears once as a pattern example, generates nothing.

**Target state**: `@retry` composes into the transport DAG as a retry wrapper.

```dag
service github.Gist {
    @rest(POST, "/gists")
    @retry(max: 3, backoff: exponential, retryable: [429, 503])
    operation Create { ... }
}
```

Generates a retry wrapper in the transport DAG:

```
caller → retry_gate → execute_transport → classify_error
              ↑                                  │
              └──── (retryable?) ←───────────────┘
```

The retry policy composes with the error classification layer and the protocol
stack's `retryable` status codes. Testgen derives:
- Mock 429 → assert retry (up to max)
- Mock 503 → assert retry
- Mock 429 x (max+1) → assert retry exhaustion error

### Phase 4: `@requires` → Resource/Capability Edges

**Current state**: `@requires` on test blocks, parsed but not enforced.

**Target state**: `@requires` becomes a structural dependency edge in the DAG.

```dag
@requires(tool: "cargo", min_version: "1.75.0")
@requires(network: true)
test gist_create_integration {
    ...
}
```

The compiler generates:
1. Resource edges in the test DAG (`tool:cargo`, `network`)
2. M10 resource admission checks (fail if resource unavailable)
3. M11 strict dry-run poison if resource not wired

This connects directly to M10 (mandatory resource declarations) — `@requires` is
the DSL surface for declaring resource dependencies.

### Phase 5: `@testgen_skip` → Consumed by Emit

Simple: wire `@testgen_skip` into `daglang-emit` test generation so marked items
are excluded from generated test files.

## Design: Category 3 Cleanup

### Immediate deletions

Remove annotations with zero semantic content:
- `@network` → redundant with `@rest`/`@shell`/`@file` transport class
- `@credential` → redundant with `@auth`
- `@external` → redundant with transport class
- `@derived_from` → documentation, not a system property
- `@ledger` → documentation, not a system property

### Migrations

- `@tool(READ)` / `@tool(WRITE)` → `TypeOp::Meta(MetadataPayload::AccessMode(...))` (M8)
- `@format(uuid)` → `@pattern("^[0-9a-f]{8}-...")` (already supported)
- `@invariant(...)` → `@contract(...)` on the parent interface (same semantics)
- `@mode(Snapshot)` → `TypeOp::Meta(MetadataPayload::ServiceMode(...))` (M8)
- `@test_hermetic` → `@hermetic` (already exists)
- `@test_integration` → `@tier(Integration)` (already exists)

### Compiler strictness

After cleanup, change the forward-compatibility behavior:

```rust
// Before:
_ => { /* Unknown annotations silently accepted */ }

// After:
_ => {
    diagnostics.push(Diagnostic::warning(
        format!("unknown annotation @{}", name),
        "annotation is not consumed by any compiler phase",
    ));
}
```

In strict mode (`--strict`), unknown annotations become errors. This prevents
new inert annotations from accumulating.

## Migration Path

1. **Phase 0: Audit and delete** — Remove Category 3 deletions. Migrate duplicates.
   Add unknown-annotation warnings. No behavioral change in generated code.

2. **Phase 1: `@contract` proof obligations** — Depends on M12 (proof nodes).
   Generate typed test obligations from `@contract` annotations. CI gate for
   uncovered implementations.

3. **Phase 2: `@error_map` classification** — Depends on M16 (protocol stack).
   Wire error classification into transport DAG. Testgen derives per-status tests.

4. **Phase 3: `@retry` policy** — Depends on Phase 2. Wire retry wrapper into
   transport DAG. Testgen derives retry/exhaustion tests.

5. **Phase 4: `@requires` resources** — Depends on M10 (resource declarations).
   Wire `@requires` as structural resource edges.

6. **Phase 5: `@testgen_skip` consumption** — Independent. Wire into emit.

## Scope & Non-Goals

### In scope
- Delete or migrate Category 3 annotations
- Add unknown-annotation compiler warnings (strict mode: errors)
- Wire `@contract` into proof obligation generation
- Wire `@error_map` into transport error classification
- Wire `@retry` into transport retry policy
- Wire `@requires` into resource edge generation
- Wire `@testgen_skip` into emit

### Not in scope
- Changing Category 1 annotations (they work)
- Full protocol stack implementation (M16 handles that)
- Runtime retry/circuit-breaking middleware (future work)
- Contract verification at runtime (compile-time obligations only)

## Relationship to Other Tasks

- **M8** (metadata separation): Category 3 `@tool`/`@mode` migrate to `TypeOp::Meta`
- **M10** (resource declarations): `@requires` becomes the DSL surface for M10
- **M12** (coercion proof nodes): `@contract` proof obligations use the same framework
- **M16** (protocol stack): `@error_map` and `@retry` compose into the protocol layers
- **M21** (structural primitives): No direct dependency, but both are "make the compiler
  derive from structure" tasks
