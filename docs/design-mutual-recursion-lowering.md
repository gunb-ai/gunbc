> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 3 Stage 3a.1

# Design DB-9 R2 — Mutual recursion lowers to substrate Loop

**Design blocker:** DB-9
**Consumers:** Lane 3 Stage 3a.1 (M2 feature parity — mutual recursion)
**Status:** Approved (R2.1). Substrate records use Track 9 integrity primitives (`NonEmptyList`, `NonSingletonList`, `ParamRef`, `TransformRef`). Supersedes DB-9 R1.
**Doc revision:** R2.1 — 2026-04-17

---

## Summary

Mutual recursion lowers at `lower.rs` time into a `Behavior::Loop` whose `bound` carries a new `Descent` variant pointing at a `Cluster` in a `Dag.clusters` sidecar table. Each member function's body keeps real `Transform` edges to its real callees — call topology is preserved. The substrate gains **one coproduct-variant addition** (`LoopBound::Descent`), **one field on the reflected `type Dag` + matching Rust `Dag` struct** (`clusters: List<Cluster>`), and three terminal types (`Cluster`, `MemberDescent`, `IntraClusterCall`); **no new `Behavior` variant**. `IntraClusterCall` is a typed handle into the authoritative `Transform` node — not a caller/callee compression. Lenses (termination, complexity) and the inference pass read cluster membership from the sidecar via `ClusterId` and per-edge structure through the `Transform` handle — single authority, no re-derivation.

---

## Constraints (non-negotiable)

1. **INVARIANTS:555** — *"Mutual recursion (SCC) on children → `descend` over SCC | Bounded by |SCC|"*. Lowering produces a bounded Loop.
2. **INVARIANTS:665** — *"|SCC| with shared measure → `descend` over SCC-ordered nodes"*. The bound carries the shared measure.
3. **SELF_HOSTING §2.4** — *"Real call topology preserved"*: each function's `Transform.target` points at the real callee declaration, not a synthetic ring.
4. **THESIS:604-629** — Five `Behavior` variants. No sixth. R2 uses the existing `Loop`; no new `Behavior` variant.
5. **Invariant D-2 (single authority)** — `compute_mutually_recursive` in `lower.rs` is the sole SCC computation in v3. Downstream consumers read the sidecar; they do not re-detect.

---

## Design

### Substrate changes

**Extend `LoopBound` to a coproduct** (`src/v3/std/substrate.dag`):

```dag
// 🟢 TERMINAL. LoopBound distinguishes runtime-cardinality loops
// from compile-time structural-descent (SCC) loops. Each variant
// carries exactly the carrier it needs; illegal combinations
// (Cardinality with members, Descent with a count port) are
// unrepresentable. Dissolution receipt stamped in-doc below.
type LoopBound
  = Cardinality { count: PortId }        // existing: fold / descend-on-source
  | Descent { cluster: ClusterId }       // new: SCC cluster descent

// 🟢 TERMINAL. Handle into Dag.clusters sidecar.
type ClusterId = Int

// 🟢 TERMINAL. Structural descent witness for an SCC. Members stay
// as ordinary peer Binds in Dag.nodes; their bodies' Transform
// edges remain the authoritative call topology. This record is
// a typed index over that authority, not a copy of it — it names
// which members the SCC contains and gives lenses a direct handle
// list for the N intra-cluster call-site Transforms so they can
// point diagnostics at the exact failing Transform without
// re-scanning bodies.
//
// Field types use the substrate integrity primitives introduced in
// the Track 9 graduation (see `src/v3/std/substrate.dag`) — illegal
// states (empty/singleton members, negative position, non-Transform
// call target) are unrepresentable by the type, not merely avoided
// by the producer.
type Cluster {
  members: NonSingletonList<MemberDescent>        // ≥2 by type shape
  intra_cluster_calls: NonEmptyList<IntraClusterCall>  // ≥1 by type shape (an SCC has ≥1 intra-cluster edge)
}

// 🟢 TERMINAL. A single typed handle that names a specific formal
// parameter of a specific cluster member. The per-member decreasing
// parameter is not a bare integer — it is a structural reference
// into the member's authoritative parameter list. `member` and
// `slot` are recoverable through ParamRef's accessors; they are not
// duplicated as MemberDescent fields because the relation between
// member and slot is what the carrier guarantees.
type MemberDescent {
  param: ParamRef
}

// 🟢 TERMINAL. Typed handle to the authoritative `Transform` node
// inside a member body. The Transform IS the call edge; this
// record is a typed reference, not a caller/callee compression.
// Callee is recoverable via `transform.target = Callable(decl)`;
// caller is recoverable via the enclosing Bind; source span lives
// on the Transform itself. No callsite identity is lost — two
// `a → b` call sites are two distinct Transforms, hence two
// distinct `IntraClusterCall` entries.
//
// `transform: TransformRef` (not raw NodeId) statically witnesses
// that the target resolves to a `Behavior::Transform` — non-Transform
// targets are unrepresentable at the type level.
type IntraClusterCall {
  transform: TransformRef
}
```

**The reflected `type Dag` gains the sidecar** (`src/v3/std/substrate.dag`):

```dag
type Dag {
  declarations: List<Declaration>
  nodes: List<Behavior>
  ports: List<DagPort>
  clusters: List<Cluster>       // NEW: SCC facts for mutual-recursion Loops
}
```

**The Rust `Dag` struct mirrors it** (`src/v3/compiler/src/dag.rs`):

```rust
pub struct Dag {
    /* existing fields (nodes, declarations, ports, primitives,
       substrate_markers, realization_metas, stdlib_types, ...) */
    clusters: Vec<Cluster>,    // indexed by ClusterId; write-once at lowering
}
```

Both surfaces must match. The reflected `type Dag` is the one `.dag` lens consumers (termination, complexity) walk; the Rust `Dag` is the one lowering writes. The reflection invariant (FieldBinding check in `m2_field_access_binding_test.rs`) enforces the two stay in sync. Matches the established sidecar pattern (`PrimitiveCache`, `SubstrateMarkers`, `RealizationMetaCache`, `StdlibTypeCache`, `optional_match_disjs` — see dag.rs:1140-1162). Write-once from lowering's perspective; lenses and inference are pure readers.

### Dissolution receipt — `LoopBound` coproduct (four-pattern check)

Per `feedback_coproduct_dissolution`, every new coproduct must pass the four-pattern dissolution check before being stamped `🟢 TERMINAL`. Receipt stamped here, not deferred:

**Pattern 1 — Fact placement (multiple consumers, different DAG locations).** Both variants are attached to the same `LoopNode.bound` slot at the same DAG location. The variant discriminates KIND OF BOUND, not WHERE THE FACT LIVES. No fact-placement compression to dissolve. ✓ does not apply

**Pattern 2 — Variant-is-data (same shape, different label).** `Cardinality { count: PortId }` and `Descent { cluster: ClusterId }` carry structurally different payloads. `PortId` is a runtime value handle; `ClusterId` is a compile-time sidecar index. They neither share a common payload shape nor differ only in label. ✓ does not apply

**Pattern 3 — Algebraic-form (traces to intro/elim of algebraic structures).** Both variants express "evidence that a loop terminates," but the algebraic origins are genuinely distinct:
- `Cardinality` reduces to cardinal-number bound on an iterable source (runtime port value ≤ source cardinality).
- `Descent` reduces to structural-descent well-foundedness on an SCC (compile-time measure decreases on every intra-cluster edge).
There is no shared algebraic form these dissolve into — they are two distinct termination authorities (runtime vs compile-time). ✓ does not apply

**Pattern 4 — Dimensional (flat enum hides M-dimensional record).** The variants don't share a coordinate space. There is no `count + cluster` hidden record where one coordinate is `None` in each variant; the two fields are genuinely alternative representations of the bound, not orthogonal dimensions. ✓ does not apply

**Conclusion:** `LoopBound` is structurally irreducible. The coproduct is the correct shape. Stamp stands: `🟢 TERMINAL`.

Dissolution receipts for the other new types (records, not coproducts) — `Cluster`, `MemberDescent`, `IntraClusterCall` — the four-pattern check applies only to coproducts. These are records; the relevant check is "state-space vs behavioral invariants" (`feedback_state_space_vs_behavioral_invariants`): can the record admit illegal states? An earlier shape answered *"no, enforced by construction"* for each field. That is convention-level, not API-level: a contributor can still construct invalid records. The approved shape graduates the answer to type-level enforcement via the Track 9 substrate integrity primitives:

- `Cluster.members: NonSingletonList<MemberDescent>` — empty and singleton states are unrepresentable because the type has separate `first`, `second`, and `rest` fields. A zero- or one-element "cluster" cannot be constructed.
- `Cluster.intra_cluster_calls: NonEmptyList<IntraClusterCall>` — an SCC by definition has ≥1 intra-cluster edge; the type shape rejects zero-edge clusters at construction.
- `MemberDescent.param: ParamRef` — typed opaque handle whose sole producer is `param_of(member: NodeId, slot: Int) -> ParamRef?`, which returns `None` for any slot outside the member's formal-parameter arity and for a `member` that is not a Bind. Because `param_of` is the only way to construct a `ParamRef`, a `MemberDescent` value statically witnesses *both* "this is a valid member node" *and* "this is a valid parameter of that member" — the member-relative bound is part of the carrier, not constructor-time prose on `MemberDescent` itself. A raw integer index paired with a raw `NodeId` (the prior shape, or an `ArityIndex + NodeId` compromise) would have left the relation "index is valid for this member" outside the type; `ParamRef` folds the relation into the handle. `member` and `slot` are recoverable via `ParamRef` accessors rather than being duplicated as `MemberDescent` fields; single authority for "which parameter of which member" lives on the handle.
- `IntraClusterCall.transform: TransformRef` — typed handle whose sole producer is `as_transform(dag, id)`, which returns `None` on non-Transform targets. A `TransformRef` value statically witnesses its Transform-ness; consumers rely on the type, not on accompanying prose.

No illegal state-space combinations are representable by the type shape. All three stamp as `🟢 TERMINAL`.

### Per-member descent positions

Cluster members need not share a single parameter position. For example:

```
fn process(state: Config, remaining: List<Task>) =
    if empty(remaining) then state
    else help(remaining, state)          // position 1 decreases (later)

fn help(tasks: List<Task>, state: Config) =
    process(state, tail(tasks))          // position 0 decreases
```

Both members have a structurally-decreasing argument but at different positions. `MemberDescent.param` is per-member to capture this — each `ParamRef` names one formal parameter of one member, and the `Cluster.members: NonSingletonList<MemberDescent>` carries one such handle per cluster member. A single `position: Int` on `Descent` was rejected because it forced signature-alignment as an implicit compiler constraint; the move from `position: ArityIndex + member: NodeId` to `param: ParamRef` carries the "valid parameter of that member" relation on the handle itself.

### Lowering shape illustrated

For `fn a(n: Int) = if n == 0 then 0 else b(n - 1)` + `fn b(n: Int) = if n == 0 then 0 else a(n - 1)`, with one external call site `a(x)`:

```
Dag.nodes:
  Bind(BindNode { id: bind_a, name: "a", ... })       // peer
  Bind(BindNode { id: bind_b, name: "b", ... })       // peer
  Transform(TransformNode {                           // authoritative a→b edge
    id: xform_a_to_b,
    target: Callable(arrow_b_decl),
    ...
  })
  Transform(TransformNode {                           // authoritative b→a edge
    id: xform_b_to_a,
    target: Callable(arrow_a_decl),
    ...
  })
  Loop(LoopNode {
    id:          <loop_id>,
    body:        bind_a,                              // entry Bind for this call site
    bound:       LoopBound::Descent { cluster: ClusterId(0) },
    source:      <a's arg port>,
    init:        <init port>,
    result_port: <loop result port>,
    span:        <cluster span>,
  })

Dag.clusters[ClusterId(0)] = Cluster {
  members: [
    MemberDescent { param: param_of(bind_a, 0).unwrap() },   // param_of returns ParamRef?; lowering knows the slot is in-arity
    MemberDescent { param: param_of(bind_b, 0).unwrap() }
  ],
  intra_cluster_calls: [
    IntraClusterCall { transform: as_transform(xform_a_to_b).unwrap() },    // handle into the authoritative Transform above
    IntraClusterCall { transform: as_transform(xform_b_to_a).unwrap() }     // handle, not a copy — callee = transform.target, caller = enclosing Bind
  ]
}
```

Authority layering: the `Transform` nodes in `Dag.nodes` are the sole call-topology authority. The `Cluster.intra_cluster_calls` list is a typed-handle index into that authority — lowering knows which Transforms are intra-cluster (that's what the SCC computation produced), so it populates the index rather than forcing lenses to re-scan bodies. Reading the callee: `transform.target` resolves to `Callable(arrow_decl)`. Reading the caller: walk up from the Transform to its enclosing Bind. Reading the source span: `transform.span`. No facts live only in the Cluster; every fact lives on the Transform and the Cluster just indexes the subset that's intra-cluster.

### What changes in `compute_mutually_recursive`

**Before:** returns `HashSet<String>` of cycle members; its only consumer is a rejection diagnostic at lower.rs:2293.

**After R2:** returns `Vec<ClusterShape>`:

```rust
struct ClusterShape {
    members:              Vec<DeclarationId>,
    per_member_positions: HashMap<DeclarationId, usize>,
    shared_witness_ok:    bool,                       // false → fail-closed diagnostic path
    // Note: no edges field — intra-cluster Transform NodeIds are
    // resolved at lowering *after* member bodies are emitted, at
    // which point lowering has both the Transform NodeIds and the
    // ClusterShape in hand. The Cluster.intra_cluster_calls list
    // is populated with those NodeIds.
}
```

Lowering consumes this in order:
1. Lower each member's body as today — `Transform` edges remain literal, pointing at real callee declarations.
2. For each member body, identify Transforms whose `target = Callable(decl)` where `decl ∈ cluster_members`; wrap each in a `TransformRef` (via `as_transform(node_id)`) and collect as `IntraClusterCall { transform: TransformRef }` entries.
3. For each cluster member, construct a `MemberDescent { param: ParamRef }` via `param_of(member_node_id, slot)` using the per-member descent slot from `ClusterShape.per_member_positions`.
4. Allocate a `ClusterId`; populate `Dag.clusters[cluster_id]` with the `Cluster` record (`NonSingletonList<MemberDescent>` + `NonEmptyList<IntraClusterCall>`).
5. For each external call site into the cluster, emit one `Behavior::Loop` with `body = NodeId` of the entry member's Bind, `bound = LoopBound::Descent { cluster: cluster_id }`.

The "mutual recursion is not yet supported in v3" rejection diagnostic at lower.rs:2293 **deletes** in the implementation PR.

### Per-call-site Loop semantics

A cluster called from N external sites produces N `Behavior::Loop` nodes, each referencing the **same** `ClusterId`. The cluster fact is carried once in the sidecar; Loops hold handles, not copies. Per-Loop state (entry point, source port, init port, result port) varies by call site; per-cluster state (membership, per-member positions, internal edges) is shared via `ClusterId`. Single authority preserved.

### Consumers

**Termination lens** (Lane 2 — `src/v3/lenses/termination.dag`, to be created): walks `Behavior::Loop`. When `bound = Descent { cluster }`, reads `Dag.clusters[cluster]` for members and builds a map `member_node → ParamRef` by querying each `MemberDescent.param`'s `member_of()` / `slot_of()` accessors.

For each `IntraClusterCall.transform: TransformRef` in `intra_cluster_calls`, the lens resolves the edge's **caller** (the enclosing Bind of the Transform — looked up via `TransformRef` and an enclosing-Bind walk) and its **callee** (the Bind identified by `transform.target = Callable(decl)` that resolves back to one of the cluster members). It looks up **both** members' `ParamRef` entries:

- `caller_slot` = the caller member's descent slot (from the caller's `MemberDescent.param.slot_of()`)
- `callee_slot` = the callee member's descent slot (from the callee's `MemberDescent.param.slot_of()`)

These need not be the same slot — the `process` / `help` case (§"Per-member descent positions") has them at different positions. The check is:

> The argument at position `callee_slot` in the Transform's argument list (i.e. the value the callsite binds to the callee's descent param) is structurally smaller than the caller's formal parameter at position `caller_slot` (i.e. the caller's descent param).

This crosses the boundary the `ParamRef` carrier was introduced to preserve: the callee-side slot selects *which argument* is being fed into the callee's decreasing parameter, the caller-side slot selects *which formal* is the caller's decreasing measure, and the structural-smaller check relates the two. A single-slot formulation (comparing "argument at caller's slot" against "caller's param at caller's slot") would silently collapse mixed-position SCCs back into the single-position regime.

No SCC re-detection. Diagnostic on failure names the exact `TransformRef` (so `transform.span` points at the failing call site — two `a → b` sites diagnose distinctly) plus both `ParamRef` handles so the user can see which caller-param and which callee-param the check was keyed on.

**Complexity lens** (Lane 2 — `src/v3/lenses/complexity.dag`): reads `Loop.bound × cost(body)` per the standard cost rule. When `bound = Descent`, the bound reads from the cluster's shared measure. No SCC re-detection. Matches SELF_HOSTING:708-713 verbatim.

**Inference pass** (`src/v3/compiler/src/infer.rs`): pure post-lowering consumer — inference runs after lowering per lib.rs:201-202. Reads cluster membership from the sidecar for bottom-up SCC-aware type inference. No separate SCC pass in inference.

### Emission

Target languages (Rust, Go, Python) have native mutual recursion. The cluster-Loop wrapper is compile-time metadata, not a runtime construct. The emitter walks the Dag, sees the member Binds as peer declarations, and emits each as a top-level fn with real call edges preserved:

```rust
// Rust emission of Bind(a) + Bind(b):
fn a(n: i64) -> i64 { if n == 0 { 0 } else { b(n - 1) } }
fn b(n: i64) -> i64 { if n == 0 { 0 } else { a(n - 1) } }
```

No dispatch, no ring, no sum-type wrapper. Target-native.

### Shape B (SPICE / Verilog / English) — not compiler targets

These are Shape B artifacts produced by `.dag` programs (see THESIS §"Two shapes"). A `.dag` program producing a SPICE netlist is ordinary user code; if it wants to support mutual recursion in its domain model, that's a `.dag`-library concern, not a compiler concern.

---

## Pipeline ordering

```
tokenize → parse → lower → infer → emit
                    │        │        │
                    │        │        └── reads LoopBound (target-native mutual rec)
                    │        └── reads Dag.clusters via LoopBound::Descent
                    └── writes Dag.clusters; materializes cluster Loops
```

Verified: lib.rs:201 is `lower::lower(...)`, lib.rs:202 is `infer::infer(...)`. Inference is a post-lowering consumer of substrate facts.

---

## Error reporting

```
ERROR at <cluster span>: mutually-recursive cluster does not terminate

  fn a(n: Int) -> Int = if n == 0 then 0 else b(n)
                                                 ^ note: `n` passed unchanged to `b`

  fn b(n: Int) -> Int = if n == 0 then 1 else a(n - 1)

Cluster: {a, b}
Descent failure at call `a → b`:
  - `a`'s descent param: `n` (slot 0)
  - `b`'s descent param: `n` (slot 0)
  - Argument at `b`'s slot 0 (`n`) is not structurally smaller than `a`'s slot 0 (`n`).

FIX: on this call, pass a structurally-smaller value at `b`'s descent slot, e.g., `b(n - 1)`.
```

Uses the DB-1 Correction shape (source-level). The diagnostic names the failing `IntraClusterCall.transform: TransformRef` (typed handle into the authoritative `Transform`, which carries its own `span`) plus both `ParamRef` handles — the caller's descent param (for the expected-smaller side) and the callee's descent param (for the slot the argument is being fed into). In the mixed-position case, slot labels in the diagnostic come from the respective `ParamRef.slot_of()` calls and may differ; both are shown so the user can see exactly which caller-formal and which callee-formal the check was keyed on.

---

## Rejected alternatives

| Alternative | Why rejected |
|---|---|
| `MutualLoop` 6th `Behavior` variant | Violates THESIS:604-629 five-behavior commitment. |
| Lens-level SCC detection, substrate unchanged | Violates INVARIANTS:555 / :665 (mutual recursion → descend over SCC is a lowering prescription, not a lens pass). Creates parallel representation with `compute_mutually_recursive`. |
| Encoding A — no substrate change, synthesized manifest nodes | Cluster membership is re-derivable from call-graph SCC per-lens → parallel representation debt grows per consumer. |
| Encoding B — `children: List<NodeId>` on `BindNode` | Forces `BindNode` into a mode distinction (naming-Bind vs composition-Bind); `result_port` semantics become ambiguous on composition-Binds. Violates `state_space_vs_behavioral_invariants` (illegal states become representable). |
| Encoding C1 — `ClusterDeclaration` as new `Declaration` kind | `Declaration` is about type connectives and user-facing types, not compiler-computed SCC metadata. Adding a Declaration kind for lowering-only facts conflates layers. Sidecar (established pattern) is the right shape. |
| Single `position: Int` on `Descent` variant | Forces implicit signature-alignment on cluster members; fails on `process/help` case (position 0 vs position 1). Per-member `MemberDescent` captures the general case. |
| Inline `members` list on each `Loop.bound.Descent` | N copies of one lowering-output fact; inconsistent with sidecar pattern. `ClusterId` handles + shared `Dag.clusters` entry replaces. |
| `CallEdge { caller: NodeId, callee: NodeId }` as edge payload | Compresses away call-site identity: two `a → b` sites with different argument shapes collapse structurally, so the termination diagnostic can't point at the failing site without re-walking bodies. Also creates a second authority for call topology alongside the authoritative `Transform` nodes. **Adopted shape:** `IntraClusterCall { transform: NodeId }` — a typed handle into the authoritative Transform; callee/caller/span all readable through the handle. |
| Rust-only `Dag.clusters` sidecar (no reflected `type Dag` field) | The `.dag` lens consumers (`termination.dag`, `complexity.dag`) walk the reflected `type Dag`; an unreflected Rust-only sidecar dies at the lower → lens boundary. Violates facts-flow-forward. Add `clusters: List<Cluster>` to `type Dag` in `src/v3/std/substrate.dag` alongside the Rust struct. |
| Deferring LoopBound dissolution receipt to implementation PR | Stamp-before-receipt — a new substrate coproduct earning `🟢 TERMINAL` without the four-pattern check creates a hole that propagates. Run the check in §Dissolution receipt above; receipt stamped in-doc. |
| Raw `List<T>` / `Int` / `NodeId` on `Cluster` / `MemberDescent` / `IntraClusterCall` with construction-only invariants | Admits illegal states (`Cluster.members` empty/singleton, negative / out-of-arity `position`, non-Transform `transform`). Relies on "enforced by construction" — convention-level, not API-level. Use `NonSingletonList<MemberDescent>` / `NonEmptyList<IntraClusterCall>` / `ParamRef` / `TransformRef` so the invariants live on the type shape (ROADMAP Track 9 substrate integrity primitives). |
| `MemberDescent { member: NodeId, position: ArityIndex }` — separate member + non-negative index fields | `ArityIndex` alone only makes non-negativity structural; the *member-relative* bound (`position < member-arity`) still sits outside the shape. **Adopted shape:** `MemberDescent { param: ParamRef }` — a single typed handle whose sole constructor `param_of(member, slot) -> ParamRef?` fails closed for any invalid (member, slot) pair. The "valid parameter of this member" relation lives on the handle. `ArityIndex` is at most an input to `param_of`, not a stored field. |
| Annotate each `Bind` with `cluster_id` | Parallel authority: call graph IS the authority for cluster membership; annotation duplicates and can drift. |
| Require users to declare `mutual` explicitly | Compiler detects SCCs automatically; user annotation is ceremony. |
| Defer mutual recursion | `compiler.dag` requires it (M2 feature parity). Cannot defer. |

---

## Open questions

None blocking. Two deliberately-deferred refinements for the implementation PR to note but not resolve:

1. **Signature-alignment normalization.** A future refinement could normalize cluster signatures at lowering so all members share a canonical parameter position, simplifying `MemberDescent` to a single `slot: ArityIndex` + `members: List<NodeId>` shape. Not in scope for the current approved design — would be a user-facing constraint requiring its own thesis check, and the `ParamRef` carrier captures the general case cleanly enough that the simplification has no urgency.
2. **`compute_mutually_recursive` migration to `std/graph.dag`.** SELF_HOSTING:710-713 anticipates SCC as a library function in `std/graph.dag`. The implementation keeps the Rust SCC routine as the transitional authority. Swap to `.dag` is a producer-side change when the `.dag` compiler migrates; consumers read `LoopBound::Descent` / `Dag.clusters` either way — no consumer churn.

---

## Acceptance (Lane 3 Stage 3a.1 owns)

**Substrate invariants:**
- [ ] `type Behavior` in `src/v3/std/substrate.dag` still has exactly 5 variants (`Value | Transform | Branch | Loop | Bind`)
- [ ] `type LoopBound` is a 2-variant coproduct (`Cardinality { count }`, `Descent { cluster }`)
- [ ] `Cluster`, `MemberDescent`, `IntraClusterCall` terminal types present
- [ ] `type Dag` in `src/v3/std/substrate.dag` carries `clusters: List<Cluster>` (matches Rust Dag sidecar; reflection FieldBinding test passes)
- [ ] `Dag.clusters` sidecar present; write-once from lowering

**Positive fixtures (SCC sizes 2, 3, 5):**
- [ ] Compile without diagnostics
- [ ] One `Behavior::Loop` emitted per external call site with `bound = LoopBound::Descent`
- [ ] `Dag.clusters[cluster_id]` populated with `NonSingletonList<MemberDescent>` (each carrying a `ParamRef` constructed via `param_of`) + `NonEmptyList<IntraClusterCall>` (each carrying a `TransformRef` constructed via `as_transform`)
- [ ] Real call-graph edges preserved in member bodies (Transforms point at real callees; no ring)
- [ ] Complexity lens reads cost via standard rule — no SCC re-detection
- [ ] Termination lens verifies descent — no SCC re-detection
- [ ] Rust / Go / Python emission produces N separate `fn` declarations calling each other

**Negative fixtures:**
- [ ] SCC with no shared descent measure (argument preserved or grows on some edge) → fail-closed diagnostic naming the failing `IntraClusterCall.transform: TransformRef`'s span + **both** the caller's and callee's `MemberDescent.param: ParamRef` handles (slots recoverable via `ParamRef.slot_of`; the two slots may differ in mixed-position SCCs)
- [ ] Mixed-position SCC fixture (e.g. `process(state, remaining)` / `help(tasks, state)`) with a valid descent: lens verifies the argument at the *callee*'s descent slot is smaller than the *caller*'s descent param — not the caller's slot — and compiles without diagnostics
- [ ] Two distinct `a → b` call sites with different argument shapes produce two distinct `IntraClusterCall` entries; diagnostic differentiates which site failed
- [ ] Non-SCC call pattern → single-recursion path unchanged (regression guard)

**Authority checks (single-representation invariant):**
- [ ] `compute_mutually_recursive` is the only Tarjan-like function in v3
- [ ] Termination lens: no SCC detection code
- [ ] Complexity lens: no SCC detection code
- [ ] Emission: no SCC detection code
- [ ] `test_mutual_recursion_is_rejected` in `m0_acceptance.rs:1350` flipped from rejection expectation to positive compile

**C1-class dissolution check (substrate extension ritual):**
- [ ] Four dissolution patterns re-run against `LoopBound::Descent` and the new terminal types. Expected outcome: no dissolution (Cardinality and Descent are genuinely distinct authorities — runtime-port vs compile-time-structural). Stamp the result in the PR body so reviewers see the check was done.

**Documentation:**
- [x] This design doc lives at `docs/design-mutual-recursion-lowering.md` and describes the approved lowering (live state per INVARIANTS — Documentation Describes Live State)
- [x] Lane 3 / post-L1.5 references point at this doc’s framing
- [x] ROADMAP Stage 3a.1 row updated to reflect approved design

---

## Associations

- **Lane 3 Stage 3a.1** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — approved design for that sub-stage
- **DB-1 Correction shape** ([design-correction-shape.md](./design-correction-shape.md)) — cluster termination diagnostics emit Corrections
- **DB-7 Symbolic cost** ([design-symbolic-cost-algebra.md](./design-symbolic-cost-algebra.md)) — reads `LoopBound::Descent.cluster` for recursion depth bound
- **`src/v3/std/substrate.dag`** — `LoopBound` grows `Descent` variant; `Cluster`, `MemberDescent`, `IntraClusterCall` terminal types added; `type Dag` gains `clusters: List<Cluster>` field; substrate integrity primitives (`NonEmptyList<T>`, `NonSingletonList<T>`, `ParamRef`, `TransformRef`) declared here and consumed by the records above — all added together in the DB-9 Lane 3 Stage 3a.1 implementation PR
- **ROADMAP Track 9** — substrate integrity primitives graduation ledger; `IndexedElement<T>.index` in `src/v3/std/list.dag` is the planned second consumer
- **`src/v3/compiler/src/dag.rs`** — `Dag.clusters: Vec<Cluster>` sidecar
- **`src/v3/compiler/src/lower.rs`** — `compute_mutually_recursive` return upgraded; cluster-Loop construction replaces rejection
- **`src/v3/compiler/src/infer.rs`** — reads `LoopBound::Descent` for bottom-up cluster inference
- **Lenses** (`src/v3/lenses/`) — termination (new) + complexity consume `Dag.clusters` via `ClusterId`
- **Thesis anchor** — THESIS:604-629 (five behaviors, no sixth)
- **Invariants anchor** — INVARIANTS:555 + :665 (mutual recursion → descend over SCC — honored via lowering)
- **SELF_HOSTING anchor** — §2.4 (lowering preserves real call topology — honored)

