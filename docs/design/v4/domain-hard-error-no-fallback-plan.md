# Domain Hard-Error No-Fallback Plan (Items 5, 7, 8)

**Status**: Revised draft for review  
**Date**: 2026-02-25  
**Scope**: Remaining post-PR1-PR5 items except report-coverage lint (item 6 intentionally out of scope)

## Why this revision

The original direction was correct, but still had places where "metadata leakage" could recreate string-coupled behavior:

1. treating `u32` IDs as "stable identity"
2. putting backend details (paths/symbol names) inside semantic asset requirements
3. carrying redundant identity fields in lowered nodes

This revision keeps the same no-fallback goals and tightens the model so semantic authority is explicit and durable.

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

## Core objective

1. Compile DSL to semantic runtime ops (no name heuristics).
2. Derive required ops/assets from reachable semantics.
3. Resolve assets via explicit producers.
4. Emit only when all requirements are satisfied.
5. Produce deterministic artifacts and deterministic diagnostics.

In short: compile succeeds only when the semantic contract is complete.

## Core model

### 1) Stable key vs interned ID (authority split)

Do not treat sequential `u32` as stable identity. Use two layers:

```rust
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StableNodeKey {
    pub origin: OriginKey,       // module path + local semantic position
    pub kind: StableNodeKind,    // callable/pipeline/primitive node class
}

pub struct SemanticNodeId(pub u32); // interned handle only
```

Rules:

1. `Stable*Key` is the source of truth for ordering, hashing, receipts, diagnostics.
2. `u32` IDs are internal handles only (performance/memory).
3. All cross-stage canonicalization uses stable keys, never allocation order.

### 2) Simplified lowered semantic reference

Lowered nodes carry only one semantic anchor:

```rust
pub struct LoweredSemanticRef {
    pub node_id: SemanticNodeId,
}
```

Owner relationships are queryable via side tables:

1. `node_id -> module`
2. `node_id -> callable/pipeline owner`
3. `node_id -> source span`

This avoids mismatch bugs from duplicating IDs on each lowered node.

### 3) Runtime operation identity as structured key

Use semantic keys, not numeric domain-op IDs:

```rust
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum RuntimeOpKey {
    Primitive(PrimitiveOpKind),
    ServiceTransport { service: ServiceKey, phase: ServicePhase },
    DomainCallable(DomainOpKey),
}

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DomainOpKey {
    pub namespace: InternedStr,
    pub name: InternedStr,
    pub version: Option<u32>,
}
```

Authoring contract:

1. Default inference from fully-qualified semantic declaration is supported.
2. Explicit override via DSL annotation is supported for rename-stable external identity.

No `DomainOpId(u32)` is exposed as semantic identity.

### 4) Semantic asset keys (no backend leakage)

Asset requirements must describe what is required, not how to embed it:

```rust
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AssetKey {
    pub namespace: AssetNamespace,
    pub name: InternedStr,
    pub content_type: AssetContentType,
    pub variant: Option<AssetVariant>,
}

pub struct EmbeddedAssetRequirement {
    pub key: AssetKey,
    pub required_by: RuntimeOpKey,
}
```

`layer1_path` and `native_embed_symbol` do not belong in semantic requirements.
Those are target/layer emission details produced later by an embed plan.

### 5) BackendSpec as single authority for op support + asset requirements

Introduce explicit backend contract:

```rust
pub trait BackendSpec {
    fn supports(&self, op: &RuntimeOpKey) -> bool;
    fn required_assets(&self, op: &RuntimeOpKey) -> Vec<AssetKey>;
    fn asset_producer(&self) -> &dyn EmbeddedAssetProducer;
}
```

This prevents hidden registries from reappearing in CLI or emitters.

### 6) Asset production contract

```rust
pub struct EmbeddedAsset {
    pub key: AssetKey,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

pub trait EmbeddedAssetProducer {
    fn produce(&self, key: &AssetKey) -> Result<EmbeddedAsset, AssetProduceError>;
}
```

Rules:

1. Missing producer for required asset is a hard error.
2. Producer failure is a hard error.
3. No stub/default content in emitter or CLI.

## Reachability contract

Hard errors depend on reachability, so semantics must be explicit:

1. `compile-time reachable`: present in static closure from selected entrypoints.
2. `potentially runtime reachable`: reachable under dynamic branch/dispatch uncertainty.

Compile validation is conservative over potentially reachable ops/assets.
No fallback is allowed for potentially reachable requirements.

## Determinism contract

Compile emits deterministic receipt:

```rust
pub struct CompileReceipt {
    pub compile_config_digest: String,   // target/layer/features/toolchain versions
    pub source_digest: String,
    pub semantic_graph_digest: String,   // stable node/edge keys only
    pub required_runtime_ops_digest: String,
    pub required_assets_digest: String,  // includes asset bytes hashes
    pub emitted_files_digest: String,
}
```

Rules:

1. Canonical ordering is by stable key/file path only.
2. Diagnostics are collected then sorted deterministically before rendering.
3. Absolute paths are excluded from digests.

## Compiler stage responsibilities

### Resolve/typecheck

1. Build stable semantic keys and deterministic intern tables.
2. Validate runtime-op annotations.
3. Produce key-based symbol metadata.

### Lower

1. Attach `LoweredSemanticRef`.
2. Attach `RuntimeOpKey` per node.
3. Preserve obligation-derived service phases structurally.

### Derive

1. Compute conservative reachable op set.
2. Validate backend op support via `BackendSpec::supports`.
3. Collect required assets via `BackendSpec::required_assets`.
4. Canonically sort requirements by key.

### Driver

1. Resolve asset bytes via `BackendSpec::asset_producer`.
2. Fail hard on unresolved or failed assets.
3. Build `AssetStore` keyed by `AssetKey`.

### Emit

1. Consume lowered graph + `AssetStore` + backend embed plan.
2. Never inspect module names for behavior.
3. Never generate fallback handlers/content.
4. Hard-fail unsupported op or missing asset content.

## Error model (hard errors only)

1. `UnimplementedRuntimeOp { op_key, target, layer, node_key }`
2. `MissingEmbeddedAssetProducer { asset_key, required_by }`
3. `EmbeddedAssetProductionFailed { asset_key, cause }`
4. `MissingEmbeddedAssetContent { asset_key }`
5. `DeterminismViolation { stage, detail }`

Diagnostics must include semantic keys and be emitted in deterministic order.

## Migration plan

### Phase A: Key model + contracts

1. Add stable key types and interning layer.
2. Collapse lowered refs to `SemanticNodeId` only.
3. Add `RuntimeOpKey` and `AssetKey` scaffolding.

### Phase B: BackendSpec authority

1. Introduce `BackendSpec` interface.
2. Move op support + required-asset mapping behind spec.
3. Keep behavior same, but route through new contracts.

### Phase C: No-fallback asset wiring

1. Add producer plumbing through driver.
2. Delete CLI embed registry logic.
3. Delete emitter stub fallback content and module checks.

### Phase D: No-fallback runtime wiring

1. Delete passthrough fallback config/handler.
2. Hard-fail any reachable unsupported op.
3. Remove remaining module/name heuristic classification paths.

### Phase E: Determinism hardening

1. Add compile-config digest and canonical diagnostic ordering.
2. Extend deterministic tests to CI pipeline and directory compile.
3. Require receipt + emit manifest equality across repeated runs.

## Concrete deletion list

Delete these patterns once migrated:

1. `EMBED_REGISTRY` in CLI.
2. `embed_layer1_handler_data` module/path matching logic.
3. `is_makegen_module(...)` checks in native emitters.
4. `MAKEGEN_STUB_CONTENT` and any stub fallback.
5. `allow_unimplemented_passthrough` and `UnimplementedPassthrough`.
6. Module-name fallback tables in handler classification.
7. Name-prefix heuristics where semantic metadata exists.

## Test plan

### Unit tests

1. Stable key canonical ordering is deterministic.
2. Runtime-op classification derives from semantic metadata.
3. Required assets derive from reachable ops via `BackendSpec`.
4. Missing producer and producer failures hard-fail.
5. No fallback handler/content is emitted.

### Integration tests

1. `compile --layer 1` makegen succeeds with producer, fails without producer.
2. `compile --target go/c/mips` uses same semantic asset contract.
3. Pragma/CI fail-fast on unsupported reachable ops.
4. Diagnostic ordering is deterministic across repeated runs.

### Determinism tests

1. canonical-json determinism for single-file and `dsl/pipelines/ci.dag`.
2. emit-manifest determinism for repeated emit runs.
3. compile-receipt determinism for repeated runs.

## Review decisions requested

1. Runtime op annotation shape:
   - Option A: structured annotation (`@runtime_op(ns.name)` or object form) with inferred default and explicit override.
   - Option B: inference only.
2. Producer ownership boundary:
   - Option A: producers in tool-domain crates, wired through backend spec (recommended).
   - Option B: producers in emitter crate.
3. Rollout strategy:
   - Option A: short migration flag window that switches pipeline selection only; both paths remain hard-error/no-fallback (recommended).
   - Option B: immediate hard switch.

## Summary

The design now cleanly separates semantic identity from runtime handles, keeps asset requirements semantic, and centralizes backend op/asset authority in a testable contract. This removes hidden registries and fallback behavior while strengthening determinism and long-term maintainability.
