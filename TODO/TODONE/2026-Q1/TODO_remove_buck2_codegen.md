# Remove Buck2 / Hermetic Build Infrastructure

**Status**: DONE
**Date**: 2026-02-04
**Completed**: 2026-02-04

The `buck-out/gen/` codegen output and Buck2 hermetic build infrastructure
was planned but never actually used. Removed all of it.

---

## What was removed

### Phase 1: Broken binary targets ✓

- [x] Remove `[[bin]]` sections from `lib/tools/gist/Cargo.toml`
- [x] Remove `[[bin]]` sections from `lib/tools/deps/Cargo.toml`
- [x] Remove `[[bin]]` sections from `lib/tools/buck2/Cargo.toml`
- [x] Verify `cargo test` works without `--lib` flag

### Phase 2: Buck2 crate ✓

- [x] Deleted entire `lib/tools/buck2/` crate
- [x] Removed from workspace Cargo.toml
- [x] Removed from gunbc-app dependencies
- [x] Removed Buck2Op from WorkspaceOp enum
- [x] Removed subdags/buck2.rs
- [x] Removed Makefile targets (buck2, buck2-dry)

### Phase 3: Codegen output infrastructure ✓

- [x] Changed output paths from `buck-out/gen/` to `target/codegen/` in main.rs
- [x] Removed codegen stamp logic from makegen/render.rs
- [x] Changed ci/ops.rs existence check to use Cargo.toml
- [x] Removed `ensure-codegen` targets and dependencies from Makefile (later reintroduced)
- [x] Removed `buck-out/` from registry.rs core_outputs()

### Phase 4: Remaining references ✓

- [x] Updated graph_mock.rs to use Cargo.toml
- [x] Updated ci.rs mock to use Cargo.toml
- [x] Updated all tests in render.rs
- [x] Kept `.gitignore` entry (harmless) (later removed)

---

## Post-completion changes (2026-02-06+)

- `ensure-codegen` was reintroduced in `Makefile` as the bootstrap-safe entrypoint.
- `.gitignore` no longer includes `/buck-out/` because Buck2 outputs were fully removed.

---

## Result

- `cargo test` works without workarounds
- `cargo build` works
- `cargo clippy` passes
- All 1200+ tests pass
