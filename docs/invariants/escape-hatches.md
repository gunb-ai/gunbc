### Escape Hatches (why violations keep recurring)

Each invariant below has a **structural prevention** that makes
violations unrepresentable. But violations keep recurring because the
codebase still has escape hatches — API surfaces where the wrong thing
is easy and the right thing is hard. Five escape hatches account for
the majority of all recurring violations:

| Escape hatch | What it enables | Structural fix |
|---|---|---|
| `String` return type in emitter | Hardcoded target syntax | Graph rendering — emitter walks graph, renderer produces strings |
| `node.name` field | Name-based dispatch anywhere | Delete `Node.name` — structural properties + edges only |
| `List<String>` fact storage | Copied string lists that go stale | `List<Node>` edges to definitions |
| Error sentinels in `Node` | Fabricated valid-looking error output | Typed wrappers (`InferredNode` pattern) at every boundary |
| Hand-editable generated code | Parallel implementations that diverge | Committed binary + regenerate→diff→empty CI gate |
| Raw `Node` in type rendering | Shape-based heuristic dispatch (connective/children guessing) | `TypeRendering` descriptor — precomputed, unambiguous, fail-closed |
| Adapter / bridge functions | Transitional state between old and new representations calcifies into permanent shape | Rework every consumer in the same PR as the representation change — no adapter ever lands |

Eliminating these six surfaces makes the invariants self-enforcing.
The invariants become properties of the API, not rules you have to
remember.

Active liabilities and their measured costs are tracked in the
**Open Debt** section at the bottom of this file.

The invariant headings in this document are also the canonical theme
labels for ratchets, review feedback, and queue planning. A review queue
branch must declare exactly one primary theme from this list and stop
before taking a second review item from a different theme, so CI
failures stay attributable to a single ratchet. Review queue branches
must also keep each commit strictly scoped to that invariant fix: no
unrelated helper cleanup, dead-code removal, or opportunistic
refactoring unless it is directly required for the fix to compile and
pass tests.

