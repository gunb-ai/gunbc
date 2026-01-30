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

### 6. Implicit Dependencies Through Structure

**Dependencies should be expressed through usage, not explicit lists.** When you use something, you depend on it. The system should deduce dependencies from the graph structure rather than requiring manual bookkeeping.

Bad:
```rust
// Explicit dependency list that must be kept in sync
fn lint() {
    REQUIRED_TOOLS.push("clippy");  // Easy to forget!
    run_clippy();
}
```

Good:
```rust
// Dependency is implicit through usage
Node::opaque("lint", ..., LintOp)
    .requires(&cli::CLIPPY)  // I need clippy → framework handles the rest
```

This principle applies broadly:
- **Tool dependencies**: Using a tool = depending on it
- **Data flow**: Connecting ports = depending on that data
- **Resource access**: Accessing a resource = needing that resource

---

## E2E Design Philosophy Examples

### Example 1: CI Lint depends on Clippy

**Before** (explicit management):
```rust
// CI had to track all tool dependencies in a central list
fn build_ci_graph() -> Dag<CIOp> {
    // ... build DAG ...
}

fn get_required_tools() -> Vec<ToolDef> {
    vec![CLIPPY, RUSTFMT, CARGO]  // Manual list, can drift from actual usage
}

fn run_ci() {
    for tool in get_required_tools() {
        ensure_installed(&tool);  // Upfront satisfiability check
    }
    execute(build_ci_graph());
}
```

**After** (implicit through structure):
```rust
// Lint node declares what it needs, framework handles acquisition
fn build_ci_graph() -> Dag<CIOp> {
    // ...
    let lint = Node::opaque("lint", inputs, outputs, CIOp::Lint)
        .requires(&cli::CLIPPY);  // "I need clippy"
    // ...
}

fn run_ci() {
    execute(build_ci_graph());  // Framework sees .requires(), handles upsert automatically
}
```

The dependency is co-located with the usage. No separate list to maintain.

### Example 2: Boundary Detection (Structural Inference)

**Bad approach** (annotation-based):
```rust
#[boundary]  // Annotation that could be forgotten or misplaced
fn write_to_gist(content: String) -> Url { ... }
```

**Good approach** (structural):
```rust
// Boundaries are INFERRED from unconnected output ports
// No annotation needed - if data leaves the DAG, it's a boundary
let gist_node = Node::opaque(
    "create_gist",
    vec![port("content", "String")],
    vec![port("url", "Url")],  // If nothing consumes this → boundary
    CreateGistOp,
);
```

The framework detects boundaries by analyzing graph structure, not by trusting annotations.

### Example 3: Resource Conflicts (Structural Detection)

**Bad approach** (manual conflict declaration):
```rust
fn lint() {
    CONFLICTS_WITH.push("format");  // Must remember to declare
    run_clippy();
}
```

**Good approach** (resource-based):
```rust
// Tools declare their access patterns
pub static CLIPPY: CliToolDef = CliToolDef {
    // ...
    access_mode: AccessMode::Read,  // Can run in parallel
};

pub static RUSTFMT: CliToolDef = CliToolDef {
    // ...
    access_mode: AccessMode::Write,  // Modifies files
};

// Framework detects conflicts from resource access patterns
// Nodes using RUSTFMT won't run in parallel with each other
```

Conflicts are deduced from access patterns, not manually declared.

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

This pattern appears throughout the codebase:
- **Tool acquisition**: Check if installed → install if missing (see Capability-Based Tool Acquisition)
- **Codegen in CI**: Check if generated files exist → run codegen if missing (see Bootstrap Pattern)
- **Dependency management**: Check dep status → install/update if needed

### Bootstrap Pattern (Self-Healing CI)

When a tool needs generated code to compile, but that tool is what generates the code, you have a **bootstrap problem**. The solution: give the bootstrap tool a handwritten entry point that uses the resource acquisition pattern internally.

**The Problem**:
```
CI workflow → needs codegen → to build gunbc-ci → which runs codegen (circular!)
```

**The Solution** (`gunbc-ci`):
```rust
// lib/tools/ci/src/main.rs - HANDWRITTEN, not generated
// This file exists specifically to break the bootstrap cycle

fn main() {
    let dag = build_ci_graph();
    execute_with_mode(&dag, mode);  // prep node handles codegen
}
```

The CI graph has a `prep` node that uses the **upsert pattern** for generated code:

```rust
// lib/tools/ci/src/ops.rs
fn execute_prep(_inputs: HashMap<String, Value>) -> Result<...> {
    // Check: do generated files exist?
    if !registry.needs_codegen() {
        return Ok(success_without_codegen);
    }
    
    // Create: run codegen if missing
    run_config_command(&config.codegen_command)?;
    
    Ok(success_with_codegen)
}
```

**Key Files**:
- `lib/tools/ci/src/main.rs` — Handwritten entry point (NOT in codegen registry)
- `lib/tools/ci/src/ops.rs` — `execute_prep()` with upsert pattern
- `lib/tools/makegen/src/registry.rs` — `needs_codegen()` check
- `.github/workflows/ci.yml` — Just runs `cargo run -p gunbc-ci`

**Why This Matters**:
1. CI is **self-healing** — missing generated code is automatically created
2. **No explicit codegen step** in workflows — the tool handles it
3. **Fast path** when code exists — skips codegen entirely
4. **Same pattern as tool acquisition** — check → create if needed

**When to Use This Pattern**:
- Bootstrap tools that generate code for other tools
- Any tool that might run before its dependencies are generated
- CI/CD pipelines that need to be robust to fresh checkouts

### Capability-Based Tool Acquisition (Primary Pattern)

CLI tools are acquired through a capability system that makes it **structurally impossible** to use a tool without acquiring it first.

```rust
// 1. Define tool (core/ir/src/transport/cli.rs)
pub static CLIPPY: CliToolDef = CliToolDef {
    id: "clippy",
    check_cmd: &["cargo", "clippy", "--version"],
    install_cmd: Some(&["rustup", "component", "add", "clippy"]),
    run_cmd: &["cargo", "clippy"],
    description: "Rust linter",
    access_mode: AccessMode::Read,  // Tool defines its own exclusivity
};

// 2. Declare requirement (DAG definition)
Node::opaque("lint", inputs, outputs, LintOp)
    .requires(&cli::CLIPPY)  // Framework injects upsert sub-DAG

// 3. Use capability (operation implementation)
fn execute_lint(inputs: HashMap<String, Value>) -> Result<...> {
    // ToolHandle provided by framework after acquisition
    let _clippy = inputs.get("tool:clippy").unwrap();
    
    // Run clippy (tool guaranteed available)
    CliToolOp::run(&cli::CLIPPY, &["--all-targets"]).execute()?
}
```

**Key insight**: `ToolHandle` cannot be constructed directly — it only comes from acquisition. This means:

- No way to bypass the upsert pattern
- No `Command::new()` calls scattered through the codebase
- Framework handles check/install/run automatically
- Tool defines its own resource access patterns (exclusivity)

**Consumer vs Tool Separation**:
- Consumer (e.g., Lint node): Only asks "can I use clippy?" via `.requires()`
- Tool (e.g., Clippy): Defines check/install commands AND access mode
- Framework: Handles acquisition, scheduling, and resource conflict detection

**Clippy Enforcement**: Direct `Command::new()` is disallowed by clippy.toml. Exceptions must use `#[allow(clippy::disallowed_methods)]` with documentation.

### Tool Dependency Graph (For Planning/Satisfiability)

Tools also define how they can be installed across platforms. This is the planning layer (separate from runtime acquisition above).

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

**Two Systems**: `ToolDef` is for planning/satisfiability; `CliToolDef` is for runtime acquisition with `.requires()`. Use `CliToolDef` for new code.

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

### Adding a New CLI Tool

**For runtime acquisition** (new preferred pattern):

1. Define as `CliToolDef` in `core/ir/src/transport/cli.rs`:
   ```rust
   pub static RUFF: CliToolDef = CliToolDef {
       id: "ruff",
       check_cmd: &["ruff", "--version"],
       install_cmd: Some(&["pip", "install", "ruff"]),
       run_cmd: &["ruff"],
       description: "Python linter",
       access_mode: AccessMode::Read,  // Or Write/Exclusive if it modifies state
   };
   ```
2. Add to the tool registry in `core/exec/src/execute.rs:get_tool_by_id()`
3. Use in nodes via `.requires(&cli::RUFF)`

**For planning/satisfiability** (platform-aware installation):

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

### Creating a Bootstrap Tool (Self-Healing)

If your tool needs to run before generated code exists (like CI):

1. **Don't add to codegen registry** — it won't be in `core/codegen/src/registry.rs`
2. **Create handwritten `src/main.rs`** — minimal entry point that calls your graph builder
3. **Add a prep node** that uses the upsert pattern:
   ```rust
   fn execute_prep(...) -> Result<...> {
       if !needs_codegen() {
           return Ok(skip);  // Fast path
       }
       run_codegen()?;  // Create if missing
       Ok(success)
   }
   ```
4. **Update `Cargo.toml`** to use `src/main.rs` instead of generated path:
   ```toml
   [[bin]]
   path = "src/main.rs"  # NOT the generated buck-out path
   ```

See `lib/tools/ci/` for the canonical example of this pattern.

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

### Capability-Based Tool Acquisition

Implemented a capability system for CLI tool dependencies that makes it structurally impossible to use a tool without acquiring it:

- `core/ir/src/transport/cli.rs` — `CliToolDef`, `ToolHandle`, execution functions
- `core/ir/src/node.rs` — `.requires(&tool)` method on Node
- `core/exec/src/execute.rs` — Automatic tool acquisition during execution
- `core/ir/src/signature.rs` — Tool ports excluded from workflow signatures

Key pattern: `Node::opaque(...).requires(&cli::CLIPPY)` declares dependency, framework acquires tool and provides `ToolHandle` through inputs.

### CLI Tool Dependency Graph (Planning Layer)

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

### Bootstrap Pattern for CI (Self-Healing Codegen)

Implemented a self-healing CI pipeline that handles the circular dependency between `gunbc-ci` and codegen:

- `lib/tools/ci/src/main.rs` — Handwritten entry point (breaks bootstrap cycle)
- `lib/tools/ci/src/ops.rs` — `execute_prep()` uses upsert pattern for codegen
- `core/codegen/src/main.rs` — `cigen` command generates CI YAML
- `.github/workflows/ci.yml` — Generated workflow that just runs `cargo run -p gunbc-ci`

**Key insight**: `gunbc-ci` is NOT in the codegen registry. It has a handwritten `main.rs` because it's the bootstrap tool that runs codegen for all other tools. The `prep` node checks if generated files exist and creates them if missing — same upsert pattern used for tool acquisition.

**Pattern reference**: See "Bootstrap Pattern (Self-Healing CI)" in Key Patterns section above.

**Related files**:
- `TODO/ci-dag-rendering.md` — Design document for CI YAML generation
- `core/ir/src/transport/ci/render.rs` — `CiRenderer` trait for multi-provider support
