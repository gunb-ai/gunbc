# Consolidation: Generic Ops and Rendering DAGs

**Status**: Ongoing
**Date**: 2026-02-03

Tracking for misplaced generic ops, duplicated patterns, and rendering
workflows that should become DAGs. Add items as they're discovered.

---

## 1. Generic ops living in domain crates

Ops that aren't domain-specific but currently live in a specific tool.
Extract to `lib/primitives` or `core/ir` when a second consumer appears.

### HashOp — stable ID generation

**Where it lives**: `lib/review/src/lib.rs` (`hash_finding_id()`)
and `lib/blob/src/lib.rs` (`BlobMeta::compute_hash()`)

**Problem**: Two different hash implementations for the same purpose
(stable content-addressed IDs). Review uses SHA256, blob uses
`DefaultHasher`. Any tool producing findings, diagnostics, or
content-addressed artifacts needs this.

**Proposed**: Generic `StableHashOp` in `lib/primitives`:

```rust
/// Compute a stable content hash from N input strings.
/// Produces a hex-encoded truncated SHA256.
pub struct StableHashOp {
    /// Number of bytes to keep (default 16 → 32 hex chars).
    pub truncate_bytes: usize,
}
```

Inputs: `parts` (list of strings to hash with `:` separator)
Output: `hash` (hex string)

**Consumers**: review findings, blob metadata, any dedup scenario.

### MergeOutputs — cardinality-aware collection

**Where it lives**: `lib/review/src/lib.rs` (`MergeOutputs`)

**Problem**: The null→0, object→1, array→N cardinality handling is
universal. The dedup-by-id logic is domain-specific. These are mixed
in one op.

**Proposed**: Split into two concerns:

1. **Engine-level** (cardinality design doc): The execution engine
   should normalize cardinality before ops see it. Once that lands,
   the defensive deserialization in MergeOutputs disappears.
2. **Generic DeduplicateOp** in `lib/primitives`: Dedup a JSON array
   by a configurable ID field. Review's `MergeOutputs` becomes a
   thin wrapper that calls this.

**Blocked on**: Cardinality-transparent execution (separate design doc).

### FormatDiffArtifact — structured-to-text rendering

**Where it lives**: `lib/review/src/lib.rs` (`FormatDiffArtifact`)

**Problem**: Joining a `MapStrStr` of filenames→chunks into a formatted
string is not review-specific. Gist does similar rendering. Any tool
that processes per-file data and needs a combined view would use this.

**Proposed**: Move to `lib/primitives` or `lib/markdown` as
`FormatMapOp`:

```rust
/// Join a MapStrStr into a formatted string.
/// Each entry rendered as "--- {key}\n{value}".
pub struct FormatMapOp {
    pub separator: String,     // default "\n\n"
    pub entry_format: String,  // default "--- {key}\n{value}"
    pub empty_text: String,    // default "(empty)"
}
```

---

## 2. Rendering workflows that should become DAGs

Currently, rendering is done in plain functions. These are pure
transforms (structured data → text) which is exactly what DAG ops
are for. Moving them into DAGs would make them testable, composable,
and interceptable.

### Makefile rendering

**Where it lives**: `gunbc-dag/src/makegen/render.rs` (479 lines)

**Current**: `MakefileRenderer` struct with `render_makefile()` function
and many `render_*` helpers. Implements `Renderable` trait.

**Why DAG**: Makefile rendering is a pipeline:
1. Scan workspace → crate list
2. Load tool registry → target definitions
3. Render targets → target blocks
4. Render help → help text
5. Assemble → final Makefile with header

Each step is a pure transform. As a DAG, individual render steps
could be tested, mocked, or swapped (e.g., render to Justfile
instead of Makefile).

**Complexity**: Medium. The renderer has ~15 helper functions.
Converting to a DAG means each becomes a node. Worth doing when
adding a second output format (Justfile, Taskfile, etc.).

### CI workflow rendering

**Where it lives**: `core/ir/src/transport/ci/render.rs`
and provider-specific renderers (GitHub Actions YAML).

**Current**: `CiRenderer` trait with `render()` method producing YAML.

**Why DAG**: CI rendering is already structured as:
1. Collect jobs from DAG transport nodes
2. Map to provider-specific YAML structure
3. Render YAML text

A DAG version would make provider-switching composable and testable.

**Complexity**: Low-medium. The trait is clean.

### Code generation (graph.rs, cli)

**Where it lives**: `core/codegen/src/dag_gen.rs`, `cli_gen.rs`

**Current**: Functions that generate Rust source code for DAG binaries.

**Why DAG**: Codegen is a multi-step pipeline:
1. Read tool registry → tool definitions
2. Generate GraphOp enum → Rust source
3. Generate CLI parser → Rust source
4. Generate main.rs → Rust source
5. Write files

Each step is pure. Template rendering already uses a generic
`Template` struct. Making this a DAG would let us generate for
other languages/frameworks.

**Complexity**: High. Codegen is meta-circular (the DAG generates
code that builds DAGs). Worth doing only if we need codegen for
non-Rust targets.

---

## 3. Duplicated patterns

### GraphOp wrapper enums

Every tool defines a union enum:

```
ReviewGraphOp  { Blob, Git, Review, Llm, Transport }
GistGraphOp    { Git, Gist, ..., Transport }
CIGraphOp      { CI, Prepare, Transport, CliTool }
MakegenGraphOp { Makegen, Primitive, Transport }
DepsGraphOp    { Deps, Transport }
```

**dag-pattern-ux.md Phase 4** proposes `ToolGraphOp<DomainOp>`:

```rust
pub enum ToolGraphOp<D> {
    Domain(D),
    Primitive(PrimitiveOp),
    Transport(TransportOps),
}
```

This eliminates 5+ enum definitions. Lower priority — the current
enums work, they're just boilerplate.

### Config node pattern

Review uses `LoadPipelineConfig(ReviewPipelineConfig)`. CI uses
`EnvOp::Ci`. Both are zero-input nodes that emit build-time constants.

Not worth abstracting yet — the pattern is simple enough that each
tool can implement it. Extract if a third tool needs it.

---

## 4. Rendering infrastructure that's well-placed

These are already generic and in the right location:

| What | Where | Status |
|------|-------|--------|
| `Renderable` trait | `core/ir/src/render.rs` | Generic, good |
| `Template` struct | `core/codegen/src/template.rs` | Generic, good |
| `FormatOp` / `ConcatOp` | `lib/primitives/src/data.rs` | Generic, good |
| `CollectionOp` (Map/Filter/Fold) | `lib/primitives/src/collection.rs` | Generic, good |
| `ParseOp` (JSON/TOML) | `lib/primitives/src/data.rs` | Generic, good |
| `MarkdownOp` | `lib/markdown/src/lib.rs` | Domain-specific, correct |
| `build::port/edge/optional` helpers | `core/ir/src/dag.rs` | Generic, correct |

---

## Tasks

- [ ] Extract `hash_finding_id` to `lib/primitives` as `StableHashOp`
- [ ] Unify blob hash with review hash (both should use SHA256)
- [ ] Extract `FormatDiffArtifact` to `lib/primitives` as `FormatMapOp`
- [ ] Design rendering DAG for Makefile generation (when adding Justfile)
- [ ] Design rendering DAG for CI workflow generation (when adding second provider)
- [ ] Consider `ToolGraphOp<D>` generic wrapper (dag-pattern-ux.md Phase 4)
- [ ] Split `MergeOutputs` dedup from cardinality handling (blocked on engine work)

## Notes

- "Extract when second consumer appears" — don't prematurely abstract.
  The first consumer defines the interface, the second validates it.
- Rendering DAGs are high value but not urgent. The current functions
  are pure and testable. DAGs add composability and interceptability.
- The cardinality design doc subsumes the MergeOutputs generalization.
  Once the engine handles cardinality, all merge-like ops simplify.
- `Renderable` trait is a good foundation. Rendering DAGs would use
  ops that implement `Executable`, with `Renderable` for the final
  output formatting step.
