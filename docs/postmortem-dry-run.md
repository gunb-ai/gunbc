# Post-Mortem: Dry-Run Implementation Patterns

## Summary

This document analyzes the dry-run implementations across `gunbc-deps`, `gunbc-makegen`, and `gunbc-gistgen`, identifying deficits and proposing improvements.

## Current Implementations

### gunbc-deps (after refactor)

**Pattern**: Operation swapping at graph build time

```rust
let install_op = match (mode, self.dry_run) {
    (Mode::Check, _) => DepOp::FailIfMissing { name },
    (Mode::Upsert, false) => DepOp::InstallCommand { name, cmd },
    (Mode::Upsert, true) => DepOp::PreviewInstall { name, cmd },
};
```

**Deficits**:
- `dry_run: bool` is manually threaded through `build_graph()` → `GraphBuilder` → `add_upsert()`
- Each new external operation requires adding a preview variant
- No infrastructure-level awareness of what touches the world

### gunbc-makegen

**Pattern**: Sink operation swapping at graph build time

```rust
let sink_op = if dry_run {
    MakegenOp::PrintStdout
} else {
    MakegenOp::WriteFile
};
```

**Deficits**:
- `dry_run: bool` passed explicitly to `build_makegen_dag()`
- Manual decision about which operation is the "sink"
- If new external operations are added, must remember to handle them
- No declaration of which nodes touch the world

### gunbc-gistgen

**Pattern**: SubDAG swapping with `BoundaryDeclaration`

```rust
let gist_subdag = match mode {
    UnderstandingMode::Real => build_gist_real(GistgenOp::Gist),
    UnderstandingMode::Mock => build_gist_mock(GistgenOp::Gist),
};
```

With boundary declarations:
```rust
DagMetadata {
    boundary_declarations: vec![
        BoundaryDeclaration {
            node: NodeId("extract_gist_url".into()),
            port: PortName("gist_url".into()),
            external_type: external_types::github_gist(),
        },
    ],
}
```

**Partial solution**: Boundaries are declared in metadata, but swapping is still manual.

**Deficits**:
- Mode is manually propagated via `UnderstandingMode` enum
- The swap logic is in the graph builder, not the infrastructure
- `BoundaryDeclaration` exists but doesn't drive automatic behavior

## Common Deficit: Manual Threading

All three implementations share the same fundamental problem:

1. **Manual mode propagation**: A `dry_run` or mode flag must be explicitly passed through the call chain
2. **Manual sink identification**: The developer must know which operations touch the world
3. **Manual variant creation**: Each external operation needs a preview/mock variant
4. **No automatic detection**: The infrastructure doesn't use boundary declarations to provide dry-run

## Ideal Architecture

The infrastructure should provide dry-run **for free** based on declared boundaries:

### 1. Declare External Boundaries (already exists in gistgen)

```rust
DagMetadata {
    boundary_declarations: vec![
        BoundaryDeclaration {
            node: NodeId("install_node"),
            port: PortName("installed"),
            external_type: ExternalType::Shell,
        },
    ],
}
```

### 2. Operations Declare Their Nature

```rust
pub trait Operation: Executable {
    /// Returns the mock/preview variant of this operation
    fn preview_variant(&self) -> Option<Box<dyn Operation>>;

    /// Does this operation write to the world?
    fn writes_world(&self) -> bool;
}
```

Or simpler — mark operations with an attribute/flag:

```rust
#[derive(Debug, Clone)]
pub enum DepOp {
    CheckCommand { ... },           // Observe
    InstallCommand { ... },         // WritesWorld
    ResolveUpsert { ... },          // Pure
}

impl DepOp {
    pub fn writes_world(&self) -> bool {
        matches!(self, DepOp::InstallCommand { .. })
    }

    pub fn to_preview(&self) -> Self {
        match self {
            DepOp::InstallCommand { name, cmd } => DepOp::PreviewInstall {
                name: name.clone(),
                cmd: *cmd
            },
            other => other.clone(),
        }
    }
}
```

### 3. Infrastructure Swaps Automatically

```rust
pub fn build_graph_with_mode(mode: ExecutionMode) -> Dag<Op> {
    let mut dag = build_graph();

    if mode == ExecutionMode::DryRun {
        // Automatically swap all world-writing operations to their preview variants
        dag.transform_nodes(|node| {
            if node.body.writes_world() {
                node.body = node.body.to_preview();
            }
        });
    }

    dag
}
```

Or at execution time:

```rust
pub fn execute_with_mode(dag: &Dag<Op>, mode: ExecutionMode) -> Result<Log, Error> {
    let executor = Executor::new()
        .with_mode(mode);  // Executor intercepts world-writes in dry-run

    executor.run(dag)
}
```

### 4. No Manual Threading

The call site becomes:

```rust
// Before (manual)
let dag = build_graph(mode, dry_run);

// After (automatic)
let dag = build_graph();
let result = execute(dag, ExecutionMode::DryRun);
```

## Action Items

1. **Extend `BoundaryDeclaration`** to be used by all crates, not just gistgen
2. **Add `writes_world()` method** to operation traits or enums
3. **Add `to_preview()` method** for automatic variant swapping
4. **Move dry-run logic to executor** or provide a `transform_for_dry_run()` helper
5. **Remove manual `dry_run` threading** from all graph builders

## Conclusion

The current implementations work but require manual wiring that's easy to get wrong. The infrastructure already has pieces of the solution (`BoundaryDeclaration`) but doesn't use them to provide automatic dry-run. By moving this logic to the infrastructure layer, we get:

- Consistent dry-run behavior across all tools
- Less per-crate boilerplate
- Harder to forget handling a world-write
- Clear declaration of what touches external systems
