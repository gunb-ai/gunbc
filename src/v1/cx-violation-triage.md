> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1: Structural** (CX gate)
> See also: [cx-design.md](cx-design.md), [cx-computation-model.md](cx-computation-model.md)

# CX Violation Triage: 526 → 0

## Current state

526 complexity violations. Each is a function where the analyzer
produces CostUnknown — it can't determine the recursion descent pattern
and falls back to `SameArgumentCall → Forever → CostUnknown`.

## Root cause analysis

The 526 violations decompose into TWO categories:

### Category 1: Direct unknowns (280 functions)

Functions that ARE recursive but the analyzer can't recognize their
descent pattern. These are the PRIMARY violations — the actual analyzer
gaps.

| Group | Count | Root cause |
|-------|-------|-----------|
| parse_* | 73 | Parser SCC: mutual recursion bounded by token consumption. Analyzer can't thread TokenPosition evidence across SCC boundaries. |
| emit_* (shared) | 67 | Emitter recursion on Node trees. These walk `texpr.children` or `node.children` recursively. Analyzer doesn't recognize `children` as structural descent (needs `is_child_accessor_in_model`). |
| emit_go_* | 31 | Same as shared emitters — Node tree walks. |
| emit_rust_* | 25 | Same. |
| serialize_* | 10 | Serializer walks Node trees recursively. Same root cause as emitters. |
| resolve_* | 9 | Resolver walks module/type graphs. Similar to emitter — structural descent on graph nodes. |
| node_*/walk_* | 5 | Explicit Node tree walkers. |
| cost_*/classify_* | 4 | Complexity analyzer's own recursive functions (cost_of_expr walks Node trees). |
| collect_* | 4 | Collection-walking helpers. |
| Other | 52 | Mix: DFS (2, graph recursion), template application, type walkers, miscellaneous tree recursion. |

**Single root cause for ~230 of 280:** Recursion on Node tree structure
(children/fields) where the analyzer doesn't recognize `children` as
a structural descent witness. These functions ARE O(n) where n = tree
size, but the analyzer sees `SameArgumentCall` because it can't connect
`node.children` to TreeSize descent.

**Single root cause for ~73 of 280:** Parser mutual recursion where
TokenPosition evidence isn't threaded through SCCs.

### Category 2: Composed unknowns (227 functions)

Functions that are NOT themselves recursive, but CALL functions from
Category 1. Their costs include `?` because a callee has CostUnknown.

| Group | Count | Cause |
|-------|-------|-------|
| emit_* (shared) | 68 | Call emit helpers that walk Node trees |
| parse_* | 53 | Call parser functions in the SCC |
| emit_go_* | 26 | Call shared emitters |
| emit_rust_* | 6 | Call shared emitters |
| Other | 71 | Callers of recursive functions |

**These resolve automatically** when Category 1 is fixed. They have
no independent root cause — they're transitive consumers of unknown costs.

### Remaining (19 functions)

19 functions with composed unknowns but no `where` clause. These are
simpler compositions where the unknown callee is the dominant term.

## Fix path: 3 structural improvements → ~0 violations

### Fix 1: Node tree descent recognition (~230 violations)

**Problem:** Functions like `cost_of_expr(texpr)`, `emit_typed_expr(texpr)`,
`serialize_node(node)` recurse on `texpr.children` or `node.children`.
The analyzer sees the recursive call but can't determine that `children`
is a STRUCTURAL descent (each child is a subtree, strictly smaller).

**Root cause:** `collect_descent_vars` doesn't recognize field access
on `children` as descent evidence. It looks for specific patterns
(Option unwrap, list skip, arithmetic) but not "iterate over structural
children of a Node."

**Fix:** Wire `is_child_accessor_in_model` (or equivalent) into the
descent evidence collection. When a function iterates over `node.children`
via fold/map and recurses on each child, the descent evidence is
`TreeSize { param: "texpr" }` or equivalent.

**Estimated impact:** ~230 direct violations + ~200 composed = ~430 total.

### Fix 2: Parser SCC TokenPosition threading (~73 violations)

**Problem:** Parser functions form a large SCC (mutual recursion).
Each function advances the token stream, but the SCC analysis can't
compose TokenPosition evidence across mutual call boundaries.

**Root cause:** `classify_scc_recursion_pattern` collects evidence
per-member but can't prove that the COMBINED SCC advances tokens
on every cycle. Individual members may have `ProgressSame` on some
paths while relying on callees to advance.

**Fix:** Thread `parser_always_advancing` evidence through the SCC
cycle detection. If every cycle through the SCC includes at least one
member with `ProgressStrict` on TokenPosition, the SCC is bounded
by `|tokens|`.

**Estimated impact:** ~73 direct + ~53 composed = ~126 total.

### Fix 3: Graph DFS worklist (2 violations)

**Problem:** `dfs_finish_order` and `dfs_collect_component` recurse
on graph neighbors with a visited set. Bounded by |V| but the .dag
language can't express "visited set size decreases."

**Fix:** Worklist-based iteration (requires `repeat` primitive from
ROADMAP I1/I2). Or: accept these 2 as known-bounded and annotate
with ExplicitCount.

**Estimated impact:** 2 direct + cascading through graph utilities.

## Summary

| Fix | Direct | Composed | Total | Blocked on |
|-----|--------|----------|-------|-----------|
| Node tree descent | ~230 | ~200 | ~430 | is_child_accessor_in_model |
| Parser SCC tokens | ~73 | ~53 | ~126 | SCC TokenPosition threading |
| Graph DFS worklist | 2 | ~10 | ~12 | repeat primitive (I1/I2) |
| **Total** | **~305** | **~263** | **~568** | |

Note: totals exceed 526 because some composed violations resolve from
multiple fixes. The actual reduction will be measured by the ratchet.

## Design principle

Each fix makes a CATEGORY of violations structurally impossible:
- After Fix 1: no function that recurses on Node.children can produce CostUnknown
- After Fix 2: no parser SCC member can produce CostUnknown
- After Fix 3: no graph DFS function can produce CostUnknown

The violation count is the honest measure. It ratchets down as each
category is eliminated. When it reaches 0, CostUnknown can be deleted
from CostExpr — because no code path can produce it.
