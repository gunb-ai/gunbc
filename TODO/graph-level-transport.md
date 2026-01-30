# DAG Elegance: Pure Nodes & Pattern Consolidation

**Status**: In Progress (Architecture Refinement)
**Date**: 2026-01-30

## Goal

Bring the codebase into compliance with the **Graph Invariants** defined in [`docs/design/overview.md`](../docs/design/overview.md#graph-invariants):

| Invariant | Summary |
|-----------|---------|
| **I1. Node Purity** | Every node is pure or a Transport Execute node |
| **I2. Transport Boundary** | All I/O flows through `TransportRequest` → Execute → `TransportResponse` |
| **I3. Observable I/O** | All I/O visible as explicit graph nodes |
| **I4. Minimal Graph** | Minimum nodes, maximum pattern reuse |
| **I5. Deterministic Ordering** | Fan-in has canonical edge ordering |

---

## Architecture Evolution

### Phase 1: Pure Execute Functions (COMPLETED for CI)

The CI tool was migrated to have pure `execute_*` functions with no hidden I/O:
- `CIGraphOp` union type with explicit transport nodes
- All ops decomposed into `Prepare*` → `TransportOps::Execute` → `Parse*` chains
- No `execute_transport()` calls in `ops.rs`

### Phase 2: Structural I/O Enforcement (CURRENT DIRECTION)

**The root cause**: Two ways to do I/O exist:
1. Correct: `TransportOps::Execute` nodes (visible, interceptable)
2. Escape hatch: `execute_transport()` called inside ops (hidden, not interceptable)

**The fix**: Remove the escape hatch structurally — make `execute_transport()` not callable from tool crates.

```rust
// lib/transport/src/lib.rs - make execute_transport non-public
pub use ops::TransportOps;  // Only export the DAG node type
// execute_transport is NOT exported
```

**Key insight**: Custom pure ops are fine. The goal is NOT "primitives only" — that's massive cognitive load. The invariant is simpler:

> **No `execute_transport()` calls outside `TransportOps::Execute`**

**Migration order**:
1. Build transport chain helper (make correct path cheap)
2. Migrate gist (proof of concept)
3. Make `execute_transport()` non-public (close escape hatch)
4. Migrate remaining tools (deps → buck2 → bootstrap)

See: `~/.cursor/plans/pure_node_enforcement_bf6bf5f3.plan.md`

---

## Current State

### What's Done

1. **Transport Compliance (I2)**: All I/O routes through `lib/transport` via `execute_transport()`. No direct `std::fs` or `Command::new` calls remain in tool crates.

2. **CI Tool Migration (I1, I3)**: The CI tool has pure `execute_*` functions with explicit transport nodes. **However**, this is now considered an intermediate step - the target is pure DAG composition with no custom code.

### What's Violated

**I1, I3** are violated in remaining tools (gist, deps, buck2, bootstrap, makegen): I/O is hidden inside opaque nodes via `execute_transport()` calls, not exposed in graph structure.

**Side door identified**: `lib/fs` uses direct `std::fs` and `Command::new`, bypassing transport entirely.

```
Current (violates I1, I3):
┌──────────────────────────────────┐
│ LoadManifest (opaque, impure)    │
│                                  │
│   execute_transport() inside     │  ← I/O hidden, not interceptable
│                                  │
└──────────────────────────────────┘

Target (satisfies I1, I3):
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ PrepareRead  │──▶│ Execute      │──▶│ ParseToml    │
│ (pure)       │   │ (transport)  │   │ (pure)       │
└──────────────┘   └──────────────┘   └──────────────┘
                         ↑
                 DryRun intercepts here
```

---

## Invariant Audit

### Summary Table

| Tool | I1 (Purity) | I2 (Transport) | I3 (Observable) | I4 (Minimal) | I5 (Ordering) |
|------|-------------|----------------|-----------------|--------------|---------------|
| ci | ✅ **All pure** | ✅ Uses transport | ✅ **Explicit nodes** | ⚠️ Not using SubDag | ✅ N/A |
| gist | ❌ Impure ops | ✅ Uses transport | ❌ Hidden I/O | ❌ Ad-hoc loop | ✅ N/A |
| deps | ❌ Impure ops | ✅ Uses transport | ❌ Hidden I/O | ❌ Ad-hoc upsert | ✅ N/A |
| buck2 | ❌ Impure ops | ✅ Uses transport | ❌ Hidden I/O | ❌ Ad-hoc | ✅ N/A |
| makegen | ⚠️ Registry I/O | ✅ Uses transport | ❌ Hidden I/O | ✅ OK | ✅ N/A |
| bootstrap | ❌ Impure ops | ✅ Uses transport | ❌ Hidden I/O | ✅ OK | ✅ N/A |
| clippy | ✅ Uses UpsertBuilder | ✅ Uses transport | ✅ SubDag exposed | ✅ Uses pattern | ✅ N/A |

### Detailed Violations

#### CI Tool (`lib/tools/ci/`) - ✅ MIGRATED

The CI tool has been fully migrated to the pure node pattern. See [CI Migration Details](#ci-migration-details) below.

**Remaining opportunity**: The Lint stage uses the same prepare/execute/parse pattern as other stages. A future enhancement could integrate `build_clippy_upsert()` as a SubDag for I4 compliance.

| Location | Status |
|----------|--------|
| `CIOp::*` | ✅ All decomposed into pure Prepare/Parse ops |
| `CIGraphOp` | ✅ Union type with CI, PrepareFileExists, PrepareShell, Transport |
| Transport nodes | ✅ 6+ explicit `TransportOps::Execute` nodes visible in graph |

#### Gist Tool (`lib/tools/gist/`)

| Node | Invariant | Issue | Fix |
|------|-----------|-------|-----|
| `ListFiles` | I1, I3 | Hides `git ls-files` shell call | Decompose: `PrepareShellOp` → `Execute` → `ParseLines` |
| `ReadFiles` | I1, I3, I4 | Hides N file reads, ad-hoc loop | Use `LoopBuilder` with `PrepareFileReadOp` → `Execute` body |

#### Deps Tool (`lib/tools/deps/`)

| Node | Invariant | Issue | Fix |
|------|-----------|-------|-----|
| `LoadManifest` | I1, I3 | Hides file read via `execute_transport()` | Decompose: `PrepareFileReadOp` → `Execute` → `ParseToml` |
| `GenerateScripts` | I1, I3 | Calls `DepsManifest::load()` (reads file again) | Pass manifest data from LoadManifest output |
| `ExecuteInstalls` | I1, I3, I4 | Hides shell install via `execute_transport()`, ad-hoc loop | Use `LoopBuilder` of `UpsertBuilder` |

#### Buck2 Tool (`lib/tools/buck2/`)

| Node | Invariant | Issue | Fix |
|------|-----------|-------|-----|
| `ParseCargoToml` | I1, I3 | Hides file read | Decompose: `PrepareFileReadOp` → `Execute` → `ParseToml` |
| `GenerateTargets` | I1, I3 | Hides `.exists()` checks | Use `PrepareFileExistsOp` → `Execute` per check |

#### Bootstrap Tool (`lib/tools/bootstrap/`)

| Node | Invariant | Issue | Fix |
|------|-----------|-------|-----|
| `ScanWorkspace` | I1, I3 | Hides `find` command | Decompose: `PrepareShellOp("find")` → `Execute` → `ParseLines` |

#### Makegen Tool (`lib/tools/makegen/`)

| Location | Invariant | Issue | Fix |
|----------|-----------|-------|-----|
| `registry.rs:needs_codegen()` | I1, I3 | Hides `.exists()` checks | Use `PrepareFileExistsOp` → `Execute` (or accept as build-time-only) |

---

## Reconciliation Plan

### Completed

- [x] **CI I2 violation** - `execute_setup_deps` now uses `FileRequest::exists()` via transport
- [x] **CI Full Migration** - All ops decomposed into pure Prepare/Parse chains with explicit transport nodes

### Phase 1: Node Decomposition (I1, I3)

Convert impure opaque nodes to pure chains. Each impure node becomes:

```
[PrepareOp (pure)] → [TransportOps::Execute] → [ParseOp (pure)]
```

**Reference Implementation: CI Tool**

The CI tool demonstrates the complete pattern. Key files:
- `lib/tools/ci/src/graph.rs` - `CIGraphOp` union type, explicit transport nodes
- `lib/tools/ci/src/ops.rs` - Pure `Prepare*` and `Parse*` ops, no `execute_transport()` calls

**Priority order** (CI complete, others remaining):
1. ~~**ci** - 5 ops to decompose (Build, Test, Lint, Prep, SetupDeps)~~ ✅ **DONE**
2. **gist** - 2 ops (ListFiles, ReadFiles) 
3. **deps** - 3 ops (LoadManifest, GenerateScripts, ExecuteInstalls)
4. **buck2** - 2 ops (ParseCargoToml, GenerateTargets)
5. **bootstrap** - 1 op (ScanWorkspace)

### Phase 2: Pattern Consolidation (I4)

Replace ad-hoc patterns with canonical builders from `core/ir/src/patterns/`:

| Ad-hoc Pattern | Canonical Pattern | Locations | Benefit |
|----------------|-------------------|-----------|---------|
| Check-then-create | `UpsertBuilder` | deps installs | Idempotent, observable |
| Loop with internal I/O | `LoopBuilder` | gist ReadFiles | Each iteration interceptable |
| N sequential commands | `LoopBuilder` | deps ExecuteInstalls | Parallelizable |

**Example: Deps ExecuteInstalls with LoopBuilder + UpsertBuilder**

```rust
// Each dependency install becomes an Upsert:
let install_dep = UpsertBuilder::new("install_dep")
    .with_check(|dep| PrepareShellOp(dep.verify_cmd))   // Pure
    .with_create(|dep| PrepareShellOp(dep.install_cmd)) // Pure  
    .with_resolve(|dep| PrepareShellOp(dep.verify_cmd)) // Pure
    .build();

// Loop over all dependencies:
let install_all = LoopBuilder::new("install_all")
    .with_input("deps", Cardinality::ZeroOrMore)
    .with_body(install_dep)  // The Upsert sub-DAG
    .with_output("results", Cardinality::ZeroOrMore)
    .build();
```

### Phase 3: Verification Checklist

For each tool after refactoring:

| Check | How to Verify |
|-------|---------------|
| I1 satisfied | All ops either pure or `TransportOps::Execute` |
| I2 satisfied | `cargo clippy` passes (no `disallowed_methods`) |
| I3 satisfied | DryRun test intercepts all I/O nodes |
| I4 satisfied | Graph uses pattern builders, no ad-hoc loops/upserts |
| I5 satisfied | N/A (already implemented) |

### Phase 4: SubDag Integration (I4 Enhancement)

After Phase 1-3, consider replacing simple prepare/execute/parse chains with SubDag patterns where appropriate:

| Tool | Current | Enhancement |
|------|---------|-------------|
| ci (Lint) | `PrepareLint` → `Execute` → `ParseLint` | Replace with `build_clippy_upsert()` SubDag |
| gist (ReadFiles) | Manual loop | Use `LoopBuilder` with file read body |
| deps (ExecuteInstalls) | Manual iteration | Use `LoopBuilder` of `UpsertBuilder` |

**CI Lint SubDag Enhancement**:
```rust
// Current (I1, I3 satisfied but I4 could be better):
CIGraphOp::CI(CIOp::PrepareLintCommand)
CIGraphOp::Transport(TransportOps::Execute)  
CIGraphOp::CI(CIOp::ParseLintResult)

// Future (uses existing clippy pattern for I4):
CIGraphOp::ClippyUpsert(build_clippy_lint_all())  // SubDag handles check/install/run
```

This requires adding a `ClippyUpsert` variant to `CIGraphOp` that wraps `Node<CliToolOp>`.

### Not In Scope (Build-Time Exceptions)

- **makegen registry** `needs_codegen()` - runs at build time, not DAG execution time
- **codegen** - bootstrap code, can't use DAG (chicken/egg)

---

## CI Migration Details

The CI tool serves as the reference implementation for the pure node pattern.

### Union Type Pattern

```rust
// lib/tools/ci/src/graph.rs
pub enum CIGraphOp {
    CI(CIOp),                           // Domain-specific pure ops
    PrepareFileExists(PrepareFileExistsOp),  // Pure file check (local, with embedded path)
    PrepareShell(PrepareShellOp),       // Pure shell preparation
    Transport(TransportOps),            // Boundary (actual I/O)
}
```

### Pure Op Pattern

```rust
// lib/tools/ci/src/ops.rs - all pure, no execute_transport() calls
pub enum CIOp {
    // SetupDeps stage
    ParseDepsExists,
    
    // Prep stage  
    PrepareCodegenExistsCheck,
    ParseCodegenExists,
    PrepareCodegenCommand,
    ParseCodegenResult,
    
    // Build stage
    PrepareBuildCommand,
    ParseBuildResult,
    
    // Test stage
    PrepareTestCommand,
    ParseTestResult,
    
    // Lint stage
    PrepareLintCommand,
    ParseLintResult,
    
    // Report (already pure)
    Report,
}
```

### Graph Structure

```
SetupDeps: PrepareFileExists(deps.toml) → Execute → ParseDepsExists
Prep:      PrepareCodegenExistsCheck → Execute → ParseCodegenExists
           → PrepareCodegenCommand → Execute → ParseCodegenResult  
Build:     PrepareBuildCommand → Execute → ParseBuildResult
Test:      PrepareTestCommand → Execute → ParseTestResult     (parallel)
Lint:      PrepareLintCommand → Execute → ParseLintResult     (parallel)
Report:    Report (pure)
```

### Metrics

- **Before**: 6 opaque nodes hiding I/O
- **After**: ~20 nodes with all I/O at explicit `TransportOps::Execute` boundaries
- **No `execute_transport()` calls** in ops.rs
- **No `println!`** in ops.rs

---

## Node Categories (Target State)

After reconciliation, all nodes fall into these categories:

| Category | Properties | Examples |
|----------|------------|----------|
| **Prepare** | Pure, outputs `TransportRequest` | `PrepareFileReadOp`, `PrepareShellOp` |
| **Execute** | Transport boundary, I/O happens here | `TransportOps::Execute` |
| **Transform** | Pure, data transformation | `ParseOp::Toml`, `FormatOp::*` |
| **Logic** | Pure, decision/validation | `ValidateOp`, `FilterOp`, `MapOp` |
| **Pattern** | SubDAG composition | `Upsert`, `Loop`, `Branch`, `Transaction` |

---

---

## Structural Enforcement Details

### The Invariant

> **No `execute_transport()` calls outside `TransportOps::Execute`**

Custom pure ops are fine. The goal is visible I/O, not "primitives only."

### What's Needed

1. **Transport chain helper**: Make prepare → execute → parse wiring cheap
2. **Close escape hatch**: Make `execute_transport()` non-public to tools
3. **Refactor side doors**: `lib/fs` uses direct `std::fs` — convert to pure `Prepare*` ops

### Example: Correct Pattern (CI does this)

```rust
// Custom pure op - GOOD (no I/O)
fn execute_parse_build_result(inputs) -> Result<...> {
    let response = inputs.get("response")?;  // Just parsing
    let success = response.exit_code == 0;
    Ok(outputs)
}

// Graph has explicit transport nodes:
// PrepareBuildCommand → TransportOps::Execute → ParseBuildResult
//                              ↑
//                      DryRun intercepts here
```

### Example: Violation (to be fixed)

```rust
// Op with hidden I/O - BAD
fn execute_scan_workspace(inputs) -> Result<...> {
    let response = execute_transport(&request)?;  // Hidden!
    // ... process response ...
    Ok(outputs)
}
```

Fix: Split into `PrepareScan` → `Execute` → `ParseScanResult`

---

## Related

- [`docs/design/overview.md#graph-invariants`](../docs/design/overview.md#graph-invariants) - Canonical invariant definitions
- [`TODONE/transport-compliance.md`](TODONE/transport-compliance.md) - Prerequisite work (I2 satisfied)
- [`TODO/opaque-op-tool-mismatch.md`](opaque-op-tool-mismatch.md) - Related problem (tool deps hidden in ops)
- [`core/ir/src/patterns/`](../core/ir/src/patterns/) - Existing pattern builders
- `~/.cursor/plans/pure_node_enforcement_bf6bf5f3.plan.md` - Detailed implementation plan
