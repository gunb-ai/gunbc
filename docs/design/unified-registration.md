# Unified Registration Model

> **Status**: Stream 1 (`all_tools()` elimination) DONE. Remaining unification tracked in [`TODO/tasks.md`](../../TODO/tasks.md).

> **Goal**: All registrable units (tools, DAGs, testgen targets, resources,
> transports) use the same auto-discovery pattern. Adding a new unit means
> annotating it — not updating a manual list.

---

## Current State: Six Registration Islands

### 1. Testgen Targets (the gold standard)
**Location**: `core/testgen-registry/`, `core/testgen-registry-macros/`
**Pattern**: Proc macro + `inventory` crate → compile-time auto-discovery

```
#[testgen_target(name = "...", output = "...", module = "...", builder = "...")]
pub fn my_mock_spec() -> MockSpec { ... }
    ↓ proc macro generates:
inventory::submit!(TestgenTarget { ... })
    ↓ binary collects:
iter_targets() → all registered targets, zero manual wiring
```

- `TestgenTarget` struct holds all metadata (`&'static str` fields for const registration)
- `inventory::collect!` discovers all submissions across crate boundaries
- `origin_crate` field + path rewriting handles cross-crate module paths
- Validation test (`mock_spec_registration.rs`) catches undecorated functions

**What's good**: Zero manual lists. Adding a testgen target = annotate one function.
Forgetting the annotation is caught by a test. The proc macro enforces required fields
at compile time.

**What's wrong**: Nothing. This is the pattern to follow.

### 2. Tool Definitions (the pain point)
**Location**: `core/codegen/src/registry.rs` — `all_tools()` function
**Pattern**: Hardcoded `Vec<ToolDef>` (~360 lines)

```rust
pub fn all_tools() -> Vec<ToolDef> {
    vec![
        ToolDef::new("gunbc-gist", "gist", "Snapshot tool", GraphBuilderId::Gist, "")
            .entrypoint(CliEntrypoint::new("repo_path", "String").short('r'))
            .entrypoint(...)
            .boundary("fs_env", vec![...])
            .invocation(CargoInvocation::composed("gist", "dag")),
        // ... 6 more tools, each 30-50 lines
    ]
}
```

**What's good**: Single place with all tool metadata. Builder pattern is readable.

**What's wrong**: Adding a tool requires:
1. `ToolDef::new()` entry in `all_tools()` (~30 lines)
2. `GraphBuilderId` enum variant + `as_str()` match arm (2 places in `cli_gen.rs`)
3. Graph builder function in tool crate (e.g., `build_gist_graph()`)
4. MockSpec in `graph_mock.rs` (separate registration via testgen)
5. Binary target in Cargo.toml (auto-derived from `.invocation()`, but requires it to be set)
6. Makefile target (auto-derived from `.invocation()`)

Steps 1-2 are manual and forgettable. The `all_tools()` vec is the #1 place where
tools get silently lost. There's no compile error if a tool crate exists but isn't
registered.

### 3. Graph Builders (string-coupled dispatch)
**Location**: `core/codegen/src/cli_gen.rs` — `GraphBuilderId` enum
**Pattern**: Enum → `as_str()` → string function name in generated code

```rust
pub enum GraphBuilderId {
    Gist,       // → "build_gist_graph"
    Makegen,    // → "build_makegen_graph"
    Deps,       // → "build_deps_graph"
    Review,     // → "build_diff_review_graph"
    Bootstrap,  // → "build_bootstrap_graph"
}

impl GraphBuilderId {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gist => "build_gist_graph",
            // ...
        }
    }
}
```

**What's good**: Enum is exhaustive — can't use an unregistered builder.

**What's wrong**: String coupling. If a tool renames its builder function, `as_str()`
silently emits the wrong name. The generated CLI compiles (it's a string template),
but fails at runtime. Should be function pointers or trait objects.

### 4. Boundary Mock Definitions (dual-source problem)
**Location**: `core/codegen/src/registry.rs` (for CLI dry-run) AND `graph_mock.rs` files (for testgen)
**Pattern**: Same boundaries defined in two unrelated places

```
registry.rs:   .boundary("fs_env", vec![("fs:write", "FilesystemHandle::cross_platform()")])
graph_mock.rs: .boundary("fs_env", "fs:write", mock_fs_handle())
```

**What's good**: Each serves its purpose (CLI generation vs. test generation).

**What's wrong**: They can desync. If a DAG adds a new boundary node, you must update
both places manually. There's no validation that they agree.

### 5. Resource Definitions (manual glob patterns)
**Location**: `core/ir/src/resource/defs.rs`
**Pattern**: Hardcoded `InputPattern::glob(...)` lists

```rust
pub fn codegen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(ResourceId::build("generated_cli"));
    def = def.with_input(InputPattern::glob("core/codegen/src/**/*.rs"));
    def = def.with_input(InputPattern::glob("core/ir/src/**/*.rs"));
    // ...
    def
}
```

**What's good**: Explicit about what inputs affect freshness.

**What's wrong**: Glob patterns are hardcoded strings. If a source file moves or a new
input crate is added, the pattern silently becomes stale. Freshness checks pass but
miss real changes.

### 6. Makefile & CI Targets (auto-derived, but fragile root)
**Location**: `gunbc-app/src/makegen/registry.rs`, `core/codegen/src/main.rs`
**Pattern**: Derived from `all_tools()` — automatic once a tool is registered

```
all_tools() → tools with .invocation() → Makefile targets (auto)
all_tools() → tools with .invocation() → Justfile targets (auto)
all_tools() → tools → CLI generation (auto)
config.ci + tools.cigen.dag + leaf CI serializer modules → CI YAML (DSL-owned for ci tool only)
```

**What's good**: Downstream derivation is automatic. Once a tool is in `all_tools()`,
it gets a Makefile target, CLI binary, etc.

**What's wrong**: The root (`all_tools()`) is manual. If a tool isn't registered there,
everything downstream is missing silently.

---

## The Problem

### Fragmented Discovery

Six systems, three different registration mechanisms (inventory, hardcoded vec, enum
dispatch), zero shared vocabulary for "here's a registrable unit, discover it."

### One Manual Bottleneck

All downstream automation (Makefile targets, CLI generation, binary registration)
flows from `all_tools()` — a single manual vec. This is the #1 source of "forgot to
register" bugs. Testgen solved this problem completely; tools haven't.

### Dual Definitions

Boundary mocks exist in both `registry.rs` and `graph_mock.rs`. These should be a
single source of truth. The MockSpec already has all boundary information — the
registry shouldn't need to duplicate it.

### String-Coupled Dispatch

`GraphBuilderId::as_str()` maps enum variants to function name strings. This coupling
survives compilation but breaks at code-generation time. Should use function pointers.

---

## Proposed Model

### Core Insight: Registration is an Upsert

Every registration follows the same shape:
1. **Declare** — "this unit exists and has these properties"
2. **Collect** — "gather all declared units at link time"
3. **Derive** — "compute downstream artifacts from the collection"

This is Check → Create → Resolve (Upsert) applied to build-time metadata.

### Layer 1: Unified Registry Trait

A common vocabulary for registrable units:

```rust
/// A unit that can be auto-discovered at link time.
///
/// All registrable things (tools, testgen targets, resources) implement
/// this trait to provide identity and derivation metadata.
pub trait Registrable {
    /// Unique identifier within its registry kind.
    fn id(&self) -> &str;

    /// Originating crate (for path rewriting).
    fn origin_crate(&self) -> &str;
}
```

### Layer 2: Tool Registration (`#[tool_target]` macro)

Following the testgen pattern exactly:

```rust
// In lib/tools/gist/src/lib.rs (or graph.rs):

#[tool_target(
    name = "gist",
    crate_name = "gunbc-gist",
    description = "Code snapshot tool",
    builder = "crate::graph::build_gist_graph()",
    invocation = "CargoInvocation::composed(\"gist\", \"dag\")",
)]
pub fn gist_tool_def() -> ToolConfig {
    ToolConfig::new()
        .entrypoint(CliEntrypoint::new("repo_path", "String")
            .short('r').help("Repository path"))
        .entrypoint(CliEntrypoint::new("extensions", "String")
            .with_cardinality(Cardinality::ZERO_OR_MORE))
}
```

The proc macro generates:
```rust
inventory::submit!(ToolRegistration {
    origin_crate: env!("CARGO_CRATE_NAME"),
    name: "gist",
    // ... all metadata
    configure: gist_tool_def,
});
```

And `all_tools()` becomes:

```rust
pub fn all_tools() -> Vec<ToolDef> {
    iter_tool_targets()
        .map(|reg| reg.to_tool_def())
        .collect()
}
```

### Layer 3: Boundary Unification

MockSpec already contains boundary definitions. The tool registry should derive
its boundary mocks from the MockSpec instead of duplicating them:

```rust
#[tool_target(
    name = "gist",
    mock_spec = "crate::graph_mock::gist_mock_spec",  // single source of truth
    // ...
)]
```

The CLI generator reads boundary information from the linked MockSpec. One
definition, two consumers.

### Layer 4: Graph Builder Resolution

Replace `GraphBuilderId` enum with function pointers stored in the registration:

```rust
pub struct ToolRegistration {
    // ...
    /// Function that builds the DAG (no string coupling)
    pub build_graph: fn() -> Dag<Box<dyn Executable>>,
}
```

Generated CLIs call `(registration.build_graph)()` instead of emitting a string
function name. The compiler enforces the function exists and has the right signature.

**Alternative** (if type erasure is too complex): Keep the string-based codegen but
derive the string from the registration macro, which validates the expression at
macro expansion time (like testgen's `builder = "..."` already does).

### Layer 5: Resource Input Discovery

Instead of hardcoded globs, resources declare their input crates:

```rust
#[resource_def(
    id = "generated_cli",
    input_crates = ["gunbc-codegen", "gunbc-ir", "gunbc-exec"],
    extra_patterns = ["clippy.toml", "Cargo.toml"],
)]
```

The macro resolves crate names to directory paths via `cargo metadata` and generates
glob patterns automatically. If a crate is renamed or moved, the resolution updates.

---

## Migration Path

### Phase 1: Tool Registry crate (non-breaking)
- Create `core/tool-registry/` and `core/tool-registry-macros/`
- Define `ToolRegistration` struct (mirrors `TestgenTarget` design)
- Define `#[tool_target]` proc macro
- `inventory::collect!(ToolRegistration)` + `iter_tool_targets()`
- **Files**: `core/tool-registry/src/lib.rs`, `core/tool-registry-macros/src/lib.rs`, `Cargo.toml`

### Phase 2: Annotate existing tools
- Add `#[tool_target(...)]` to each tool's graph.rs or lib.rs
- Keep `all_tools()` as a shim that delegates to `iter_tool_targets()`
- Verify byte-identical CLI and Makefile output
- Delete `GraphBuilderId` enum (replaced by registration metadata)
- **Files**: `lib/tools/gist/src/lib.rs`, `lib/tools/deps/src/lib.rs`, `lib/review/src/lib.rs`, `gunbc-app/src/makegen/`, `gunbc-app/src/ci/`, `gunbc-app/src/bootstrap/`

### Phase 3: Boundary unification
- `ToolRegistration` gains `mock_spec` field (path to MockSpec function)
- CLI generator reads boundaries from MockSpec instead of ToolDef
- Remove `.boundary()` calls from `all_tools()` / tool registrations
- **Files**: `core/codegen/src/cli_gen.rs`, `core/codegen/src/registry.rs`

### Phase 4: Resource discovery
- Add `#[resource_def]` macro (optional — evaluate if enough resources justify it)
- Or: resource definitions derive input patterns from crate dependency graph
- **Files**: `core/ir/src/resource/defs.rs`

### Phase 5: Validation
- Add validation test (like `mock_spec_registration.rs`) that checks:
  - Every tool crate with a `build_*_graph` function has `#[tool_target]`
  - Every `#[tool_target]` has a corresponding `#[testgen_target]`
  - No orphan registrations
- **Files**: `gunbc-app/tests/tool_registration.rs`

---

## What This Enables

| Today | With Unified Model |
|-------|-------------------|
| `all_tools()` is a 360-line manual vec | `all_tools()` collects from inventory — zero manual entries |
| Adding a tool = 10+ touch points | Adding a tool = annotate one function + implement graph |
| Boundaries defined in 2 places | Single source of truth (MockSpec) |
| `GraphBuilderId` string coupling | Function pointers or validated macro strings |
| Resource globs are hardcoded | Resource inputs derived from crate dependencies |
| "Forgot to register" is silent | Validation test catches orphans |

---

## Relationship to Unified Emission

The Emit pattern (`docs/design/unified-emission.md`) describes *how* registered
units produce output. The Registration model describes *how units are discovered*.
They compose:

```
Registration (discovery)  →  Configuration (what to build)  →  Emission (how to render)
    #[tool_target]              ToolDef + MockSpec                Prepare → Format → Write
```

Registration feeds the Emit pipeline: auto-discovered tools flow into the rendering
DAG that produces CLIs, Makefiles, CI YAML, and test suites.

---

## Verification

After each phase:
```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
make testgen   # verify generated output unchanged
make           # verify Makefile targets unchanged
```

Phase 2 additionally: verify generated CLI main.rs files are byte-identical
before/after.

Phase 5 additionally: run validation tests that catch orphan tools/specs.
