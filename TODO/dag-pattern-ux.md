# DAG Pattern UX Improvements

**Status**: Design
**Date**: 2026-01-30

## Goal

Reduce DAG construction boilerplate and improve consistency across tools by providing higher-level builder patterns for common operations.

## Problem Statement

Building DAGs in gunbc is structurally sound but verbose. The "correct" patterns require significant boilerplate, which:

1. **Increases cognitive load** — Authors must remember the 3-node transport chain pattern
2. **Invites inconsistency** — Each tool defines slightly different PrepareXxx ops
3. **Hides intent** — The actual operation is buried in wiring code

The goal is to make the correct patterns as easy to use as the incorrect (hidden I/O) patterns would be.

## Current Pain Points

### 1. Transport Chain Boilerplate

Every transport operation requires 3 nodes with manual wiring:

```rust
// Current: ~15-20 lines per transport chain
let prepare_node = Node::opaque(
    "prepare-list-files",
    vec![port("repo_path", "String")],
    vec![port("request", "TransportRequest")],
    GistGraphOp::PrepareListFiles,
);
let prepare = builder.add_node(prepare_node)?;

let execute_node = Node::opaque(
    "execute-list-files",
    vec![port("request", "TransportRequest")],
    vec![port("response", "TransportResponse")],
    GistGraphOp::Transport(TransportOps::Execute),
);
let execute = builder.add_node_after(execute_node, &prepare)?;

let parse_node = Node::opaque(
    "parse-list-files",
    vec![port("response", "TransportResponse")],
    vec![port("files", "StrList")],
    GistGraphOp::ParseListFiles,
);
let parse = builder.add_node_after(parse_node, &execute)?;

builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
builder.add_edge(execute.out("response"), parse.in_port("response"))?;
```

This pattern is repeated for every I/O operation in every tool.

### 2. Op Enum Wrapping

Each tool must define its own wrapper enum:

```rust
// lib/tools/gist/src/graph.rs
pub enum GistGraphOp {
    PrepareListFiles,
    ParseListFiles,
    PrepareReadFiles,
    ParseReadFiles,
    // ... many more
    Transport(TransportOps),
}

// lib/tools/ci/src/graph.rs  
pub enum CIGraphOp {
    CI(CIOp),
    PrepareFileExists(PrepareFileExistsOp),
    PrepareShell(PrepareShellOp),
    Transport(TransportOps),
    CliTool(CliToolOp),
}

// lib/tools/makegen/src/graph.rs
pub enum MakegenGraphOp {
    Makegen(MakegenOp),
    Primitive(PrimitiveOp),
    Transport(TransportOps),
}
```

Every tool reinvents this union pattern.

### 3. PrepareXxx Ops Repetition

Common operations are redefined with slight variations:

```rust
// In gist: PrepareListFiles, PrepareReadFiles, PrepareReadFile
// In ci: PrepareFileExists, PrepareShell, PrepareBuild, PrepareTest
// In bootstrap: PrepareScanWorkspace, PrepareFileWrite
// In deps: PrepareLoadManifest, PrepareExecuteInstalls
```

Many of these are just thin wrappers around shell commands or file operations.

## Proposed Solutions

### Phase 1: Transport Chain Builder (Highest Value)

Add methods to `DagBuilder` that emit complete transport chains:

```rust
impl<T> DagBuilder<T> {
    /// Create a shell command chain: prepare -> execute -> parse
    /// Returns handles to all three nodes for additional wiring.
    pub fn shell_chain(
        &mut self,
        name: &str,
        command: &str,
        args: &[&str],
    ) -> Result<TransportChain<T>, BuilderError>
    where
        T: From<TransportOps> + From<PrepareShellOp> + From<ParseShellOp>,
    {
        // Creates 3 nodes, wires them, returns handles
    }

    /// Create a file read chain: prepare -> execute -> parse
    pub fn file_read_chain(
        &mut self,
        name: &str,
        path_input: PortRef,  // or embedded path
    ) -> Result<TransportChain<T>, BuilderError>;

    /// Create a file write chain: prepare -> execute
    pub fn file_write_chain(
        &mut self,
        name: &str,
        path_input: PortRef,
        content_input: PortRef,
    ) -> Result<TransportChain<T>, BuilderError>;

    /// Create a file exists check chain
    pub fn file_exists_chain(
        &mut self,
        name: &str,
        path: &str,
    ) -> Result<TransportChain<T>, BuilderError>;
}

/// Handle to a transport chain for additional wiring
pub struct TransportChain<T> {
    pub prepare: NodeRef,
    pub execute: NodeRef,
    pub parse: Option<NodeRef>,  // None for write operations
}
```

**Usage:**

```rust
// Before: 15+ lines
// After: 1 line
let list_files = builder.shell_chain("list-files", "git", &["ls-files", "-z"])?;

// Wire the output
builder.add_edge(list_files.parse.unwrap().out("stdout"), next_node.in_port("files"))?;
```

### Phase 2: Centralized Prepare/Parse Ops

Move common ops to `lib/primitives/src/io.rs`:

```rust
// lib/primitives/src/io.rs

/// Prepare a shell command (pure - outputs TransportRequest)
#[derive(Debug, Clone)]
pub struct PrepareShellOp {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

/// Parse shell response (pure - extracts stdout/stderr/exit_code)
#[derive(Debug, Clone)]
pub struct ParseShellOp;

/// Prepare file read (pure - outputs TransportRequest)
#[derive(Debug, Clone)]
pub struct PrepareFileReadOp {
    pub path: String,
}

/// Parse file read response (pure - extracts content)
#[derive(Debug, Clone)]
pub struct ParseFileReadOp;

/// Prepare file write (pure - outputs TransportRequest)
#[derive(Debug, Clone)]
pub struct PrepareFileWriteOp;

/// Prepare file exists check (pure - outputs TransportRequest)
#[derive(Debug, Clone)]
pub struct PrepareFileExistsOp {
    pub path: String,
}

/// Parse file exists response (pure - extracts bool)
#[derive(Debug, Clone)]
pub struct ParseFileExistsOp;
```

Tools can then import and wrap these instead of redefining:

```rust
pub enum GistGraphOp {
    // Domain-specific ops
    FilterByExtension { extensions: Vec<String> },
    CollectFileContents,
    
    // Reuse primitives
    Primitive(PrimitiveOp),
    Transport(TransportOps),
}
```

### Phase 3: Pattern Composition Shortcuts

Build on existing patterns (`UpsertBuilder`, `LoopBuilder`, etc.) with convenient shortcuts:

```rust
impl<T> DagBuilder<T> {
    /// Shortcut for UpsertBuilder with transport chains
    pub fn upsert_with_transport(
        &mut self,
        name: &str,
        check_chain: TransportChain<T>,
        create_chain: TransportChain<T>,
    ) -> Result<NodeRef, BuilderError>;

    /// Shortcut for LoopBuilder over a list with transport body
    pub fn loop_with_transport(
        &mut self,
        name: &str,
        items_port: PortRef,
        body_chain: impl FnOnce(&mut DagBuilder<T>) -> Result<TransportChain<T>, BuilderError>,
    ) -> Result<NodeRef, BuilderError>;
}
```

### Phase 4: Generic Op Union (Advanced)

Consider a standard "tool graph op" pattern:

```rust
/// Standard union type for tool graphs
pub enum ToolGraphOp<DomainOp> {
    /// Domain-specific pure operations
    Domain(DomainOp),
    /// Reusable primitive operations
    Primitive(PrimitiveOp),
    /// Transport boundary
    Transport(TransportOps),
}

// Tools just define their domain ops:
pub type GistGraphOp = ToolGraphOp<GistOp>;
pub type CIGraphOp = ToolGraphOp<CIOp>;
```

This is more invasive but eliminates the enum boilerplate entirely.

## Priority Order

1. **Transport Chain Builder** (Phase 1) — Highest impact, lowest risk
2. **Centralized Prepare/Parse Ops** (Phase 2) — Enables Phase 1, reduces duplication
3. **Pattern Shortcuts** (Phase 3) — Nice-to-have after basics work
4. **Generic Op Union** (Phase 4) — Consider after patterns stabilize

## Migration Path

1. Implement `TransportChainBuilder` as extension methods on `DagBuilder`
2. Add centralized ops to `lib/primitives`
3. Migrate one tool (suggest: gist) as proof of concept
4. If successful, migrate remaining tools incrementally
5. Deprecate per-tool PrepareXxx definitions

## Success Criteria

- [ ] New transport chains require < 5 lines instead of 15+
- [ ] Common ops (shell, file read/write, file exists) defined once
- [ ] Existing tools continue to work during migration
- [ ] DryRun interception still works correctly

## Related Files

- `core/ir/src/builder.rs` — DagBuilder implementation
- `core/ir/src/patterns/` — Existing pattern builders
- `lib/primitives/src/io.rs` — Where centralized ops would live
- `lib/tools/*/src/graph.rs` — Tool graphs to migrate

## Candidates Across Codebase

### Summary Statistics

| Metric | Count | Notes |
|--------|-------|-------|
| Total `TransportOps::Execute` usages | 45 | Across 14 files |
| Wrapper enum definitions | 8 | `XxxGraphOp` patterns |
| Transport chains documented | ~17 | `Prepare -> Execute -> Parse` |
| Centralized PrepareXxx ops (primitives) | 6 | Already in `lib/primitives/src/io.rs` |
| Duplicated PrepareXxx ops | 2 | CI defines its own |

### Transport Chains Per Tool

| Tool | Chains | Example Chains |
|------|--------|----------------|
| **gist** | 3 | `ListFiles`, `ReadFiles` (batch), `ReadFile` (single) |
| **ci** | 6+ | `FileExists`, `CodegenExists`, `CodegenCmd`, `Build`, `Test`, `Lint` |
| **deps** | 2 | `LoadManifest`, `ExecuteInstalls` |
| **bootstrap** | 3 | `ScanWorkspace`, `MakefileWrite`, `GitignoreWrite` |
| **buck2** | 2 | `ParseCargoToml`, `FileWrite` |
| **makegen** | 1 | `FileWrite` |
| **clippy** | 4 | Uses SubDag pattern via `CliToolOp` |

### Wrapper Enum Definitions (Candidates for Generic Pattern)

```
lib/tools/gist/src/graph.rs      → GistGraphOp (13 variants)
lib/tools/ci/src/graph.rs        → CIGraphOp (5 variants)
lib/tools/deps/src/graph.rs      → DepsGraphOp (2 variants)
lib/tools/bootstrap/src/graph.rs → BootstrapGraphOp (3 variants)
lib/tools/buck2/src/graph.rs     → Buck2GraphOp (3 variants)
lib/tools/makegen/src/graph.rs   → MakegenGraphOp (3 variants)
lib/tools/ci/src/ops.rs          → CIOp (12 variants)
lib/tools/deps/src/ops.rs        → DepsOp (6 variants)
```

### Existing Centralized Ops (lib/primitives/src/io.rs)

Already have these - **tools should use them**:
- `PrepareFileWriteOp` — used by bootstrap, buck2, makegen
- `PrepareFileReadOp` — available but not used
- `PrepareFileExistsOp` — **duplicated** in CI
- `PrepareShellOp` — **duplicated** in CI
- `PrepareDirectoryListOp` — available but not used
- `HttpRequestOp` — available but not used

### Duplication Candidates (Quick Wins)

**DONE: CI now uses primitives' embedded ops.**

Added to `lib/primitives/src/io.rs`:
- `EmbeddedFileExistsOp { path: String }` — for hardcoded paths
- `EmbeddedShellOp { command, args }` — for hardcoded commands

CI imports and uses these instead of defining its own.

Two patterns now exist in primitives:
- **Port-based** (`PrepareFileExistsOp`, `PrepareShellOp`) — dynamic values from upstream
- **Embedded** (`EmbeddedFileExistsOp`, `EmbeddedShellOp`) — hardcoded values

### Largest Graph Files (Most Boilerplate)

| File | Lines | Transport Chains |
|------|-------|------------------|
| `lib/tools/gist/src/graph.rs` | 1050 | 3 |
| `lib/tools/ci/src/graph.rs` | 756 | 6+ |
| `lib/tools/deps/src/graph.rs` | 260 | 2 |
| `lib/tools/bootstrap/src/graph.rs` | 250 | 3 |

### Assessment: Worth It or Premature?

**Done (High Value / Low Risk):**
1. ~~**Consolidate CI's duplicate ops**~~ ✅ DONE — Added `EmbeddedXxxOp` to primitives, CI imports them
2. **Document "one way to do it"** → Update AGENT.md with primitives pattern

**Moderate Value:**
3. **Transport chain builder** → Would save ~10 lines per chain × 17 chains = ~170 lines
4. **Generic `ToolGraphOp<D>`** → Would eliminate 8 enum definitions

**Possibly Premature:**
5. **Full migration of all tools** → Current code works, migration has risk
6. **Pattern composition shortcuts** → Existing builders work, just verbose

### Recommendation

~~Start with **low-risk consolidation**:~~
~~1. Fix CI to use primitives (remove duplicates)~~ ✅ DONE

Next steps:
1. Add `TransportChainBuilder` as optional convenience
2. Document the "golden path" in AGENT.md
3. Let tools migrate incrementally as they're touched

Don't force a full rewrite - the current code is correct, just verbose.

## Related Concepts

- Transport compliance (completed) — This builds on the pure ops foundation
- Language module — Similar "centralize common patterns" approach
- Pattern builders — Existing infrastructure to build on
