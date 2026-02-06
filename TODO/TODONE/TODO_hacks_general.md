# Hacks Resolved (2026-02-05, updated 2026-02-06)

**Status**: ✅ Done
**Date**: 2026-02-05 (latest: 2026-02-06)

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

## Lint config doesn't match crate boundaries (2026-02-05)

**Resolved**: `gunbc-infra` crate extracted.

- `hash.rs`, `manifest.rs`, `ResourceId` moved to `core/infra/`
- Codegen/testgen hashing centralized via `ResourceDef` + `compute_key_from_def`
- `hash_parts()` centralized in `core/infra/src/hash.rs`
- 5 `#[allow(clippy::disallowed_methods)]` pragmas eliminated from ir/resource/
- `lib/blob` and `lib/primitives` now delegate to infra for hashing
- All re-exports preserved — zero downstream breakage

## Resource model design issues #6, #8, #9, #10, #11 (2026-02-05)

All resolved during `gunbc-infra` extraction:

- **#6**: Duplicate codegen hash logic → `ResourceDef` + `compute_key_from_def`
- **#8**: `GUNBC_EXEC_MODE` env var → threaded via DAG edges from `runner_env`
- **#9**: ResourceHandle forgery → `pub(crate)` acquire, per-process secret
- **#10**: `compute_key` lacks manifest → signature now accepts `&ResourceManifest`
- **#11**: SimpleResource empty hash → uses `compute_key_from_def`

## Test type/cardinality mismatch in fan_in test (2026-02-05)

**Resolved**: Changed output to `list("items", "String")` to match input.

Files:
- `core/exec/src/execute.rs` (test around line 1150)

## CI codegen check is brittle (file existence hack) (2026-02-05)

**Resolved**: CI pipeline uses manifest-based freshness checking. See
`TODO/design-unified-resource-model.md` for Phases 1-5.

- `core/ir/src/resource/` — ContentHash, ResourceManifest, ExecMode
- `core/codegen/src/main.rs` — writes manifest after codegen
- `gunbc-dag/src/bin/testgen.rs` — writes manifest after testgen
- `gunbc-dag/src/ci/ops.rs` — manifest-based freshness check
- `gunbc-dag/src/bin/ci.rs` — `--mode=verify|ensure` flag

## `can_coerce_to` duplicates `satisfies` on Cardinality (2026-02-06)

**Resolved**: Removed `Cardinality::can_coerce_to()` method and its tests.
The method was logically identical to `satisfies()` — the "additional"
clause was the same predicate restated. Zero production callers existed.
If lossless coercion rules are needed later, they should be designed
from scratch with explicit allowed-coercion rules (see `core/ir/src/coerce.rs`).

Files:
- `core/ir/src/types.rs`

## Tool targets blanket-depend on ensure-codegen (2026-02-06)

**Resolved**: Added `needs_generated_cli: bool` field to `ToolInfo`.
Codegen-derived tools (from `from_tool_def`) set `true`; manual tools
(ci, pragma, build-all) set `false` via `.manual()` builder method.
Renderer conditionally includes `ensure-codegen` dependency only when needed.

Files:
- `gunbc-dag/src/makegen/registry.rs`
- `gunbc-dag/src/makegen/render.rs`

## Meta-target fix_prerequisites are untyped strings (2026-02-06)

**Resolved**: Replaced `fix_prerequisites: Vec<&'static str>` with
`Vec<FixAlias>` enum. `FixAlias` has `FmtFix` and `LintFix` variants,
each resolving to a target name string at render time via `target_name()`.
Renaming a fix alias now causes a compile error.

Files:
- `gunbc-dag/src/makegen/registry.rs`
- `gunbc-dag/src/makegen/render.rs`
