# CLI Tool Dependency Graph

**Completed**: January 2026

## Summary

Implemented a unified system for CLI tool dependency management where:
- Tools are `ToolDef` structs with install options
- Package managers are also `ToolDef` structs
- Platforms map to available PMs
- Satisfiability is checked before execution
- `deps.toml` is generated from the registry

## Files Created/Modified

- `core/ir/src/transport/tool.rs` — Core types (ToolDef, InstallInputs, registries)
- `core/ir/src/transport/github/cli.rs` — GH_TOOL definition
- `lib/tools/deps/src/tool_upsert.rs` — Integration with Installer, deps.toml generation

## Key Design Decisions

1. **No enums for install methods** — Package manager IDs are strings
2. **Package managers as tools** — APT, BREW, etc. are `ToolDef`s with empty install_options
3. **Platform hierarchy** — ubuntu → linux for PM inheritance
4. **No "script" fallback** — Each install method must be a properly modeled PM
