# R3 Evaluator — E0 Body-Evaluator API Scaffold

**Status:** PROPOSAL — docs-only API scaffold for the body-evaluator
entrypoint. This brief names the Rust module / function shape that E1,
E2, and E3 will fill, so those slices can ship independently without
absorbing each other's surface and without smuggling carrier
duplication. **No Rust code, no carrier mirrors, no fixture changes
land in this slice.**

**Parent:**
[`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
("Implementation Slices" — E1 through E9; this brief precedes E1).
**Carrier authorities:**
[`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) — `Value`,
`NamedField`, `EvalFrame`, `EvalStateStack`, `EvalStrategy`,
`InputEvaluationOrder`.
**Behavior authority:**
[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) +
`src/v3/compiler/src/dag.rs` — `Behavior = Value | Transform | Branch |
Loop | Bind` (`dag.rs:1905-1911`).
**Implementation rules:**
[`docs/briefs/r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md).

## Why E0 exists

The dispatch brief sequences body-evaluator work as E1 (`Value`), E2
(`EvalFrame` / `EvalStateStack`), E3 (`Transform`), E4 (`Branch`), E5
(`Loop`), E6 (`Bind`), each as small slices over a common evaluator
API. That API does not exist in `src/v3/compiler/src/` today:

- `grep -rn "EvalFrame\|EvalStrategy\|EvalStateStack" src/v3/compiler/src/`
  finds only the bootstrap snapshot rows
  (`bootstrap_generated.rs:992-995`) — these are read-only `Declaration`
  rows, **not instantiable Rust mirror types**.
- The closest neighbor is `src/v3/compiler/src/lens_apply.rs::EvalCtx`
  (`:531 fn eval_transform`), which operates on `FieldValue` for the
  *lens application* path. Extending that into the body evaluator would
  conflate two lanes the runner-authority ratchet brief specifically
  separates; PR-E E0 must not do this.

Without an agreed API, E1 and E2 each have to invent the entrypoint
shape, and E3 has nothing to depend on. E0's job is to lock that shape
docs-only so the later slices ship as fills, not redesigns.

## Substrate-vs-Rust carrier status (read-only audit)

| Carrier             | `.dag` authority                               | Rust mirror today |
|---------------------|------------------------------------------------|------------------|
| `Value` / `NamedField` | `src/v3/std/runtime.dag:18-49` (PR-A.1, #1243) | **None instantiable.** Lives only as `Declaration` rows in `bootstrap_generated*.rs`. |
| `EvalFrame { bindings: Map<PortId, Value> }` | `src/v3/std/runtime.dag:62-64` (PR-A.2, #1255) | **None instantiable.** Same story. |
| `EvalStateStack { frames: List<EvalFrame> }` | `src/v3/std/runtime.dag:73-75` (PR-A.2) | **None instantiable.** Same story. |
| `EvalStrategy = ApplicativeOrder { input_order }` | `src/v3/std/runtime.dag:82` (PR-A.3) | **None instantiable.** Same story. |
| `InputEvaluationOrder = LeftFirst` | `src/v3/std/runtime.dag:87` (PR-A.3) | **None instantiable.** Same story. |
| `Behavior = Value | Transform | Branch | Loop | Bind` | `src/v3/std/substrate.dag` | **Live Rust enum** at `src/v3/compiler/src/dag.rs:1905-1911`. |
| `Dag` | `src/v3/std/substrate.dag` | Live Rust struct (`dag.rs`). |

Behavior + Dag are already mirrored. The five runtime carriers are
**not** — they are substrate-side declarations consumed by the bootstrap
but never lowered into `dag.rs` (or any new file) as Rust types. E1
through E3 cannot construct or compare them without that bridge.

**Out of E0's scope:** authoring the Rust mirror types. That work is
either (a) a separate Substrate-Rust bridge slice that auto-generates
mirrors from the `Declaration` rows, or (b) a small per-slice add as
each evaluator slice lands its first usage. E0 names the gap and lets
E1's first carrier add be the dependency-direction-setting one (E1's
acceptance test exercises `Value::LiteralValue`, which is the single
required carrier add for the slice). E0 does not pre-declare it.

## Proposed API surface

The shape below is the **API contract** E1 / E2 / E3 will implement.
No Rust types in this brief; only the names, signatures, and rule
references. Each later slice fills the function body for one
`Behavior` variant.

### Module path

`src/v3/compiler/src/body_evaluator.rs` — single new module. No
extension of `lens_apply.rs`; that file stays the lens-application
authority. Body evaluator is its own module so the runner-authority
ratchet's "no parallel test-predicate language in Rust" rule applies
cleanly.

### Public entrypoint

```text
pub fn evaluate_body(
    dag: &Dag,
    body: NodeId,
    initial_stack: EvalStateStack,
    strategy: EvalStrategy,
) -> Result<Value, EvalDiagnostic>;
```

- `dag` — already-compiled program DAG (consumes `compile_to_dag`'s
  output; no parsing inside the evaluator).
- `body` — the entry node identifier (`NodeId`); for top-level
  evaluation this is the bind/transform the caller selected.
- `initial_stack` — the caller-supplied starting `EvalStateStack`
  containing one `EvalFrame` with parameter bindings already
  registered (per PR-B.1 §First implementation target item 1).
  Construction of the initial frame is the caller's responsibility,
  not the evaluator's.
- `strategy` — must be `ApplicativeOrder { input_order: LeftFirst }`
  for E1-E5; any other inhabitant returns
  `EvalDiagnostic::UnsupportedStrategy` (per PR-B.1 §Strategy and the
  PR-A.3 audit's eager-baseline lock).
- Returns `Value` on success, `EvalDiagnostic` fail-closed on every
  detectable problem.

### Internal dispatch entrypoint

```text
fn eval_node(
    dag: &Dag,
    node: NodeId,
    stack: &mut EvalStateStack,
    strategy: &EvalStrategy,
) -> Result<Value, EvalDiagnostic>;
```

Resolves `node` via `dag.node_opt(&node)` (the Rust Dag API for
behavior lookup at `src/v3/compiler/src/dag.rs:3037`; `Dag::node(id) ->
&Behavior` is the panicking variant at `:3025`). A `None` from
`node_opt` is fail-closed `EvalDiagnostic::ResolveError` (missing
node), not a panic. The evaluator must NOT call `dag.declaration(…)`
to resolve a node — declarations and nodes are separate authorities
in `Dag` (declarations are type-shape; nodes are `Behavior`
inhabitants), and conflating them creates parallel-authority debt.

After resolution, pattern-matches on `Behavior`, dispatches to
`eval_value` (E1), `eval_transform` (E3), `eval_branch` (E4),
`eval_loop` (E5), or `eval_bind` (E6).

### Port-level evaluation contract

```text
fn eval_port(
    dag: &Dag,
    port: PortId,
    stack: &mut EvalStateStack,
    strategy: &EvalStrategy,
) -> Result<Value, EvalDiagnostic>;
```

Runtime data dependencies in a body flow through `PortId` facts, not
node ids. `Transform.inputs`, `Branch.input`, `Loop.source` / `init`
are all `PortId` references; consumers need a `Value` for each cited
port. `eval_port` is the **single port-level resolution authority**
the per-`Behavior` evaluators consume:

1. **Frame lookup first.** Call `frame_lookup(stack, port)`. If the
   port is bound (parameter binding or upstream `Bind` /
   producer-node write), return the `Value` immediately.
2. **Producer demand-eval.** If the port is unbound, look up
   `dag.port_opt(&port).and_then(|p| p.produced_by)` per the existing
   Rust API at `src/v3/compiler/src/dag.rs:3237` (port lookup) and
   `:3061` (Bind-chain producer follow). If a producer `NodeId`
   exists, call `eval_node(dag, producer, stack, strategy)` to
   compute it; the producer's body is responsible for `frame_bind`-ing
   its `result_port`. After `eval_node` succeeds, **re-do**
   `frame_lookup(stack, port)`; the value must now be bound (else
   producer-side invariant violation, fail-closed
   `EvalDiagnostic::FrameDepthInvariantViolation`-class).
3. **Fail-closed otherwise.** Port absent from the DAG entirely is
   `EvalDiagnostic::ResolveError`; port present but neither bound nor
   producer-backed is `EvalDiagnostic::UnboundPort(port)`.

Per-`Behavior` evaluators **must not** call `frame_lookup` directly
for input-port resolution; they call `eval_port`, which keeps
demand-evaluation centralized. Direct `frame_lookup` is reserved for
internal frame-discipline (E2) helpers and for assertion-style
checks. This separation is the single port-resolution authority the
runtime data-flow facts (`PortId` everywhere) require — without it
each `Behavior` evaluator would re-invent demand-eval and `Dag` port
lookup, violating Facts-Flow-Forward and creating per-Behavior
parallel resolution paths.

`eval_port` is **not implemented in E0**; it is part of E2's
contract (E2 owns frame + port discipline). The signature is locked
here so E3/E4/E5/E6 can depend on it without redesign.

### Per-`Behavior` slice fills

Each later slice fills exactly one of these signatures. None are
implemented in E0; they all return `EvalDiagnostic::NotYetImplemented`
in the scaffold.

- **E1 — `eval_value`:**
  `fn eval_value(node: &ValueNode) -> Result<Value, EvalDiagnostic>`
  per PR-B.1 §B.1.1.
- **E2 — frame + port discipline helpers** (no `Behavior` dispatch;
  pure state + port-resolution plumbing):
  - `fn frame_lookup(stack: &EvalStateStack, port: PortId) -> Result<Value, EvalDiagnostic>`
    (innermost-first walk; unbound = `EvalDiagnostic::UnboundPort`).
    Used internally by `eval_port`; **per-`Behavior` evaluators call
    `eval_port`, not `frame_lookup`, for input resolution**.
  - `fn frame_bind(stack: &mut EvalStateStack, port: PortId, value: Value) -> Result<(), EvalDiagnostic>`
    (top-frame only; `map_get` pre-check; duplicate =
    `EvalDiagnostic::DuplicateBind`).
  - `fn push_frame(stack: &mut EvalStateStack)` /
    `fn pop_frame(stack: &mut EvalStateStack) -> Result<(), EvalDiagnostic>`
    (paired discipline; pop on empty is an internal-invariant
    `Diagnostic`).
  - `eval_port` (signature in §"Port-level evaluation contract"
    above) — central demand-eval authority; combines `frame_lookup`
    and producer-node `eval_node` recursion behind one port-level
    boundary.
- **E3 — `eval_transform`:**
  `fn eval_transform(dag, t: &TransformNode, stack, strategy) -> Result<Value, EvalDiagnostic>`
  per PR-B.1 §B.1.2 and §B.1.6 / §B.1.7. `t.inputs` is `Vec<PortId>`
  per `dag.rs:1721`, **not** `Vec<NodeId>`. Eager evaluation under
  `LeftFirst` is **port-resolution via `eval_port`** (the port-level
  authority defined above): for each `port` in `t.inputs`
  left-to-right, call `eval_port(dag, port, stack, strategy)` to get
  the `Value`. `eval_port` handles both bound-parameter and
  producer-node demand-eval cases internally, so `eval_transform`
  does not branch on those cases. After all inputs resolve, dispatch
  on `t.target` (`Callable` / `FieldProject` / `Operator`) and write
  the result to `t.output` via `frame_bind` so downstream consumers
  can read.
- **E4 — `eval_branch`:**
  `fn eval_branch(dag, b: &BranchNode, stack, strategy) -> Result<Value, EvalDiagnostic>`
  per PR-B.1 §B.1.3 (resolved-variant dispatch only; no wildcard).
- **E5 — `eval_loop`:**
  `fn eval_loop(dag, l: &LoopNode, stack, strategy) -> Result<Value, EvalDiagnostic>`
  per PR-B.1 §B.1.4 (`Cardinality` arm executable; `Descent` arm
  fail-closed residual).
- **E6 — `eval_bind`:**
  `fn eval_bind(dag, b: &BindNode, stack) -> Result<Value, EvalDiagnostic>`
  per PR-B.1 §B.1.5 (`map_get` pre-check + top-frame `frame_bind`).

### Fail-closed error carrier

```text
pub enum EvalDiagnostic {
    NotYetImplemented(&'static str),
    UnsupportedStrategy(String),
    UnsupportedArrowBody(&'static str),
    UnsupportedOperator(String),
    UnsupportedTarget(String),
    LoopBoundDescentResidual { cluster: ClusterId },
    LoopCardinalityWitnessMissing(String),
    UnboundPort(PortId),
    DuplicateBind(PortId),
    BranchUnresolvedVariant { name: String },
    BranchShapeMismatch(String),
    FrameDepthInvariantViolation { entry: usize, exit: usize },
    ResolveError(String),
}
```

The variants are exactly the dissolution-trigger entries from PR-B.1's
fail-closed catalog — one variant per Diagnostic source PR-B.1 may
emit. **Adding a new variant is a STOP+PING** (per dispatch rule):
either route the underlying gap through P1 substrate-fact-introduction
or extend the catalog in PR-B.1's seed brief first.

`EvalDiagnostic` lives in `body_evaluator.rs` next to the entrypoints;
it is **not** a substrate carrier. It is the typed Rust failure mode
the Rust evaluator returns; the corresponding `.dag` `Diagnostic`
shape (if/when one is needed) is a separate Substrate-routed slice.

## What each slice fills (handoff matrix)

| Slice | Fills | New mirror types this slice may add | STOP+PING boundary |
|------|-------|------------------------------------|---------------------|
| **E0 (this brief)** | API names + module skeleton (optional: returns `NotYetImplemented` for all). | None. | Adding any carrier mirror is out of E0. |
| **E1** | `eval_value` body. | `Value` Rust mirror (single use; minimum needed: `LiteralValue(LiteralBits)` arm). Document this as the first carrier add. | If `Value` mirror needs more than `LiteralValue` to compile the slice, STOP+PING. |
| **E2** | `frame_lookup` / `frame_bind` / `push_frame` / `pop_frame`. | `EvalFrame` / `EvalStateStack` Rust mirrors (consume the existing `Map<K,V>` Rust shape — likely `BTreeMap<PortId, Value>` for deterministic iteration). | If `Map<PortId, Value>` needs a new substrate carrier, STOP+PING. |
| **E3** | `eval_transform` body. | `EvalStrategy` / `InputEvaluationOrder` Rust mirrors (single inhabitant each at PR-A.3 lock). | If `Transform` needs `EvalThunk`, lazy strategy, or memoization, STOP+PING. |
| **E4** | `eval_branch` body. | None new (`BranchNode` / `BranchPattern` / `PayloadBinding` already in `dag.rs`). | If `Branch` needs a wildcard or catch-all, STOP+PING. |
| **E5** | `eval_loop` body. | None new (`LoopNode` / `LoopBound` already in `dag.rs`). | If `Descent` arm execution needed, STOP+PING (residual). |
| **E6** | `eval_bind` body. | None new. | Duplicate-bind discipline is the slice's own boundary. |

## Optional tiny code companion (this brief)

This brief authorizes — but does **not** require — a sub-tiny code
slice that adds:

1. `src/v3/compiler/src/body_evaluator.rs` containing only the
   `EvalDiagnostic` enum (above) — **no `evaluate_body` signature**,
   because that signature names `Value` / `EvalStateStack` /
   `EvalStrategy`, none of which have Rust mirrors today and the
   companion is required to be carrier-neutral.
2. `pub mod body_evaluator;` line in `src/v3/compiler/src/lib.rs`.

That code is **strictly carrier-neutral**: it adds zero mirror types
for `Value` / `EvalFrame` / `EvalStateStack` / `EvalStrategy`. The
`evaluate_body` signature shipped above is the **API contract**, not
production Rust — it lands inside the E1 PR alongside the first
carrier mirror (`Value`) so the function can actually compile. Until
then, the optional companion gives only the typed `EvalDiagnostic`
enum and an empty module so E1 doesn't have to create the file and
`lib.rs` wiring at the same time as its first behavior body.

If even this much code feels like scope creep, E0 stays docs-only and
E1 absorbs the module-creation overhead.

The companion is gated on reviewer comfort: if the api-review bots
flag it as scope expansion, drop it and ship docs-only.

## Out of scope (this brief, E0)

- Implementing **any** evaluator behavior (`Value`, `Transform`,
  `Branch`, `Loop`, `Bind` are all `NotYetImplemented` placeholders if
  the optional code lands).
- Authoring `Value` / `EvalFrame` / `EvalStateStack` / `EvalStrategy`
  Rust mirror types.
- Any new substrate carrier or `TestPredicate` variant.
- Any change to `lens_apply.rs` or extension of `EvalCtx`.
- Memoization (no `EvalMemoKey` work; eager-no-memo is the baseline).
- `EvalThunk`, `NormalOrder`, lazy strategy.
- Witness construction (PR-B witness half).
- Any test fixture beyond the structural acceptance gates already
  named in the dispatch brief.

## Acceptance gates (this brief, E0)

- ✅ Public entrypoint signature named with explicit input/output types
  and strategy parameter.
- ✅ Internal dispatch entrypoint named.
- ✅ Per-`Behavior` slice fills enumerated, each with a PR-B.1 §
  reference and a dedicated function signature.
- ✅ `EvalDiagnostic` carrier shape names every PR-B.1 fail-closed
  catalog entry as a typed variant.
- ✅ Substrate-vs-Rust carrier status table makes the missing-mirror
  gap auditable.
- ✅ Per-slice handoff matrix names what each later slice may add and
  its STOP+PING boundary.
- ✅ Docs-only PR (or, optionally, docs + a single carrier-neutral
  skeleton file).

## STOP+PING boundary (E0 itself)

E0 must NOT propose, declare, or mirror:

- A new `TestPredicate` variant.
- A new `Value` variant (substrate-side).
- A new `EvalStrategy` / `InputEvaluationOrder` inhabitant.
- A new `Behavior` variant.
- A new substrate carrier of any kind.
- A second body-evaluator entrypoint or `EvalCtx`-style sibling in
  `lens_apply.rs`.

If during E0 review any reviewer concludes that one of the above is
required to make the API contract executable, **STOP and escalate** —
do not silently broaden. The dispatch brief sequences these
prerequisites through P1 / Substrate Manager, not through worker code.
