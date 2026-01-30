# Agent Onboarding Guide — gunbc

This document captures project structure, design philosophy, and implementation patterns for AI agents working on this codebase.

---

## Quick Context

**gunbc** is a Rust-based workflow IR (Intermediate Representation) system. The core idea: **everything is a DAG** (Directed Acyclic Graph). Workflows, types, validations, and resource management are all expressed as graphs of nodes connected by typed edges.

**Key principle**: If it validates, it is structurally sound.

---

## Related Projects

### the-gunbai

The `the-gunbai` repository (referenced in SPEC.md) is the **design origin**:

- **Design docs**: `the-gunbai/docs/design/v2/` contains the foundational thinking
- **deps.toml pattern**: gunbai pioneered the pattern of generating `deps.toml` from tool definitions
- **Package manager docs**: gunbai has `package_manager.md` files documenting PM behaviors

When the user mentions "follow the gunbai pattern" or "like in the-gunbai", they mean:
1. Declarative tool definitions that are the **source of truth**
2. Generated artifacts (deps.toml, scripts) derived from those definitions
3. Explicit dependency graphs for pre-flight satisfiability checks

---

## Core Architecture

### DAG Structure

```
Dag<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
}

Node<T> {
    id: NodeId,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    body: NodeBody<T>,  // Opaque(T) or SubDag(Dag<T>)
}

Port {
    name: PortName,
    type_id: TypeId,
    cardinality: Cardinality,  // Zero, One, ZeroOrOne, ZeroOrMore, OneOrMore
}

Edge {
    from: (NodeId, PortName),
    to: (NodeId, PortName),
}
```

### Key Files

| Path | Purpose |
|------|---------|
| `core/ir/src/dag.rs` | Core DAG data structures |
| `core/ir/src/node.rs` | Node definition |
| `core/ir/src/builder.rs` | DAG construction with validation |
| `core/ir/src/patterns/` | Higher-order patterns (upsert, branch, loop, etc.) |
| `core/ir/src/transport/` | I/O abstractions (REST, file, shell, GitHub) |
| `core/exec/` | Execution engine |
| `lib/tools/` | Tool-specific implementations |
| `docs/design/overview.md` | Detailed design philosophy |
| `SPEC.md` | Formal specification |
| `TODO/` | Design plans for upcoming features |

---

## Design Principles (Critical)

### 1. Causality is a DAG

Effects cannot precede causes. Dependencies are acyclic. Information flows forward.

### 2. No Meta-Annotations

**All behavior must be expressed through the type system, not annotations.** If something can change observable behavior, it must be a node/edge/type, not metadata.

Bad: A "guard" annotation that skips required values
Good: `cardinality: ZeroOrOne` to express optionality

### 3. Metadata Erasure is Semantics-Preserving

If you can remove all metadata without changing behavior, you've correctly separated structure from decoration.

### 4. Nodes are Pure, Boundaries are Structural

- **Nodes**: Pure transformations (inputs → outputs)
- **Boundaries**: Where DAG meets the world (I/O, transport)
- Boundaries are inferred from unconnected ports, not annotated

### 5. Types Express Behavior

Instead of runtime checks, express constraints as types. Type validation can itself be a DAG (`Dag<TypeOp>`).

---

## Key Patterns

### Upsert Pattern

The foundational idempotent operation pattern:

```
Check → Create → Resolve
```

Used in `core/ir/src/patterns/upsert.rs`:

```rust
UpsertBuilder::new("install_tool")
    .with_check(op)    // Verify existence
    .with_create(op)   // Create if missing
    .with_resolve(op)  // Verify success
    .build()
```

### Tool Dependency Graph (Recently Implemented)

Tools define how they can be installed. Package managers are also tools.

```rust
// core/ir/src/transport/tool.rs
pub struct ToolDef {
    pub id: &'static str,
    pub command: &'static str,
    pub verify: &'static str,
    pub install_options: &'static [InstallOption],
    pub depends_on: &'static [&'static str],
}

pub struct InstallOption {
    pub via: &'static str,  // PM id: "apt", "brew", "cargo"
    pub inputs: InstallInputs,
}
```

**Key insight**: No enum for install methods. Package manager IDs are strings. Tools provide inputs, PMs provide the upsert mechanics.

**Satisfiability**: Given available PMs on a platform, check if required tools can be installed before running a workflow.

---

## Crate Organization

```
core/
├── ir/          # Core IR types (Dag, Node, Edge, Port, patterns)
├── exec/        # Execution engine
├── test/        # Testing utilities, mock system
├── codegen/     # Code generation
└── testgen/     # Test generation

lib/
├── primitives/  # Primitive operations
├── tools/       # Tool implementations
│   ├── deps/    # Dependency management (Installer, manifest, tool_upsert)
│   ├── gist/    # GitHub Gist tool
│   ├── ci/      # CI tool
│   └── ...
└── transport/   # Transport implementations
```

---

## User Preferences (Important)

Based on previous conversations:

1. **Avoid enums for extensible concepts** — prefer data-driven approaches with string IDs
2. **Package managers are first-class** — model them as tools themselves, not just install methods
3. **No "script" fallbacks** — each install method must be a properly modeled PM
4. **Graph structure over file structure** — tools can be scattered, but the dependency graph must be explicit
5. **Integration with existing patterns** — new code should use existing `UpsertBuilder`, `Installer`, etc.
6. **Platform hierarchy** — platforms can have parents (ubuntu → linux) for PM inheritance

---

## Common Tasks

### Adding a New Tool

1. Define as `ToolDef` constant (in `tool.rs` or contextual location like `github/cli.rs`)
2. List install options with PM associations
3. Add to `default_tool_registry()` if it should be globally available
4. The upsert and deps.toml generation come for free via `tool_upsert.rs`

### Adding a New Package Manager

1. Define as `ToolDef` with empty `install_options` (base PM)
2. Update `pm_to_platform` mapping in `tool_upsert.rs` if needed
3. Ensure `Installer.generate_install_cmd()` handles the new PM

### Adding a New Pattern

1. Create in `core/ir/src/patterns/`
2. Follow the `*Builder` pattern (see `UpsertBuilder`, `BranchBuilder`)
3. Return a `Dag<T>` or `Node<T>` from `.build()`

---

## Testing Conventions

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of each file
- Static test data uses `static` constants for `'static` lifetime requirements
- Mock specs are in `*_mock.rs` files for test generation

---

## Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p gunbc-ir

# Specific module
cargo test -p gunbc-ir tool::

# With output
cargo test -- --nocapture
```

---

## Git Conventions

- Don't commit unless explicitly asked
- Use descriptive commit messages focusing on "why" not "what"
- The repo uses standard Rust formatting (cargo fmt)

---

## When in Doubt

1. Read `docs/design/overview.md` — it's comprehensive
2. Check `SPEC.md` for formal definitions
3. Look at existing patterns in `core/ir/src/patterns/`
4. The `builder.rs` tests show valid DAG construction
5. Ask about the `the-gunbai` repo if something seems inspired by it

---

## Recent Work (January 2026)

### CLI Tool Dependency Graph

Implemented a unified system for CLI tool dependency management:

- `core/ir/src/transport/tool.rs` — Core types (ToolDef, InstallInputs, registries)
- `core/ir/src/transport/github/cli.rs` — GH_TOOL definition
- `lib/tools/deps/src/tool_upsert.rs` — Integration with Installer, deps.toml generation

This replaced the approach in `TODO.md` (trait-based with enums) with a data-driven approach where:
- Tools are `ToolDef` structs with install options
- Package managers are also `ToolDef` structs (with empty install_options)
- Platforms map to available PMs
- Satisfiability is checked before execution
- deps.toml is generated from the registry
