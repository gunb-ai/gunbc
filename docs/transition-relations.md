# Transition Relations: The Upstream Unification

Part of: [ROADMAP.md §PERF](../ROADMAP.md#perf-eliminate-unnecessary-work) |
[THESIS.md §Concept unification](../THESIS.md#concept-unification) |
[INVARIANTS.md](../INVARIANTS.md)

## The insight

Multiple emitter "exclusions" currently modeled as independent
side-tables — TCO, callable-set, match-bound, owned-after-unwrap,
field-access style — are all consumers of the same upstream fact
that doesn't exist yet: **the per-param transition relation on
recursive/call edges.**

## What a transition relation is

A normal function model says:
```
enter fresh frame → run body → return
```

That's the static view. But a recursive call, especially a tail
call, is actually:
```
update frame state vector → iterate
```

The difference isn't a flag on the function. It's a property of
the **edge** from caller to callee. And that edge has per-parameter
facets:

| Facet | Meaning | Ownership implication |
|-------|---------|----------------------|
| **Reassigned** | Param's slot is overwritten with a new value at the next iteration | Needs owned (moved, not borrowed) |
| **PassThrough** | Param stays the same across iterations (fold accumulator style with no update) | Can borrow |
| **Consumed** | Param's value is moved into a child operation and not reused | Moves at use site |
| **Fresh** | Param was freshly constructed at the call site | Owned, new value |

This is the language of **frame transitions** — what happens to
each slot across a call. A function's whole behavior is the set
of transition relations on its call edges.

## Why this unifies current exclusions

### TCO

Current model: `is_tco_eligible: Bool` flag on function items.
Emitter: if set, disable borrowing for all params, emit loop with
reassignment.

What TCO actually means: the recursive edge transitions some
params via `Reassigned` and others via `PassThrough`. Only
`Reassigned` params need ownership. Currently we paint the whole
function owned because we lack the per-param view.

With transition relations: emission reads per-param. `Reassigned`
params emit owned. `PassThrough` params emit borrowed. No
function-level flag, no exclusion.

### Callable-set exclusion

Current model: `callable_set` collected by scanning expressions
for value-level function references. If a function is in the set,
its params can't be borrowed (would change the signature).

What "callable" actually means: this function has an edge of a
different kind — **closure capture** rather than direct call.
The transition for that edge type is inherently different
(captured by value, frozen signature).

With transition relations: closure-capture edges are a distinct
transition type. Params observed by capture edges can't borrow.
Params observed by direct-call edges can. Same function, different
decisions per edge.

### Last-use move

Current model: `movable` set for bindings with `fan_out == 1`.
Emitter moves at that single use site.

What last-use actually means: the use is a `Consumed` transition
— the value flows into an operation and doesn't return. For
`fan_out == 1`, the single use is necessarily consumed. For
`fan_out > 1`, only the terminal use is consumed; earlier uses
are `PassThrough`. Currently we only model `fan_out == 1`.

With transition relations: each use site gets a transition facet
(`Consumed`, `PassThrough`, `Shared`). The terminal-use analysis
is the facet computation. No separate `movable` set.

### Match-bound names clone

Current model: match-bound bindings are flagged as "must clone"
because Rust can't move out of a pattern destructure of a shared
owner.

What match-binding actually means: the transition from the
scrutinee to the bound name is **projection from a shared owner**.
The binding is a reference to a field of a value that outlives
the binding's scope. In transition terms: the binding is `Shared`
(not `Consumed`), so it can't be moved.

With transition relations: the scrutinee-to-binding transition
carries `Shared` explicitly. Emission reads it and clones
correctly. Not a special case — a structural consequence.

### Owned after `Rc::try_unwrap`

Current model: after `Rc::try_unwrap`, the binding is marked
owned and can be moved.

What this actually means: the `try_unwrap` operation IS a
transition — from `Shared` (Rc ref) to `Consumed` (unique
owner). The transition relation changes along the data flow.

With transition relations: transitions flow through operations.
`try_unwrap` produces an owned binding. Emission reads the
current transition state per binding, not a separate set.

### CX descent evidence

Current model: `SubValueRelation` per argument, `DescentEvidence`
per call site. Classified per-param in isolation.

What descent evidence actually means: it's the structural
component of the transition — "is this param a sub-value of
the corresponding input?" It IS a facet of the transition
relation, just not called that.

With transition relations: descent evidence is a field of the
per-param transition. CX proof construction (M1 Step 3) is
literally building transition relations on recursive edges.
Same data, unified name.

## The pattern

Every current "exclusion" is evidence of a missing upstream fact.
The exclusions cluster because they're all downstream consumers
of the same fact — **what happens to each param across a call
edge** — which currently doesn't have a home.

The sustainable version:

```
Transition relation (upstream, per edge, per param)
      │
      ├─→ CX descent evidence (M1 Step 3)
      ├─→ Ownership movable / last-use (Stream B)
      ├─→ Emission borrow / move / clone decisions
      ├─→ TCO loop-with-reassignment lowering
      └─→ Closure capture signature freezing
```

One upstream fact. Multiple mechanical consumers. No exclusion
lists. Adding a new consumer means reading the transition, not
creating a new side-table.

## Why TCO should eventually disappear as an explicit model

Under the transition relation framing, "TCO-eligible" isn't a
property a function has. It's a consequence:

> A function is TCO-eligible if and only if every recursive call
> on it is a tail position AND the resulting transition relation
> allows frame-reassignment lowering.

Both conditions are structural. Neither requires a flag. The
emitter's TCO rewrite becomes: "for any tail-position recursive
call edge with a `Reassigned` transition vector, emit as a
loop update." Functions where every call satisfies this emit as
loops. Functions where some calls don't fall back to normal
calls. No `is_tco_eligible` boolean.

**The eventual refactor:** delete the explicit TCO detection
pass. Replace it with a **regression test**: "tail-recursive
functions in this test suite still emit as loop lowering." If
the test passes, TCO works. If it fails, the transition relation
or the emission strategy is wrong. The test verifies emergent
behavior, not explicit modeling.

This is the thesis principle in action: model the root fact,
get the behavior for free, verify by regression test that the
free behavior still emerges.

## Connection to existing work

### M1 Step 3 (lexicographic proof construction)

The path to M1 Step 3 was framed as "wire TerminationProof from
`std/termination.dag` into CX, replacing per-argument heuristic
classification with proof constructors."

That work IS the transition relation. The lexicographic proof is
literally a multi-dimensional transition vector: "dimension 1
decreased, dimension 2 non-increasing" is the same as "param 1
was `Consumed`, param 2 was `PassThrough`."

M1 Step 3 was scoped as CX-only. Reframed as transition
relations, it unlocks emission wins too.

### Stream D (-3 vs expected -137)

Stream D merged with the expectation that per-field provenance
would fire on shrinking token lists, dissolving 132+ parser
violations. Actual impact: -3.

Why? Because the parser now passes shrinking lists, but CX still
classifies per-argument in isolation. It sees "tokens argument
changed" without seeing "the transition is `Consumed`, producing
a sub-list." The structural relationship is there; CX can't
compose it.

**The transition relation is the missing piece.** With it, Stream
D's benefit materializes: CX sees the full transition vector on
each parser recursive call, proves descent by lexicographic
composition, and the -137 follows.

### Stream B clone elision

Stream B Layer 1 (last-use move) and Layer 3 (borrow propagation)
are both consumers of transition relations. Layer 1 wants
`Consumed` vs `PassThrough` per use. Layer 3 wants `PassThrough`
vs `Reassigned` per param. Both want the same upstream data.

Currently they're designed as separate passes. Under transition
relations, they're one pass (compute the transitions) with
multiple consumers (emit reads them).

## What this doesn't promise

Transition relations are **not a silver bullet for the perf
crisis.** The perf crisis is dominated by heap allocations
(String clones, mostly), not by missing borrow decisions. The
perf session should continue on M2 Node.name deletion — that's
the highest heap-allocation source and doesn't depend on this
framing.

Transition relations are the **correct long-term model** for
the CX, ownership, and emission passes. They dissolve the
current "exclusion" clutter. But they require designing and
wiring a new upstream concept, which is a significant piece of
work.

## Status

**Design direction.** This doc captures the framing. Implementation
is downstream of:

- M2 Node.name deletion (immediate perf, independent)
- M1 Step 3 (lexicographic proof construction — the CX side of
  transition relations)
- Stream B Layer 1 redesign (last-use as `Consumed` facet)

When enough of the downstream work is done with this framing in
mind, the transition relation emerges as the upstream model that
all of them share.

**Not:** an immediate implementation target. The perf session
should not stop and implement this before continuing Node.name
deletion.

**Is:** the framing for how CX, ownership, and emission should
relate going forward. New work in any of these areas should ask:
"is this consuming an upstream fact, or inventing a new
side-table?" If the latter, reconsider.
