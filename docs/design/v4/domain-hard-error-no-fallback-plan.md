# Compile+Link Hard-Error Plan (Items 5, 7, 8)

**Status**: Revised draft for review  
**Date**: 2026-02-25  
**Scope**: Remaining post-PR1-PR5 items except report-coverage lint (item 6 intentionally out of scope)

## Why this framing

The right simplification is to treat runtime handlers and embedded assets as linkable extern symbols.

Model:

1. Compile DSL into target-agnostic IR with symbol references.
2. Link IR against a target/layer backend that resolves extern funcs/assets.
3. Missing symbol resolution is a hard compile/link error.
4. Emit artifacts and deterministic receipts.

This replaces string heuristics, passthrough fallbacks, and stub asset paths with one consistent contract.

## Non-negotiable constraints

1. No fallback execution paths.
2. Missing required semantics/data is a hard error.
3. No module-name dispatch tables in CLI/emitter/runtime.
4. Determinism is a first-class contract.

## Scope

This plan covers:

1. Item 5: ID brittleness (string-coupled classification)
2. Item 7: deterministic compilation contract expansion
3. Item 8: layer-1 embedded asset multi-module semantics

This plan does not cover:

1. Item 6 report coverage lint

## DSL surface

Use explicit extern declarations:

1. `func ... { ... }` for DSL-defined behavior.
2. `extern func ...;` for runtime-provided behavior that must link.
3. `extern asset ...;` for compile-time assets that must link.

This removes the need for ad-hoc runtime annotations and emitter-side name heuristics.

## Minimal identity model

Use two IDs only:

1. `SymbolId` for named program symbols (funcs, pipelines, assets), derived from canonical symbol paths.
2. `NodeId` for graph structure nodes, derived by canonical ordering.

Implementation may intern IDs to `u32`, but stable keys are the ordering/hash authority.

## Minimal IR model

Use three operation forms:

```rust
pub enum OpRef {
    Intrinsic(IntrinsicOp), // primitives/pattern/transport phases
    Call(SymbolId),         // call DSL-defined symbol
    Extern(SymbolId),       // call extern func (must link)
}
```

Transport behavior remains explicit via intrinsic transport ops.

## Linker contract

Backends provide extern resolution:

```rust
pub trait Backend {
    fn resolve_extern_func(&self, sym: SymbolId) -> Option<BackendFn>;
    fn resolve_extern_asset(&self, sym: SymbolId) -> Option<ResolvedAsset>;
}
```

Link step:

1. Compute reachable graph slice from selected entrypoints.
2. Collect required extern funcs/assets.
3. Resolve all symbols through backend.
4. Hard-fail on any missing symbol.

## Assets as extern symbols

Do not run a separate special asset-requirement model. Assets are symbols:

1. `extern asset` declarations add required asset symbols.
2. Backends resolve asset bytes at link time.
3. Resolved asset bytes are part of determinism receipts.

If extern funcs need assets, use one of:

1. Explicit DSL declaration (`@requires_asset(...)` style contract).
2. Backend manifest mapping extern func symbol to required asset symbols.

Either way, requirements remain symbol-based and link-validated.

## Determinism contract

Keep receipt small and aligned with emit-manifest:

```rust
pub struct CompileReceipt {
    pub source_digest: Digest,
    pub program_ir_digest: Digest,
    pub required_extern_funcs_digest: Digest,
    pub required_assets_digest: Digest,
    pub resolved_assets_digest: Digest, // includes asset bytes hashes
    pub emit_manifest_digest: Digest,
}
```

Rules:

1. Canonical ordering for symbols, nodes, and files.
2. No absolute machine-local paths in digests.
3. Diagnostic set and ordering are deterministic.

## Error model (hard errors only)

Primary link errors:

1. `MissingExternFunc { symbol, required_by_node }`
2. `MissingExternAsset { symbol, required_by_symbol }`

Secondary errors:

1. `UnsupportedIntrinsic { intrinsic, target, layer, node }`
2. `DeterminismViolation { stage, detail }`

No passthrough behavior and no stub asset fallback.

## Reachability semantics

Hard errors are checked over a conservative reachable set:

1. statically reachable nodes from selected entrypoints
2. potentially reachable nodes under dynamic branch uncertainty

Potentially reachable extern requirements must resolve; no fallback is allowed.

## Compiler/Linker stage responsibilities

### Resolve/typecheck

1. Build canonical symbol table.
2. Validate `extern` declarations and usage.

### Lower

1. Lower calls to `Call(SymbolId)` or `Extern(SymbolId)`.
2. Keep transport phases as explicit intrinsic ops.

### Link

1. Collect required extern symbols from reachable graph.
2. Resolve symbols against backend.
3. Emit hard missing-symbol errors deterministically.

### Emit

1. Consume linked program + resolved assets.
2. Emit outputs plus deterministic emit manifest.

## Migration plan

### Phase A: Introduce externs

1. Add `extern func` and `extern asset` to parser/typechecker/lowering.
2. Lower extern invocations to `OpRef::Extern(SymbolId)`.
3. Keep behavior parity by routing existing runtime ops through extern symbols.

### Phase B: Add linker with hard missing-symbol errors

1. Implement link step and backend resolution interfaces.
2. Require all extern funcs/assets to resolve.
3. Start failing hard for unresolved extern requirements.

### Phase C: Migrate runtime handlers/assets to extern symbols

1. Convert runtime-only domain handlers into extern declarations.
2. Move asset resolution to extern asset linking.
3. Remove resolver paths that depend on `(module, name)` tables.

### Phase D: Remove fallback surfaces

1. Delete passthrough fallback controls and handler variants.
2. Delete stub embedded asset fallbacks.
3. Delete CLI embed registries and module-name embedding checks.

### Phase E: Determinism hardening

1. Add compile receipt hashes tied to emit-manifest.
2. Add CI determinism gates for single-file + CI pipeline + directory compile.
3. Verify deterministic diagnostic ordering.

## Concrete deletion list

Delete these patterns once migrated:

1. `EMBED_REGISTRY` in CLI.
2. `embed_layer1_handler_data` module/path matching logic.
3. `is_makegen_module(...)` checks in native emitters.
4. `MAKEGEN_STUB_CONTENT` and stub fallbacks.
5. `allow_unimplemented_passthrough` and `UnimplementedPassthrough`.
6. Module/name fallback tables and prefix heuristics in runtime classification.
7. Builder/helper paths explicitly designed for passthrough fallback behavior.

## Test plan

### Unit tests

1. canonical symbol ordering and deterministic symbol hashes.
2. lowering correctly distinguishes `Call` vs `Extern`.
3. linker reports missing extern func/asset with deterministic diagnostics.
4. no fallback handler/content paths remain reachable.

### Integration tests

1. layer-1 compile links required extern funcs/assets and fails when missing.
2. go/c/mips compile paths share same extern symbol requirements.
3. CI and pragma compile fail-fast on unresolved extern requirements.

### Determinism tests

1. canonical-json determinism for single-file and `dsl/pipelines/ci.dag`.
2. emit-manifest determinism across repeated runs.
3. compile-receipt determinism across repeated runs.

## Open review decisions

1. Extern asset dependency expression:
   - Option A: DSL-side explicit requirements per extern func.
   - Option B: backend-side extern manifest per symbol.
2. Backend implementation boundary:
   - Option A: backend/link crates own symbol implementations (recommended).
   - Option B: CLI wires symbol implementations directly.
3. Rollout strategy:
   - Option A: short migration switch that toggles pipeline path only; both paths stay hard-error/no-fallback (recommended).
   - Option B: immediate hard switch.

## Summary

Compile+link framing gives one minimal, enforceable contract: extern symbols must resolve or the build fails. This preserves your no-fallback requirement while reducing model complexity and removing string-coupled registries.
