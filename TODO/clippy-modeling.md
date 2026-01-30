# Clippy Modeling

**Status**: Draft
**Date**: 2026-01-29

## Goal

Model clippy as both:
1. **A tool dependency** — clippy needs to be installed, verified, and can be upserted
2. **A meta-understanding** — how we USE clippy to enforce architectural constraints (transport pattern)

This follows the-gunbai pattern where tools are not just dependencies but have associated "understandings" — documentation of HOW they're configured and WHY.

## Current State

### What We Have

1. **clippy.toml** — enforces transport pattern via `disallowed-methods`:
   - Blocks `std::fs::*` ops (use `PrepareFileReadOp` instead)
   - Blocks `std::process::Command::new` (use `PrepareShellOp` instead)

2. **Makefile target** — `make clippy` runs `cargo clippy --all-targets -- -D warnings`

3. **CI integration** — Lint step runs clippy as part of the pipeline

4. **BuildConfig** — `lint_command` defined in makegen registry

### What's Missing

1. **No ToolDef for clippy** — can't do satisfiability checks
2. **No documentation of clippy configuration** — WHY these rules exist
3. **No validation that clippy.toml is correct** — could drift from intent
4. **No crate-level allowances documented** — some crates bypass rules

## Design

### 1. Clippy as ToolDef

```rust
// core/ir/src/transport/tool.rs

/// Clippy linter (part of Rust toolchain)
pub static CLIPPY: ToolDef = ToolDef {
    id: "clippy",
    command: "cargo-clippy",  // or just cargo clippy
    verify: "cargo clippy --version",
    install_options: &[
        // clippy comes with rustup
        InstallOption {
            via: "rustup",
            inputs: InstallInputs::component("clippy"),
        },
    ],
    depends_on: &["rust"],  // clippy needs rust
};
```

**Note**: Need to extend `InstallInputs` for rustup components:

```rust
pub struct InstallInputs {
    pub packages: Option<&'static [&'static str]>,
    pub crate_name: Option<&'static str>,
    pub git_url: Option<&'static str>,
    pub component: Option<&'static str>,  // NEW: for rustup component add
}
```

### 2. Clippy Configuration as Data

Model the clippy.toml configuration as structured data:

```rust
// lib/tools/lint/src/config.rs (new crate or in existing location)

/// A clippy disallowed method rule
pub struct DisallowedMethod {
    pub path: &'static str,
    pub reason: &'static str,
    pub category: LintCategory,
}

pub enum LintCategory {
    /// Direct I/O that should use transport layer
    TransportBypass,
    /// Other architectural violations
    Architecture,
    /// Style preferences
    Style,
}

/// Clippy configuration for gunbc
pub struct ClippyConfig {
    pub disallowed_methods: Vec<DisallowedMethod>,
    pub crate_allowances: Vec<CrateAllowance>,
}

pub struct CrateAllowance {
    pub crate_name: &'static str,
    pub allows: &'static [&'static str],
    pub reason: &'static str,
}
```

### 3. Configuration Source of Truth

Like the-gunbai, define the configuration in Rust and GENERATE clippy.toml:

```rust
pub fn gunbc_clippy_config() -> ClippyConfig {
    ClippyConfig {
        disallowed_methods: vec![
            DisallowedMethod {
                path: "std::fs::read",
                reason: "Use PrepareFileReadOp + TransportOps::Execute for transport compliance",
                category: LintCategory::TransportBypass,
            },
            DisallowedMethod {
                path: "std::fs::write",
                reason: "Use PrepareFileWriteOp + TransportOps::Execute for transport compliance",
                category: LintCategory::TransportBypass,
            },
            DisallowedMethod {
                path: "std::process::Command::new",
                reason: "Use PrepareShellOp + TransportOps::Execute for transport compliance",
                category: LintCategory::TransportBypass,
            },
            // ... more
        ],
        crate_allowances: vec![
            CrateAllowance {
                crate_name: "gunbc-transport",
                allows: &["std::fs::*", "std::process::Command::new"],
                reason: "Transport executor is the designated I/O boundary",
            },
            CrateAllowance {
                crate_name: "gunbc-primitives",
                allows: &["std::fs::*", "std::process::Command::new"],
                reason: "Deprecated ops, allowed for backwards compatibility",
            },
            CrateAllowance {
                crate_name: "gunbc-codegen",
                allows: &["std::fs::*"],
                reason: "Bootstrapper - can't use transport (chicken/egg)",
            },
        ],
    }
}
```

### 4. Generate clippy.toml

```rust
pub fn generate_clippy_toml(config: &ClippyConfig) -> String {
    let mut output = String::from("# Generated from lint config - do not edit manually\n");
    output.push_str("# Regenerate with: cargo run -p gunbc-lint -- generate\n\n");
    
    output.push_str("disallowed-methods = [\n");
    for method in &config.disallowed_methods {
        output.push_str(&format!(
            "    {{ path = \"{}\", reason = \"{}\" }},\n",
            method.path, method.reason
        ));
    }
    output.push_str("]\n");
    
    output
}
```

### 5. Per-Crate Allowance Documentation

Generate a markdown doc explaining WHY each crate has allowances:

```rust
pub fn generate_lint_docs(config: &ClippyConfig) -> String {
    // Generate docs/lint-allowances.md
    // Lists each crate with #![allow(clippy::disallowed_methods)] and WHY
}
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Clippy Integration                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ToolDef (clippy)          ClippyConfig                     │
│  ├── id: "clippy"          ├── disallowed_methods[]         │
│  ├── verify                │   ├── path                     │
│  ├── install_via: rustup   │   ├── reason                   │
│  └── depends_on: [rust]    │   └── category                 │
│                            └── crate_allowances[]           │
│           │                         │                       │
│           ▼                         ▼                       │
│    Satisfiability           Generate clippy.toml            │
│    Check                    Generate lint-allowances.md     │
│                                                             │
│           │                         │                       │
│           ▼                         ▼                       │
│    CI Lint Step             Validated Config                │
│    (cargo clippy)           (source of truth)               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Tasks

- [ ] **Extend InstallInputs** — add `component` field for rustup
- [ ] **Add CLIPPY ToolDef** — in tool.rs with rustup install option
- [ ] **Add RUSTUP ToolDef** — as a package manager (base tool)
- [ ] **Create ClippyConfig struct** — model clippy.toml as data
- [ ] **Implement gunbc_clippy_config()** — source of truth in Rust
- [ ] **Generate clippy.toml** — from config (like deps.toml)
- [ ] **Document crate allowances** — generate lint-allowances.md
- [ ] **Add to CI** — verify clippy.toml matches generated version

## Notes

### Reference: the-gunbai Pattern

In the-gunbai, tools have associated markdown files that document:
- What the tool does
- How it's configured
- Why specific settings exist

This is the "understanding" pattern — not just THAT a tool is installed, but HOW it's used.

### Crate Allowances Strategy

Current crates with `#![allow(clippy::disallowed_methods)]`:
- `lib/transport/` — legitimate I/O boundary
- `lib/primitives/` — deprecated, should migrate
- `core/codegen/` — bootstrap code, can't use transport
- `lib/tools/ci/` — minor Path::exists check

Long-term: reduce allowances by migrating to transport pattern everywhere possible.

### Open Questions

1. Should clippy config live in a new `gunbc-lint` crate or in `core/ir`?
2. Should we track clippy VERSION requirements (like GH_CLI_MIN_VERSION)?
3. How do we handle clippy.toml for workspace vs per-crate?
