# Graph-Level Transport Exposure

**Status**: Draft
**Date**: 2026-01-30

## Goal

Make all I/O visible as explicit graph nodes, not hidden inside opaque operations. This enables:
- Full DryRun interception at the graph level (not just transport layer)
- Complete observability of I/O in DAG visualization
- Proper use of existing pattern builders (UpsertBuilder, LoopBuilder, TransactionBuilder)

## Current State

After the transport compliance refactor, all I/O goes through `execute_transport()` from `lib/transport`. However, this I/O is hidden **inside** opaque nodes:

```
Current (I/O hidden inside opaque):
┌─────────────────────────────────────┐
│ CIOp::Build (opaque)                │
│                                     │
│   execute_transport(ShellRequest)   │  <-- I/O hidden here
│                                     │
└─────────────────────────────────────┘
```

The executor's `is_transport_execution_node()` checks if a node has `TransportRequest` inputs. Since the opaque nodes don't have `TransportRequest` inputs, they won't be intercepted in DryRun mode at the graph level.

## Target State

I/O should be visible as explicit graph nodes:

```
Target (I/O explicit in graph):
┌────────────────────┐   ┌─────────────────────┐   ┌─────────────────┐
│ PrepareShellOp     │──▶│ TransportOps::Exec  │──▶│ ParseOutput     │
│ ("cargo build")    │   │ (interceptable!)    │   │ (pure)          │
│ (pure)             │   │                     │   │                 │
└────────────────────┘   └─────────────────────┘   └─────────────────┘
                                   ↑
                                   └── DryRun intercepts HERE
```

## Benefits

1. **Full DryRun interception** - Transport nodes are intercepted, mocked values flow through graph
2. **Observability** - I/O visible in DAG visualization tools
3. **Composability** - Can wrap transport nodes in Retry, Circuit Breaker, etc.
4. **Pattern alignment** - Matches existing Prepare* + Execute pattern

## Design

### Tool-Specific Refactors

#### CI Tool

Current:
```rust
// ops.rs - I/O hidden
fn run_config_command(command: &[&str]) -> Result<...> {
    let request = TransportRequest::Shell(...);
    execute_transport(&request)?  // Hidden I/O
}
```

Target:
```rust
// graph.rs - I/O explicit as nodes
let prepare_build = builder.add_node(..., CIOp::PrepareBuild)?;  // Pure: outputs TransportRequest
let execute = builder.add_node(..., TransportOps::Execute)?;      // Interceptable
let parse = builder.add_node(..., CIOp::ParseBuildResult)?;       // Pure

builder.add_edge(prepare_build.out("request"), execute.in_port("request"))?;
builder.add_edge(execute.out("stdout"), parse.in_port("stdout"))?;
```

#### Gist Tool (LoopBuilder Pattern)

Current:
```rust
// graph.rs - reads multiple files inside opaque node
GistGraphOp::ReadFiles  // I/O hidden, reads all files internally
```

Target:
```rust
// Use LoopBuilder for reading multiple files
let read_loop = LoopBuilder::new("read_files")
    .with_input("paths", "StrList", Cardinality::ZeroOrMore)
    .with_body(build_single_file_read_dag())  // PrepareFileReadOp -> Execute
    .with_output("contents", "StrList")
    .build();
```

#### Deps Tool (UpsertBuilder Pattern)

Current:
```rust
// ops.rs - install check hidden
installer.is_installed(verify_cmd)  // Command::new hidden
```

Target:
```rust
// Use UpsertBuilder for idempotent install
let install_tool = UpsertBuilder::new("install_tool")
    .with_check(DepsOp::PrepareVerify)     // PrepareShellOp(verify) -> Execute -> exit_code check
    .with_create(DepsOp::PrepareInstall)   // PrepareShellOp(install) -> Execute
    .with_resolve(DepsOp::PrepareVerify)   // Verify again
    .build();
```

### New Pure Ops Needed

Each tool needs pure "Prepare" ops that output `TransportRequest`:

| Tool | Current Opaque | Target Pure Ops |
|------|---------------|-----------------|
| ci | `CIOp::Build` | `CIOp::PrepareBuild`, `CIOp::ParseBuildResult` |
| ci | `CIOp::Test` | `CIOp::PrepareTest`, `CIOp::ParseTestResult` |
| ci | `CIOp::Lint` | Use `cli::CLIPPY` upsert directly |
| gist | `GistGraphOp::ListFiles` | `PrepareShellOp("git ls-files")` + parse |
| gist | `GistGraphOp::ReadFiles` | `LoopBuilder` with `PrepareFileReadOp` |
| deps | `DepsOp::LoadManifest` | `PrepareFileReadOp` + `ParseOp::Toml` |
| deps | `DepsOp::ExecuteInstalls` | `UpsertBuilder` per dependency |
| buck2 | `Buck2Op::ParseCargoToml` | `PrepareFileReadOp` + `ParseOp::Toml` |

## Tasks

### Phase 1: CI Tool
- [ ] Create `CIOp::PrepareBuild` - pure op that outputs `TransportRequest::Shell`
- [ ] Create `CIOp::PrepareBuildResult` - pure op that parses stdout/stderr/exit_code
- [ ] Update `build_ci_graph()` to expose transport nodes
- [ ] Same for Test, Lint ops

### Phase 2: Gist Tool (LoopBuilder)
- [ ] Create body DAG for single file read: `PrepareFileReadOp` -> `TransportOps::Execute`
- [ ] Use `LoopBuilder` for `ReadFiles` node
- [ ] Expose `ListFiles` as `PrepareShellOp` -> `Execute` -> parse

### Phase 3: Deps Tool (UpsertBuilder)
- [ ] Create `DepsOp::PrepareVerify` - pure op that outputs `TransportRequest::Shell`
- [ ] Create `DepsOp::PrepareInstall` - pure op that outputs `TransportRequest::Shell`
- [ ] Use `UpsertBuilder` for each dependency
- [ ] Expose manifest load as `PrepareFileReadOp` -> `Execute` -> `ParseOp::Toml`

### Phase 4: Buck2 Tool
- [ ] Expose `ParseCargoToml` as `PrepareFileReadOp` -> `Execute` -> `ParseOp::Toml`
- [ ] Expose `.exists()` checks as `PrepareFileExistsOp` -> `Execute`

### Phase 5: Bootstrap Tool
- [ ] Expose `ScanWorkspace` as `PrepareShellOp("find")` -> `Execute` -> parse

## Notes

### Why Incremental?

Each tool can be migrated independently. The current state (I/O through transport internally) is functional and tested. This graph-level exposure is an enhancement for better observability and interception.

### Testing Strategy

For each migrated tool:
1. Verify DryRun mode intercepts transport nodes
2. Verify mocked values flow through correctly
3. Verify real execution still works

### Related Work

- `TODO/opaque-op-tool-mismatch.md` - Related issue about ops hiding tool dependencies
- `TODONE/transport-compliance.md` - Prerequisite work (completed)
