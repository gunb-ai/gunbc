# Clippy Modeling

**Status**: Complete
**Date**: 2026-01-29

## Goal

Model clippy as both:
1. **A tool dependency** — clippy needs to be installed, verified, and can be upserted ✅ DONE
2. **Configuration as code** — clippy.toml generated from `ClippyConfig` struct ✅ DONE

This follows the fractal DAG pattern where tool crates define both tool operations AND their configuration.

## Current State

### What We Have (Implemented)

1. **clippy.toml** — enforces transport pattern via `disallowed-methods`:
   - Blocks `std::fs::*` ops (use `PrepareFileReadOp` instead)
   - Blocks `std::process::Command::new` (use `PrepareShellOp` instead)

2. **Makefile target** — `make clippy` runs `cargo clippy --all-targets -- -D warnings`

3. **CI integration** — Lint step runs clippy as part of the pipeline

4. **BuildConfig** — `lint_command` defined in makegen registry

5. **✅ `cli::CLIPPY` CliToolDef** (`core/ir/src/transport/cli.rs:444`):
   ```rust
   pub static CLIPPY: CliToolDef = CliToolDef {
       id: "clippy",
       check_cmd: &["cargo", "clippy", "--version"],
       install_cmd: Some(&["rustup", "component", "add", "clippy"]),
       run_cmd: &["cargo", "clippy"],
       description: "Rust linter",
       access_mode: AccessMode::Read,
   };
   ```

6. **✅ `tool::CLIPPY` ToolDef** (`core/ir/src/transport/tool.rs:548`):
   ```rust
   pub static CLIPPY: ToolDef = ToolDef {
       id: "clippy",
       command: "cargo",
       verify: "cargo clippy --version",
       install_options: &[], // Installed as rust component via rustup
       depends_on: &["cargo"],
   };
   ```

7. **✅ `gunbc-clippy` crate** (`lib/tools/clippy/`):
   - `build_clippy_upsert()` — Fractal sub-DAG (check → install → run)
   - `build_clippy_lint_all()` — Standard `--all-targets -- -D warnings`
   - `Clippy::upsert_and_run()` — Imperative convenience wrapper

8. **✅ Generic CLI tool upsert pattern** (`core/ir/src/transport/cli.rs`):
   - `build_cli_upsert(tool, args)` — Creates fractal sub-DAG for any CLI tool
   - `build_cli_ensure(tool)` — Check + install without run
   - `ToolHandle` — Capability-based access (can't use tool without acquiring)

9. **✅ `ClippyConfig` struct** (`lib/tools/clippy/src/config.rs`):
   - `ClippyConfig` — Main config struct with disallowed_methods and crate_allowances
   - `DisallowedMethod` — Individual method rule (path + reason)
   - `CrateAllowance` — Crate bypass documentation
   - `ClippyConfig::transport_pattern()` — Preset for transport pattern enforcement
   - `generate_clippy_toml()` — Renders config to TOML format
   - `ClippyConfigRenderer` — Implements `Renderable` for standard header

10. **✅ Codegen integration** (`core/codegen/src/main.rs`):
    - `cargo run -p gunbc-codegen -- clippy-toml` — Generates clippy.toml

### What's Still Missing

1. **CI verification** — Verify clippy.toml matches generated version in CI pipeline
2. **Lint allowances doc** — Generate docs/lint-allowances.md explaining crate exceptions

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

### Implemented: Two-System Tool Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Clippy Tool Integration (COMPLETE)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  CliToolDef (cli.rs:444)           ToolDef (tool.rs:548)                │
│  ├── id: "clippy"                  ├── id: "clippy"                     │
│  ├── check_cmd                     ├── command: "cargo"                 │
│  ├── install_cmd (rustup)          ├── verify                           │
│  ├── run_cmd                       └── depends_on: ["cargo"]            │
│  └── access_mode: Read                                                  │
│           │                                  │                          │
│           ▼                                  ▼                          │
│  build_cli_upsert()               Platform Satisfiability               │
│  (fractal sub-DAG)                (deps.toml generation)                │
│           │                                                             │
│           ▼                                                             │
│  gunbc-clippy crate                                                     │
│  ├── build_clippy_upsert()                                              │
│  ├── build_clippy_lint_all()                                            │
│  └── Clippy::upsert_and_run()                                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

### Implemented: Configuration Modeling

┌─────────────────────────────────────────────────────────────────────────┐
│                    ClippyConfig (IMPLEMENTED)                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  lib/tools/clippy/src/config.rs                                         │
│  ├── ClippyConfig                                                       │
│  │   ├── disallowed_methods: Vec<DisallowedMethod>                      │
│  │   ├── crate_allowances: Vec<CrateAllowance>                          │
│  │   └── large_error_threshold: Option<u32>                             │
│  │                                                                      │
│  ├── ClippyConfig::transport_pattern()  → Preset config                 │
│  ├── generate_clippy_toml()             → Render to TOML                │
│  └── ClippyConfigRenderer               → Implements Renderable         │
│                                                                         │
│  core/codegen/src/main.rs                                               │
│  └── cmd_clippy_toml()                  → `codegen clippy-toml`         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Tasks

### Tool Modeling (COMPLETE ✅)

- [x] **Add CLIPPY CliToolDef** — Runtime tool acquisition (`cli.rs:444`)
- [x] **Add CLIPPY ToolDef** — Platform-aware satisfiability (`tool.rs:548`)
- [x] **Add RUSTUP CliToolDef** — For component management (`cargo/src/ops.rs:68`)
- [x] **Create gunbc-clippy crate** — Full integration (`lib/tools/clippy/`)
- [x] **Implement build_clippy_upsert()** — Fractal sub-DAG pattern
- [x] **Implement build_cli_upsert()** — Generic pattern for any CLI tool
- [x] **Implement ToolHandle** — Capability-based tool access

### Configuration Modeling (COMPLETE ✅)

The config modeling lives in `lib/tools/clippy/` following the same pattern as other tool crates:

| Crate | Config Type | Generated File |
|-------|-------------|----------------|
| `lib/tools/deps/` | `Manifest` | `deps.toml` |
| `lib/tools/makegen/` | `BuildConfig` | `Makefile` |
| `lib/tools/ci/` | `WorkflowConfig` | `ci.yml` |
| `lib/tools/clippy/` | `ClippyConfig` ✅ | `clippy.toml` ✅ |

Tasks:
- [x] **Create ClippyConfig struct** — in `lib/tools/clippy/src/config.rs`
- [x] **Implement ClippyConfig::transport_pattern()** — preset for transport pattern enforcement
- [x] **Generate clippy.toml** — `generate_clippy_toml()` function
- [x] **Add to codegen** — `cargo run -p gunbc-codegen -- clippy-toml`
- [ ] **Add to CI** — verify clippy.toml matches generated version

### Note on InstallInputs.component

The original design proposed adding `InstallInputs.component` for rustup. This was NOT implemented because a simpler approach was taken: `CliToolDef.install_cmd` directly specifies `["rustup", "component", "add", "clippy"]`. This achieves the same goal without extending the `InstallInputs` type.

## Notes

### Reference: the-gunbai Pattern

In the-gunbai, tools have associated markdown files that document:
- What the tool does
- How it's configured
- Why specific settings exist

This is the "understanding" pattern — not just THAT a tool is installed, but HOW it's used.

### Crate Allowances Strategy

Current crates with `#![allow(clippy::disallowed_methods)]`:
- `lib/transport/` — legitimate I/O boundary (executes transport requests)
- `core/codegen/` — bootstrap code, can't use transport (chicken/egg)

**Removed allowances** (after transport compliance refactor 2026-01-30):
- `lib/primitives/` — no longer has direct I/O ops (ReadFileOp, WriteFileOp, etc. deleted)
- `lib/tools/ci/` — now uses transport layer for all I/O
- `lib/tools/deps/` — now uses transport layer for all I/O
- `lib/tools/bootstrap/` — now uses transport layer for all I/O

The transport compliance refactor eliminated all direct I/O from tool crates. See `TODONE/transport-compliance.md`.

### Open Questions

1. Should clippy config live in a new `gunbc-lint` crate or in `core/ir`?
2. Should we track clippy VERSION requirements (like GH_CLI_MIN_VERSION)?
3. How do we handle clippy.toml for workspace vs per-crate?
