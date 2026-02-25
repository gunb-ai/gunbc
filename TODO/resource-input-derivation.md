# Resource Input Derivation — Tasks

**Last updated**: 2026-02-25
**Design**: `docs/design/resource-input-derivation.md`
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Lane 1: Scope-Derived Resource Inputs

**Goal**: Replace all hardcoded `const` glob arrays and per-resource derivation functions with declarative `InputScope` declarations resolved from `WorkspaceLayout`. Prevent the 2026-02-25 class of freshness bug (missing inputs cause silent skip) by making completeness structurally testable.

**Design reference**: `docs/design/resource-input-derivation.md`

### Track A: Core Abstraction

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **RD-1** | **Add `InputScope` enum and `InputCategory`**: Define `InputScope` (Crate, CrateGroup, Category, Toolchain, Resource) and `InputCategory` (Dsl, WorkspaceConfig) in `core/ir/src/resource/`. Add `resolve_scope()` that expands an `InputScope` into `Vec<InputPattern>` using `WorkspaceLayout`. | -- | M | Planned |
| **RD-2** | **Add `with_scope()` and `resolve()` to `ResourceDef`**: Builder method that accumulates `InputScope` entries. `resolve()` expands all scopes via `resolve_scope()` and populates `inputs`. | RD-1 | S | Planned |
| **RD-3** | **Fallback resolution path**: `resolve_scope()` without a `WorkspaceLayout` falls back to static defaults. Add `InputScope::to_fallback_patterns()` that produces hardcoded patterns for isolated/offline runs. Test that fallback output matches live resolution output in normal workspace. | RD-1 | S | Planned |

### Track B: Migration

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **RD-4** | **Convert `codegen_resource_def()`**: Replace manual glob lists in `core/ir/src/resource/defs.rs` with `InputScope::Crate("gunbc-codegen")`, `Crate("gunbc-ir")`, `Crate("gunbc-dag")`, `Category(Dsl)`, `Toolchain`. Delete `CODEGEN_INPUT_GLOBS`, `CODEGEN_INPUT_FILES`, `derive_codegen_input_patterns()`, `DERIVED_CODEGEN_INPUTS`. | RD-2 | S | Planned |
| **RD-5** | **Convert repo-level resources**: Replace `REPO_SOURCE_INPUT_GLOBS`, `REPO_CONFIG_INPUT_FILES`, `with_repo_inputs()`, `derive_repo_source_input_globs()`, `derive_repo_config_input_files()` in `gunbc-dag/src/resources.rs` with scope declarations on `makefile_resource_def()`, `gitignore_resource_def()`, `deps_config_resource_def()`. | RD-2 | M | Planned |
| **RD-6** | **Convert `testgen_resource_def()`**: Replace `TESTGEN_INPUT_GLOBS`, `derive_testgen_input_globs()`, `DERIVED_TESTGEN_GLOBS` in `gunbc-dag/src/resources.rs`. | RD-2 | S | Planned |

### Track C: Completeness Invariant

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **RD-7** | **Completeness test**: Add a workspace-level test that walks `WorkspaceLayout.crates` and verifies every crate contributing source to a resource's output is covered by at least one `InputScope::Crate` or `CrateGroup` entry. Catches the exact class of bug that caused the 2026-02-25 freshness skip. | RD-4, RD-5, RD-6 | M | Planned |
| **RD-8** | **Delete dead code**: Remove the 4 `OnceLock` statics, remaining `const` arrays (if not used by fallback path), and old derivation functions. Verify re-exports in `core/ir/src/resource/mod.rs` are updated. | RD-7 | S | Planned |

### Dependency guide

1. `RD-1 -> RD-2 -> (RD-4, RD-5, RD-6)` — core abstraction then parallel migration
2. `(RD-4, RD-5, RD-6) -> RD-7 -> RD-8` — completeness test after all conversions, then cleanup
3. `RD-1 -> RD-3` — fallback path can be done in parallel with Track B

### Exit criteria

1. Zero hardcoded glob-path `const` arrays remain in resource definitions.
2. All 5 resource defs (codegen, testgen, makefile, gitignore, deps_config) use `InputScope` declarations.
3. A completeness test catches any crate added to the workspace that contributes to a resource but is not declared in its scopes.
4. Fallback patterns for offline runs are validated against live resolution.
5. `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings` clean.

### Files touched (aggregate)

| File | Changes |
|------|---------|
| `core/ir/src/resource/scope.rs` (new) | `InputScope`, `InputCategory`, `resolve_scope()` |
| `core/ir/src/resource/def.rs` | `with_scope()`, `resolve()` on `ResourceDef` |
| `core/ir/src/resource/mod.rs` | Re-export new types |
| `core/ir/src/resource/defs.rs` | Replace manual globs with scope declarations; delete constants + derivation fn |
| `gunbc-dag/src/resources.rs` | Replace manual globs with scope declarations; delete constants + derivation fns |
