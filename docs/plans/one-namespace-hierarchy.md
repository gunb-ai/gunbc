# One namespace hierarchy (owner direction 2026-09-25)

## The ruling

There is ONE hierarchy, and every context uses it.

A node's position is its path in the namespace graph `x.y.z`. Walking that path
from a node to the root yields every alignment level: the issue's own node,
each named ancestor, the implied nodes (the repo level — "gunbc" — and every
folder/namespace segment in between), up to the root.

- **Alignment is the namespace walk.** Working an issue nested in the
  hierarchy means aligning to each parent in turn, up from your own node. The
  alignment ladder's chain (task → projects → root) becomes a derivation over
  the namespace path, not a separately maintained relation.
- **Implied nodes are first-class levels.** "gunbc" (the repo) and every
  intermediate namespace segment exist as alignment levels even when no issue
  or project row names them. A level's existence does not depend on a row;
  rows attach to levels, levels come from the namespace.
- **A project is an instantiation of a time-bounded namespace chunk of work** —
  usually one touching a namespaced program (e.g. `gunbc.spark.*`). A project
  is not a parallel tree; it is the namespace slice
  `(subtree root, time window, work intent)`.
- **Issue hierarchy is the same graph.** Upstream/downstream (parent/child)
  relations between issues are contiguity in the namespace; a child issue's
  default parent is its namespace parent. Blockers remain the SEPARATE
  must-complete-before-start dimension (dependency edges, not hierarchy).

## What this unifies (the three graphs we currently carry)

1. The module/file namespace (`gunbc.roadmap.roadmap_page`, folder paths) —
   today implicit.
2. The alignment ladder's hand-maintained task→project→root rows
   (`roadmap_alignment.dag`) — today a parallel relation.
3. The issue tracker's parent/child fields (`extdeps.google.issue_tracker`) —
   today a third relation.

Target: one namespace graph; (2) and (3) become projections/annotations of it.

## Consequences queued

- The alignment chain derivation becomes: walk the node's namespace path,
  materialize implied levels, overlay declared rows (projects/issues) at their
  levels. `AlignmentChainRowUndeclared` then means "a row references a
  namespace level that does not exist," and ambiguity means "two declared
  owners for one level."
- The issue hierarchy (nested tickets, tracker lane) derives its nesting from
  the same walk — the nested-menu rendering is the namespace tree with issues
  attached.
- Dispatch alignment (the pending authority consolidation) consumes this:
  the brief renders every level from the node to the root, each with its
  alignment standing.
- Witness discipline: one derivation, three projections — a change to the
  namespace graph must move all three projections together; a projection that
  can move independently is a derivation defect.

## Open questions (owner, when convenient)

- Namespace identity for implied levels: is `gunbc.spark` a first-class node
  with its own standing/state, or only a waypoint that aggregates its
  descendants' standing? (Affects the cadence math and the dashboard's tree.)
- Do filesystem folders and dag module segments share one namespace by law,
  or are they two namespaces with a declared mapping? (They are currently
  close but not identical: `dag/gunbc/roadmap/` ↔ `gunbc.roadmap.*`.)
- Project time-bounds: does a project's window attach to the namespace slice
  itself, or to one instantiation row, allowing sequential instantiations of
  the same slice?
