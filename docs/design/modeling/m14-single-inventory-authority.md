# M14: Single Inventory Authority

**Status**: Design
**Lane**: C (Process Contract Drift)
**Depends on**: M13, M20

## Problem

While `iter_tool_targets()` is the canonical source for tool registrations,
several downstream lists are still manually maintained:
- `WorkspaceBinary` enum in `binaries.rs` (12 variants, manually kept in sync)
- DSL module→tool mapping in makegen
- Binary entrypoints in Makefile

Adding a new tool requires edits in multiple locations.

## Current State

The `WorkspaceBinary` enum already calls `registry_invocation()` which looks
up the tool registration from inventory. But the enum variants themselves are
manually maintained.

## Design

### 1. Derive WorkspaceBinary from inventory

Replace the manually maintained `WorkspaceBinary` enum with a runtime lookup:

```rust
pub fn workspace_binary(tool_name: &str) -> Option<CargoInvocation> {
    iter_tool_targets()
        .find(|t| t.tool_name == tool_name && t.has_invocation)
        .map(|t| {
            let pkg = t.package.unwrap_or("dag");
            let bin = t.binary.unwrap_or(t.tool_name);
            CargoInvocation::composed(bin, pkg)
        })
}
```

Keep `WorkspaceBinary` as a convenience enum but derive its variants from
inventory at test time:

```rust
#[test]
fn workspace_binary_enum_matches_inventory() {
    let inventory_binaries: BTreeSet<&str> = iter_tool_targets()
        .filter(|t| t.has_invocation)
        .map(|t| t.tool_name)
        .collect();
    let enum_binaries: BTreeSet<&str> = WorkspaceBinary::ALL
        .iter()
        .map(|b| b.tool_name())
        .collect();
    assert_eq!(inventory_binaries, enum_binaries);
}
```

### 2. Provides/consumes metadata on ToolRegistration

Add optional `provides` and `consumes` fields:

```rust
pub struct ToolRegistration {
    // ... existing fields ...
    pub provides: &'static [&'static str],  // e.g., ["Makefile", "deps.toml"]
    pub consumes: &'static [&'static str],  // e.g., ["target/codegen/.stamp"]
}
```

This enables the generator edge graph (M20) to derive producer→consumer
relationships from a single source.

### 3. Drift test

```rust
#[test]
fn adding_tool_requires_only_one_registration() {
    // Verify that every tool in makegen registry traces back to
    // exactly one ToolRegistration entry (no manual additions needed)
}
```

## Files

- `core/tool-registry/src/lib.rs` — provides/consumes fields
- `gunbc-app/src/binaries.rs` — derive from inventory
- `gunbc-app/tests/tool_registration.rs` — drift tests

## References

- `core/tool-registry/src/lib.rs:21-96` — ToolRegistration
- `gunbc-app/src/binaries.rs` — WorkspaceBinary enum
- `docs/design/modeling/repo-self-understanding.md` — M20 generator edges
