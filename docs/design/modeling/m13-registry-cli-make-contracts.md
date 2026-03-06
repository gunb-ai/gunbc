# M13: Registry→CLI→Make Contract Tests

**Status**: Design
**Lane**: C (Process Contract Drift)
**Blocks**: M14

## Problem

Tool entrypoints flow through three representations: `ToolRegistration`
(inventory) → `ToolDef` (codegen) → `ToolInfo` (makegen). Each hop can drop
or mismap fields. Existing contract tests in `tool_registration.rs` verify
name-level parity but don't check entrypoint semantics (repeatable flags,
cardinality, default values).

## Current Contract Tests

1. `derive_tool_defs_matches_inventory()` — name parity
2. `makegen_default_registry_matches_codegen_registry()` — generated-cli lockstep
3. `codegen_cli_discovery_avoids_tool_registry_inventory()` — separation of concerns

## Design

### 1. Entrypoint round-trip harness

For each tool with entrypoints:
```
ToolRegistration.entrypoints_json
  → CliEntrypoint::from_json()
  → ToolInfo::from_tool_def() (entrypoints with make_var)
  → EntrypointParam
```

Test: Parse entrypoints from JSON, convert to ToolDef, convert to ToolInfo,
verify no field is lost or mistyped.

### 2. Cardinality contract

```rust
#[test]
fn repeatable_flags_survive_roundtrip() {
    for tool in derive_tool_defs() {
        for ep in &tool.entrypoints {
            if ep.cardinality.allows_many() {
                // This entrypoint should be repeatable in CLI and Make
                // Verify the CLI flag supports multiple values
                // Verify the Make variable accepts space-separated values
            }
        }
    }
}
```

### 3. Default value contract

```rust
#[test]
fn default_values_survive_roundtrip() {
    for tool in derive_tool_defs() {
        for ep in &tool.entrypoints {
            if let Some(default) = &ep.default_value {
                // The default should appear in both CLI --help and Make ?= assignment
            }
        }
    }
}
```

### 4. make_var↔CLI flag bijection

Every entrypoint with `make_var` must have a corresponding CLI flag.
Every CLI flag that appears in Makefile must have a `make_var`.

## Files

- `gunbc-app/tests/tool_registration.rs` — new contract tests
- `core/codegen/src/registry.rs` — derive_tool_defs()
- `gunbc-app/src/makegen/registry.rs` — ToolInfo::from_tool_def()

## References

- `core/tool-registry/src/lib.rs` — ToolRegistration struct
- `core/codegen/src/cli_gen.rs` — CliEntrypoint
- `gunbc-app/src/makegen/registry.rs:642-678` — from_tool_def conversion
