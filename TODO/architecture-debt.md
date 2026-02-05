# Architecture Debt: Root Cause Analysis

> **Created**: 2026-02-05
> **Status**: Phase A COMPLETE, Phase B in progress
> **Priority**: Phase B — Fix naive hashing (mtime fast path)
>
> This document consolidates all TODO debt under a unified root cause analysis.
> Individual TODOs are tracked here with their relationship to the core issues.

## Executive Summary

The codebase had accumulated debt because **infrastructure code had no home**.
The `gunbc-infra` crate extraction (Phase A) is now complete, which fixed:

1. ~~Lint fights (`#[allow]` pragmas everywhere)~~ — **FIXED**: 5 pragmas eliminated
2. ~~Code duplication (same logic in multiple places)~~ — **FIXED**: single `compute_codegen_input_hash()`
3. Circular dependency workarounds (strings instead of function refs) — still open
4. Incomplete abstractions (traits that can't access what they need) — still open
5. ~~Non-uniform hashing~~ — **FIXED**: `hash_parts()` centralized in infra, blob uses `ContentHash`

**Remaining**: Phase B (mtime fast path) and Phase C (design fixes).

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

**Recent commits adding more exemptions:**
```
c3d753c (2026-01-31) "Add clippy::disallowed_methods exemption for testgen binary
                      (code generator needs direct filesystem access)"
```
Comment in code: "same exemption as gunbc-codegen" — showing the pattern continues.

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

### Issue 5: Non-Uniform Hashing — ✅ RESOLVED

All hashing is now centralized in `gunbc-infra::hash`:

| Caller | Hash Function | Status |
|--------|--------------|--------|
| `codegen/main.rs` | `gunbc_infra::codegen_hash::compute_codegen_input_hash()` | ✅ Canonical |
| `ci/ops.rs` | Same as above (was duplicate, now single source) | ✅ Fixed |
| `testgen.rs` | `compute_testgen_input_hash()` | ✅ Uses HashBuilder |
| `lib/blob` | `gunbc_infra::hash::ContentHash::from_bytes()` | ✅ Fixed (was DefaultHasher) |
| `lib/review` | `gunbc_infra::hash::hash_parts()` via StableHashOp | ✅ Fixed (was colon-separator) |
| `lib/primitives` | `gunbc_infra::hash::hash_parts()` (StableHashOp delegates) | ✅ Canonical |

**Hash algorithm policy (enforced by infra)**:
- **Content hashing**: `ContentHash::from_bytes()` / `HashBuilder` — full SHA-256, 64 hex chars
- **Multi-part ID hashing**: `hash_parts()` — SHA-256 with length-prefix encoding, truncated to 32 hex chars
- **File hashing**: `HashBuilder::update_file()` — path+NUL+length+content+NUL encoding

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

### Tier 1: ✅ DONE (Extract gunbc-infra)

All resolved by infra extraction:

| Issue | Status |
|-------|--------|
| `#[allow(disallowed_methods)]` pragmas in ir/resource/*.rs | ✅ 5 pragmas deleted |
| Duplicate `compute_codegen_input_hash` | ✅ Single fn in `gunbc_infra::codegen_hash` |
| Non-uniform hashing (blob, review, primitives) | ✅ All delegate to `gunbc_infra::hash` |
| `ManagedResource::compute_key` lacks manifest | Open (move to infra in future) |
| `SimpleResource` silent empty hash | Open (fix during future move) |
| `check_state` computes keys when missing | ✅ Fixed (2026-02-05) |

### Tier 2: Design Fixes (After Infra Extraction)

| Issue | Description | Fix |
|-------|-------------|-----|
| Naive hashing | O(n) file reads per check | mtime fast path |
| No hash caching | Same files hashed repeatedly | Per-file hash cache |
| `ResourceHandle` forgeable | `acquire()` is pub | ✅ `acquire()` now `pub(crate)`; still need cap validation on deserialize |
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

### Phase A: ✅ DONE — Extract gunbc-infra

Completed. `gunbc-infra` is a leaf crate with:
- `hash.rs` — `ContentHash`, `HashBuilder`, `hash_parts()` (moved from ir + primitives)
- `manifest.rs` — `ResourceManifest`, `ManifestEntry` (moved from ir)
- `codegen_hash.rs` — `compute_codegen_input_hash()` (deduplicated from codegen + dag)
- `lib.rs` — `ResourceId` (moved from ir)

Crates updated to use infra: `gunbc-ir`, `gunbc-codegen`, `gunbc-dag`, `gunbc-primitives`, `gunbc-lib-blob`.
All re-exports preserved — zero downstream breakage.

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

| Metric | Before Phase A | After Phase A | After Phase B |
|--------|----------------|---------------|---------------|
| `#[allow]` pragmas in ir/resource | 5 | ✅ 0 | 0 |
| Duplicate hash logic | 2 places | ✅ 1 place | 1 place |
| Crates with direct sha2/hex deps | 4 (ir, blob, primitives, infra) | ✅ 1 (infra) | 1 (infra) |
| Hash implementations | 3 (ContentHash, DefaultHasher, StableHash) | ✅ 1 (all in infra) | 1 |
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
- `TODO/refactor-pressure.md` — Recurring root patterns that cause rework
- `TODO/design-unified-resource-model.md` — Resource model design
- `TODO/design-resource-performance.md` — Performance considerations
- `TODO/design-resource-acquisition.md` — Resource trait design
