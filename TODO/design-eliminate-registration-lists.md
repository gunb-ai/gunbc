# Eliminate Registration Lists: Close the DSL-Runtime Gap

**Status**: PROPOSED
**Date**: 2026-02-21
**Track**: Cleanup — eliminate hardcoded metadata duplication
**Prerequisite**: CL1-CL8 completed (hardcoded lists consolidated)

## Vision

The DSL is the only programming language for tool, workflow, and pipeline logic.
Rust is infrastructure — compiler, executor, transport adapters — not a fallback
for "complex" logic. There is no escape hatch. If something can't be written in
DSL today, that's a missing DSL feature to be fixed, not a reason to write Rust.

## Problem Statement

The Rust runtime maintains handwritten registries that duplicate metadata the DSL
compiler already knows. Today (post-CL1-CL8), adding a new DSL module still
requires touching Rust code in up to 4 places. Worse, 5 modules implement their
function bodies in Rust — pure computations (string rendering, list filtering,
JSON construction) that belong in DSL but leak into Rust because the DSL lacks
expression-level primitives.

**Goal**: Make it so that adding or modifying any tool, callable, workflow, or
configuration requires **only DSL changes**. Zero Rust edits. Drift is
structurally impossible. Rust as escape hatch is eliminated.

## Why can't we "just write DSL" for all of this?

Short answer: **we almost can.** The remaining Rust exists for three reasons,
none of which are fundamental.

### What's already DSL-only

**Every tool graph is 100% DSL-compiled at runtime.** There are zero hand-coded
Rust DAGs. When `build_pragma_graph()` runs, it calls:

```
dsl/tools/pragma.dag -> daglang_driver::compile -> Dag<LoweredOp> -> resolve -> Dag<DynOp>
```

All graph structure, wiring, and orchestration comes from the DSL. The `emit`
phase can even generate complete Rust/Go/C source code from the compiled DAG.
The DSL already expresses:
- Module dependencies and imports
- Function signatures (inputs, outputs, types)
- Graph topology (data flow, parallelism, stages)
- Resource annotations (`@file(READ/WRITE)`, `@hermetic`, `@mock_response`)
- Service protocol specs (REST endpoints, shell commands, field mappings)
- Pipeline stage ordering

### The three reasons Rust code still exists

#### Reason 1: Leaf-node function bodies (missing DSL feature)

The DSL declares functions like `fn render_clippy_toml(directives) -> String`
but the **body** is implemented in Rust (`PragmaOp::RenderClippy`). These are
pure computations: string templating, list filtering, JSON serialization. The
DSL has the type system for this, but no expression language for function bodies.

This is a **missing DSL feature**, not an architectural limitation. Evidence:
- `InfraToolOp` duplicates logic already expressed in `dsl/tools/infra.dag`
  (match/filter/count) — the DSL CAN express it, but Rust reimplements it
- `PragmaOp` renders strings from config — expressible with string interpolation
- `MakegenOp::LoadRegistry` serializes a Rust struct — needs a DSL-side data
  source or FFI mechanism

**Fix**: Add expression-level DSL support (string ops, list ops, arithmetic).
See "DSL Language Features Required" below for the complete inventory.

#### Reason 2: The resolver (unnecessary — compiler already has the information)

`resolve_lowered_dag()` maps `LoweredOp` (compiler output) to `DynOp`
(executable). The `LoweredOp` already carries module, name, obligation category,
and service metadata — everything needed to route. But the resolver maintains
its own copy of this routing table:

| Registry | What it duplicates |
|---|---|
| `PASSTHROUGH_CALLABLES` (30+ entries) | "These callables exist and are passthrough" — compiler already validated this |
| `resolve_domain()` match arms (6 modules) | "These modules have custom ops" — could be inventory-discovered |
| `resolve_std_resources()` name match | "These resources exist" — compiler knows from `std/resources.dag` |

This is **entirely eliminable** without DSL changes. The compiler proves
callables exist; the resolver should trust that proof. See Changes 1-2 below.

#### Reason 3: Workflow specs are Rust-constructed DAGs (should be DSL pipelines)

The workflow builders (`gist_workflow_spec`, `bootstrap_workflow_spec`, etc.)
construct `Dag<WorkflowUnit>` objects in Rust using `dag.add_node()`/
`dag.add_edge()`. This is the same thing the DSL does — defining graph topology
— but bypassing the compiler entirely.

Meanwhile, `pipelines/ci.dag` and `pipelines/sdlc.dag` already express
pipelines in DSL that the compiler handles. The 12 remaining Rust-constructed
workflows exist because they were written before the DSL pipeline feature was
mature enough.

This is a **migration gap**, not a limitation. The DSL's `pipeline` construct
can express everything the Rust builders do. Evidence: `pipelines/ci.dag` is
the most complex workflow and it's fully DSL.

**Fix**: Migrate workflow builders to `dsl/pipelines/*.dag` files. The process
unit claims (currently in `process_registry.rs`) can be derived from the DSL's
`@file(READ/WRITE)` annotations, which already exist but aren't extracted.

### Summary: What blocks "just write DSL"

| Blocker | Category | How many registries it causes | Fix |
|---|---|---|---|
| Resolver doesn't trust compiler | Architecture gap | 3 (PASSTHROUGH_CALLABLES, match arms, resource names) | Default-passthrough + inventory (this design) |
| Workflow specs in Rust | Migration gap | 2 (TOOL_WORKFLOWS, process_registry) | Migrate to DSL pipeline definitions |
| No function body expressions | Missing DSL feature | 1 (custom Executable impls) | DSL expression language |

None of these are fundamental. All three are fixable.

## DSL Language Features Required

Auditing every custom `Executable` impl (22 op variants across 5 modules)
reveals the exact language primitives the DSL needs to eliminate Rust as an
escape hatch. Every computation in these modules is pure — no I/O, no FFI,
no unsafe — just data transformation between transport boundaries.

### Feature 1: String interpolation and templating

**Used by**: PragmaOp (3 variants), BootstrapOp (4), CodegenOp (5), BuildOp (7)

The most common pattern. Rust uses `format!()`, `write!()`, and string
concatenation to build output strings from structured inputs.

```
// Current Rust (pragma/ops.rs):
format!("# {}\n{}", header.render(), body)

// DSL equivalent:
let result = "# ${header.render()}\n${body}"
```

**Required primitives**:
- `"${expr}"` — interpolation within string literals
- Multi-line string literals (template blocks)
- String concatenation (`+` or implicit adjacency)

### Feature 2: String methods

**Used by**: BootstrapOp (parsing shell output), CodegenOp (path normalization)

```
// Current Rust:
line.trim()
line.strip_prefix("crates/")
path.replace('\\', "/")
output.lines()
name.contains('/')
text.is_empty()
text.ends_with('/')
```

**Required primitives**:
- `.trim()`, `.lines()`, `.split(sep)`
- `.strip_prefix(s)`, `.strip_suffix(s)`
- `.replace(old, new)`
- `.contains(s)`, `.starts_with(s)`, `.ends_with(s)`
- `.is_empty()`, `.len()`

### Feature 3: List operations

**Used by**: PragmaOp (allowlist rendering), MakegenOp (registry serialization),
BootstrapOp (crate name extraction), CodegenOp (path verification)

```
// Current Rust:
rules.iter().map(|r| r.render()).collect::<Vec<_>>()
crate_names.sort()
patterns.dedup()
expected_paths.iter().all(|p| found.contains(p))
```

**Required primitives**:
- `.map(fn)`, `.filter(fn)` — transform/select
- `.sort()`, `.dedup()` — ordering
- `.join(sep)` — list to string
- `.any(fn)`, `.all(fn)` — predicate testing
- `.len()` — count
- `.push(item)`, list literal `[a, b, c]`
- `.contains(item)` — membership

### Feature 4: Pattern matching and conditionals

**Used by**: All 5 modules

```
// Current Rust:
match response {
    TransportResponse::Shell(shell) => ...,
    _ => Err(...)
}
if build_success && !skip_tests { ... }
```

**Required primitives**:
- `match expr { pattern => body, ... }` — exhaustive matching
- `if cond { a } else { b }` — conditional expressions
- `let ... = ...` — binding with destructuring
- Boolean operators: `&&`, `||`, `!`

### Feature 5: Integer arithmetic and comparison

**Used by**: CodegenOp (manifest freshness), BuildOp (exit code checking),
MakegenOp (counting)

```
// Current Rust:
testgen_targets.len()
response.exit_code == 0
```

**Required primitives**:
- `+`, `-`, `*`, `/`, `%` — arithmetic
- `==`, `!=`, `<`, `>`, `<=`, `>=` — comparison
- Integer literals

### Feature 6: Structured data construction

**Used by**: MakegenOp (JSON building for Makefile rendering)

```
// Current Rust:
serde_json::json!({
    "tools": tools.iter().map(|t| json!({"name": t.short_name})).collect::<Vec<_>>(),
    "testgen_targets": targets,
})
```

**Required primitives**:
- Object literals: `{ key: value, ... }`
- Nested construction: objects containing lists containing objects
- This is close to what the DSL already has for `@mock_response` blocks

### Feature 7: DSL-accessible data sources

**Used by**: MakegenOp, BootstrapOp, CodegenOp, PragmaOp

Currently, pure configuration data is embedded in Rust source files and accessed
via Rust API calls. This data has no reason to live in Rust — it's declarative
configuration that belongs in DSL data files.

**Data currently hiding in Rust**:

| Data | Location | Nature |
|---|---|---|
| Clippy allowlist rules (8 rules) | `policy/pragma.rs` | Static config: crate selectors, suffix paths, rationales |
| Dead code allow rules (5 rules) | `policy/pragma.rs` | Static config: crate names, relative paths |
| Pragma allow lints (3 lints) | `policy/pragma.rs` | Static list of lint IDs |
| Crate policies (1 entry) | `policy/pragma.rs` | Static config: crate name + policy flags |
| Tool registry (12 tools) | `gunbc-tool-registry` | Static config: tool names, packages, binaries |
| Testgen specs | `gunbc-testgen-registry` | Static config: test module names, DAG paths |
| Build config | `gunbc-makegen` | Static config: cargo commands, feature flags |
| Gitignore categories (14 categories) | `gunbc-makegen` | Static config: path patterns per category |
| Codegen path templates | `codegen/ops.rs` | Static config: `target/codegen/bin`, stamp paths |
| Workspace layout | `gunbc-ir` | Derivable from DSL module structure |

**Required mechanism**:
- `@data` or `data` blocks in DSL for declaring static configuration
- `import data from "config/pragma-policy.dag"` — DSL-to-DSL data imports
- The compiler resolves data references at compile time, not runtime

This eliminates the last category of "I need Rust because the data lives there."
The data moves to DSL, the logic that consumes it is already expressible with
Features 1-6 above.

### Coverage matrix: Features vs. modules

| Module | Variants | F1 String | F2 Methods | F3 Lists | F4 Match | F5 Arith | F6 Data | F7 Sources |
|---|---|---|---|---|---|---|---|---|
| **pragma** | 3 | YES | YES | YES | YES | - | - | YES |
| **makegen** | 3 | YES | - | YES | YES | YES | YES | YES |
| **bootstrap** | 4 | YES | YES | YES | YES | - | - | YES |
| **codegen** | 5 | YES | YES | YES | YES | YES | - | YES |
| **build** | 7 | YES | - | - | YES | YES | - | - |

Every module is fully covered by these 7 features. No module requires anything
beyond basic data transformation primitives.

## Design: Phase 1 — Resolver trusts compiler (immediate, no DSL changes)

### Change 1: Default-passthrough resolver (eliminates PASSTHROUGH_CALLABLES)

**Current**: `resolve_domain()` checks custom resolvers, then
`PASSTHROUGH_CALLABLES`, then returns `unknown_callable` error.

**Proposed**: Default to passthrough for any callable the compiler validated.

```rust
fn resolve_domain(
    node_id: &str,
    module: &str,
    name: &str,
    outputs: &[Port],
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // 1. Custom resolvers (modules with non-passthrough Executable impls).
    if let Some(result) = resolve_custom(node_id, module, name, outputs) {
        return result;
    }
    // 2. Service/workspace transport (generic, spec-driven).
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(node_id, module, name, service_metadata);
    }
    // 3. Resource lifecycle (generic, name-driven).
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // 4. Default: passthrough. The compiler validated this callable exists.
    //    No list needed — if it compiled, it's resolvable.
    Ok(DynOp::new(PassthroughOp {
        output_port_names: declared_output_names(outputs),
    }))
}
```

**Why this is safe**: The DagLang compiler validates every callable reference
resolves to a declared `fn`/`func`. If `LoweredOp::Callable` reaches the
resolver, the callable exists. Passthrough is correct for any callable without
custom side-effect logic, and those are already handled by steps 1-3.

**What this eliminates**: `PASSTHROUGH_CALLABLES` (9 modules, 30+ names).
Adding a new passthrough callable requires **zero Rust changes**.

### Change 2: Inventory-based custom resolver registration (eliminates match arms)

Custom resolvers register themselves co-located with their `Executable` impls:

```rust
// In gunbc-dag/src/pragma/ops.rs:
inventory::submit!(DomainResolver {
    module: "tools.pragma",
    resolve: resolve_pragma,
});

fn resolve_pragma(node_id: &str, name: &str, outputs: &[Port])
    -> Option<Result<DynOp, ResolveError>>
{
    match name {
        "render_clippy_toml" => Some(Ok(DynOp::new(PragmaOp::RenderClippy))),
        "pragma" => Some(Ok(DynOp::new(PragmaEntrypointOp))),
        _ => None, // fall through to default passthrough
    }
}
```

**What this eliminates**: The `match module { ... }` dispatch. Adding a custom
module means adding the impl + registration in one file — `resolve.rs` never
needs editing.

Returning `None` for unrecognized callables is the key: even modules with custom
ops can have passthrough callables mixed in. No need to enumerate every callable.

### Change 3: Structural test assertions (eliminates brittle counts)

Replace `assert_eq!(dag.nodes.len(), 9)` (11+ instances) with:

```rust
assert!(spec.dag.has_node("gist.branch_resolution"));
assert!(spec.dag.is_connected());
assert!(spec.dag.has_single_sink());
```

## Design: Phase 2 — Workflows migrate to DSL (medium-term)

### Change 4: Express workflows as DSL pipelines

The 12 Rust-constructed workflow specs should become `dsl/workflows/*.dag` files,
compiled and resolved exactly like `pipelines/ci.dag` already is. This
eliminates:
- `TOOL_WORKFLOWS` registry (14 entries)
- `default_process_unit_registry()` (~80 entries)
- All `*_workflow_spec()` builder functions

### Change 5: Derive process unit claims from DSL annotations

The DSL already has `@file(READ, "{path}")` and `@file(WRITE, "{path}")`
annotations. The compiler's derivation phase (`DerivedArtifacts`) already
extracts `ResourceUsage` per node. The process unit claims can be generated
from this:

```
DSL:     @file(WRITE, "clippy.toml")
Derived: ResourceUsage { resource: "Filesystem", usage: "Write" }
Claim:   UnitClaim::write("file:workspace")
```

This closes the loop: DSL annotations -> compiler derivation -> process claims.
No Rust registry needed.

## Design: Phase 3 — DSL expression language (eliminates Rust escape hatch)

This is the core goal, not an optional long-term aspiration. Phases 1 and 2
remove registration boilerplate; Phase 3 eliminates the reason Rust is used
for business logic at all.

### Change 6: Expression-level DSL support

Add the 7 feature categories documented in "DSL Language Features Required"
above. This is a language evolution within the existing DagLang compiler —
the type system, module system, and graph semantics are unchanged.

### Change 7: Migrate configuration data to DSL data sources

Move all static configuration currently embedded in Rust source files into
DSL data files:

```
dsl/config/pragma-policy.dag    -- clippy rules, lint policies, crate policies
dsl/config/tool-registry.dag    -- tool names, packages, binaries
dsl/config/build.dag            -- cargo commands, feature flags
dsl/config/codegen-paths.dag    -- path templates, stamp files
```

The compiler resolves these at compile time. The data is version-controlled,
diffable, and requires zero Rust knowledge to modify.

### Change 8: Migrate custom Executable impls to DSL function bodies

With Features 1-7 available, each custom module migrates from Rust to DSL:

| Module | Rust ops to migrate | What replaces them |
|---|---|---|
| `tools.infra` | Filter/count/format (5 ops) | **Delete** — already redundant with `dsl/tools/infra.dag` |
| `tools.build` | Boolean cascade + string summary (7 ops) | DSL conditionals + string interpolation |
| `tools.pragma` | Config rendering (3 ops) | DSL string interpolation + data imports |
| `tools.bootstrap` | Shell output parsing + crate extraction (4 ops) | DSL string methods + list ops |
| `tools.codegen` | Path checking + manifest freshness (5 ops) | DSL string methods + conditionals + data sources |
| `tools.makegen` | Registry load + JSON construction (3 ops) | DSL data sources + structured data literals |

After this, zero `Executable` impls exist outside the compiler/executor
infrastructure. The `resolve_custom()` path in Change 1 becomes empty.
The inventory registrations from Change 2 disappear. The resolver reduces to:

```rust
fn resolve_domain(...) -> Result<DynOp, ResolveError> {
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(...);
    }
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // Everything is passthrough. The DSL handles all logic.
    Ok(DynOp::new(PassthroughOp { ... }))
}
```

## What stays in Rust (by design, not by escape hatch)

These are infrastructure concerns, not business logic. They don't duplicate
DSL metadata and don't grow when tools/workflows are added:

| Component | Why it's Rust | Grows when... |
|---|---|---|
| DagLang compiler | Language implementation | New DSL syntax is added |
| DAG executor | Runtime engine | New execution semantics are added |
| Transport adapters (Shell, REST, Filesystem) | System boundary / FFI | New transport protocols are added |
| Resource handle types | Capability system | New resource kinds are added |
| `WorkspaceBinary` (12 entries) | Build system (Cargo binary names) | New crate binaries are added |
| `STANDARD_SYMBOLS` (40 entries) | UI/presentation | New display symbols are added |
| `FORBIDDEN_CALLS` (guardrails) | Architectural constraint | New safety rules are added |

The key distinction: **infrastructure grows with the platform, not with the
domain.** Adding a new tool, workflow, or pipeline should never require touching
any of these.

## Implementation order

| Phase | Changes | Eliminates | Size |
|---|---|---|---|
| **1a** | Default passthrough (Change 1) | `PASSTHROUGH_CALLABLES` | S |
| **1b** | Structural tests (Change 3) | 13+ brittle count assertions | S |
| **1c** | Inventory resolvers (Change 2) | `match module` dispatch | M |
| **2a** | Workflow DSL migration (Change 4) | `TOOL_WORKFLOWS` + builder fns | L |
| **2b** | Derived claims (Change 5) | `process_registry` (80+ entries) | M |
| **3a** | DSL expression language (Change 6) | The escape hatch itself | L |
| **3b** | Data source migration (Change 7) | Config data in Rust files | M |
| **3c** | Custom op migration (Change 8) | All 5 custom `Executable` modules (27 ops) | L |

Phase 1a is the highest-value immediate win. Phase 3 is where the vision
is realized: Rust is infrastructure, DSL is everything else.

## Success criteria

**After Phase 1** (Rust-only, no DSL changes):
- New passthrough callable: **0 Rust files**
- New custom-behavior module: **1 file** (impl + inventory, co-located)
- New resource: **0 Rust files** (already done)

**After Phase 2** (workflow migration):
- New workflow: **1 DSL file** (no Rust)
- New process unit: **0 files** (derived from DSL annotations)

**After Phase 3** (DSL expressions + data migration):
- New tool of any complexity: **1 DSL file** (no Rust at all)
- New configuration/policy: **1 DSL data file** (no Rust at all)
- Rust code only needed for: new transport adapters, new resource handle types
- Custom `Executable` impls: **0** (down from 27 op variants across 5 modules)
- The concept of "escape to Rust" no longer exists
