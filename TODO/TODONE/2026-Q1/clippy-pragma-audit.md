# Clippy `disallowed_methods` Pragma Audit

**Status**: Complete (2026-02-05)

## Summary

The workspace `clippy.toml` disallows direct use of `std::fs::*` and
`std::process::*` to enforce that all I/O flows through the transport layer
or other designated infrastructure. Code that legitimately needs direct access
uses `#[allow(clippy::disallowed_methods)]` with the guardrails script
(`tools/check-disallowed-methods.sh`) enforcing an exact allowlist.

**After this audit**: 20 pragmas remain across 9 files. All are at legitimate
I/O boundaries. The 4 test-helper pragmas that were in `core/ir/src/resource/managed.rs`
have been eliminated by moving the helpers to `core/infra/src/test_utils.rs`
(where the crate-level `clippy.toml` allows fs operations).

---

## Crate-Level Exemptions (2 files, 2 pragmas)

These crates use `#![allow(clippy::disallowed_methods)]` at the crate root,
exempting all code in the crate.

### `core/codegen/src/lib.rs` (1)

**Justification**: The codegen crate generates Rust source files, writes
`Cargo.toml` entries, and manages the `bin/` symlink directory. Every module
in this crate performs file I/O as its core function. A crate-level exemption
is appropriate.

**Alternative considered**: Per-function pragmas on each `write_file` call.
Rejected — would add ~20 pragmas with no additional safety.

### `lib/transport/src/lib.rs` (1)

**Justification**: The transport crate IS the I/O boundary. It implements
`execute_shell`, `execute_file`, `execute_git`, etc. Direct `Command::new`
and `fs::*` usage is the crate's entire purpose.

**Alternative considered**: None — this is the canonical I/O layer.

---

## Function-Level Exemptions (7 files, 18 pragmas)

### `core/codegen/src/main.rs` (4)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `run_cargo_build` | `Command::new("cargo")` | Bootstrapper — builds the transport layer itself |
| `setup_bin_directory` | `fs::*` (symlink, create_dir, etc.) | Sets up bin/ symlink directory |
| `cmd_rollback` | `fs::remove_dir_all`, `fs::remove_file` | Transactional rollback of generated files |
| `codegen_clis` | `fs::create_dir_all`, `fs::write` | Generates CLI main.rs source files |

**Why not use transport**: This binary bootstraps the build system. The
transport layer doesn't exist yet when codegen runs for the first time.

### `core/ir/src/transport/cli.rs` (7)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `WhichResolver::resolve` | `Command::new("which")` | Tool path discovery |
| `upsert_tool` | (delegates) | Convenience wrapper |
| `upsert_tool_with` | `fs::metadata` | Check/install/run tool lifecycle |
| `execute_check` | `Command::new(tool.binary)` | Run tool's check command |
| `execute_install` | `Command::new(tool.install_cmd)` | Run tool's install command |
| `execute_run` | `Command::new(tool.binary)` | Run tool with args |
| `execute_run_with_path` | `Command::new(path)` | Run tool with resolved binary path |

**Why not use transport**: These functions ARE the abstraction. They implement
the `CliTool` upsert pattern that other code uses instead of `Command::new`.

### `core/ir/src/transport/github/cli.rs` (3)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `is_gh_installed` | `Command::new("gh")` | Check if GitHub CLI exists |
| `gh_installed_version` | `Command::new("gh")` | Parse gh version |
| `is_gh_authenticated` | `Command::new("gh")` | Check auth status |

**Why not use transport**: Same as `cli.rs` — these are the GitHub CLI
integration layer. Wrapping `Command::new("gh")` with the proper error
handling and version parsing.

### `gunbc-app/src/bin/testgen.rs` (1)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `main` | `fs::write`, `fs::create_dir_all` | Writes generated test source files |

**Why not use transport**: Same as codegen — this is a code generator binary
that writes Rust source files. It runs at build time, not through the DAG.

### `lib/tools/deps/src/installer.rs` (1)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `is_installed` | `Command::new(verify_cmd)` | Verify tool installation |

**Why not use transport**: The installer IS the tool management layer.
It runs arbitrary verify commands to check if deps are installed.

### `lib/tools/deps/src/manifest.rs` (1)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `DepsManifest::load` | `fs::read_to_string` | Load deps.toml manifest |

**Why not use transport**: The manifest loader runs before the DAG is
constructed. It reads a single config file to determine what deps to install.

### `lib/transport/src/executor.rs` (1)

| Function | Disallowed method | Reason |
|----------|-------------------|--------|
| `execute_shell` | `Command::new` | THE shell execution boundary |

**Why not use transport**: This IS the transport executor. All
`TransportRequest::Shell` requests flow through this single function.

---

## Eliminated Pragmas

### `core/ir/src/resource/managed.rs` (was 4, now 0)

Previously had 4 pragmas on test helpers: `temp_dir`, `write_file`,
`cleanup_dir`, and `TestProviderResource::create`.

**Fix**: Moved helpers to `core/infra/src/test_utils.rs`. Since `core/infra`
has an empty `clippy.toml` (allowing fs ops), no pragmas needed. The
`managed.rs` test module now delegates to `gunbc_infra::test_utils::*`.

Also deduplicated `core/infra/src/freshness.rs` which had its own inline
`temp_dir()` — now uses the shared `crate::test_utils::temp_dir`.

---

## Guardrails Script Fixes

`tools/check-disallowed-methods.sh` had two bugs discovered during this audit:

1. **Regex `\!` invalid in rg**: The pattern used `\!?` but rg's Rust regex
   engine doesn't recognize `\!` as a valid escape. Fixed to `!?`.

2. **`--exclude-dir=bin` too broad in grep**: The grep fallback excluded all
   directories named `bin`, which caught `src/bin/testgen.rs`. Removed since
   `--include='*.rs'` is sufficient to filter out compiled binaries.

---

## Policy

All remaining 20 pragmas are at legitimate I/O boundaries:
- **Bootstrapper binaries** (codegen, testgen): write source files at build time
- **Transport layer**: THE I/O abstraction that other code delegates to
- **CLI tool layer**: implements check/install/run for external tools
- **Manifest loader**: reads config before DAG construction

New test code that needs filesystem access should use
`gunbc_infra::test_utils::{temp_dir, write_file, cleanup_dir}` instead of
adding new `#[allow]` pragmas.
