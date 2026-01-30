# Design Gap: Opaque Operations Hide Tool Dependencies

## Problem Statement

Currently, opaque operations (like `CIOp::Lint`) can use tools internally without the DAG being aware. This creates a synchronization problem:

```rust
// graph.rs - must manually add .requires()
Node::opaque("lint", ..., CIOp::Lint).requires(&cli::CLIPPY)

// ops.rs - must manually call clippy
fn execute_lint(...) {
    CliToolOp::run(&CLIPPY, &["--all-targets"]).execute()?;
}
```

These are **two separate places** that must stay in sync. If someone:
- Adds a tool call in the operation but forgets `.requires()` → runtime error
- Removes a tool call but leaves `.requires()` → unnecessary dependency

This violates the design principle: **"Dependencies should be expressed through usage, not explicit lists."**

## Root Cause

The issue is that `CIOp::Lint` is **opaque** - the DAG cannot see inside it. The clippy usage is hidden in hand-written Rust code that executes at runtime.

## Ideal Design

Lint should be **structurally composed** from a clippy tool invocation:

```rust
// Instead of opaque hand-written code:
CIOp::Lint  // What does this do? DAG can't tell.

// Lint should BE a sub-DAG:
let lint = build_cli_upsert(&cli::CLIPPY, &["--all-targets", "--", "-D", "warnings"]);
```

If Lint is a sub-DAG containing a clippy node, the dependency is structural:
- The DAG sees the clippy node
- No separate `.requires()` needed - dependency is implicit through composition
- Codegen can generate the execution code from the DAG structure

## Current Workaround

The capability-based pattern (`.requires()` + `ToolHandle` through inputs) provides **runtime** safety:
- If `.requires()` is missing, the operation fails with a clear error message
- But this is discovered at runtime, not at DAG validation/codegen time

## Future Solution Options

1. **Make operations sub-DAGs, not opaque code**
   - Lint IS a clippy sub-DAG, not hand-written code calling clippy
   - Dependencies become structural through composition

2. **Codegen operations from declarations**
   - Operations are generated from DAG definitions
   - Both `.requires()` and the tool call come from the same source
   - No mismatch possible

3. **Static analysis of operation code**
   - Scan operation implementations for tool references
   - Verify nodes declare those dependencies
   - Complex, requires proc macros or external tooling

## Related Files

- `lib/tools/ci/src/ops.rs` - Hand-written `execute_lint` that calls clippy
- `lib/tools/ci/src/graph.rs` - Manual `.requires(&cli::CLIPPY)` on lint node
- `core/ir/src/transport/cli.rs` - `CliToolDef` and `ToolHandle` definitions

## Status

**Deferred** - Current approach is functional with runtime safety. Revisit when:
- Codegen is more mature
- More operations need tool dependencies
- Pattern for "operations as sub-DAGs" is established
