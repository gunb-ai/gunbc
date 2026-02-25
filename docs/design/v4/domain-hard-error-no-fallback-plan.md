# Domain Hard-Error No-Fallback Plan (Items 5, 7, 8)

**Status**: Draft for review  
**Date**: 2026-02-25  
**Scope**: Remaining post-PR1-PR5 items except report-coverage lint (item 6 is intentionally out of scope)

## Why this design exists

The current pain is not just "too many hardcoded strings." The deeper issue is missing domain contracts between:

1. DSL semantics
2. runtime handler semantics
3. embedded compile-time assets
4. backend determinism guarantees

Without those contracts, compiler behavior is implemented via:

1. module/name string matching
2. static ad-hoc registries in CLI/emitter
3. passthrough fallback behavior for unimplemented handlers
4. stub fallback content for missing embedded assets

This design replaces that with explicit domain modeling and hard-error semantics.

## Non-negotiable constraints

1. No fallback execution paths.
2. Missing required semantics/data is a hard compile error.
3. No module-name dispatch tables in CLI/emitter.
4. Determinism is a first-class compile contract, not just a test side effect.

## Scope

This plan covers:

1. Item 5: ID brittleness (string-coupled classification)
2. Item 7: deterministic compilation contract expansion
3. Item 8: Layer-1 embedded asset multi-module semantics

This plan does not cover:

1. Item 6 report coverage lint

## What we are actually trying to accomplish

The real objective is:

1. Compile the DSL graph using semantic IDs, not ad-hoc names.
2. Determine required runtime handlers and required embedded assets from semantics.
3. Materialize assets through explicit producers.
4. Emit code only if all semantic requirements are satisfied.
5. Produce deterministic artifacts and a deterministic compile receipt.

In short: "compile succeeds only when the domain model is complete."

## Domain model

### 1) Semantic identity model (replaces string coupling)

Introduce stable IDs generated during resolve/typecheck:

```rust
pub struct ModuleId(pub u32);
pub struct CallableId(pub u32);
pub struct PipelineId(pub u32);

pub struct SemanticNodeId(pub u32); // node-level semantic identity
```

Lowered nodes keep semantic references:

```rust
pub struct LoweredSemanticRef {
    pub module_id: ModuleId,
    pub callable_id: Option<CallableId>,
    pub pipeline_id: Option<PipelineId>,
    pub semantic_node_id: SemanticNodeId,
}
```

Rules:

1. IDs are allocated by compiler passes, not handwritten.
2. IDs are deterministic for the same source graph.
3. Emit/runtime classification reads IDs + semantic metadata, not `(module, name)` strings.

### 2) Runtime operation model (replaces handler name heuristics)

Each lowered node with runtime behavior has a `RuntimeOpKind`:

```rust
pub enum RuntimeOpKind {
    Primitive(PrimitiveOpKind),
    ServiceTransport(ServicePhase), // Prepare/Execute/Parse from obligation category
    DomainCallable(DomainOpId),     // explicit semantic op id from DSL metadata
}

pub struct DomainOpId(pub u32); // interned from DSL declaration, deterministic
```

How `DomainCallable` is authored:

1. Near-term: explicit DSL annotation on callable/pipeline declarations (recommended).
2. Long-term: remove host handlers by compiling expression bodies directly; then `DomainCallable` shrinks.

Required property:

1. Every reachable `RuntimeOpKind` must map to an implementation in the selected target/layer.
2. If not, compile fails with `UnimplementedRuntimeOp`.

### 3) Embedded asset model (replaces CLI embed registry + stubs)

Define embedded assets as first-class semantic requirements:

```rust
pub struct EmbeddedAssetId(pub u32); // deterministic symbol id

pub struct EmbeddedAssetRequirement {
    pub asset_id: EmbeddedAssetId,
    pub required_by: RuntimeOpKind,
    pub layer1_path: Option<String>,
    pub native_embed_symbol: Option<String>,
    pub content_type: AssetContentType,
}

pub struct EmbeddedAsset {
    pub asset_id: EmbeddedAssetId,
    pub bytes: Vec<u8>,
    pub sha256: String,
}
```

Define producer contract:

```rust
pub trait EmbeddedAssetProducer {
    fn produce(&self, asset_id: EmbeddedAssetId) -> Result<EmbeddedAsset, AssetProduceError>;
}
```

Rules:

1. Required assets are derived from reachable runtime op semantics.
2. Missing producer for a required asset is a hard error.
3. Producer failure is a hard error.
4. No stub/default content exists in emitter or CLI.
5. CLI does not maintain asset lists.

### 4) Determinism model (first-class contract)

Compile emits a deterministic receipt:

```rust
pub struct CompileReceipt {
    pub source_digest: String,
    pub semantic_graph_digest: String,
    pub required_runtime_ops_digest: String,
    pub required_assets_digest: String,
    pub emitted_files_digest: String,
}
```

Rules:

1. Ordering is canonical at every stage (`node_id`, `edge`, `asset_id`, file paths).
2. Asset bytes are part of determinism contract.
3. Emit manifest and compile receipt are both deterministic artifacts.

## Allowed vs disallowed static sets

Allowed:

1. Finite domain enums (`RuntimeOpKind`, `ServicePhase`, target/layer matrix).
2. External dependency identifiers where values are inherently arbitrary.

Disallowed:

1. Module/name string match tables for runtime classification.
2. CLI-local static embedded-asset registries.
3. Stub constants used as fallback content.
4. Passthrough fallback handlers that mask unimplemented behavior.

## Compiler pipeline responsibilities

### Resolve/typecheck

1. Build stable symbol identities (`ModuleId`, `CallableId`, `PipelineId`).
2. Validate any runtime-op annotations.
3. Emit deterministic symbol tables.

### Lower

1. Attach `LoweredSemanticRef` and `RuntimeOpKind` to reachable nodes.
2. Preserve obligation-derived service phases structurally.

### Derive

1. Compute execution slice from selected entrypoints (avoid unreachable-node false failures).
2. Compute `required_runtime_ops`.
3. Compute `required_assets`.
4. Produce deterministic `required_*` ordering.

### Driver

1. Resolve assets through producers for `required_assets`.
2. Fail hard on any unresolved requirement.
3. Pass a complete `AssetStore` to emitters.

### Emit (all targets/layers)

1. Consume `RuntimeOpKind` and `AssetStore` only.
2. Never inspect module-name strings for behavior.
3. Never generate fallback handlers/content.
4. Fail hard on unsupported runtime op or missing asset.

## Error model (hard errors only)

Introduce explicit compile errors:

1. `UnimplementedRuntimeOp { op, target, layer, node_id }`
2. `MissingEmbeddedAssetProducer { asset_id, required_by }`
3. `EmbeddedAssetProductionFailed { asset_id, cause }`
4. `MissingEmbeddedAssetContent { asset_id }`
5. `DeterminismViolation { stage, detail }`

Error messages must include:

1. semantic op identity
2. node id
3. target/layer
4. exact missing requirement

## Migration plan

### Phase A: Model introduction (no behavior change yet)

1. Add semantic IDs and `RuntimeOpKind` fields.
2. Add `required_assets` derivation.
3. Add compile receipt generation.

### Phase B: Switch to semantic authority

1. Update emit/runtime classification to consume `RuntimeOpKind`.
2. Remove module/name match dispatch in emit paths.
3. Route service transport by obligation-derived phase only.

### Phase C: Embedded asset hard contract

1. Introduce `EmbeddedAssetProducer` plumbing in driver.
2. Remove CLI `EMBED_REGISTRY` and ad-hoc embed injection logic.
3. Remove emitter stub fallback content and module checks.

### Phase D: Remove fallback execution

1. Remove `allow_unimplemented_passthrough`.
2. Remove `UnimplementedPassthrough`.
3. Classify only reachable nodes; hard-fail if reachable op has no implementation.

### Phase E: Determinism hardening

1. Expand deterministic tests to `dsl/pipelines/ci.dag` and directory compile.
2. Assert identical compile receipt and emit manifest across repeated runs.

## Concrete deletion list

Delete these implementation patterns once new contract is wired:

1. `EMBED_REGISTRY` in CLI
2. `embed_layer1_handler_data` module/path matching logic
3. `is_makegen_module(...)` checks in native emitters
4. `MAKEGEN_STUB_CONTENT` and any `unwrap_or` stub fallback
5. `allow_unimplemented_passthrough` config and passthrough handler kind
6. module-name fallback tables in handler classification
7. name-prefix heuristics where semantic metadata exists

## Test plan

### Unit tests

1. runtime-op classification from semantic metadata
2. required-asset derivation from reachable execution slice
3. missing asset producer hard-fails
4. producer error hard-fails
5. no fallback handlers emitted

### Integration tests

1. `compile --layer 1` makegen succeeds with produced asset, fails without producer
2. `compile --target go/c/mips` makegen succeeds with same asset contract
3. pragma and CI compile paths fail-fast on unsupported reachable ops

### Determinism tests

1. canonical-json determinism for single-file and CI pipeline
2. emit manifest determinism for repeated emit runs
3. compile receipt determinism for repeated runs

## Review decisions requested

1. DSL annotation form for domain runtime ops:
   - Option A: `@runtime_op("<id>")` on callable/pipeline definitions
   - Option B: infer from type/obligation only (works for primitives/services but not all domain callables)
2. Producer ownership boundary:
   - Option A: producer implementations live in tool-domain crates (recommended)
   - Option B: producer implementations live in emitter crate
3. Rollout strictness:
   - Option A: gated by feature flag for one short migration window
   - Option B: immediate hard-error switch (recommended if CI is ready)

## Summary

This plan intentionally shifts authority from ad-hoc string tables to typed domain semantics. The compiler succeeds only when semantics, handler implementations, and required assets are all present and deterministic. There are no fallback paths.
