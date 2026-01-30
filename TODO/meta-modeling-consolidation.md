# Meta-Modeling Consolidation

**Status**: In Progress
**Date**: 2026-01-29
**Updated**: 2026-01-30

## Goal

Consolidate all build artifact generation (Makefile, .gitignore, ci.yml) around a single source of truth: the `ToolRegistry` and `BuildConfig`. Eliminate hardcoded strings scattered across tools.

## Current State Summary

### What's Done ✅

| Component | Status | Location |
|-----------|--------|----------|
| `BuildConfig` | ✅ Single source of truth | `makegen/src/registry.rs` |
| `ToolRegistry` | ✅ Declarative tool list | `makegen/src/registry.rs` |
| `MakefileRenderer` | ✅ Generates from registry | `makegen/src/render.rs` |
| `WorkflowConfig` | ✅ Typed CI configuration | `core/ir/src/transport/github_actions.rs` |
| `Integration` | ✅ GitHub Actions with `provides_tools` | `github_actions.rs` |
| `RunnerImage` | ✅ Runner tool tracking | `github_actions.rs` |
| `check_satisfiability()` | ✅ Tool availability check | `github_actions.rs` |
| CI YAML generation | ✅ `WorkflowConfig.render()` | `github_actions.rs` |
| Caching | ✅ In generated ci.yml | `.github/workflows/ci.yml` |

### What's Remaining ❌

| Component | Issue | Fix |
|-----------|-------|-----|
| Bootstrap Makefile | Hardcoded in `ops.rs` | Use `render_makefile()` |
| Bootstrap .gitignore | Hardcoded in `ops.rs` | Create `GitignoreRenderer` |
| makegen tool list | Hardcoded in `default_registry()` | Derive from central registry |
| Help text | Manually maintained | Generate from tool descriptions |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Meta-Modeling Architecture                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────┐      ┌─────────────────┐                      │
│  │  ToolRegistry   │      │  BuildConfig    │                      │
│  │  (tools, meta)  │      │  (commands)     │                      │
│  └────────┬────────┘      └────────┬────────┘                      │
│           │                        │                                │
│           └────────────┬───────────┘                                │
│                        │                                            │
│                        ▼                                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Renderers                                 │   │
│  │                                                              │   │
│  │  MakefileRenderer   GitignoreRenderer   WorkflowConfig      │   │
│  │  (exists)           (to create)         (exists)            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                        │                                            │
│           ┌────────────┼────────────┐                              │
│           ▼            ▼            ▼                              │
│       Makefile    .gitignore    ci.yml                             │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Consumers                                 │   │
│  │                                                              │   │
│  │  makegen          bootstrap         codegen (cigen)         │   │
│  │  (uses registry)  (should use)      (uses WorkflowConfig)   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Design

### 1. GitignoreRenderer (New)

Similar to `MakefileRenderer`, create a gitignore renderer:

```rust
// lib/tools/makegen/src/gitignore.rs

pub struct GitignoreConfig {
    /// Language-specific patterns
    pub language_patterns: Vec<LanguagePatterns>,
    /// IDE patterns (.idea/, .vscode/)
    pub ide_patterns: Vec<&'static str>,
    /// OS patterns (.DS_Store, Thumbs.db)
    pub os_patterns: Vec<&'static str>,
    /// Build system patterns (derived from BuildConfig)
    pub build_patterns: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct LanguagePatterns {
    pub name: &'static str,
    pub patterns: Vec<&'static str>,
}

impl LanguagePatterns {
    pub fn rust() -> Self {
        Self {
            name: "rust",
            patterns: vec![
                "/target/",
                "**/*.rs.bk",
                "Cargo.lock",
            ],
        }
    }
}

impl GitignoreConfig {
    pub fn default_rust() -> Self {
        Self {
            language_patterns: vec![LanguagePatterns::rust()],
            ide_patterns: vec![".idea/", ".vscode/", "*.swp", "*.swo", "*~"],
            os_patterns: vec![".DS_Store", "Thumbs.db"],
            build_patterns: vec!["buck-out/"],
        }
    }
}

pub struct GitignoreRenderer<'a> {
    pub config: &'a GitignoreConfig,
}

impl Renderable for GitignoreRenderer<'_> {
    fn generator_name(&self) -> &str { "gunbc-makegen" }
    fn regenerate_command(&self) -> &str { "make makegen" }
    fn render_content(&self) -> String {
        // Render gitignore from config
    }
}
```

### 2. Bootstrap Uses Makegen

Instead of hardcoded strings, bootstrap imports from makegen:

```rust
// lib/tools/bootstrap/src/ops.rs

use gunbc_makegen::{
    ToolRegistry, BuildConfig, 
    render_makefile, render_gitignore,
    GitignoreConfig,
};

fn execute_generate_makefile(_inputs: HashMap<String, Value>) -> Result<...> {
    let registry = ToolRegistry::default_registry();
    let config = BuildConfig::cargo();
    let content = render_makefile(&registry, &config);
    
    let mut out = HashMap::new();
    out.insert("makefile_content".to_string(), Value::Str(content));
    Ok(out)
}

fn execute_generate_gitignore(_inputs: HashMap<String, Value>) -> Result<...> {
    let config = GitignoreConfig::default_rust();
    let content = render_gitignore(&config);
    
    let mut out = HashMap::new();
    out.insert("gitignore_content".to_string(), Value::Str(content));
    Ok(out)
}
```

### 3. Derive Tool List from Central Registry (Future)

Currently `ToolRegistry::default_registry()` hardcodes the tool list. Future enhancement:

```rust
// Derive from a central tool definition
pub fn default_registry() -> Self {
    let mut registry = Self::new();
    
    // Auto-discover tools from workspace
    for tool_def in discover_workspace_tools() {
        registry.register(tool_def.into());
    }
    
    registry
}
```

## Implementation Plan

### Phase 1: GitignoreRenderer

1. Create `lib/tools/makegen/src/gitignore.rs`
2. Add `GitignoreConfig` and `LanguagePatterns` structs
3. Implement `GitignoreRenderer` with `Renderable` trait
4. Add `render_gitignore()` function
5. Add tests

### Phase 2: Bootstrap Integration

1. Add `gunbc-makegen` dependency to `gunbc-bootstrap/Cargo.toml`
2. Update `execute_generate_makefile()` to use `render_makefile()`
3. Update `execute_generate_gitignore()` to use `render_gitignore()`
4. Update bootstrap tests

### Phase 3: Validation (Optional)

1. Add `make validate` target that checks generated files match source
2. Fail CI if Makefile/gitignore/ci.yml differ from what would be generated

## Tasks

### GitignoreRenderer
- [ ] Create `gitignore.rs` module in makegen
- [ ] Define `GitignoreConfig` struct
- [ ] Define `LanguagePatterns` with Rust preset
- [ ] Implement `GitignoreRenderer` with `Renderable`
- [ ] Add `render_gitignore()` public function
- [ ] Add unit tests

### Bootstrap Integration
- [ ] Add makegen dependency to bootstrap Cargo.toml
- [ ] Update `execute_generate_makefile()` to use renderer
- [ ] Update `execute_generate_gitignore()` to use renderer
- [ ] Update/fix bootstrap tests

### Documentation
- [ ] Update AGENT.md with meta-modeling patterns
- [ ] Document how to add new tools to registry

## Success Criteria

- [ ] Bootstrap generates Makefile from `ToolRegistry` (not hardcoded)
- [ ] Bootstrap generates .gitignore from `GitignoreConfig` (not hardcoded)
- [ ] Adding a new tool to registry automatically updates Makefile
- [ ] `BuildConfig` remains single source of truth for commands
- [ ] All generation is deterministic and testable
- [ ] No hardcoded tool lists anywhere

## Related Files

| File | Purpose |
|------|---------|
| `lib/tools/makegen/src/registry.rs` | ToolRegistry, BuildConfig |
| `lib/tools/makegen/src/render.rs` | MakefileRenderer |
| `lib/tools/bootstrap/src/ops.rs` | Hardcoded generation (to fix) |
| `core/ir/src/transport/github_actions.rs` | WorkflowConfig, Integration |

## Notes

### Why Not a Separate `gunbc-meta` Crate?

Option considered but deferred:
- Extract `ToolRegistry`, `BuildConfig`, `GitignoreConfig` to shared crate
- Pros: Cleaner dependencies, no makegen→bootstrap coupling
- Cons: More crates to maintain, over-engineering for current needs

**Decision**: Start with bootstrap depending on makegen. Extract later if needed.

### CI YAML Generation Already Done

The `WorkflowConfig` in `github_actions.rs` already:
- Models integrations with `provides_tools`
- Tracks runner capabilities
- Generates full ci.yml via `Renderable`
- Validates tool satisfiability

This is the reference for how makegen/bootstrap should work.
