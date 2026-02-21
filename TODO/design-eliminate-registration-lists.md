# Eliminate Registration Lists: Default-Passthrough Resolver + Inventory Discovery

**Status**: PROPOSED
**Date**: 2026-02-21
**Track**: Cleanup — eliminate hardcoded metadata duplication
**Prerequisite**: CL1-CL8 completed (hardcoded lists consolidated)

## Problem Statement

The Rust runtime maintains handwritten registries that duplicate metadata the DSL
compiler already knows. Today (post-CL1-CL8), adding a new DSL module still
requires touching Rust code in up to 4 places:

| What you add | Rust code you must also touch |
|---|---|
| New `.dag` callable (passthrough) | `PASSTHROUGH_CALLABLES` in `resolve.rs` |
| New `.dag` callable (custom behavior) | match arm in `resolve_domain()` + new `Executable` impl |
| New workflow | `TOOL_WORKFLOWS` in `spec_builders.rs` + builder fn |
| New process unit | `default_process_unit_registry()` in `process_registry.rs` |

**Goal**: Make it so that adding a new DSL module/callable/workflow requires
**zero** Rust registration edits. The architecture should make drift structurally
impossible, not just tested-for.

## Root Cause

The `LoweredOp` enum already carries everything the resolver needs:

```
LoweredOp::Callable {
    module: String,                              // "tools.build"
    name: String,                                // "build_all"
    obligation: ObligationCategory,              // None, ServiceTransport*, Resource*
    service_metadata: Option<ServiceCallMetadata>, // REST/Shell specs
    ...
}
```

The resolver *could* dispatch entirely from this data. It doesn't because the
current architecture requires an explicit mapping from every `(module, name)`
pair to a concrete `Executable`. But for ~80% of callables, the `Executable` is
identical: forward inputs to outputs (passthrough).

## Design

### Change 1: Default-passthrough resolver (eliminates PASSTHROUGH_CALLABLES)

**Current**: `resolve_domain()` checks custom resolvers, then `PASSTHROUGH_CALLABLES`,
then returns `unknown_callable` error.

**Proposed**: `resolve_domain()` checks custom resolvers, then service transport,
then resource lifecycle, then **defaults to passthrough for any remaining callable**.

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

**Why this is safe**: The DagLang compiler already validates that every callable
reference resolves to a declared `fn`/`func` in the target module. If a
`LoweredOp::Callable` reaches the resolver, the compiler has proven the callable
exists. Passthrough is correct for any callable without custom side-effect logic
(I/O, resource acquisition, etc.), and those categories are already handled by
steps 1-3.

**What this eliminates**: The entire `PASSTHROUGH_CALLABLES` const (9 modules,
30+ callable names). Adding a new passthrough callable to any DSL module requires
**zero Rust changes**.

### Change 2: Inventory-based custom resolver registration (eliminates match arms)

**Current**: `resolve_domain()` has a `match module { ... }` with 6 arms for
modules with custom `Executable` impls.

**Proposed**: Custom resolvers register themselves via the `inventory` crate
(same pattern as `#[tool_target]`).

```rust
// In gunbc-dag/src/pragma/ops.rs:
inventory::submit!(DomainResolver {
    module: "tools.pragma",
    resolve: resolve_pragma,
});

fn resolve_pragma(node_id: &str, name: &str, outputs: &[Port]) -> Option<Result<DynOp, ResolveError>> {
    match name {
        "render_clippy_toml" => Some(Ok(DynOp::new(PragmaOp::RenderClippy))),
        "pragma" => Some(Ok(DynOp::new(PragmaEntrypointOp))),
        // ...
        _ => None, // fall through to default passthrough
    }
}
```

The resolver collects all registered `DomainResolver` entries at startup:

```rust
fn resolve_custom(
    node_id: &str, module: &str, name: &str, outputs: &[Port],
) -> Option<Result<DynOp, ResolveError>> {
    for resolver in inventory::iter::<DomainResolver>() {
        if resolver.module == module {
            return (resolver.resolve)(node_id, name, outputs);
        }
    }
    None // no custom resolver → fall through to default passthrough
}
```

**What this eliminates**: The `match module { ... }` dispatch in `resolve_domain()`.
Adding a new module with custom behavior requires only the `Executable` impl and
an `inventory::submit!` call in the same file — the resolver never needs editing.

**Note**: Individual callable match arms within custom resolvers (e.g.,
`resolve_pragma`'s 4 arms) are **not** eliminable — they map DSL names to
specific Rust types, which is inherently manual. But they return `None` for
unknown names, falling through to passthrough instead of erroring. This means
even custom-resolver modules can have passthrough callables mixed in.

### Change 3: Inventory-based workflow spec registration (eliminates TOOL_WORKFLOWS)

**Current**: `TOOL_WORKFLOWS` is a 14-entry const array mapping names to builder
functions.

**Proposed**: Each workflow builder registers itself:

```rust
// In gunbc-dag/src/workflow/gist.rs:
inventory::submit!(WorkflowRegistration {
    canonical_name: "gist",
    aliases: &[],
    build: gist_workflow_spec,
});
```

Discovery becomes:

```rust
pub fn all_tool_workflow_names() -> Vec<&'static str> {
    inventory::iter::<WorkflowRegistration>()
        .map(|w| w.canonical_name)
        .collect()
}

pub fn tool_workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    for w in inventory::iter::<WorkflowRegistration>() {
        if w.canonical_name == name || w.aliases.contains(&name) {
            return (w.build)();
        }
    }
    Err(format!("unknown tool workflow: '{name}'"))
}
```

**What this eliminates**: The `TOOL_WORKFLOWS` const. Adding a new workflow
requires the builder function + `inventory::submit!` in the same file.

### Change 4: Derive process units from workflow DAGs (eliminates process_registry)

**Current**: `default_process_unit_registry()` has ~80 manual `pu(...)` entries
mapping process units to resource claims.

**Proposed**: When a `WorkflowSpec` is built, it already contains a `Dag` with
named nodes. Process units can be derived from the DAG topology:

```rust
impl WorkflowSpec {
    /// Derive process units from this workflow's DAG nodes.
    fn derive_process_units(&self) -> Vec<ProcessUnitSpec> {
        self.dag.nodes.iter().map(|node| {
            let claims = derive_claims_from_node(node); // see below
            pu(&self.name, &node.id.0, claims)
        }).collect()
    }
}
```

Resource claims can be inferred from node metadata:
- Nodes with `file:write` resource ports → `UnitClaim::write("file:workspace")`
- Nodes with `file:read` resource ports → `UnitClaim::read("file:workspace")`
- Nodes with `network` resource ports → `UnitClaim::write("network:*")`
- Nodes with `tool:cargo` ports → `UnitClaim::read("tool:cargo")`
- Pure nodes (no resource ports) → `vec![]`

**What this eliminates**: The entire hand-maintained process unit registry.
Adding a new workflow node automatically creates a process unit with correct
resource claims derived from the DAG's resource wiring.

**Fallback**: If claim inference isn't precise enough for some nodes, allow
explicit `#[process_claim(...)]` annotations in the DSL or on the builder.

### Change 5: Structural test assertions (eliminates brittle counts)

**Current**: 11+ tests assert exact node counts: `assert_eq!(dag.nodes.len(), 9)`

**Proposed**: Replace with structural assertions:

```rust
// Instead of: assert_eq!(spec.dag.nodes.len(), 9);
// Use:
assert!(spec.dag.has_node("gist.branch_resolution"));
assert!(spec.dag.has_node("gist.credential_resolve"));
assert!(spec.dag.has_edge_between("gist.branch_resolution", "gist.gist_create"));
// Or for pure structure validation:
assert!(spec.dag.is_connected(), "workflow DAG must be connected");
assert!(spec.dag.has_single_sink(), "workflow DAG must have one terminal node");
```

This validates the DAG's **shape** rather than its **size**, so adding a node
(e.g., a new intermediate step) doesn't break unrelated tests.

## Remaining hardcoded lists (not eliminable)

Some lists are inherently manual because they map DSL concepts to Rust-specific
behavior that can't be auto-derived:

| List | Why it stays | Mitigation |
|---|---|---|
| `WorkspaceBinary` enum (12 entries) | Maps binary names to Cargo invocation metadata. Binaries are a build system concept, not a DSL concept. | Already uses single-table macro. Consider deriving from `Cargo.toml` `[[bin]]` in a build script. |
| `MANUAL_TOOL_DEFS` (2 entries) | `pragma` needs custom `Executable`; `build` has non-standard short_name. | Already documented (CL7). Shrinks as tools move to standard path. |
| Custom `Executable` impls (5 modules) | By definition, custom behavior requires custom code. | Inventory registration (Change 2) keeps them co-located. |
| `STANDARD_SYMBOLS` (40 entries) | UI symbols are a presentation concern, not DSL metadata. | Use `const` count assertion: `const _: [(); SYMBOLS.len()] = [(); 40];` |
| `FORBIDDEN_CALLS` / `ALLOWED_FILES` (guardrails) | Architectural constraints, not DSL metadata. | Fine as-is; changes are intentional. |

## Additional findings from audit

Beyond the CL1-CL8 items and the changes above, the scanner found:

| Finding | Location | Recommendation |
|---|---|---|
| Mock spec hardcoded paths (`"tools.bootstrap::bootstrap"`) | `ci/graph_mock.rs:76-88` | Derive from `LoweredOp` node IDs in the compiled CI DAG |
| Hardcoded command counts in workflow unit tests | `unit_commands.rs:541,554` | Same as Change 5: structural assertions |
| Makefile workflow name dispatch | `makegen/render.rs:230+` | Already uses generated registry; verify coverage |

## Implementation order

| Phase | Changes | Impact | Size |
|---|---|---|---|
| **Phase 1** | Change 1 (default passthrough) | Eliminates `PASSTHROUGH_CALLABLES`. Zero-touch for new passthrough callables. | S |
| **Phase 2** | Change 5 (structural test assertions) | Eliminates 13+ brittle count assertions. | S |
| **Phase 3** | Change 2 (inventory custom resolvers) | Eliminates resolver `match module` dispatch. Co-locates ops with registration. | M |
| **Phase 4** | Change 3 (inventory workflow specs) | Eliminates `TOOL_WORKFLOWS`. Co-locates builders with registration. | M |
| **Phase 5** | Change 4 (derived process units) | Eliminates process registry. Requires claim inference logic. | L |

Phase 1 is the highest value: it makes the most common operation (adding a DSL
callable) require zero Rust changes. Phases 2-4 are incremental wins using the
proven inventory pattern. Phase 5 is the most complex but eliminates the largest
registry.

## Success criteria

After all phases:
- Adding a new `.dag` module with only passthrough callables: **0 Rust files touched**
- Adding a new `.dag` module with custom behavior: **1 Rust file** (the `Executable` impl + inventory registration, co-located)
- Adding a new workflow: **1 Rust file** (the builder + inventory registration, co-located)
- Adding a new process unit to a workflow: **0 Rust files** (derived from DAG)
- Adding a new resource to `std/resources.dag`: **0 Rust files** (already done in CL8)
