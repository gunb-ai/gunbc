# Hacks Resolved (2026-02-05)

**Status**: ✅ Done
**Date**: 2026-02-05

Additional hack cleanups moved out of `TODO_hacks`.

## buck-out/gen hardcoded in multiple locations

**Resolved**: Consolidated codegen output paths behind shared constants
(`CODEGEN_OUT_DIR`, `CODEGEN_BIN_DIR`, etc.) in `core/ir`. Removed hardcoded
`buck-out/gen` literals from runtime logic (gitignore entries remain).

Files:
- `core/ir/src/lib.rs`
- `core/codegen/src/main.rs`
- `gunbc-dag/src/codegen/*`

## Codegen source tracking directories were static

**Resolved**: Removed the hardcoded `CODEGEN_SOURCES` Makefile block from
the renderer; codegen no longer depends on a static list of sources.

Files:
- `gunbc-dag/src/codegen/render.rs`

## diff_files port cardinality/value mismatch

**Resolved**: `diff_files` now modeled as a scalar Map in the gist diff
pipeline, matching `GitOps::ParseDiff` output and existing mocks.

Files:
- `lib/gist-ops` / diff pipeline graph

## check_state hashed missing resources

**Resolved**: `check_state()` now returns `Missing` before computing keys,
avoiding unnecessary hashing when no manifest entry exists.

Files:
- `core/ir/src/resource/managed.rs`

## canonical_edge_order recomputed per node

**Resolved**: `execute_flat()` now computes canonical edge order once and
groups edges by destination node to avoid repeated sorting.

Files:
- `core/exec/src/execute.rs`

## DAG builder references centralized

**Resolved**: Tool DAG builders now reference `GraphBuilderId` (single mapping
to function names), and testgen targets are auto-discovered via
`#[testgen_target]` inventory registrations. This removes the old hardcoded
builder lists and keeps builder references in one place.

Files:
- `core/codegen/src/cli_gen.rs`
- `core/codegen/src/registry.rs`
- `core/testgen-registry-macros/src/lib.rs`
- `gunbc-dag/src/bin/testgen.rs`
