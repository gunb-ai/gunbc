# R2 PR-B.1 Eager Body Evaluator — Implementation Seed Checklist

**Status:** PROPOSAL — docs-only design seed for the executable PR-B.1 slice
that ships after Worker A lands the PR-A.3 strategy / memoization carriers.
This brief extends the PR-B.0 design surface
([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md))
with a concrete behavior-by-behavior implementation checklist, frame
discipline rules, and `Map<PortId, Value>` access discipline. **No Rust
evaluator code, substrate carriers, fixtures, lazy/normal-order rules, TC2
strict equality, or witness construction land in this brief.**

**Parent design lock:** [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md).
**Substrate carrier authority:** [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag)
(`Value`, `NamedField`, `EvalFrame`, `EvalStateStack`).
**Strategy / memo carrier authority (audit; not yet declared in `runtime.dag`):**
[`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md).
**Interpreter rules:** [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
§3.

## Hard prerequisite — Worker A PR-A.3 carriers

PR-B.1 cannot start until **all four** PR-A.3 implementation carriers exist
in `src/v3/std/runtime.dag` (or its successor PR-A.3 module slice). PR-B.1
imports them; it does not declare them.

- `EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }`
- `InputEvaluationOrder = LeftFirst`
- `EvalStateKey { state: EvalStateStack }`
- `EvalMemoKey { program: DeclarationId, node: NodeId, state_key: EvalStateKey, strategy: EvalStrategy }`

Until these land, PR-B.1 is **blocked** with no scope expansion. PR-B.0 is
the active R2 design lock; this seed brief authors PR-B.1's first-shippable
checklist for when the gate clears.

## Behavior dispatch checklist (5 of 5 substrate variants)

Each row below is one PR-B.1 acceptance check. PR-B.1 must implement each
in order; merge cannot proceed if any row's fail-closed boundary is missing.

### B.1.1 — `Value`

- **Input:** `ValueNode { id, payload: LiteralBits, result_port, span, lane2_workflow }`.
- **Rule:** return `Value::LiteralValue(payload)` directly. No frame mutation.
- **Fail-closed boundary:** none — `Value` is total over its inhabitants.

### B.1.2 — `Transform`

- **Input:** `TransformNode { id, target: TransformTarget, inputs: List<PortId>, result_port, span }`.
- **Rule:**
  1. Eager-evaluate `inputs` left-to-right per `InputEvaluationOrder::LeftFirst`,
     producing `List<Value>`. (Reads strategy from imported `EvalStrategy`.)
  2. Dispatch on `TransformTarget`:
     - `Callable(decl_id)` → resolve `decl_id`'s `ArrowBody` (see §B.1.6 below).
     - `FieldProject` → return the named field of the record-shaped input.
       Only `RecordValue(List<NamedField>)` inputs are valid; any other
       `Value` shape is fail-closed.
     - `Operator(op)` → apply the operator to its primitive operand
       inhabitants per the locked operator dispatch (see PR-A operator
       table in `dsl/std/algebra.dag`). Operators with no PR-B.1 lowering
       are **fail-closed** (see §B.1.7).
- **Fail-closed boundary:**
  - `Callable` over an unsupported `ArrowBody` variant (§B.1.6) — `Diagnostic`.
  - `Operator` not yet lowered for PR-B.1 (§B.1.7) — `Diagnostic`.
  - `FieldProject` over a non-record `Value` — `Diagnostic`.

### B.1.3 — `Branch`

- **Input:** branch node with a scrutinee port and a list of paths (path
  pattern + body `NodeId`).
- **Rule:**
  1. Eager-evaluate the scrutinee (`Value`).
  2. Select the path whose pattern matches the scrutinee per the
     pattern-match rule. Patterns are exhaustive over the scrutinee's
     `Value` shape by construction; the wildcard / catch-all path is the
     `Disj` total fallback.
  3. Push a fresh `EvalFrame` containing the path-binding (the matched
     payload bound to the path's parameter `PortId`).
  4. Evaluate the selected path's body in that frame.
  5. Pop the frame; return the body's result.
- **Fail-closed boundary:**
  - Scrutinee whose shape no path can match (substrate guarantees totality,
    so this is a structural invariant violation, not a runtime case) —
    `Diagnostic`.

### B.1.4 — `Loop`

- **Input:** loop node with init port, accumulator port, body `NodeId`, and
  `LoopBound`.
- **Rule:** dispatch on `LoopBound` per the parent design lock
  ([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md)
  Loop rule):
  - `LoopBound::Cardinality { count: PortId }`:
    1. Eager-evaluate the init.
    2. Read the cardinality witness through the declared `count` port. If
       absent / non-integer — fail-closed `Diagnostic`.
    3. For `i ∈ [0, count)`: push a fresh `EvalFrame` containing the
       accumulator binding for iteration `i`, evaluate the body, pop the
       frame, thread the result into the next iteration's accumulator.
    4. Return the final accumulator value.
  - `LoopBound::Descent { cluster: ClusterId }`: **named fail-closed
    residual at PR-B.1**. Emit a `Diagnostic` and do not iterate. PR-B.1
    must not silently broaden to descent execution; that rule belongs in a
    separate descent-execution slice consuming `std.termination`
    `DescentEvidence`.
- **Fail-closed boundary:**
  - `Cardinality` with missing or non-integer count — `Diagnostic`.
  - `Descent` (any cluster) — `Diagnostic` (residual; see above).

### B.1.5 — `Bind`

- **Input:** `BindNode` with `name`, `params: List<PortId>`, `result_port`.
- **Rule:** register the binding in the current `EvalFrame`:
  `frame.bindings = map_insert(frame.bindings, port, value)` for the
  parameter / result port being bound. **No body execution at the Bind
  site** — the body that uses this binding lives at downstream
  `Transform(Callable(...))` or `Bind` references per
  `docs/design-pb-runtime-interpreter.md` §3.2 / §3.3.
- **Fail-closed boundary:** if `frame.bindings` already contains the
  `PortId` (duplicate binding for one port in the same frame) — that
  state is unrepresentable per `Map<PortId, Value>`, so detection is
  upstream of PR-B.1; if PR-B.1 ever observes it, it is a substrate
  invariant violation and emits a `Diagnostic`.

### B.1.6 — `ArrowBody` dispatch (called from §B.1.2 `Callable`)

- `ArrowBody::UserDefined(body_node_id)`:
  1. Push a fresh `EvalFrame` onto `EvalStateStack`.
  2. Bind each `Transform` input to the corresponding parameter `PortId`
     in that frame (parameter list comes from the resolved declaration).
  3. Evaluate `body_node_id` in the new frame.
  4. Pop the frame.
  5. Return the body's `Value`.
- `ArrowBody::ExternalRealization(decl)`: dispatch to the host-bound
  implementation. PR-B.1 itself does not execute external bodies; if no
  host-binding lookup table exists at PR-B.1 land time, this is **fail-closed
  `Diagnostic`** (named residual: host-binding lookup slice).
- `ArrowBody::Pending` / `ArrowBody::NoBody` / `ArrowBody::Unparsed(span)`:
  fail-closed `Diagnostic` per `feedback_fail_closed_discipline`. These
  are evaluation-time errors, not panics or silent `None`s.

### B.1.7 — `Operator` dispatch (called from §B.1.2 `Operator(op)`)

- PR-B.1 lowers exactly the operator set the parent brief defines as
  in-scope; that set is read from the operator-dispatch authority
  (`dsl/std/algebra.dag` + the existing operator-emission tables in
  `extdeps`).
- Any operator without a PR-B.1 lowering rule is **fail-closed
  `Diagnostic`**, named residual: operator coverage slice. PR-B.1 must
  not silently coerce or skip.

## Frame push/pop discipline (`EvalStateStack`)

PR-B.1 is the only authority that mutates `EvalStateStack`. The discipline
is point-of-use scoped:

- **Push exactly when** a fresh binding scope opens: `Transform(Callable
  ArrowBody::UserDefined)` call (§B.1.6), `Branch` path body (§B.1.3),
  `Loop` iteration body (§B.1.4 `Cardinality` arm only).
- **Pop exactly when** that scope closes: after the body's `Value` is
  computed (or after a fail-closed `Diagnostic` is raised within the
  scope). Pop is not optional and must run on both success and diagnostic
  paths — PR-B.1 cannot leak a frame.
- **No top-of-stack mutation outside push/pop.** `Bind` registers within
  the current frame's `bindings` map; that is field-level mutation of
  `EvalFrame`, not stack mutation.
- **Initial stack** for a top-level evaluation is one `EvalFrame` with
  the caller-supplied parameter bindings (per PR-B.0 §First implementation
  target).
- **Frame leak detection** is structural, not heuristic: every body's
  evaluator wrapper pushes-then-pops, and PR-B.1 must enforce balanced
  push/pop with a fail-closed assertion at the wrapper boundary if depth
  on entry ≠ depth on exit. (The assertion failure case is a substrate
  invariant violation, treated as `Diagnostic`.)

## `Map<PortId, Value>` lookup / update discipline

`EvalFrame.bindings: Map<PortId, Value>` is the only binding-scope
read/write surface in PR-B.1. The discipline:

- **Lookup.** Reading a parameter / let-bound port resolves through the
  innermost-frame-first walk of `EvalStateStack.frames`. PR-B.1 reads
  through whatever lookup primitive `Map<K, V>` exposes (`map_get` /
  equivalent in `dsl/std/types.dag`). A `None` from `map_get` walking
  the entire stack is fail-closed (`Diagnostic`: unbound port at use
  site).
- **Update.** PR-B.1 only writes to the **innermost** (top-of-stack)
  frame's `bindings`. Outer frames are immutable from PR-B.1's
  perspective. Updates use the `Map<K, V>` insert primitive and replace
  the top frame in-place. (This is structural per
  `feedback_state_space_vs_behavioral_invariants` — the type already
  guarantees per-`PortId` uniqueness inside one frame; PR-B.1 enforces
  per-stack scoping rather than carrier-level uniqueness.)
- **No cross-frame writes.** PR-B.1 must not write to a non-top frame.
  Doing so would conflate "binding extension" with "outer-scope
  mutation," which the substrate does not admit.
- **No global / module-level mutable state.** Module-level `data`
  declarations are read-only.

## Strategy + memoization discipline

- **Strategy.** PR-B.1 reads `EvalStrategy` (declared by PR-A.3) and
  honors only the inhabitant `ApplicativeOrder { input_order: LeftFirst }`.
  Any other inhabitant emits a fail-closed `Diagnostic`.
- **Memoization (optional in PR-B.1).** If the slice elects to ship
  memoization, it must:
  1. Key on `EvalMemoKey { program, node, state_key, strategy }` exactly
     — no string digests, no name-only proxies.
  2. Cache only completed `Value` results. Diagnostics and partial state
     are never cached.
  3. Treat a memo miss as identical to "evaluate normally" — the cache
     must not introduce any observable behavior change relative to a
     no-memo run on the same input.
- **No `EvalThunk` use.** Eager-only baseline. `EvalThunk` is unreachable
  in PR-B.1 by construction.

## Fail-closed catalog (single index)

Every `Diagnostic` PR-B.1 emits matches one of these named residuals or
substrate-invariant cases. Each row names the dissolution trigger.

| Source                                              | Variant kind          | Dissolution trigger                                          |
|-----------------------------------------------------|-----------------------|--------------------------------------------------------------|
| `ArrowBody::ExternalRealization`                    | host-binding gap      | Host-binding lookup slice (separate worker brief)            |
| `ArrowBody::Pending`                                | substrate residual    | Pipeline slice that lowers Pending to executable             |
| `ArrowBody::NoBody`                                 | substrate residual    | Author the body or mark non-executable upstream              |
| `ArrowBody::Unparsed(span)`                         | parser lag            | Parser catches up; M2 surface coverage                       |
| `LoopBound::Descent { cluster }`                    | execution residual    | Descent-execution slice consuming `std.termination`          |
| `Operator(op)` not yet lowered                      | operator coverage gap | Operator-coverage slice (per-op rule + receipt)              |
| `Transform(FieldProject)` over non-record `Value`   | invariant violation   | Substrate guarantee — never reached on well-typed program    |
| `EvalStrategy` inhabitant other than `ApplicativeOrder/LeftFirst` | strategy residual     | PR-A.3 expansion + lazy / `EvalThunk` slice                  |
| `LoopBound::Cardinality` count missing / non-int    | runtime data gap      | Caller-side rule: cardinality witness must be available      |
| Unbound `PortId` at use site                        | resolve gap           | Resolution pass; should be caught upstream (Diagnostic here) |
| Frame depth on exit ≠ on entry                      | invariant violation   | PR-B.1 internal assertion; never reached on correct evaluator|

## Out of scope (this brief, PR-B.1 seed)

- Rust evaluator code, substrate carriers, fixtures, lazy / normal-order
  evaluation, TC2 strict-equality strengthening, witness construction,
  cross-target harness execution, `EvalThunk`, `ClosureValue`. All deferred
  per PR-B.0 R3 residual list.
- Worker dispatch instructions for PR-B.1 implementation. This brief is
  the design seed; dispatch happens after the PR-A.3 prerequisite carriers
  land.
- Any modification of PR-B.0 or PR-A.3 audit content. This brief consumes
  both as authorities.

## Acceptance gates (this brief)

- ✅ Five `Behavior` variants each have a one-paragraph rule + fail-closed
  boundary.
- ✅ Frame push/pop discipline rules cover every push site and every pop
  site, with a balanced-stack invariant.
- ✅ `Map<PortId, Value>` lookup/update rules name innermost-frame
  precedence and prohibit non-top writes.
- ✅ Fail-closed catalog enumerates every `Diagnostic` source PR-B.1 can
  emit, each with a dissolution trigger.
- ✅ Hard prerequisite (PR-A.3 carriers) named with all four carrier
  declarations.
- ✅ Docs-only PR. No Rust, no substrate, no fixtures.
