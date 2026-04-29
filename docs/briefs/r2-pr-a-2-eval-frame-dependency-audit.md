# R2 PR-A.2 EvalFrame / EvalStateStack — Dependency Audit

**Status:** AUDIT — produced while PR-A.2 is blocked on PR-A.1 (`Value` authority)
and on a fail-closed unique-binding carrier for `Map<PortId, Value>`. Docs only;
no carrier declarations land here.

**Parent authority:** [`docs/briefs/r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md)
(merged in #1197). **PB-Runtime authority:**
[`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §3.2–3.3.

## Purpose

Inventory the substrate PR-A.2 will consume so that, once PR-A.1 lands and a
unique-binding carrier is available, PR-A.2 reduces to a small structural diff
(carrier declarations + strengthened `TestClaim`) with zero parallel-authority
debt. Per `feedback_enumerate_before_substrate`: list every consumer-side
question before editing substrate.

## Existing authorities PR-A.2 will import

All atomic identity handles already exist; PR-A.2 imports them rather than
redeclaring:

| Concept             | Authority                                                   | Notes                                                  |
|---------------------|-------------------------------------------------------------|--------------------------------------------------------|
| `PortId`            | `src/v3/std/substrate_minimal.dag:26`                       | opaque atomic id; realized as Rust newtype in `dag.rs` |
| `NodeId`            | `src/v3/std/substrate_minimal.dag:25`                       | opaque atomic id                                       |
| `DeclarationId`     | `src/v3/std/substrate_minimal.dag:27`                       | opaque atomic id                                       |
| `LiteralBits`       | `src/v3/std/substrate.dag:30`                               | needed transitively via `LiteralValue`                 |
| `LoopBound`         | `src/v3/std/substrate.dag:342`                              | needed transitively via `CardinalityValue`             |
| `List<T>`           | `src/v3/std/list.dag:43`                                    | `EvalStateStack.frames: List<EvalFrame>` consumes this |
| `TestClaim` / `Compiles` / `TestSuite` | `std.verification`                            | already used by the slice-0 fixture                    |

## Authorities PR-A.2 cannot import yet (the actual blockers)

### 1. `Value` and `NamedField` — owned by PR-A.1

No `Value`, `LiteralValue`, `RecordValue`, `VariantValue`, `NodeRef`,
`CardinalityValue`, or `NamedField` declaration exists in `src/v3/std/`. Verified
via `grep -rn "type Value\|RecordValue\|LiteralValue\|NamedField" src/v3/std/`
at HEAD `df458e7b7` (0 matches). PR-A.1 must land first; PR-A.2 must not invent
a duplicate per `feedback_parallel_representation_debt`.

### 2. `Map<PortId, Value>` — no instantiable carrier exists

`PartialFunction` exists in v3 only as a *tag* in the kernel algebra profile
(`PartialFunctionProfile` in `src/v3/std/computation.dag:18`,
`kernel_algebra_profile` lookup at `:19`). It is metadata about an existing
type's algebraic shape, not an instantiable generic container.

`SurfaceMapEntry` (`src/v3/std/parse_surface.dag:96`) is parse-surface-only,
carries `String` keys and surface expressions, and is bound to the surface
parser. It is structurally wrong for runtime evaluator state (key type, value
type, and authority layer all mismatch).

No `type Map<K, V>` declaration exists anywhere under `src/v3/std/` or
`src/v3/`. The merged PR-A.0 brief (§Substrate Targets) is explicit:

> If `Map<K, V>` is not yet available in the v3 runtime module when PR-A.2
> lands, PR-A.2 must either port the existing `Map = PartialFunction` authority
> or add a fail-closed unique-binding carrier before mirroring evaluator state;
> it must not fall back to a duplicate-admitting `List<EvalBinding>`.

There is no "existing `Map = PartialFunction` authority" to port today; the
`PartialFunctionProfile` tag does not constitute a carrier. So PR-A.2 (or a
prerequisite slice) must declare a fail-closed unique-binding carrier as a
genuine substrate addition. Routing question for the Substrate Manager: does
this prerequisite live (a) inside PR-A.2's scope, (b) as a separate Substrate
Manager workstream, or (c) bundled into PR-A.1? The merged brief permits (a),
but does not require it.

## Candidate module path

PR-A.0 §Substrate Targets calls for "a runtime module under the v3 std/runtime
surface once that directory is introduced by the Evaluator or PB-Runtime lane."
At HEAD, `src/v3/std/runtime.dag` does not exist. Existing convention places
each std concept in a flat file under `src/v3/std/`
(`substrate.dag`, `computation.dag`, `lens.dag`, `verification.dag`, etc.), not
in subdirectories. Recommended path:

- `src/v3/std/runtime.dag` — single authority module declaring `Value`
  (PR-A.1), `NamedField` (PR-A.1), `EvalFrame` (PR-A.2), `EvalStateStack`
  (PR-A.2), and later `EvalThunk` / `EvalStrategy` / `EvalMemoKey` (PR-A.3).

This matches the brief's "one runtime authority module" requirement
(§Implementation Gates item 1) and the existing flat layout convention. If the
Substrate Manager prefers a `std/runtime/` subdirectory, PR-A.1 should
establish that — PR-A.2 follows whatever PR-A.1 picks rather than relitigating.

## Imports PR-A.2 will need (post-unblock)

Assuming PR-A.1 lands `Value` and `NamedField` in `src/v3/std/runtime.dag` and
a unique-binding carrier (call it `UniqueBindings<K, V>`, exact name TBD by the
authority that lands it) is available, PR-A.2's runtime-module diff is:

```dag
module std.runtime  // already opened by PR-A.1

import std.substrate_minimal { PortId }
import std.list { List }
// + whatever module declares the unique-binding carrier

type EvalFrame {
  bindings: UniqueBindings<PortId, Value>  // or Map<PortId, Value>
}

type EvalStateStack {
  frames: List<EvalFrame>
}
```

Plus a strengthened claim in
`src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag` —
replacing the slice-0 `Compiles` predicate with a claim that exercises the
declared carriers structurally (exact predicate shape depends on what
verification primitives are available; PR-A.2 can decide once the carrier
declarations exist).

## Out of scope for PR-A.2

Per dispatch and parent brief: no `Value` authority (PR-A.1), no `EvalThunk` /
`EvalStrategy` / `EvalMemoKey` (PR-A.3), no body evaluator (PR-B), no
`ClosureValue` (forbidden; would be a P1 substrate change). No Rust mirror in
this audit; that is post-`.dag`-substrate work.

## Pre-merge checklist for PR-A.2 itself (when unblocked)

1. Confirm PR-A.1 has landed and `Value` / `NamedField` are importable from
   `std.runtime`.
2. Confirm a unique-binding carrier exists, OR include one in PR-A.2's scope
   with Substrate Manager sign-off.
3. Declare `EvalFrame` and `EvalStateStack` in `src/v3/std/runtime.dag`.
4. Strengthen
   `src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag` past
   `Compiles` to a structural claim that names the new carriers.
5. No `ClosureValue`. No `List<EvalBinding>`. No second runtime `Value`
   authority.
