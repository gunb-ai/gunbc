# Dry-Run via Sentinel World-Write Nodes

## Context

Dry-run is currently implemented by manually threading a `dry_run` flag through
per-tool graph builders (e.g., swapping `WriteFile` to `PrintStdout`). This
creates duplicated logic and makes it easy to miss new world-writing operations.

The IR treats unconnected output ports as exports, not world writes. Therefore,
"dangling outputs" cannot be used to infer side effects. We need an explicit and
consistent representation of world writes.

## Goals

- **Deepest dry-run**: execute the graph as far as possible and only swap the
  final world-writing operations.
- **Small, explicit set of world-write nodes** that are easy to find and audit.
- **Enforced preview**: every world-write operation must have a dry-run variant.
- **Graph-time decisions**: dry-run should be applied by a shared transform
  before execution, not via runtime flags inside ops.
- **Safe default logging**: in dry-run, terminal output is assumed safe and can
  be enabled for every node via a shared observer.

## Non-Goals

- Inferring world writes from unconnected outputs (incompatible with IR rules).
- Reworking the executor or lowering model.
- Defining a single universal policy for which writes are safe; this will evolve.

## Options Considered

### Option A: Boundary metadata with effect (Read/Write)

Add `BoundaryEffect` to `BoundaryDeclaration` and treat any `Write` boundary as a
world-write. Dry-run swaps ops that sit on write boundaries.

**Pros**: Minimal graph changes; reuses existing metadata.
**Cons**: Non-structural; easier to forget; harder to validate without tooling.

### Option B: Op-level `writes_world()` + `to_preview()`

Each op declares whether it writes and provides a preview variant.

**Pros**: Local, easy to implement incrementally.
**Cons**: Scales poorly; no single structural indicator in the DAG.

### Option C: Sentinel world-write nodes (preferred)

Introduce a small, explicit set of nodes that represent world writes (e.g.
`HTTP::Request`, `TCP::Connect`, `FS::Write`, `Process::Exec`). All real-world
writes are funneled through these nodes. Dry-run swaps only these nodes.

**Pros**: Structural, explicit, easy to audit; a small set of known nodes.
**Cons**: Requires consistent insertion; new external writes must add a sentinel.

## Proposed Direction

Adopt **sentinel world-write nodes** and a dry-run transform that swaps only
those nodes to preview variants. The transform runs at graph-time (pre-exec).

### Dry-Run Behavior

- **Deepest dry-run**: execute the entire graph, swapping only sentinel
  world-write nodes to preview variants.
- **Default logging**: in dry-run, a shared observer logs node execution and
  summaries to stdout (safe by assumption).
- **Preview enforcement**: if a sentinel node lacks a preview variant, dry-run
  fails fast.

### Sentinel Node Examples

- `External::TCP::Connect`
- `External::HTTP::Request`
- `External::FS::Write`
- `External::Process::Exec` (if command execution is a write)

## Tradeoffs

- **Explicitness vs. flexibility**: sentinel nodes make writes visible and
  auditable but require discipline to route all writes through them.
- **Deepest dry-run** works well for most cases, but some write steps may be
  semantically required to generate IDs or side effects; those writes must be
  simulated (preview op) or explicitly permitted.
- **Observer logging** provides a uniform dry-run trace without forcing every
  op to print.

## Open Questions

- Final list of sentinel nodes (HTTP/TCP/FS/Process? DB? GitHub Gist?).
- Whether to keep `BoundaryDeclaration` as supplemental metadata or replace it
  with sentinel nodes entirely.
- How to represent "preview writes" in metadata (if retained).
- Policy for writes that are safe/necessary even in dry-run (temp, reversible).

## Next Steps

1. Decide on the minimal sentinel set and naming conventions.
2. Add a dry-run transform that swaps sentinel nodes to preview variants.
3. Add a dry-run observer that logs node execution by default.
4. Update one tool (e.g., makegen) to use the shared dry-run transform.
