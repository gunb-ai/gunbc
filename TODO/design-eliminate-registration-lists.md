# Eliminate Registration Lists: Close the DSL→Runtime Gap

**Status**: PROPOSED
**Date**: 2026-02-21
**Track**: Cleanup — eliminate hardcoded metadata duplication
**Prerequisite**: CL1-CL8 completed (hardcoded lists consolidated)

## Problem Statement

The Rust runtime maintains handwritten registries that duplicate metadata the DSL
compiler already knows. Today (post-CL1-CL8), adding a new DSL module still
requires touching Rust code in up to 4 places.

**Goal**: Make it so that adding DSL modules/callables/workflows requires **zero**
Rust registration edits. Drift should be structurally impossible.

## Why can't we "just write DSL" for all of this?

Short answer: **we almost can.** The remaining Rust exists for three reasons,
only one of which is fundamental.

### What's already DSL-only

**Every tool graph is 100% DSL-compiled at runtime.** There are zero hand-coded
Rust DAGs. When `build_pragma_graph()` runs, it calls:

```
dsl/tools/pragma.dag → daglang_driver::compile → Dag<LoweredOp> → resolve → Dag<DynOp>
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

#### Reason 1: Leaf-node function bodies (temporary — DSL doesn't implement yet)

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
This is a language evolution, not an architecture change. Each function body
migrated from Rust to DSL eliminates one custom `Executable` impl.

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
| Leaf-node Rust impls | Fundamental (for now) | 0 (these don't create registries) | Not a registry problem |

The registries are caused by the first two. Neither is fundamental.

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

This closes the loop: DSL annotations → compiler derivation → process claims.
No Rust registry needed.

## Design: Phase 3 — DSL function bodies (long-term)

### Change 6: Expression-level DSL support

Add basic expression support to the DSL language:
- String interpolation / templating
- List operations (map, filter, join)
- Arithmetic and comparison
- Pattern matching

This allows migrating the 5 custom `Executable` modules to pure DSL:

| Module | Current Rust ops | DSL-expressible? |
|---|---|---|
| `tools.pragma` | String rendering from config | Yes, with string interpolation |
| `tools.makegen` | Registry load + Makefile render | Partially — needs data source for registry |
| `tools.bootstrap` | Shell request prep + output parsing | Yes, with service transport patterns |
| `tools.codegen` | Build-time file existence checks | Partially — needs build metadata access |
| `tools.infra` | Filter/count/format | **Already expressed in DSL** — Rust version is redundant |

After Phase 3, the only Rust code is the compiler itself, the executor, and
the transport adapters. Everything else is DSL.

## Remaining lists (inherently non-DSL)

| List | Why it stays |
|---|---|
| `WorkspaceBinary` (12 entries) | Build system concept (Cargo binary names), not DSL metadata |
| `STANDARD_SYMBOLS` (40 entries) | UI/presentation concern |
| `FORBIDDEN_CALLS` (guardrails) | Architectural constraint, not domain metadata |

These are fine — they don't duplicate DSL metadata.

## Implementation order

| Phase | Changes | Eliminates | Size |
|---|---|---|---|
| **1a** | Default passthrough (Change 1) | `PASSTHROUGH_CALLABLES` | S |
| **1b** | Structural tests (Change 3) | 13+ brittle count assertions | S |
| **1c** | Inventory resolvers (Change 2) | `match module` dispatch | M |
| **2a** | Workflow DSL migration (Change 4) | `TOOL_WORKFLOWS` + builder fns | L |
| **2b** | Derived claims (Change 5) | `process_registry` (80+ entries) | M |
| **3** | DSL expressions (Change 6) | Custom `Executable` impls (5 modules) | XL |

Phase 1a is the highest-value immediate win. Phase 3 is the endgame where
"just write DSL" becomes literally true.

## Success criteria

**After Phase 1** (Rust-only, no DSL changes):
- New passthrough callable: **0 Rust files**
- New custom-behavior module: **1 file** (impl + inventory, co-located)
- New resource: **0 Rust files** (already done)

**After Phase 2** (workflow migration):
- New workflow: **1 DSL file** (no Rust)
- New process unit: **0 files** (derived from DSL annotations)

**After Phase 3** (DSL expressions):
- New tool with only pure computation: **1 DSL file** (no Rust at all)
- Rust code only needed for: new transport adapters, new resource handle types
