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

## Completed Issues (Phases A–C)

All original debt items from Phases A (infra extraction), B (mtime fast path), and
C (design fixes) are resolved. Key outcomes:

- **Lint pragmas**: 5 eliminated from `core/ir/src/resource/`; ~19 remain, all in
  legitimate I/O boundary layers. Documented in `tools/disallowed-methods-allowlist.txt`.
- **Hash unification**: All hashing delegates to `gunbc_infra::hash`. SHA-256 policy
  enforced by infra. `ResourceDef` + `compute_key_from_def` = single source of truth.
- **Mtime fast path**: `core/infra/src/freshness.rs` — 0 file reads when inputs unchanged.
- **Design fixes**: `ResourceHandle` per-process secret, `exec_mode` via DAG edges,
  `PrepLevel::dep_name()`, `GraphBuilderId` enum.
- **Infra crate**: `gunbc-infra` is leaf crate with `hash.rs`, `manifest.rs`, `ResourceId`.

For full evidence and migration details, see git history (2026-02-05 commits).

---

### Tier 3: Extensions (Feature Work)

| Feature | Depends On | Priority |
|---------|-----------|----------|
| deps.toml tracking | Infra extraction | High |
| Makefile tracking | Infra extraction | Medium |
| .gitignore tracking | Infra extraction | Medium |
| Per-tool test tracking | Performance fixes | Low |
| ToolHandle unification | Design fixes | Low |

---

## Remaining: Phase D — Extensions

Add new resources to the model as needed. Infrastructure is in place.
Phases A (infra extraction), B (mtime fast path), C (design fixes) are all complete.

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
- `TODO/TODONE/design-resource-performance.md` — Performance considerations (DONE)
- `TODO/TODONE/design-resource-acquisition.md` — Resource trait design (DONE)
