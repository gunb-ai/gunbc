# Architecture Debt: Root Cause Analysis

> **Created**: 2026-02-05
> **Status**: Active tracking document
> **Priority**: URGENT — Extract gunbc-infra as the next major task
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
5. **Non-uniform hashing** (different algorithms in different places)

**The fix**: Extract `gunbc-infra` crate. This is a force multiplier that
unblocks most other cleanups.

---

## Concrete Evidence from Codebase

### Issue 1: Lint Fights — 20+ `#[allow]` Pragmas

The `clippy.toml` disallows `std::fs::*` and `Command::new` to enforce transport
abstraction. But infrastructure code legitimately needs these, so we have pragmas:

**In `core/ir/` (should be I/O-free):**
```
core/ir/src/resource/hash.rs:34:    #[allow(clippy::disallowed_methods)]
core/ir/src/resource/hash.rs:116:   #[allow(clippy::disallowed_methods)]
core/ir/src/resource/manifest.rs:88:  #[allow(clippy::disallowed_methods)]
core/ir/src/resource/manifest.rs:115: #[allow(clippy::disallowed_methods)]
core/ir/src/resource/manifest.rs:189: #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:326:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:417:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:426:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:665:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:687:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:727:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/cli.rs:762:     #[allow(clippy::disallowed_methods)]
core/ir/src/transport/github/cli.rs:182: #[allow(clippy::disallowed_methods)]
core/ir/src/transport/github/cli.rs:196: #[allow(clippy::disallowed_methods)]
core/ir/src/transport/github/cli.rs:219: #[allow(clippy::disallowed_methods)]
```

**In binaries/generators:**
```
core/codegen/src/main.rs:135:  #[allow(clippy::disallowed_methods)]
core/codegen/src/main.rs:152:  #[allow(clippy::disallowed_methods)]
core/codegen/src/main.rs:190:  #[allow(clippy::disallowed_methods)]
core/codegen/src/main.rs:577:  #[allow(clippy::disallowed_methods)]
core/codegen/src/main.rs:819:  #[allow(clippy::disallowed_methods)]
gunbc-dag/src/bin/testgen.rs:190: #[allow(clippy::disallowed_methods)]
gunbc-dag/src/bin/testgen.rs:356: #[allow(clippy::disallowed_methods)]
```

**Pattern**: Infrastructure code scattered across crates, each needing exemptions.

### Issue 2: Duplicate Code — Same Hash Logic in Two Places

**core/codegen/src/main.rs:790-813:**
```rust
fn compute_codegen_input_hash() -> io::Result<ContentHash> {
    let builder = HashBuilder::new();
    let (builder, codegen_count) = builder.update_glob("core/codegen/src/**/*.rs")?;
    let (builder, ir_count) = builder.update_glob("core/ir/src/**/*.rs")?;
    let builder = builder.update_file("core/codegen/Cargo.toml")?;
    let builder = builder.update_file("core/ir/Cargo.toml")?;
    let rust_version = env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let builder = builder.update_str(&rust_version);
    Ok(builder.finalize())
}
```

**gunbc-dag/src/ci/ops.rs:337-355:** (nearly identical)
```rust
fn compute_codegen_input_hash() -> Result<ContentHash, std::io::Error> {
    let builder = HashBuilder::new();
    let (builder, _) = builder.update_glob("core/codegen/src/**/*.rs")?;
    let (builder, _) = builder.update_glob("core/ir/src/**/*.rs")?;
    let builder = builder.update_file("core/codegen/Cargo.toml")?;
    let builder = builder.update_file("core/ir/Cargo.toml")?;
    let rust_version = std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let builder = builder.update_str(&rust_version);
    Ok(builder.finalize())
}
```

**Why duplicated?** `gunbc-dag` can't import from `gunbc-codegen` (would create
dependency on code generator). Should be in shared `gunbc-infra` crate.

### Issue 3: String-Based Function References (Circular Dep Workaround)

**core/codegen/src/cli_gen.rs:22-25:**
```rust
pub struct CliBinaryDef {
    pub tool_name: String,
    pub description: String,
    pub graph_builder: String,  // "build_gist_graph" as STRING
    pub graph_builder_args: String,
}
```

**gunbc-dag/src/bin/testgen.rs:110-152:** (hardcoded dispatch)
```rust
targets.push(TestgenTarget {
    dag: gunbc_dag::build_bootstrap_graph().unwrap(),  // ACTUAL function
    ...
});
targets.push(TestgenTarget {
    dag: gunbc_dag::build_ci_graph().unwrap(),  // ACTUAL function
    ...
});
```

**Why strings?** `gunbc-codegen` (registry) can't reference `gunbc-dag` (builders)
because that would be circular. So registry stores string names, testgen has
hardcoded function calls. Renaming a builder silently breaks at runtime.

### Issue 4: Naive Hashing — O(n) File Reads Per Check

**Current call sites hash everything:**

```rust
// codegen/main.rs - hashes ~100 files
let (builder, codegen_count) = builder.update_glob("core/codegen/src/**/*.rs")?;
let (builder, ir_count) = builder.update_glob("core/ir/src/**/*.rs")?;

// testgen.rs - hashes ~200 files
let (builder, dag_count) = builder.update_glob("gunbc-dag/src/**/*.rs")?;
let (builder, ir_count) = builder.update_glob("core/ir/src/**/*.rs")?;
let (builder, lib_count) = builder.update_glob("lib/**/src/**/*.rs")?;

// ci/ops.rs - hashes same ~100 files AGAIN
let (builder, _) = builder.update_glob("core/codegen/src/**/*.rs")?;
let (builder, _) = builder.update_glob("core/ir/src/**/*.rs")?;
```

**Total per CI run**: ~400 file reads even when nothing changed.

**Should be**: 0 file reads (mtime fast path) or ~3 file reads (only changed files).

### Issue 5: Non-Uniform Hashing — Different Algorithms in Different Places

**Hash implementations in the codebase:**

| Location | Algorithm | Purpose |
|----------|-----------|---------|
| `core/ir/src/resource/hash.rs` | SHA-256 (sha2 crate) | Resource freshness keys |
| `lib/blob/src/lib.rs` | `DefaultHasher` (64-bit) | BlobMeta content hash |

**Problem**: Two different hashing strategies for content identity:

1. **SHA-256 (resource model)** — Cryptographically strong, 256-bit, hex-encoded
2. **DefaultHasher (blob)** — Fast but weak, 64-bit, collision-prone

**lib/blob/src/lib.rs:158-166:**
```rust
fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Simple hash for now - could use SHA256 later
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
```

**Note**: Comment says "could use SHA256 later" — should be unified.

**Hash/staleness call sites inventory:**

| Caller | Hash Function | Inputs | Output |
|--------|--------------|--------|--------|
| `codegen/main.rs` | `compute_codegen_input_hash()` | codegen+ir sources, Cargo.toml, RUSTC_VERSION | manifest `build:generated_cli` |
| `ci/ops.rs` | `compute_codegen_input_hash()` | (DUPLICATE of above) | freshness check |
| `testgen.rs` | `compute_testgen_input_hash()` | dag+ir+lib sources, codegen key | manifest `build:generated_tests` |
| `lib/blob` | `BlobMeta::compute_hash()` | blob content | metadata field |

**Post-infra extraction**:
- Single `compute_codegen_input_hash()` in `gunbc-infra`
- Migrate `BlobMeta` to use infra's `ContentHash` or keep separate (if perf-critical)
- Document hash algorithm policy: SHA-256 for identity, DefaultHasher only for perf-critical non-security uses

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

## Known Limitations

### RUSTC_VERSION Environment Variable

The hash computation for codegen includes `RUSTC_VERSION` from the environment:

```rust
let rust_version = env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
let builder = builder.update_str(&rust_version);
```

**Limitation**: If `RUSTC_VERSION` is not set, hash defaults to "unknown". This means:
- Hash doesn't change when rustc version changes (unless env var is set)
- CI must set `RUSTC_VERSION` explicitly for proper cache invalidation

**Why not run `rustc --version` directly?**

The codebase uses a resource/upsert model where all I/O goes through the transport
abstraction. Adding direct `Command::new("rustc")` calls would:
1. Bypass the transport abstraction (lint violation)
2. Add another `#[allow(clippy::disallowed_methods)]` pragma
3. Contradict the design goal of making I/O observable

**Proper fix (post-infra extraction)**:
- Add a `RustcVersion` resource to the model
- Have gunbc-infra provide a `compute_rustc_version()` function
- Let the resource system cache and track this like any other input

**Current workaround**: Set `RUSTC_VERSION` in CI/build scripts.

---

## References

- `TODO_hacks` — Detailed hack descriptions
- `TODO/design-unified-resource-model.md` — Resource model design
- `TODO/design-resource-performance.md` — Performance considerations
- `TODO/design-resource-acquisition.md` — Resource trait design
