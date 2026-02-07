# TODO: Graph Helper Refactoring

**Created:** 2026-02-06
**Updated:** 2026-02-07

Five refactoring opportunities to reduce graph-building boilerplate, ordered by priority.

---

## 1. Generic Content Upsert Helper

**Status:** Done
**Effort:** Medium
**Location:** `core/ir/src/patterns/content_upsert.rs`

### Problem

The 6-node content freshness pattern (render → prepare_read → execute_read → compare → prepare_write → execute_write) is copy-pasted across 4 binary workflows. Each instance wires the same 8 edges with the same port names. Only testgen has a local helper, and it's hardcoded to `TestgenGraphOp`.

### Design

A generic builder function in `core/ir/src/patterns/`:

```rust
pub struct ContentUpsertChain<T> {
    pub generate: NodeRef<T>,
    pub execute_write: NodeRef<T>,
}

pub fn add_content_upsert_chain<T: Clone>(
    builder: &mut DagBuilder<T>,
    name: &str,
    generate_node: Node<T>,
    prepare_read_op: T,
    prepare_write_op: T,
    compare_op: T,
    transport_op: T,
) -> Result<ContentUpsertChain<T>, BuilderError>
```

The function:
1. Adds the generate node as-is (caller controls inputs/outputs)
2. Stamps out the 5 infrastructure nodes with `{name}_` prefixed IDs
3. Wires all 8 standard edges (content → compare, content → write, read chain, compare → skip, write chain)
4. Returns refs to generate (for upstream wiring) and execute_write (for downstream/boundary access)

Port contract (fixed by the helper):
- Generate node must have output `port("content", "String")`
- Execute write node gets skippable shape with outputs: `optional("{name}_response", "TransportResponse")`, `port("skip", "Bool")`, etc.

### Migration Targets

| File | Current State | Chains | Lines Saved (est.) |
|------|--------------|--------|--------------------|
| `gunbc-dag/src/testgen_dag/graph.rs` | Has local `add_upsert_chain` helper | N (dynamic) | Replace local helper with shared one (~100 lines) |
| `gunbc-dag/src/pragma/graph.rs` | 3 chains wired manually | 3 | ~200 lines |
| `gunbc-dag/src/bootstrap/graph.rs` | 2 chains wired manually | 2 | ~140 lines |
| `gunbc-dag/src/makegen/graph.rs` | 1 chain wired manually | 1 | ~70 lines |

### Steps

1. Add `content_upsert.rs` to `core/ir/src/patterns/`
2. Re-export from `patterns/mod.rs`
3. Migrate testgen first (already has a helper to compare against)
4. Migrate pragma (3 chains — good stress test of the API)
5. Migrate bootstrap and makegen
6. Delete testgen's local `add_upsert_chain`
7. Add handbook entry (see item 4 below)

### Open Questions

- Should the helper accept a path-injection strategy? Currently some graphs hardcode the path in the generate node, others wire it from upstream. The helper could accept an optional `path_source: Option<OutputRef<T>>` to handle both.
- Should the compare node accept `check_mode`? Some chains use it, some don't. Could be an optional input left unwired by default.

---

## 2. Adopt Transport Triplet Helpers

**Status:** Done (migrated: build, codegen, CI, gist, deps, review, llm-ops)
**Updated:** 2026-02-07
**Effort:** Low
**Location:** Existing helpers in `core/ir/src/patterns/transport_triplet.rs`

### Problem

`add_transport_triplet` and `add_skippable_transport_triplet` exist but only CI uses them. Every other graph wires the same prepare → execute → parse edges manually.

### Design

No new API needed — the helpers already exist. This is a pure migration task. If the API proves awkward for any graph, improve it rather than skipping that graph.

Potential API improvement: the current helpers require callers to pass full input/output port lists for prepare and parse nodes. A variant that accepts only the *domain-specific* ports and auto-appends the standard transport ports (`request`, `skip`, `response`) would reduce noise.

### Migration Targets

| File | Triplets | Skippable? | Notes | Status |
|------|----------|------------|-------|--------|
| `gunbc-dag/src/codegen/graph.rs` | 3 | Mixed | exists_check (no), codegen (no), stamp_write (no parse) | Done |
| `gunbc-dag/src/build/graph.rs` | 3 | Mixed | build (no), test (yes), clippy (yes) | Done |
| `gunbc-dag/src/ci/graph.rs` | 6 | Mixed | deps_exists (no), testgen/build/test/guardrail/verify (yes) | Done |
| `lib/tools/gist/src/graph.rs` | 5-8 | No | Mode-dependent; loop body has inner triplet | Done |
| `lib/tools/deps/src/graph.rs` | 2-3 | No | install + generate graphs | Done |
| `lib/review/src/graph.rs` | 2-4 | No | Varies by review mode | Done |
| `lib/llm-ops/src/graph.rs` | 1 | No | chat completion | Done |

### Steps

1. Audit current helper API against each migration target — identify any friction
2. If needed, add a convenience variant that auto-appends standard ports
3. Migrate codegen (small, 3 triplets)
4. Migrate build (small, 3 triplets with mixed skippability)
5. Migrate library tools (gist, deps, review, llm)
6. Verify no behavior change via `cargo test --workspace`

### Edge Cases

- **Stamp write in codegen** has no parse node (2-node triplet, not 3). The helper assumes 3 nodes. Either skip this one or add a `add_transport_pair` variant.
- **Loop body triplet in gist** is inside a SubDag. The helper should work on any `DagBuilder<T>` regardless of nesting, but verify.
- **Auth chain in review/llm** (resolve_auth → credential_env → execute) isn't a standard triplet — leave these manual.

---

## 3. Generic `convert_dag` for Workspace

**Status:** Done
**Effort:** Low
**Location:** `gunbc-dag/src/workspace/convert.rs` (new file)

### Problem

Every tool in `workspace/subdags/` defines `convert_foo_node` and `convert_foo_dag` functions that do identical structural recursion. The only thing that varies is the op mapping closure.

### Design

```rust
pub fn convert_dag<S, T>(dag: Dag<S>, f: &impl Fn(S) -> T) -> Dag<T> {
    Dag {
        nodes: dag.nodes.into_iter().map(|n| convert_node(n, f)).collect(),
        edges: dag.edges,
    }
}

fn convert_node<S, T>(node: Node<S>, f: &impl Fn(S) -> T) -> Node<T> {
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
            NodeBody::SubDag(dag) => NodeBody::SubDag(convert_dag(dag, f)),
        },
        examples: node.examples,
    }
}
```

Each subdag builder collapses to:

```rust
pub fn build_ci_subdag() -> Node<WorkspaceOp> {
    let dag = build_ci_graph();
    let converted = convert_dag(dag, &convert_ci_op);
    Node::subdag("ci", converted)
}
```

### Migration Targets

| File | Current Functions | Replaced By |
|------|------------------|-------------|
| `workspace/subdags/bootstrap.rs` | `convert_bootstrap_node`, `convert_bootstrap_dag` | `convert_dag(&convert_bootstrap_op)` |
| `workspace/subdags/ci.rs` | `convert_ci_node`, `convert_ci_dag` | `convert_dag(&convert_ci_op)` |
| `workspace/subdags/clippy.rs` | `convert_clippy_node`, `convert_clippy_dag` | `convert_dag(&convert_clippy_op)` |
| `workspace/subdags/deps.rs` | `convert_deps_node`, `convert_deps_dag` | `convert_dag(&convert_deps_op)` |
| `workspace/subdags/gist.rs` | `convert_gist_node`, `convert_gist_dag` | `convert_dag(&convert_gist_op)` |
| `workspace/subdags/makegen.rs` | `convert_makegen_node`, `convert_makegen_dag` | `convert_dag(&convert_makegen_op)` |

The per-tool `convert_foo_op` mapping functions stay (they encode the domain-specific variant mapping). Only the structural recursion is eliminated.

### Steps (completed)

1. Added `gunbc-dag/src/workspace/convert.rs` with `convert_dag` and `convert_node`
2. Migrated subdags that need op remapping (CI, gist)
3. Simplified subdags that only need `convert_node` (clippy, languages)
4. Removed per-tool `convert_*` helpers (none remain)

### Open Question

Decision: keep `convert_dag`/`convert_node` in `gunbc-dag/src/workspace/convert.rs` for now. If other crates need op remapping, move to `core/ir` later.

---

## 4. Handbook Entry for Content Upsert Pattern

**Status:** Done
**Effort:** Low
**Location:** `docs/handbook.md`

### Problem

The content upsert pattern is the most common graph pattern in binary workflows (4 of 7 use it) but has no handbook entry. The existing "Upsert" entry (A.2.1) covers tool acquisition, which is a different pattern.

### Design

Add section **A.2.4 Content Upsert** (or A.12, depending on numbering preference) to the pattern catalog:

**Content to cover:**
- Intent: Idempotent file generation — render expected content, read current file, compare, skip write if fresh
- Structure: 6-node chain (generate → prepare_read → execute_read → compare → prepare_write → execute_write)
- Invariants: Compare node (BlobOps::CompareContent) produces `skip: Bool` and `skip_reason: String`; write transport is skippable
- Relationship to A.11 (Freshness): A.11 covers the mtime fast path at the infra level; content upsert is the graph-level pattern that uses content hashing when mtime says "maybe stale"
- Relationship to A.2.1 (Upsert): A.2.1 is resource acquisition (check → create → resolve); content upsert is file generation (render → read → compare → write). Different intent, different shape.
- Helper API: reference `add_content_upsert_chain` (once item 1 is done)
- Examples: testgen (dynamic N chains), pragma (3 static parallel chains), bootstrap (2 parallel chains after scan), makegen (1 chain)

### Steps (completed)

1. Added A.2.4 Content Upsert entry in `docs/handbook.md`
2. Updated the entry to reference the shared `add_content_upsert_chain` helper

---

## 5. GraphOp Composition Helper

**Status:** Not started
**Effort:** Medium
**Location:** `core/ir/src/ops.rs` (new) or `lib/primitives/`

### Problem

Four binary graphs define nearly identical op enums:

```rust
// This shape appears in makegen, pragma, bootstrap, testgen
pub enum FooGraphOp {
    Foo(FooOp),                           // domain-specific
    PrepareFileRead(PrepareFileReadOp),    // shared
    PrepareFileWrite(PrepareFileWriteOp),  // shared
    Blob(BlobOps),                         // shared
    Transport(TransportOps),               // shared
}
```

Each also has a mechanical `Executable` impl that dispatches each variant.

### Design Options

**Option A: Generic wrapper enum**

```rust
pub enum FileOpsGraph<D> {
    Domain(D),
    PrepareFileRead(PrepareFileReadOp),
    PrepareFileWrite(PrepareFileWriteOp),
    Blob(BlobOps),
    Transport(TransportOps),
}

impl<D: Executable> Executable for FileOpsGraph<D> { ... }
```

Then: `type MakegenGraphOp = FileOpsGraph<MakegenOp>;`

Pros: Zero boilerplate for new file-upsert tools.
Cons: Adds a layer of indirection. Workspace conversion functions need to handle the generic wrapper. Might not play well with graphs that need *additional* infrastructure variants (CI has `CliTool`, gist has `Git`).

**Option B: Derive macro**

```rust
#[derive(GraphOp)]
#[graph_op(transport, file_read, file_write, blob)]
pub enum MakegenGraphOp {
    Makegen(MakegenOp),
}
```

Generates the full enum + `Executable` dispatch.

Pros: Flexible (opt-in to which infra variants you need).
Cons: Another proc macro to maintain.

**Option C: Do nothing beyond item 1**

If the content upsert helper (item 1) absorbs the need to pass `PrepareFileReadOp`/`PrepareFileWriteOp`/`BlobOps` into graph builders, the op enum boilerplate becomes less painful because you interact with it less often. The remaining dispatch code is ~15 lines per enum, which may not justify a new abstraction.

### Migration Targets (if Option A)

| Graph | Current Enum | Would Become |
|-------|-------------|-------------|
| `testgen_dag/ops.rs` | `TestgenGraphOp` (5 variants) | `FileOpsGraph<TestgenOp>` |
| `pragma/ops.rs` | `PragmaGraphOp` (5 variants) | `FileOpsGraph<PragmaOp>` |
| `bootstrap/ops.rs` | `BootstrapGraphOp` (5 variants) | `FileOpsGraph<BootstrapOp>` |
| `makegen/ops.rs` | `MakegenGraphOp` (5 variants) | `FileOpsGraph<MakegenOp>` |

**Not migrated** (different shape): codegen (2 variants), build (2 variants), CI (5 variants but different infra ops), gist, review, deps, llm.

### Recommendation

Defer this until item 1 is done. If the content upsert helper makes the op enum pain tolerable, skip this entirely (Option C). If it doesn't, prefer Option A over Option B — a generic enum is simpler than a proc macro and easier to debug.

### Steps (if proceeding)

1. Complete item 1 first
2. Evaluate remaining boilerplate pain
3. If still warranted, implement `FileOpsGraph<D>` in `core/ir/src/ops.rs`
4. Migrate the 4 binary graphs
5. Update workspace conversion to handle the generic wrapper
