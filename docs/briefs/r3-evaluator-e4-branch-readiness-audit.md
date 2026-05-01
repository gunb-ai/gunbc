# R3 Evaluator — E4 Branch Readiness / Blocker Audit

**Status:** AUDIT — docs-only. Records the exact E1 prerequisites that
must land before PR-E E4 (Branch arm coverage) can be implemented as
the bounded slice the dispatch brief intends. **No Rust, no substrate,
no fixture changes land in this slice.**

**Authorities:**
- [`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
  §"Implementation Slices" → E4 spec; §"Parallelization" sequencing.
- [`docs/briefs/r3-evaluator-e0-body-evaluator-api-scaffold.md`](r3-evaluator-e0-body-evaluator-api-scaffold.md)
  (#1371) — body-evaluator API contract (`evaluate_body`, `eval_node`,
  `eval_port`, `EvalDiagnostic`).
- [`docs/briefs/r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md)
  §B.1.3 — Branch evaluation rule.
- E2 implementation in `src/v3/compiler/src/lib.rs::evaluator` (#1374)
  — `EvalFrame<V>` and `EvalStateStack<V>`.

## State at HEAD

- E0 brief landed docs-only — no Rust evaluator API exists yet.
- E2 frame helpers landed as `pub mod evaluator { … }` inline in
  `src/v3/compiler/src/lib.rs` with `EvalFrame<V>` / `EvalStateStack<V>`
  generic on the value type `V`. No runtime `Value` carrier; `V` is a
  type parameter the slice never instantiates.
- **E1 has not landed.** Verified at `origin/main` `8f051650`:
  - `grep -rn "enum Value\b" src/v3/compiler/src/` returns 0 hits for a
    runtime `Value` carrier (only `ValueNode` / `FieldValue` /
    `SubValueRelation`, none of which are the runtime `Value` from
    `src/v3/std/runtime.dag`).
  - No `eval_value`, `eval_port`, or `eval_node` exist outside
    `lens_apply.rs::EvalCtx` — and the lens-apply module operates on
    `FieldValue`, not the runtime `Value`. Per E0 brief §"Why E0
    exists", extending `lens_apply.rs` would create the parallel-authority
    debt the runner-authority ratchet brief is meant to ratchet down,
    so the body evaluator must live in its own module.
- `BranchNode` / `BranchPattern` / `BranchPath` / `PayloadBinding`
  Rust types exist in `src/v3/compiler/src/dag.rs`; no new mirror
  needed for E4's substrate inputs.

## Supersedes (E0 contract corrections)

This audit corrects two E0 contract holes the merged E0 brief
(`r3-evaluator-e0-body-evaluator-api-scaffold.md`, #1371) left
unspecified. **These corrections supersede E0 for E1 implementation
purposes** until a follow-on docs amendment to E0 mirrors the wording:

- **E0 §"Port-level evaluation contract" step 1 is superseded** by this
  audit's §2 "Port membership check first" — `eval_port` must call
  `dag.port_opt(&port)` *before* `frame_lookup`, not after; a `None`
  port returns `EvalDiagnostic::ResolveError(port)`. The E0 sequence
  "frame lookup first" admits caller-supplied bindings on
  nonexistent `PortId`s and violates Fail-Closed (per the merged
  reviewer finding on E0).
- **E0 §"Per-`Behavior` slice fills" `eval_value` signature is
  superseded** by this audit's §2a "Producer-bind invariant" — every
  per-`Behavior` evaluator (including `eval_value`) takes
  `stack: &mut EvalStateStack` and calls `stack.bind_top(node.result_port,
  value)` before returning. Without this, `eval_port`'s
  producer-demand-eval branch cannot cache a `Value`-produced port
  and Facts-Flow-Forward fails.

E1's implementation MUST follow this audit's wording for these two
points; the merged E0 wording is the older draft. A short follow-on
PR amending E0 is the cleanest landing for the contract; until then
this audit is the authoritative reference for E1 / E4 consumers.

## E1 prerequisites E4 depends on (concrete inventory)

E4 cannot ship as a bounded slice until **all four** of these land via
E1 (or a tiny E1.0 unblock per the routing the parent already accepted):

### 1. Runtime `Value::VariantValue` tag/payload access

PR-B.1 §B.1.3 step 1 says E4's scrutinee evaluation must produce a
`Value::VariantValue { tag: DeclarationId, payload: Value }` (per
`src/v3/std/runtime.dag:46-47`). The Rust mirror E1 must declare needs
at minimum:

```text
pub enum Value {
    LiteralValue(LiteralBits),                            // E1 baseline
    VariantValue { tag: DeclarationId, payload: Box<Value> },
    // RecordValue / NodeRef / CardinalityValue may stay
    // NotYetImplemented arms until later slices need them; E4
    // requires only LiteralValue (parameter passes) + VariantValue
    // (scrutinee shape).
}
```

E4 needs **read access** to `tag: DeclarationId` and `payload: Value`
from a `Value::VariantValue` to perform `ResolvedVariant(decl) ==
tag` matching and to bind the payload via E2's `frame_bind`.

### 2. `eval_port` (port-level demand-eval authority)

Per E0 §"Port-level evaluation contract", `eval_port(dag, port,
stack, strategy) -> Result<Value, EvalDiagnostic>` is the single
port-resolution authority. E4's scrutinee evaluation calls
`eval_port(dag, b.input, stack, strategy)` to get the scrutinee
`Value`. Without `eval_port`, E4 either redoes demand-eval inline
(parallel-authority debt) or cannot resolve the scrutinee at all.

`eval_port`'s implementation depends on:
- **Port membership check first.** Before any frame lookup,
  `eval_port` must call `dag.port_opt(&port)`; if `None`, fail
  closed with `EvalDiagnostic::ResolveError(port)` immediately.
  This makes `Dag` port-membership a substrate fact at the
  boundary, so a caller-supplied initial frame containing a
  `PortId` not in the program cannot evaluate successfully (per
  Fail-Closed and "Every Dependency Is A Substrate Fact"). The E0
  brief's port-level contract said "frame lookup first" without
  this prerequisite — E1's implementation must perform the
  membership check ahead of frame lookup, and a follow-on docs
  amendment should clarify E0's wording. Order is:
  `dag.port_opt(&port)` → `frame_lookup(stack, port)` →
  producer-node demand-eval via `produced_by`.
- `EvalStateStack::lookup(port)` — already shipped in E2 as
  `evaluator::EvalStateStack<V>::lookup` (`lib.rs::evaluator` per
  #1374). Generic on `V`; needs to be instantiated at `Value`.
- `dag.port_opt(&port).and_then(|p| p.produced_by)` — already
  available in `dag.rs:3237` + the producer-follow logic at `:3061`.
- Recursive `eval_node` for producer demand-eval — see (3) below.

### 2a. Producer-bind invariant — every `Behavior` evaluator binds its `result_port`

E0's port contract says `eval_port`'s producer-demand-eval branch
calls `eval_node(producer)` and then re-does `frame_lookup(port)`,
expecting the value to be bound. That requires **every per-`Behavior`
evaluator to `frame_bind(node.result_port, value)` on success** —
including `eval_value`. E0's `eval_value` signature was
`fn eval_value(node: &ValueNode) -> Result<Value, EvalDiagnostic>`
with no `stack` parameter, which would prevent the binding.

E1's signature must be:
```text
fn eval_value(
    node: &ValueNode,
    stack: &mut EvalStateStack,
) -> Result<Value, EvalDiagnostic>;
```

`eval_value` returns the `Value` AND calls
`stack.bind_top(node.result_port, value.clone())?` before returning,
so subsequent `eval_port` calls for `node.result_port` resolve via
`frame_lookup` instead of re-triggering demand-eval. Same invariant
for `eval_transform` (binds `t.output`), `eval_loop` (binds
`l.result_port`), `eval_branch` (binds `b.result_port`), and
`eval_bind` (already binds explicitly).

This is "Facts Flow Forward" at the port boundary: a producer's
output is durable in the frame stack after evaluation, not
recomputed each time a downstream port reader looks it up. (E0's
`eval_value` signature was a docs slip; E1 must fix it.)

### 3. `eval_node` dispatch shell

E0 §"Internal dispatch entrypoint" specifies
`eval_node(dag, node, stack, strategy) -> Result<Value, EvalDiagnostic>`
that resolves via `dag.node_opt(&node)` and pattern-matches on
`Behavior` to dispatch to per-`Behavior` handlers. E4 fills only
`eval_branch`; it needs `eval_node` itself to exist as a dispatch
shell so its branch-body recursion is well-defined.

E1 (or E1.0) ships the shell with all five arms returning a
fail-closed `EvalDiagnostic::NotYetImplemented(...)` placeholder
except `eval_value` (which E1 implements). E4 then replaces the
`Branch` arm's `NotYetImplemented` with `eval_branch`.

### 4. Frame `bind`/`push`/`pop` discipline reaching `EvalStateStack<Value>`

E2's `EvalFrame<V>` / `EvalStateStack<V>` are generic. E4's frame
operations (push fresh `EvalFrame<Value>`, `bind_top(payload_port,
payload)`, evaluate body, `pop_frame()`) require `V = Value`.
Instantiation lives in E1 (or wherever the runtime `Value` mirror
lands). E4 must consume the instantiated stack, not introduce a new
`Value`-shaped frame mirror.

`EvalFrameError` from E2 (`EmptyStateStack` / `DuplicateBinding` /
`UnboundPort`) maps to the matching `EvalDiagnostic` variants from
E0's catalog. E4 propagates these unchanged; it does not invent new
diagnostic variants.

## What E4 will do once unblocked (preview, not implemented here)

Per PR-B.1 §B.1.3, E4 implements `eval_branch(dag, b: &BranchNode,
stack, strategy) -> Result<Value, EvalDiagnostic>`:

1. `let scrutinee = eval_port(dag, b.input, stack, strategy)?;`
2. Match `scrutinee` to `Value::VariantValue { tag, payload }`;
   anything else → `EvalDiagnostic::BranchShapeMismatch`.
3. Walk `b.paths`, find the unique `BranchPath` whose
   `pattern: ResolvedVariant(decl)` has `decl == tag`;
   - `pattern: UnresolvedVariant { name, .. }` → `EvalDiagnostic::BranchUnresolvedVariant { name }`.
   - No matching path → `EvalDiagnostic::BranchShapeMismatch` (substrate
     guarantees totality on well-typed programs; this case is an
     invariant violation diagnostic).
4. `stack.push_frame(EvalFrame::empty())`. If the path has a
   `binding: PayloadBinding { binding_name, payload_port }`, call
   `stack.bind_top(payload_port, *payload)?`.
5. Evaluate `eval_node(dag, path.body, stack, strategy)` and capture
   the `Value`.
6. `stack.pop_frame()?`.
7. Return the body `Value`. Pop runs on both success and
   `Diagnostic` paths so a balanced-stack invariant holds.

Tests E4 must include (per dispatch §E4 acceptance):
- Resolved variant match — branch picks the path whose
  `ResolvedVariant.tag` equals the scrutinee `VariantValue.tag`;
  payload binding visible inside the body frame.
- `UnresolvedVariant` fail-closed diagnostic at evaluation time.
- No matching path / scrutinee shape mismatch fail-closed diagnostic.
- (Implicit) frame depth on entry == frame depth on exit, both on
  success and on diagnostic paths.

## Cross-references

- PR-E dispatch brief E4 spec — `r3-evaluator-dispatch.md` §E4.
- PR-B.1 Branch rule — `r2-pr-b-1-eager-evaluator-implementation-seed.md` §B.1.3.
- E0 API contract — `r3-evaluator-e0-body-evaluator-api-scaffold.md`
  (#1371).
- E2 frame helpers — #1374 implementation; module `evaluator` inline
  in `src/v3/compiler/src/lib.rs`.

## Out of scope (this audit)

- Implementing `eval_branch`, `eval_value`, `eval_port`, `eval_node`,
  or the runtime `Value` mirror.
- Authoring substrate carriers.
- Adding tests or fixtures.
- Choosing whether E1 or a tiny E1.0 ships first — the parent already
  accepted the routing; this audit just records the dependency
  shape.

## STOP+PING boundary (E4 itself, when unblocked)

Per E0 brief, E4 must NOT:
- Add a wildcard / catch-all `BranchPattern` variant — substrate has
  only `UnresolvedVariant | ResolvedVariant`.
- Add a new `Value` variant or a parallel runtime carrier.
- Mutate non-top frames, fork its own demand-eval helper, or call
  `frame_lookup` directly for input resolution (must use `eval_port`).
- Touch `lens_apply.rs::EvalCtx`.
- Invent new `EvalDiagnostic` variants — propagate the existing
  catalog.
