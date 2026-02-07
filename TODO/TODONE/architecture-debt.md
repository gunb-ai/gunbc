# Architecture Debt: Root Cause Analysis

> **Created**: 2026-02-05
> **Completed**: 2026-02-05
> **Status**: ALL PHASES COMPLETE — moved to TODONE
>
> This document consolidates all TODO debt under a unified root cause analysis.
> Phases A–C resolved the structural debt. Remaining extension features and
> known limitations migrated to `TODO/consolidation.md §16`.

## Executive Summary

The codebase had accumulated debt because **infrastructure code had no home**.
The `gunbc-infra` crate extraction (Phase A) is now complete, which fixed:

1. ~~Lint fights (`#[allow]` pragmas everywhere)~~ — **FIXED**: 5 pragmas eliminated
2. ~~Code duplication (same logic in multiple places)~~ — **FIXED**: shared `ResourceDef` + `compute_key_from_def()`
3. Circular dependency workarounds (strings instead of function refs) — still open
4. Incomplete abstractions (traits that can't access what they need) — still open
5. ~~Non-uniform hashing~~ — **FIXED**: `hash_parts()` centralized in infra, blob uses `ContentHash`

**Remaining extension work migrated to**: `TODO/consolidation.md §16`.

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
| Codegen content-hash manifest | Infra extraction | **High** |
| deps.toml tracking | Infra extraction | High |
| Makefile tracking | Infra extraction | Medium |
| .gitignore tracking | Infra extraction | Medium |
| Per-tool test tracking | Performance fixes | Low |
| ToolHandle unification | Design fixes | Low |

**Codegen content-hash manifest**: The current freshness check relies on
glob patterns (`CODEGEN_GLOB_PATTERNS`, `CODEGEN_EXTRA_FILES`) to discover
inputs. If inputs change in ways the globs don't capture (new crate dep,
new config file), stale artifacts go undetected. Fix: store a content-hash
manifest of all actual inputs consumed during codegen, and verify against
it on next run. The infrastructure exists (`ContentHash`, `ManifestEntry`,
`input_file_count`); the gap is recording the actual input set rather than
a glob-derived approximation.

---

## Weekly Signal (2026-02-05): What the last week's changes reveal

The week's diff confirms one meta-root-cause that unifies all four
completed phases:

> **When a concept lacks a typed, structural home (IR/model/registry/
> resource), it leaks into templates, env access, string IDs, and
> ad-hoc rules — and then we refactor later to pull it back into
> structure.**

The week's work pulled multiple leaks back into structure:

| Leak type | What leaked | Structural fix |
|-----------|-------------|----------------|
| Emission | `format!()` codegen | `ValueExpr` IR + renderer |
| Registry | String-based builder refs | `GraphBuilderId` enum, dual-encoding removal |
| Config | Manual CI YAML / Makefile | Generated from DAG/tool model |
| Environment | Inline `SystemTime::now()`, `Platform::detect()` | Explicit env nodes (FsEnv, ClockEnv, PlatformEnv) |
| Hashing | Per-crate hash impls | Unified via `gunbc-infra::hash` |

### Coherence with existing plans

Strongly coherent with `TODO/TODONE/refactor-pressure.md` root causes (A–D) and
the decision rules (single source of truth, no stringly refs, no
hidden env/IO). Two watchpoints for drift:

1. **Generated artifact verification** — CI doesn't yet verify that
   generated files match their generator output. First hand-edit
   re-introduces drift. Tracked in `TODO/TODONE/refactor-pressure.md` tasks.

2. **Codegen freshness** — manifest-based model works (Phases 1-5)
   but the codegen upsert key still has a brittle path if inputs
   change in ways the glob patterns don't capture. Content-hash
   manifest (Phase D item) is the structural fix.

---

## Remaining: Phase D — Extensions

Add new resources to the model as needed. Infrastructure is in place.
Phases A (infra extraction), B (mtime fast path), C (design fixes) are all complete.

---

## Known Limitations (Resolved)

### RUSTC_VERSION — FIXED (2026-02-06)

Previously, codegen hashing read `RUSTC_VERSION` from the environment with
a fallback to `"unknown"`. This was a silent fallback that violated the
"no hidden env" invariant.

**Fix**: Replaced `InputPattern::Env("RUSTC_VERSION")` with
`InputPattern::CommandOutput("rustc", ["--version"])`. The actual compiler
version is now captured directly via the resource model, with no env var
dependency or silent default. See `core/ir/src/resource/defs.rs`.

---

## References

- `TODO_hacks` — Detailed hack descriptions
- `TODO/TODONE/refactor-pressure.md` — Recurring root patterns that cause rework
- `TODO/TODONE/design-unified-resource-model.md` — Resource model design
- `TODO/TODONE/design-resource-performance.md` — Performance considerations (DONE)
- `TODO/TODONE/design-resource-acquisition.md` — Resource trait design (DONE)
