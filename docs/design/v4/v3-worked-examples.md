# V3 Worked Examples

**Status**: Draft — January 2026
**Purpose**: Concrete examples of the fractal DAG model from
[`v3-contracts-minimal.md`](./v3-contracts-minimal.md). Shows what tools
look like when Understanding, Behavior, and Block are all the same type.

---

## The One Type

Every example uses the same structure:

```rust
struct Node<T> {
    id: NodeId,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
    body: NodeBody<T>,
}

enum NodeBody<T> {
    Opaque(T),
    SubDag(Dag<T>),
}

struct Dag<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<TypedEdge>,
}
```

---

## Example 1: `tool/zstd` — Simple Tool (One Pattern)

### At L3 (tool dependency graph)

Other tools see zstd as one opaque node:

```
Node {
    id: "tool/zstd",
    inputs: [Port("os", PlatformKind)],
    outputs: [Port("binary", ResolvedHandle)],
    body: SubDag(...)   // ← we can open this
}
```

From L3's perspective, zstd takes a platform and produces a binary handle.
That's the complete contract. Consumers (like tectonic) connect to
`outputs["binary"]` and never look inside.

### Open the node → L2 (operations)

Inside, it's an upsert pattern — a sub-DAG of three nodes:

```
Dag {
    nodes: [
        Node {
            id: "tool/zstd/check",
            inputs: [],
            outputs: [Port("state", ResourceState)],
            body: SubDag(...)   // ← can open further
        },
        Node {
            id: "tool/zstd/create",
            inputs: [Port("state", ResourceState, guard: Equals(Missing))],
            outputs: [Port("ref", GuardedOutput<ResourceRef>)],
            body: SubDag(...)
        },
        Node {
            id: "tool/zstd/resolve",
            inputs: [
                Port("state", ResourceState),
                Port("ref", GuardedOutput<ResourceRef>),
            ],
            outputs: [Port("handle", ResolvedHandle)],
            body: SubDag(...)
        },
    ],
    edges: [
        "check/state" → "create/state",
        "check/state" → "resolve/state",
        "create/ref"  → "resolve/ref",
    ],
}
```

Same `Node` type. Same `Dag` type. The upsert pattern is just a
particular DAG shape — three nodes, specific port types, one guard.

### Open "check" → L1 (execution blocks)

```
Dag {
    nodes: [
        Node {
            id: "tool/zstd/check/which",
            inputs: [],
            outputs: [Port("output", CommandOutput)],
            body: Opaque(ShellCommand("which zstd"))   // ← L0, we stop here
        },
        Node {
            id: "tool/zstd/check/parse",
            inputs: [Port("output", CommandOutput)],
            outputs: [Port("state", ResourceState)],
            body: Opaque(ParseExitCode {
                success: ResourceState::Exists,
                failure: ResourceState::Missing,
            })
        },
    ],
    edges: [
        "which/output" → "parse/output",
    ],
}
```

Still the same types. `Opaque(ShellCommand(...))` is our chosen L0 —
we trust the shell to execute it. We could open it further (syscalls,
kernel scheduling) but we choose not to.

### Open "create" → L1

```
Dag {
    nodes: [
        Node {
            id: "tool/zstd/create/download",
            inputs: [],
            outputs: [Port("archive", FilePath)],
            body: Opaque(ShellCommand("curl -L https://... -o /tmp/zstd.tar.gz"))
        },
        Node {
            id: "tool/zstd/create/extract",
            inputs: [Port("archive", FilePath)],
            outputs: [Port("binary", FilePath)],
            body: Opaque(ShellCommand("tar xzf {archive} && mv zstd /usr/local/bin/"))
        },
    ],
    edges: [
        "download/archive" → "extract/archive",
    ],
}
```

"Install" is not a separate pattern. It's upsert's Create node opened
into a sub-DAG. Download causes extract. The archive flows between them.
Same type all the way down.

---

## Example 2: `tool/tectonic` — Tool With Dependencies

### At L3

```
Dag {
    nodes: [
        Node { id: "tool/zstd",     ..., outputs: [Port("binary", ResolvedHandle)] },
        Node { id: "tool/tectonic",  ..., inputs: [Port("zstd", ResolvedHandle)],
                                          outputs: [Port("binary", ResolvedHandle)] },
    ],
    edges: [
        "zstd/binary" → "tectonic/zstd",   // tectonic depends on zstd
    ],
}
```

The dependency between tools is the same kind of edge as the dependency
between operations within a tool, and the same kind of edge as the
dependency between blocks within an operation. **One edge type.**

### Open "tectonic" → L2

Same upsert shape as zstd. Different bindings (different check command,
different install steps). The pattern is reused. The structure is
identical:

```
[Check] --[ResourceState]--> [Create] --[ResourceRef]--> [Resolve]
```

Each of these nodes can be opened into L1 sub-DAGs. Tectonic's Create
might involve downloading a release binary, verifying a checksum, and
symlinking — three nodes at L1, connected by typed edges. From L2,
Create is one node with output `ResourceRef`. From L1, it's a DAG.

---

## Example 3: `tool/gh` — Mixed Concerns (Upsert + Capabilities)

### At L3

gh is one node: inputs `[os]`, outputs `[binary, auth_token]`.

Two outputs — the binary itself (from upsert) and the auth capability
(from a separate sub-DAG).

### Open → L2

Two sub-DAGs inside, composed independently:

```
Dag {
    nodes: [
        // Upsert sub-DAG (install gh)
        Node { id: "gh/upsert/check",   ... },
        Node { id: "gh/upsert/create",  ... },
        Node { id: "gh/upsert/resolve", ... },

        // Capability: auth
        Node {
            id: "gh/auth/check-token",
            inputs: [],
            outputs: [Port("status", AuthStatus)],
            body: Opaque(ShellCommand("gh auth status"))
        },
        Node {
            id: "gh/auth/login",
            inputs: [Port("status", AuthStatus, guard: Equals(NotAuthenticated))],
            outputs: [Port("token", GuardedOutput<AuthToken>)],
            body: Opaque(ShellCommand("gh auth login"))
        },
    ],
    edges: [
        // upsert internal edges
        "upsert/check/state" → "upsert/create/state",
        "upsert/check/state" → "upsert/resolve/state",
        "upsert/create/ref"  → "upsert/resolve/ref",
        // auth internal edges
        "auth/check-token/status" → "auth/login/status",
        // cross-concern: auth depends on binary existing
        "upsert/resolve/handle" → "auth/check-token/requires_binary",
    ],
}
```

The auth capability is itself a causal chain — check token status, then
conditionally login. It has the same shape as upsert (check → guarded
create) but it's a different pattern. From L3, both sub-DAGs collapse
into the gh node's two output ports.

Note the cross-concern edge: auth depends on the binary existing. This
is just another typed edge. No special "composition" mechanism. The DAG
handles it.

---

## What Disappeared

| V1/V2 concept | V3 representation |
|---|---|
| `Understanding` struct | `Node<ToolImpl>` with `SubDag` body |
| `Behavior` struct | `Node` inside a sub-DAG |
| `Block` struct | `Node<Executable>` — leaf or sub-DAG |
| `behaviors: &[Behavior]` (flat list) | `Dag { nodes, edges }` (causal graph) |
| `upsert_phase: Option<UpsertPhase>` | Node position in the upsert sub-DAG |
| `depends_on: &[Dependency]` | Typed edges in the L3 DAG |
| `to_blocks()` (skip L2) | Recursive flattening — every level is traversed |
| GraphIR (separate type) | `Dag<Executable>` — same type, all nodes opaque |
| `PatternUse<T>` / `NotApplicable` | Sub-DAG body present or node is opaque — same enum |
| `CompositionSpec` | Just edges between nodes in the same DAG |

The entire composition/pattern/behavior vocabulary reduces to: **nodes,
edges, sub-DAGs, and opaque leaves.** Everything else was scaffolding
for a missing recursive type.

---

## The Key Observation

In V1, the tool definition and the execution graph were different types
with a lossy bridge (`to_blocks()`). In V3, they're the same type at
different zoom levels. There's no bridge because there's no gap. You
just keep opening nodes until you hit opaque leaves, then you execute.

The "levels" aren't architectural layers. They're how deep you've
recursed into `NodeBody::SubDag`. The system doesn't know about levels.
It knows about nodes.
