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
