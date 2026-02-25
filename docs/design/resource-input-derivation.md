# Resource Input Derivation

> **Status**: Draft
> **Scope**: Eliminate hardcoded input-pattern lists from resource definitions by deriving them from existing domain models.

---

## Problem

Resource freshness depends on `InputPattern` lists attached to each `ResourceDef`.
Today these lists are **manually curated**:

| Resource | Hardcoded constants | Derivation fn | Location |
|----------|-------------------|---------------|----------|
| codegen (`generated_cli`) | `CODEGEN_INPUT_GLOBS`, `CODEGEN_INPUT_FILES` | `derive_codegen_input_patterns()` | `core/ir/src/resource/defs.rs` |
| testgen (`generated_tests`) | `TESTGEN_INPUT_GLOBS` | `derive_testgen_input_globs()` | `gunbc-dag/src/resources.rs` |
| makefile | `REPO_SOURCE_INPUT_GLOBS`, `REPO_CONFIG_INPUT_FILES` | `with_repo_inputs()` | `gunbc-dag/src/resources.rs` |
| gitignore | (same) | (same) | (same) |
| deps_config | (same) | (same) | (same) |

Each constant is a hand-maintained `&[&str]` of glob patterns. Adding a crate, renaming a directory, or introducing a new input category requires manually updating 2-4 locations (constant + derivation function + fallback + test). Forgetting to update causes **silent freshness bugs** — the manifest reports "fresh" while the real inputs have changed.

### Concrete failure mode (2026-02-25)

`codegen_resource_def()` tracked `core/codegen` and `core/ir` but not `gunbc-dag` or `dsl/**/*.dag`. Changing DSL tool definitions or the codegen binary itself produced no hash change. Codegen was silently skipped. The fix was a manual patch adding the missing patterns — the same class of bug that will recur any time the workspace evolves.

### Duplication

The current codebase has **4 nearly-identical derivation functions** (`derive_codegen_input_patterns`, `derive_repo_source_input_globs`, `derive_repo_config_input_files`, `derive_testgen_input_globs`) plus a shared `with_repo_inputs` compositor. Each reimplements the same workspace-layout-or-fallback pattern. The constants they fall back to duplicate each other with minor subset variations.

---

## Design Principles

1. **Deduce over configure.** The system already knows what crates exist (`WorkspaceLayout`), what DSL files exist (`dsl/**/*.dag`), and what tools produce which outputs (`CompileOutput.output_paths`, `@outputs` annotations). Input patterns should be *derived* from these facts, not restated.

2. **Closed-world assumption.** If a resource definition doesn't account for an input, that's a correctness bug. The set of inputs should be structurally complete — impossible to silently omit.

3. **Single source of truth per concern.** Crate membership comes from `Cargo.toml`. DSL file membership comes from the filesystem. Tool output declarations come from DSL annotations. None of these should be restated in resource definitions.

---

## Proposed Model

### Core Abstraction: `InputScope`

Replace per-resource glob lists with composable **input scopes** that derive patterns from workspace structure:

```rust
/// Declares what a resource reads. Patterns are derived, not hardcoded.
pub enum InputScope {
    /// All source files in the named crate (src/**/*.rs + Cargo.toml).
    Crate(String),

    /// All source files in crates under a workspace directory (e.g., "core", "lib").
    CrateGroup(String),

    /// All files matching a category (e.g., DslFiles -> dsl/**/*.dag).
    Category(InputCategory),

    /// Toolchain version (rustc --version, etc.).
    Toolchain,

    /// Another resource's freshness key.
    Resource(ResourceId),
}

pub enum InputCategory {
    /// All DSL definition files.
    Dsl,
    /// Workspace root config (Cargo.toml, Cargo.lock).
    WorkspaceConfig,
}
```

### Resolution: `InputScope` → `Vec<InputPattern>`

A resolver expands scopes into concrete `InputPattern` values using `WorkspaceLayout`:

```rust
pub fn resolve_scope(scope: &InputScope, layout: &WorkspaceLayout) -> Vec<InputPattern> {
    match scope {
        InputScope::Crate(name) => {
            let dir = layout.crate_dir(name);
            vec![
                InputPattern::glob(format!("{dir}/src/**/*.rs")),
                InputPattern::file(format!("{dir}/Cargo.toml")),
            ]
        }
        InputScope::CrateGroup(prefix) => {
            // Walk layout.crates, filter by prefix, emit globs
        }
        InputScope::Category(InputCategory::Dsl) => {
            vec![InputPattern::glob("dsl/**/*.dag")]
        }
        InputScope::Toolchain => {
            vec![InputPattern::command_output("rustc", &["--version"])]
        }
        InputScope::Resource(id) => {
            vec![InputPattern::resource(id.clone())]
        }
        // ...
    }
}
```

### Resource definitions become declarative

```rust
// Before: 30 lines of manual globs + derivation function + fallback constant + test
pub fn codegen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(ResourceId::build("generated_cli"));
    let (globs, files) = codegen_input_patterns(); // manually curated
    for pattern in globs { def = def.with_input(InputPattern::glob(pattern)); }
    for path in files { def = def.with_input(InputPattern::file(path)); }
    def = def.with_input(InputPattern::glob("dsl/**/*.dag".to_string()));
    def = def.with_input(InputPattern::command_output("rustc", &["--version"]));
    def
}

// After: intent-level declaration, patterns derived
pub fn codegen_resource_def(layout: &WorkspaceLayout) -> ResourceDef {
    ResourceDef::new(ResourceId::build("generated_cli"))
        .with_scope(InputScope::Crate("gunbc-codegen"))
        .with_scope(InputScope::Crate("gunbc-ir"))
        .with_scope(InputScope::Crate("gunbc-dag"))
        .with_scope(InputScope::Category(InputCategory::Dsl))
        .with_scope(InputScope::Toolchain)
        .resolve(layout)
}
```

### Completeness invariant

A test walks `WorkspaceLayout.crates` and asserts that every crate contributing source code to a generated artifact is covered by at least one `InputScope::Crate` or `InputScope::CrateGroup` entry. Adding a new crate that affects codegen output without declaring it in the resource scope causes a **test failure**, not a silent freshness bug.

---

## Scope Decisions

### What this design covers

- Replacing the 5 hardcoded `const` glob arrays and 4 derivation functions
- Introducing `InputScope` as the declarative layer between resource defs and `InputPattern`
- Adding completeness tests that catch missing input declarations

### What this design does NOT cover

- **DSL-declared resources** (`@outputs`, content_upsert paths) — already derived, not hardcoded
- **Cloud resources** (`CloudConfigResource`) — already parameterized at construction time
- **Unified registration model** — tracked separately in `docs/design/unified-registration.md`

### Fallback behavior

The `OnceLock` + fallback-constant pattern remains for isolated generator runs where `WorkspaceLayout` is unavailable. But the fallback constants are **generated** (or at minimum validated) from the scope declarations, not maintained by hand.

---

## Migration Path

### Phase 1: Introduce `InputScope` and resolver

Add the `InputScope` enum and `resolve_scope()` function to `core/ir/src/resource/`.
No behavioral change — existing code continues to work.

### Phase 2: Convert `codegen_resource_def()`

Replace the manual glob lists in `core/ir/src/resource/defs.rs` with `InputScope` declarations. Delete `CODEGEN_INPUT_GLOBS`, `CODEGEN_INPUT_FILES`, `derive_codegen_input_patterns()`.

### Phase 3: Convert repo-level resources

Replace `REPO_SOURCE_INPUT_GLOBS`, `REPO_CONFIG_INPUT_FILES`, `TESTGEN_INPUT_GLOBS` and `with_repo_inputs()` in `gunbc-dag/src/resources.rs` with scope-based declarations.

### Phase 4: Completeness test

Add a workspace-level test that verifies every resource's scopes cover all crates that contribute to its output. This is the invariant that prevents the 2026-02-25 class of bug from recurring.

### Phase 5: Delete dead code

Remove the 4 `derive_*` functions, 4 `OnceLock` statics, 5 `const` arrays, and their associated tests.

---

## Invariants

1. **No hardcoded glob paths in resource definitions.** All file patterns are derived from `InputScope` + `WorkspaceLayout`.
2. **Completeness is tested.** If a crate contributes source to a resource's output, the resource's scopes must cover it.
3. **Fallback constants are validated.** If fallback constants exist for offline use, a test asserts they match what scope resolution produces.
