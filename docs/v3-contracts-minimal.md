# V3 Contracts — Minimal Design

**Status**: Draft — January 2026
**Purpose**: The logically minimal representation of the V2 design.
If a statement can be removed without breaking coherence, it doesn't belong.

**Full rationale**: [`v2-contracts-design.md`](./v2-contracts-design.md)
**Problem analysis**: [`bl1-retrospective.md`](./bl1-retrospective.md)
**Worked examples**: [`v3-worked-examples.md`](./v3-worked-examples.md)
**Foundational inspiration**: [`ac.pdf`](./ac.pdf) — Abstraction Calculus

---

## Relationship to the Abstraction Calculus

V3 is an instantiation of the AC ([`ac.pdf`](./ac.pdf)). The AC defines a
domain-agnostic framework for towers of abstraction. V3 instantiates it for
causal modeling of tool behaviors. Every structural choice in V3 traces to
an AC primitive:

**The primitive.** AC defines a primitive as Π = (U, B, L, mode) — a
universe, a codomain, a lens, and a mode. In V3, a `Node<T>` is a
primitive:

- **U** (universe) = the sub-DAG inside the node (or the opaque
  implementation if it's a leaf)
- **B** (codomain) = the typed output ports
- **L** (lens) = the compilation/flattening that extracts output from
  internal structure. `ResolvedHandle` is L applied to the upsert
  sub-DAG. The lens compresses everything inside into the output type.
- **mode** = whether this node's contract is descriptive ("what the tool
  does") or normative ("what the tool must satisfy"). V1 was descriptive.
  V3 is normative. Pattern templates are the mode-bridge.

**The kernel quotient (α).** AC defines abstraction as: x ~ y iff
L(x) = L(y). In V3, two sub-DAG implementations are equivalent if they
produce the same typed output on the same typed input. This is
**fungibility** — you can swap an opaque node for a sub-DAG (or vice
versa) without breaking consumers, because the consumer sees only L's
output (the ports), not U (the internals).

**Idempotence (α∘α = α).** Once a node is opaque, further compression is
trivial — the node is already its own equivalence class. Abstracting an
already-abstract node changes nothing.

**Re-priming.** AC says: after α, use the new carrier A₁ as the next
universe, apply a new lens. In V3, this is **opening a node**: the
sub-DAG becomes the new universe, each inner node gets its own lens
(output ports), and you repeat. Each level of the tower is one
re-priming step. The tower is the iterated application of α with
re-priming.

**Contexts.** AC defines contexts C ∈ Ctx that determine the carrier and
codomain. In V3, context is the platform/environment (Linux, macOS,
CI, local). A node's sub-DAG may differ per context. The lowering phase
resolves context — it selects the relevant sub-DAG for the current
platform. **Naturality** (the lens commutes with context restriction)
means the output type is context-independent: `ResolvedHandle` regardless
of platform. **Context meet** (C₁ ∧ C₂) means composing nodes from
different contexts requires a common context; if none exists, the
composition is undefined.

**Non-triviality guard.** AC says: don't apply α when the dispersion
score H(L) drops below a floor. In V3, this is **P3: no speculative
patterns**. Don't add a level of abstraction unless it compresses
something useful. 2+ tools justify a pattern; 1 tool doesn't.

**Stratified reflection.** AC says: level k may only evaluate objects at
level < k. In V3, the compiler (modeling layer) validates pattern specs.
The executor runs opaque leaves. The executor never evaluates modeling
concepts. No circular dependencies between levels.

**Set operations.** AC says: when B is a De Morgan algebra, set-like
operations are available. In V3, `SetSpec<T>` (from V1) provides
Universal/Empty/These with proper set semantics. Already structural.

---

## What We Do

We model cause and effect.

Not objects. Not data. Not "things that exist." We model **what causes
what, in what order, carrying what information**. A DAG is not a data
structure — it is a claim about time-gated causation. Node A must
complete before Node B can begin. The edge between them carries typed
information that B needs from A. That's it.

Every system does things in order. A causes B. B causes C. If something
goes wrong, you have to be able to point at the chain and say where:

- Did something happen between A and B?
- Between B and C?
- Before A?
- Did something happen *while* B was happening that we forgot to mention?

If you can't answer these questions from the model, the model is incomplete.

---

## Why Not Just Write Code

Normal programming also has causal structure. Functions call functions.
Data flows through arguments and return values. The structure exists —
but it's **inside the code**, invisible to anything except a human reading it.

You can't ask a program "what must happen before X?" without reading the
implementation. You can't ask "if Y fails, what is affected?" without
tracing control flow. The call graph is implicit, recoverable only by
static analysis tools that are always incomplete (Turing-completeness
guarantees this).

In the fractal DAG, the causal structure **is** the data:

- Every dependency is an edge. You query it.
- Every output is a typed port. You check it.
- Every conditional skip is a declared guard. You enumerate them.
- Completeness is structural — if a dependency isn't an edge, it doesn't exist.

This is the difference between *code that runs* and *code that reasons
about itself*. A program that only runs can do anything but prove nothing.
A program that is also a model of itself can answer questions about its
own structure before execution — what's missing, what breaks if this
changes, what tests are implied, what invariants hold.

**Normal programming: humans reason about code.**
**This: code reasons about code, because the structure is the data.**

The trade-off is expressiveness. DAGs can't loop. Typed ports can't lie.
Explicit edges can't hide. These constraints make deduction possible.
We give up Turing-complete flexibility in the modeling layer to gain
machine-checkable guarantees — completeness checking, impact analysis,
test generation, invariant propagation — all derived from the graph,
not maintained by hand.

The execution layer remains Turing-complete (opaque nodes can do
anything). The modeling layer is deliberately constrained so that
the system can reason about its own structure. That's the point.

### What Can Go Wrong

The above guarantees hold only if opaque nodes honor the contract.
Three failure modes:

**1. Hidden side effects.** If Node A writes `/tmp/data.json` and
Node B reads it, but there's no edge between them, the graph says
they're independent. The scheduler may parallelize them. Race condition.
The graph is only a source of truth if nodes communicate exclusively
through declared ports. Enforcement options: sandboxed execution,
declared I/O manifests, or runtime detection of undeclared filesystem
access. Without enforcement, the graph is a suggestion, not a guarantee.

**2. God nodes.** The value of the graph is proportional to how much
structure is in the graph vs hidden inside opaque nodes. A DAG with
two nodes ("Setup" → "Run Everything") is technically a DAG but tells
you nothing. Impact analysis says "this giant node changed" — useless.
The fractal structure mitigates this (you can always open a node into
a sub-DAG), but only if someone actually does. Preventing god nodes
requires review discipline or structural limits (max complexity per
opaque node, mandatory decomposition above a threshold).

**3. Dynamic topology.** DAGs are static. Workflow automation sometimes
requires variable-sized work: "for every user, send an email." Three
options: hide the loop inside a node (lose per-item failure tracking),
unroll into N nodes (graph explosion), or support dynamic DAG generation
at runtime (lose static analysis guarantees for that subgraph). This is
a real tension. Our current instantiation uses static topology —
variable-sized work lives inside opaque nodes. If per-item causal
tracking becomes necessary, dynamic subgraph generation is the escape
hatch, with the understanding that dynamically generated subgraphs
are not statically analyzable.

**The contract of the node:** For the graph to be a source of truth,
each opaque node must be stateless across the graph boundary — all
input arrives through declared ports, all output leaves through declared
ports, no side-channel communication. The node can do anything internally
(Turing-complete), but its interface to the graph must be honest.
Without this contract, the DAG is a visualization, not a reasoning tool.

---

## Two Kinds of Cause

Every causal link has two aspects:

- **Action** (energy) — movement. "A causes B to happen." Directed,
  sequential, kinetic. Represented by **edges** in the chain.
- **Information** (matter) — structure. "What A produces *is* this."
  Static, constraining, determining. Represented by **types** on edges.

An edge without a type is "something happens next" — no constraint.
A type without an edge is "this exists" — no movement. You need both.
The edge says *something flows*. The type says *what flows*.

**Every cause must be an explicit link — both the action and the
information.** If A must happen before B, there is an edge. If A produces
something B needs, the type is declared on both sides. If B might not
happen (because A said "nothing to do"), that skip is an explicit guard
on the edge, not an implicit absence.

No implicit ordering. No implicit data flow. No implicit skips.

---

## One Type

There is one data model in the entire system:

```rust
struct Node<T> {
    id: NodeId,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
    body: NodeBody<T>,
}

enum NodeBody<T> {
    Opaque(T),          // leaf — we trust it, don't look inside
    SubDag(Dag<T>),     // recursive — same structure inside
}

struct Dag<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<TypedEdge>,
}
```

That's the whole system. There is no separate "Understanding" type, no
separate "Behavior" type, no separate "Block" type. There is `Node` and
`Dag`. A node is either opaque (you trust it) or a sub-DAG (you can
look inside, and it's the same structure).

What were previously three different representations:

| Old type | New representation |
|---|---|
| `Understanding` | `Node<ToolImpl>` with `SubDag` body containing operations |
| `Behavior` | `Node` inside a tool's sub-DAG |
| `Block` | `Node<Executable>` — opaque leaf, or sub-DAG of finer steps |
| GraphIR | `Dag<Executable>` — the result of flattening all sub-DAGs |
| `gunbai-dag` | Generic algorithms over `Dag<T>` (topo sort, waves, etc.) |

One type. One graph. Different zoom levels.

---

## One Interface (The Fractal Structure)

The system is not a stack of different machines. It is one recursive
structure. Every level has the same interface:

```
Node = typed input ports → [black box] → typed output ports
DAG  = nodes + typed edges connecting ports
```

A node is either:

- **Opaque** — a leaf. You trust it to do its job. This is your L0.
- **A sub-DAG** — same interface inside. If you look inside, you've
  moved your L0 down one level.

We are not building a tool engine and an operation engine and a block
engine. We are building a **fractal DAG engine** — one engine that
understands nodes with typed ports, all the way up and down.

**The type at a level boundary is the complete interface.** A node's
output type tells you everything you need to operate at that level.
You never reach through it. If you need to know what happened inside
a node, you're working at the level below — you've opened the sub-DAG.
That's a different stance, not a violation.

Three properties follow from one interface:

- **Uniformity**: the DSL does not change between layers. You are always
  composing nodes into DAGs.
- **Opacity**: a node is "atomic" only because you choose not to look
  inside. To `depgen`, "install git" is an atomic node. To the executor,
  it's a DAG of operations. Same node, different stance.
- **Fungibility**: because the interface (typed ports) is identical at
  every level, you can replace an opaque node with a sub-DAG (or
  collapse a sub-DAG into an opaque node) without breaking consumers.
  The consumer sees the contract, not the implementation.

This is how physics works. A gas is a fluid (L2) or a collection of
molecules (L1). The interface (pressure, volume, temperature) works at
L2 without knowing about L1. You can model the gas as one node or as
a molecular simulation. The equations at L2 don't change.

```
L3:  [tool/zstd] ----"exists"----> [tool/tectonic]
          |
          | (open the node)
          v
L2:  [Check] --[ResourceState]--> [Create] --[ResourceRef]--> [Resolve]
                |                                                  |
                | (open the node)                                  |
                v                                                  v
L1:  [Block: "which zstd"] --[Output]--> ...              [Block: "zstd --version"]
                |
                | (open the node — or don't, this is our current L0)
                v
L0:  (opaque: we trust the shell)
```

Every box is the same shape. Every arrow is the same kind of thing.
The only difference between levels is *what you choose to model*.

---

## The Tower

Causal structure forms a tower of levels. Each level compresses the
action/matter of the level below into typed matter for the level above.

There is no privileged base. **L0 is wherever you choose to stop looking**
— the opacity boundary where you say "I trust everything below here."
The tower doesn't have a natural bottom. It has a *chosen* bottom.

From your chosen L0, you build upward:

- **Each level is one abstraction step** (the AC's α — kernel quotient).
  It takes the action/matter detail below and presents compressed typed
  matter above.
- **You add levels where you need causal structure.** If you need to
  reason about what happens *inside* a node at level k, you decompose it
  into a sub-DAG at level k-1.
- **You stop when compression stops paying.** When further decomposition
  doesn't help you answer causal questions, you've found your top.

### Our instantiation

We currently instantiate the tower with these levels:

| Level | What it captures | Unit of matter | Unit of action |
|---|---|---|---|
| L0 | Shell commands, API calls | Exit codes, bytes, responses | "Execute this" |
| L1 | Execution blocks | Typed ports (Output, ResourceRef) | Block-to-block data flow |
| L2 | Operations within a tool | Typed state (ResourceState) | Causal edges (check → create) |
| L3 | Tool dependencies | "Tool exists" | "Install before" |

L0 and L1 exist (GraphIR + `gunbai-dag`). L3 exists (`depends_on` graph).
**L2 was missing.** That's what we're adding.

But with one type (`Node<T>` / `Dag<T>`), the "levels" are just how deep
you've opened the sub-DAGs. There's no code that knows about levels.
There's just the recursive structure.

### Why L2 was the gap

We built L0→L1 (execution engine) and L3 (tool dependencies), then jumped
from L3 to L1 by converting each behavior into an isolated block
(`to_blocks()`). That jump skipped L2 — the causal structure *within*
a tool.

The fix isn't "add L2." The fix is "every node is a sub-DAG or opaque."
If we'd had one recursive type from the start, L2 wouldn't have been
skippable — opening a tool node would have forced you to provide its
internal DAG.

---

## Patterns Are Reusable Sub-DAG Templates

A pattern is a sub-DAG template — a reusable shape that tools fill in.
Upsert is:

```
Check  --[ResourceState]--> Create  --[ResourceRef]--> Resolve --[ResolvedHandle]-->
            |                                              ^
            |  (guard: only if Missing)                    |
            +--- if Exists, skip Create ------------------+
```

The template defines the shape: nodes, edges, types, guards. A tool
fills in what each node actually does. Behaviors are the nodes produced
by instantiation — not an author-written flat list.

From the level above, the entire upsert chain is one node with output
type `ResolvedHandle`. Same interface. You don't need to know
check/create/resolve happened.

---

## What This Means Concretely

### 1. If it's not in a chain, you have to say so explicitly

Every tool must address every known pattern: either "I implement this
chain" (with all nodes filled in) or "this chain doesn't apply to me"
(explicit decision, not silence).

### 2. No freeform names for things that matter

If generators, validators, or tests consume a field, that field is a
typed value — an enum, a validated ID, a declared code. Not a string.
Strings are for documentation only.

### 3. Every claim has proof

If you say a behavior is read-only, you say how that's verified: a
generated test, a harness, a derivation from other proven properties,
or a justified exception with an owner and expiry.

---

## The Compiler

The compiler is a recursive traversal of the DAG-of-DAGs:

```
Author writes:  "this tool implements upsert; here are the bindings"
                → Node<ToolImpl> with SubDag body
                         ↓
Modeling:       validates each sub-DAG is complete (all nodes, all types, all edges)
                         ↓
Lowering:       recursively flattens SubDag bodies into Opaque leaves;
                resolves guards into skip predicates; strips irrelevant branches
                → Dag<Executable> (all nodes are Opaque)
                         ↓
Execution:      gunbai-dag runs the flat Dag in dependency order
                (sees only nodes and edges — no levels, no patterns, no recursion)
```

The execution engine sees one flat DAG of opaque nodes. The fractal
structure exists only at authoring and compilation time.

---

## What's Banned

- Incomplete sub-DAGs (upsert without resolve = type error)
- Implicit skips (guarded nodes declare their guard)
- String-typed semantic fields (enums or declared codes, not `&str`)
- Convention-based protocols (no prefix parsing)
- Unverified property claims (every claim carries proof strategy)
- Nodes without causal provenance (no flat lists, no bag of behaviors)
- Speculative patterns (2+ tools before formalization)
- Leaky level boundaries (a node's output type is the complete interface)
- Separate type systems per level (one `Node<T>` / `Dag<T>`, not
  Understanding + Behavior + Block)

---

## Extension

New semantic values: declare a typed code (Lane B), use it. If 2+ tools
use it, promote to core enum (Lane A). Strings are Lane C (docs only).

New patterns: same rule. Declare it typed, implement one tool. Second
tool triggers promotion.

New levels: open a node into a sub-DAG. Same type. Same interface.
The framework doesn't change — there's nothing to change.

First tool is never blocked. Core vocabulary never bloats without evidence.
The tower grows when the domain demands it, not before.
