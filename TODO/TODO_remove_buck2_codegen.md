# Remove Buck2 / Hermetic Build Infrastructure

**Status**: TODO
**Date**: 2026-02-04
**Priority**: HIGH (blocks `cargo test` without workarounds)

The `buck-out/gen/` codegen output and Buck2 hermetic build infrastructure
was planned but never actually used. It causes friction:

- `cargo test` fails without `--lib` flag because generated binaries don't exist
- `buck-out/gen/bin/*/main.rs` paths in Cargo.toml reference non-existent files
- 30+ hardcoded references to `buck-out/` scattered across codebase

---

## Current State

### Binary targets pointing to buck-out (broken)

```toml
# lib/tools/gist/Cargo.toml
[[bin]]
name = "gunbc-gist"
path = "../../../buck-out/gen/bin/gist/main.rs"  # doesn't exist

# lib/tools/deps/Cargo.toml
[[bin]]
name = "gunbc-deps"
path = "../../../buck-out/gen/bin/deps/main.rs"  # doesn't exist

# lib/tools/buck2/Cargo.toml
[[bin]]
name = "gunbc-buck2"
path = "../../../buck-out/gen/bin/buck2/main.rs"  # doesn't exist
```

### Hardcoded paths (30+ occurrences)

- `Makefile` (4 refs) — codegen stamp file
- `core/codegen/src/main.rs` — output directories
- `core/codegen/src/registry.rs` — core_outputs()
- `gunbc-dag/src/makegen/render.rs` — Makefile generation
- `gunbc-dag/src/ci/ops.rs` — existence check
- Various test files and graph_mock.rs

---

## Cleanup Plan

### Phase 1: Remove broken binary targets ✓

- [x] Remove `[[bin]]` sections from `lib/tools/gist/Cargo.toml`
- [x] Remove `[[bin]]` sections from `lib/tools/deps/Cargo.toml`
- [x] Remove `[[bin]]` sections from `lib/tools/buck2/Cargo.toml`
- [x] Verify `cargo test` works without `--lib` flag

### Phase 2: Decide on Buck2 crate

- [ ] Audit `lib/tools/buck2/` — is it used anywhere?
- [ ] If unused: delete entire `lib/tools/buck2/` crate
- [ ] If used: keep as library, remove generated binary

### Phase 3: Remove codegen output infrastructure

- [ ] Remove `buck-out/gen/` output path from `core/codegen/src/main.rs`
- [ ] Remove codegen stamp logic from `gunbc-dag/src/makegen/render.rs`
- [ ] Remove existence check from `gunbc-dag/src/ci/ops.rs`
- [ ] Update `Makefile` codegen targets
- [ ] Remove `buck-out/` from `core/codegen/src/registry.rs` core_outputs()

### Phase 4: Consolidate remaining references

- [ ] Grep for remaining `buck-out` references
- [ ] Update or remove each occurrence
- [ ] Keep `.gitignore` entry (harmless)
- [ ] Update `TODO/consolidation.md` to mark this done

---

## Alternative: Keep codegen but fix paths

If we want to keep generating CLI binaries:

1. Move output to a standard location (e.g., `target/codegen/`)
2. Make binary targets conditional on file existence
3. Or: generate binaries directly in `src/bin/` (checked in)

This is more work and the current approach was never used, so deletion
is recommended.

---

## References

- `TODO/consolidation.md` §6 — mentions 16 `buck-out/gen` occurrences
- `TODO_hacks` — lists 4 hardcoded locations
- Error: `couldn't read lib/tools/deps/../../../buck-out/gen/bin/deps/main.rs`
