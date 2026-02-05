# Architecture Debt: Root Cause Analysis

> **Created**: 2026-02-05
> **Status**: Active tracking document
>
> This document consolidates all TODO debt under a unified root cause analysis.
> Individual TODOs are tracked here with their relationship to the core issues.

## Executive Summary

The codebase has accumulated debt because **infrastructure code has no home**.
Every feature that needs real I/O (hashing, manifests, file generation) gets
wedged into crates designed for other purposes, leading to:

1. Lint fights (`#[allow]` pragmas everywhere)
2. Code duplication (same logic in multiple places)
3. Circular dependency workarounds (strings instead of function refs)
4. Incomplete abstractions (traits that can't access what they need)

**The fix**: Extract `gunbc-infra` crate. This is a force multiplier that
unblocks most other cleanups.

---

## Core Issue: Missing Infrastructure Layer

### Current Crate Structure

```
gunbc-ir        (types, traits)     ← NO I/O allowed by lint
gunbc-exec      (DAG execution)     ← Uses transport abstraction
gunbc-dag       (tools, binaries)   ← Domain code
gunbc-codegen   (code generation)   ← Needs real I/O but can't import dag
```

### Missing Layer

```
gunbc-infra     (hashing, manifest, resource coordination)
                ← Real I/O allowed
                ← Shared by codegen, dag, future tools
```

### Why This Matters

Without `gunbc-infra`:
- Hash/manifest code lives in `gunbc-ir` with `#[allow]` pragmas
- `compute_codegen_input_hash()` duplicated in codegen AND ci/ops.rs
- `ManagedResource` trait can't properly implement `InputPattern::Resource`
- Every new infrastructure feature repeats this pattern

---

## Secondary Issue: Naive Hashing Strategy

### Current Behavior

Every freshness check does:
1. Expand all glob patterns (filesystem walk)
2. Read every matching file
3. Hash all contents
4. Compare to stored key

This is **O(files × file_size)** on every check, even when nothing changed.

### Why This Is Wrong

Make solved this in 1976: **use mtime as fast path**.

```
Fast path (99% of checks):
  manifest_mtime vs max(source_mtimes)
  If manifest newer → trust it → Fresh

Slow path (only when sources changed):
  Re-hash changed files only
  Compare keys
```

### Better: Trust Git

In a git repo, `git status --porcelain` tells us exactly what changed.
No need to walk the filesystem ourselves.

```rust
// Pseudocode for git-aware freshness
fn check_freshness(resource_id: &ResourceId) -> ResourceState {
    let manifest_entry = manifest.get(resource_id)?;

    // Fast path: ask git if anything in our input patterns changed
    let changed_files = git_status_for_patterns(&manifest_entry.input_patterns)?;

    if changed_files.is_empty() {
        return ResourceState::Fresh;
    }

    // Slow path: something changed, re-hash
    let new_key = compute_key_from_files(&changed_files)?;
    if new_key == manifest_entry.key {
        ResourceState::Fresh
    } else {
        ResourceState::Stale { reason: "inputs changed" }
    }
}
```

---

## Consolidated TODO Inventory

### Tier 1: Blocking (Extract gunbc-infra)

These all get fixed by the infra extraction:

| Issue | Current Location | After Extraction |
|-------|-----------------|------------------|
| `#[allow(disallowed_methods)]` pragmas | ir/resource/*.rs | Deleted (infra has no lint) |
| Duplicate `compute_codegen_input_hash` | codegen + ci/ops.rs | Single fn in infra |
| `ManagedResource::compute_key` lacks manifest | ir/resource/managed.rs | Can properly impl in infra |
| `SimpleResource` silent empty hash | ir/resource/managed.rs | Proper impl or remove |
| `check_state` computes keys when missing | ir/resource/managed.rs | Fix during move |

### Tier 2: Design Fixes (After Infra Extraction)

| Issue | Description | Fix |
|-------|-------------|-----|
| Naive hashing | O(n) file reads per check | mtime fast path |
| No hash caching | Same files hashed repeatedly | Per-file hash cache |
| `ResourceHandle` forgeable | `acquire()` is pub | Make pub(crate) |
| `GUNBC_EXEC_MODE` env var | Global mutable state | Pass through context |
| DAG builders as strings | Circular dep workaround | Registry in shared crate |
| `PrepLevel→deps` hardcoded | Policy in renderer | Declarative resource model |

### Tier 3: Extensions (Feature Work)

| Feature | Depends On | Priority |
|---------|-----------|----------|
| deps.toml tracking | Infra extraction | High |
| Makefile tracking | Infra extraction | Medium |
| .gitignore tracking | Infra extraction | Medium |
| Per-tool test tracking | Performance fixes | Low |
| ToolHandle unification | Design fixes | Low |

---

## Recommended Execution Order

### Phase A: Extract gunbc-infra (unblocks everything)

1. Create `gunbc-infra` crate with its own `clippy.toml` (no disallowed_methods)
2. Move `core/ir/src/resource/hash.rs` → `gunbc-infra/src/hash.rs`
3. Move `core/ir/src/resource/manifest.rs` → `gunbc-infra/src/manifest.rs`
4. Add `compute_codegen_input_hash()` to infra (single source)
5. Update `gunbc-ir` to re-export from `gunbc-infra`
6. Update `gunbc-codegen` and `gunbc-dag` to use infra
7. Delete all `#[allow(clippy::disallowed_methods)]` pragmas

**Effort**: ~2-3 hours
**Impact**: Unblocks Tier 2 and Tier 3

### Phase B: Fix naive hashing

1. Add mtime tracking to `ManifestEntry`
2. Implement mtime fast path in `check_freshness()`
3. Add per-file hash cache
4. Optional: git-aware freshness for git repos

**Effort**: ~2-3 hours
**Impact**: Freshness checks go from O(n×size) to O(1) in common case

### Phase C: Design fixes

Work through Tier 2 items as needed. Each is independent.

### Phase D: Extensions

Add new resources to the model as needed. Infrastructure is in place.

---

## Metrics to Track

| Metric | Current | After Phase A | After Phase B |
|--------|---------|---------------|---------------|
| `#[allow]` pragmas | 6+ | 0 | 0 |
| Duplicate hash logic | 2 places | 1 place | 1 place |
| Freshness check time | ~500ms? | ~500ms | <10ms |
| Files read per check | All matching | All matching | 0 (mtime) or changed only |

---

## References

- `TODO_hacks` — Detailed hack descriptions
- `TODO/design-unified-resource-model.md` — Resource model design
- `TODO/design-resource-performance.md` — Performance considerations
- `TODO/design-resource-acquisition.md` — Resource trait design
