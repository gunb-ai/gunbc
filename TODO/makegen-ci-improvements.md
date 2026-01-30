# Makefile Generation & CI Improvements

**Status**: Draft
**Date**: 2026-01-29

## Goal

Make Makefile generation and CI more competent by:
1. **Generating more from source of truth** — less hardcoded, more derived
2. **Integrating tool satisfiability** — know what tools CI needs before running
3. **Better workflow configuration** — typed, validated, generated

## Current State

### Makefile Generation (makegen)

**Good**:
- `BuildConfig` is single source of truth for build/test/lint commands
- `ToolRegistry` defines all tools and their entrypoints
- `MetaTarget` defines test/check/clippy/fmt with prep levels

**Gaps**:
- Hardcoded tool list in `ToolRegistry::default_registry()`
- No integration with new `ToolDef` system
- No validation that runner has required tools
- Help text manually maintained

### CI (ci crate)

**Good**:
- Workflow logic in Rust, not YAML
- `WorkflowConfig` with typed integrations and permissions
- `RunnerImage` knows what tools it provides
- Permissions computed from integrations

**Gaps**:
- No pre-flight satisfiability check (will tools be available?)
- No deps.toml generation for CI
- Actions hardcoded, not derived from actual needs
- No caching configuration

## Design

### 1. Integrate ToolDef with CI

Connect the new `ToolDef` system to CI workflow configuration:

```rust
// lib/tools/ci/src/graph.rs

use gunbc_ir::transport::{
    default_tool_registry, default_platform_registry, 
    check_all_satisfiable, GH_TOOL, GIT, CLIPPY,
};

/// Tools required by the CI pipeline
pub fn ci_required_tools() -> Vec<&'static ToolDef> {
    vec![
        &GIT,      // for checkout
        &CLIPPY,   // for lint
        // cargo/rustc provided by rust-toolchain action
    ]
}

/// Check if CI can run on a given runner
pub fn check_ci_satisfiability(runner: &RunnerImage) -> Result<(), Vec<UnsatisfiableError>> {
    let registry = default_tool_registry();
    let platform_registry = default_platform_registry();
    
    // Get available PMs from runner's platform
    let available_pms = platform_registry.available_pms(runner.platform_id());
    
    // Add tools provided directly by runner
    // (ubuntu-latest has git, cargo, etc. pre-installed)
    let mut all_available = available_pms;
    for tool in runner.provided_tools() {
        // Tools provided by runner are "pre-satisfied"
    }
    
    check_all_satisfiable(ci_required_tools(), &all_available, &registry)
}
```

### 2. Generate CI deps.toml

Produce a deps.toml for CI that only includes tools not provided by the runner:

```rust
pub fn generate_ci_deps_toml(runner: &RunnerImage) -> String {
    let required = ci_required_tools();
    let provided: HashSet<_> = runner.provided_tools().iter().collect();
    
    let needed: Vec<_> = required
        .into_iter()
        .filter(|t| !provided.contains(&t.id))
        .collect();
    
    generate_deps_toml(&needed, Some("# CI dependencies - tools not provided by runner"))
}
```

### 3. Derive Makefile Tools from ToolDef Registry

Instead of hardcoding tools in `ToolRegistry`, derive from the tool system:

```rust
// lib/tools/makegen/src/registry.rs

/// Generate ToolInfo from ToolDef
pub fn tool_info_from_def(def: &ToolDef, config: ToolConfig) -> ToolInfo {
    ToolInfo {
        crate_name: format!("gunbc-{}", def.id),
        short_name: def.id.to_string(),
        description: config.description,
        entrypoints: config.entrypoints,
        // ...
    }
}
```

### 4. Typed GitHub Actions

Model GitHub Actions as first-class entities with known behaviors:

```rust
// core/ir/src/transport/github_actions.rs (extend existing)

/// A GitHub Action with typed metadata
pub struct Action {
    pub id: &'static str,
    pub uses: &'static str,
    pub provides_tools: &'static [&'static str],  // tools this action installs
    pub required_permissions: Permissions,
    pub inputs: &'static [ActionInput],
    pub outputs: &'static [ActionOutput],
}

/// Known actions
pub fn checkout() -> Action {
    Action {
        id: "checkout",
        uses: "actions/checkout@v4",
        provides_tools: &["git"],  // checkout ensures git is available
        required_permissions: permissions!{ contents: read },
        inputs: &[
            ActionInput::optional("fetch-depth", "1"),
            ActionInput::optional("submodules", "false"),
        ],
        outputs: &[],
    }
}

pub fn rust_toolchain() -> Action {
    Action {
        id: "rust-toolchain",
        uses: "dtolnay/rust-toolchain@stable",
        provides_tools: &["rustc", "cargo", "clippy", "rustfmt"],
        required_permissions: Permissions::empty(),
        inputs: &[
            ActionInput::optional("toolchain", "stable"),
            ActionInput::optional("components", "clippy,rustfmt"),
        ],
        outputs: &[],
    }
}
```

### 5. Generate Workflow YAML from Config

Currently ci.yml is manually written. Generate it:

```rust
pub fn generate_workflow_yaml(config: &WorkflowConfig) -> String {
    let mut yaml = String::new();
    
    yaml.push_str("# Generated by gunbc-ci - do not edit manually\n");
    yaml.push_str(&format!("name: {}\n\n", config.name));
    
    yaml.push_str("on:\n");
    yaml.push_str("  push:\n");
    yaml.push_str("    branches: [main, master]\n");
    yaml.push_str("  pull_request:\n");
    yaml.push_str("    branches: [main, master]\n\n");
    
    // Add permissions if needed
    if config.has_permissions() {
        yaml.push_str("permissions:\n");
        for (scope, level) in &config.permissions {
            yaml.push_str(&format!("  {}: {}\n", scope, level));
        }
        yaml.push_str("\n");
    }
    
    yaml.push_str("jobs:\n");
    yaml.push_str("  ci:\n");
    yaml.push_str(&format!("    runs-on: {}\n", config.runner.id));
    yaml.push_str("    steps:\n");
    
    for action in &config.integrations {
        yaml.push_str(&format!("      - uses: {}\n", action.uses));
        // Add inputs if any
    }
    
    yaml.push_str("      - name: Run CI\n");
    yaml.push_str("        run: cargo run -p gunbc-ci -- run\n");
    
    yaml
}
```

### 6. Caching Support

Add caching to CI workflow:

```rust
pub fn cache_action() -> Action {
    Action {
        id: "cache",
        uses: "actions/cache@v4",
        provides_tools: &[],
        required_permissions: Permissions::empty(),
        inputs: &[
            ActionInput::required("path", "~/.cargo/registry\n~/.cargo/git\ntarget"),
            ActionInput::required("key", "${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}"),
        ],
        outputs: &[],
    }
}

pub fn ci_integrations_with_cache() -> Vec<Action> {
    vec![
        checkout(),
        cache_action(),
        rust_toolchain(),
    ]
}
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Build System Architecture                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ToolDef Registry              Action Registry                  │
│  (gh, git, clippy, ...)        (checkout, rust-toolchain, ...)  │
│           │                              │                      │
│           ▼                              ▼                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   WorkflowConfig                         │   │
│  │  - runner (ubuntu-latest)                                │   │
│  │  - integrations (actions used)                           │   │
│  │  - required_tools (derived from CI steps)                │   │
│  │  - permissions (computed from actions)                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│           ┌──────────────────┼──────────────────┐              │
│           ▼                  ▼                  ▼              │
│    Satisfiability     ci.yml Generation    Makefile Gen        │
│    Check              (from config)        (from config)       │
│                                                                 │
│           │                  │                  │              │
│           ▼                  ▼                  ▼              │
│    Pre-flight         .github/workflows/    Makefile           │
│    Validation         ci.yml                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Tasks

### Makefile Generation

- [ ] **Derive tool list from ToolDef** — instead of hardcoded `default_registry()`
- [ ] **Generate help text** — from tool descriptions and entrypoints
- [ ] **Add prep target generation** — ensure codegen/daggen run as needed
- [ ] **Support multiple build systems** — BuildConfig already has cargo/buck2

### CI Improvements

- [ ] **Add satisfiability check** — verify tools available before pipeline
- [ ] **Generate ci.yml** — from WorkflowConfig
- [ ] **Add caching** — cache cargo registry and target
- [ ] **Generate CI deps.toml** — only tools not provided by runner
- [ ] **Model Actions as typed entities** — with provides_tools, inputs, outputs

### Integration

- [ ] **Connect runner tools to ToolDef** — ubuntu-latest provides git, cargo, etc.
- [ ] **Validate CI config** — tools used match runner capabilities
- [ ] **Bootstrap validation** — ensure generated files match source of truth

## Notes

### Current makegen Registry

The existing `ToolRegistry::default_registry()` in `lib/tools/makegen/src/registry.rs` hardcodes:
- gunbc-gist
- gunbc-buck2
- gunbc-makegen
- gunbc-deps
- gunbc-ci
- gunbc-bootstrap
- gunbc-viz

These should be derived from a central tool registry, not duplicated.

### CI Pipeline Structure

Current pipeline:
```
SetupDeps → Prep → Build → Test  → Report
                       \→ Lint →/
```

This is good. The enhancement is making the TOOLS used at each step explicit and validated.

### Runner Tool Mapping

```rust
impl RunnerImage {
    pub fn ubuntu_latest() -> Self {
        Self {
            id: "ubuntu-latest",
            platform_id: "ubuntu",
            // Tools pre-installed on ubuntu-latest
            provided_tools: &["git", "curl", "wget", "python3"],
            // Note: rust NOT provided - needs rust-toolchain action
        }
    }
}
```

### Open Questions

1. Should we generate the entire Makefile or keep some manual sections?
2. How do we handle tool version requirements (e.g., needs git >= 2.0)?
3. Should Actions be in core/ir or lib/tools/ci?
