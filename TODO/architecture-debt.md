# Architecture Debt: Root Cause Analysis

> **Created**: 2026-02-05
> **Status**: Phase A COMPLETE, Phase B COMPLETE, Phase C COMPLETE
> **Priority**: Phase D — Extensions
>
> This document consolidates all TODO debt under a unified root cause analysis.
> Individual TODOs are tracked here with their relationship to the core issues.

## Executive Summary

The codebase had accumulated debt because **infrastructure code had no home**.
The `gunbc-infra` crate extraction (Phase A) is now complete, which fixed:

1. ~~Lint fights (`#[allow]` pragmas everywhere)~~ — **FIXED**: 5 pragmas eliminated
2. ~~Code duplication (same logic in multiple places)~~ — **FIXED**: shared `ResourceDef` + `compute_key_from_def()`
3. Circular dependency workarounds (strings instead of function refs) — still open
4. Incomplete abstractions (traits that can't access what they need) — still open
5. ~~Non-uniform hashing~~ — **FIXED**: `hash_parts()` centralized in infra, blob uses `ContentHash`

**Remaining**: Phase D (extensions).

---

## Concrete Evidence from Codebase

### Issue 1: Lint Fights — ~19 `#[allow]` Pragmas

The `clippy.toml` disallows `std::fs::*` and `Command::new` to enforce transport
abstraction. But infrastructure code legitimately needs these, so we have pragmas.

**Status (updated 2026-02-05):** Down from 22 to ~19 pragmas. The 5 pragmas in
`core/ir/src/resource/hash.rs` and `core/ir/src/resource/manifest.rs` were
eliminated by the gunbc-infra extraction. New pragmas appeared in
`lib/tools/deps/src/` (manifest loading, installer) and `lib/transport/src/executor.rs`.

**In `core/ir/` (transport layer — legitimate I/O):**
```
core/ir/src/transport/cli.rs              (7 pragmas — CLI tool execution)
core/ir/src/transport/github/cli.rs       (3 pragmas — GitHub CLI execution)
```

**In binaries/generators (approved exceptions):**
```
core/codegen/src/main.rs                  (4 pragmas — bootstrapper)
gunbc-dag/src/bin/testgen.rs              (1 pragma — generator)
```

**In lib crates (newer additions):**
```
lib/transport/src/executor.rs             (1 pragma — transport execution)
lib/tools/deps/src/installer.rs           (1 pragma — dependency installation)
lib/tools/deps/src/manifest.rs            (1 pragma — manifest loading)
```

All documented in `tools/disallowed-methods-allowlist.txt`.

**Pattern**: Pragmas are now properly scoped to I/O boundary layers. The original
issue of pragmas in `core/ir/src/resource/` (types layer) is fully resolved.

### Issue 2: Duplicate Code — Same Hash Logic in Two Places

**Fixed:** hashing inputs are now declared in a shared `ResourceDef` and computed
via `compute_key_from_def`. Both `codegen/main.rs` and `ci/ops.rs` use the same
resource definition, so there is no duplicated hashing logic.

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

Resource freshness hashing is now centralized via `ResourceDef` + `compute_key_from_def`:

| Caller | Hash Function | Status |
|--------|--------------|--------|
| `codegen/main.rs` | `compute_key_from_def(codegen_resource_def)` | ✅ Canonical |
| `ci/ops.rs` | Same resource def + key computation | ✅ Fixed |
| `testgen.rs` | `compute_key_from_def(testgen_resource_def)` | ✅ Fixed |
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
- Codegen/testgen hashing would be duplicated across tools
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
| Duplicate codegen hash logic | ✅ Shared `ResourceDef` + `compute_key_from_def` |
| Non-uniform hashing (blob, review, primitives) | ✅ All delegate to `gunbc_infra::hash` |
| `ManagedResource::compute_key` lacks manifest | ✅ Fixed (2026-02-05) |
| `SimpleResource` silent empty hash | ✅ Fixed (2026-02-05) |
| `check_state` computes keys when missing | ✅ Fixed (2026-02-05) |

### Tier 2: ✅ DONE (Design Fixes)

| Issue | Description | Status |
|-------|-------------|--------|
| Naive hashing | O(n) file reads per check | ✅ mtime fast path in `freshness.rs` |
| No hash caching | Same files hashed repeatedly | ✅ mtime avoids rehashing in common case |
| `ResourceHandle` forgeable | Static marker could be forged | ✅ Per-process random secret (`PROCESS_SECRET` in `handle.rs`) |
| `GUNBC_EXEC_MODE` env var | Global mutable state | ✅ Threaded through DAG as `exec_mode` edge from `runner_env` |
| DAG builders as strings | Circular dep workaround | ✅ Already resolved: `GraphBuilderId` enum + `stringify!()` macro = compile-time safety |
| `PrepLevel→deps` hardcoded | Policy in renderer | ✅ Moved to `PrepLevel::dep_name()` method |

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
- `lib.rs` — `ResourceId` (moved from ir)

Crates updated to use infra: `gunbc-ir`, `gunbc-codegen`, `gunbc-dag`, `gunbc-primitives`, `gunbc-lib-blob`.
All re-exports preserved — zero downstream breakage.

### Phase B: ✅ DONE — Mtime fast path

Completed. `freshness.rs` in `gunbc-infra` provides `check_freshness_mtime()`.
`ManifestEntry.input_file_count` tracks expected file count for fast invalidation.

### Phase C: ✅ DONE — Design fixes

All Tier 2 items resolved:
- `PrepLevel::dep_name()` method replaces free function in renderer
- `exec_mode` threaded through DAG edges (env var removed from source)
- `ResourceHandle` per-process secret prevents forgery
- DAG builders already compile-time safe (no action needed)

### Phase D: Extensions

Add new resources to the model as needed. Infrastructure is in place.

---

## Metrics to Track

| Metric | Before Phase A | After Phase A | After Phase B+C |
|--------|----------------|---------------|-----------------|
| `#[allow]` pragmas in ir/resource | 5 | ✅ 0 | 0 |
| Duplicate hash logic | 2 places | ✅ 1 place | 1 place |
| Crates with direct sha2/hex deps | 4 (ir, blob, primitives, infra) | ✅ 1 (infra) | 1 (infra) |
| Hash implementations | 3 (ContentHash, DefaultHasher, StableHash) | ✅ 1 (all in infra) | 1 |
| Freshness check time | ~500ms? | ~500ms | ✅ <10ms (mtime) |
| Files read per check | All matching | All matching | ✅ 0 (mtime) or changed only |
| Global env vars for state | 1 (GUNBC_EXEC_MODE) | 1 | ✅ 0 (DAG edges) |
| ResourceHandle forgery | Possible (static marker) | Same | ✅ Prevented (per-process secret) |

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
