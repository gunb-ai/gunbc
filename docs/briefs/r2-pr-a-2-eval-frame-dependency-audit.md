# R2 PR-A.2 EvalFrame / EvalStateStack — Dependency Audit

**Status:** AUDIT — produced while PR-A.2 is blocked on PR-A.1 (`Value`
authority). The map-carrier blocker the original draft of this audit asserted
has been retracted: `Map<K, V>` already exists as a single authority at
`dsl/std/types.dag:213`. Docs only; no carrier declarations land here.

**Parent authority:** [`docs/briefs/r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md)
(merged in #1197). **PB-Runtime authority:**
[`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §3.2–3.3.

## Purpose

Inventory the substrate PR-A.2 will consume so that, once PR-A.1 lands,
PR-A.2 reduces to a small structural diff (carrier declarations + strengthened
`TestClaim`) with zero parallel-authority debt. Per
`feedback_enumerate_before_substrate`: list every consumer-side question
before editing substrate.

## Existing authorities PR-A.2 will import

All atomic identity handles and the keyed-collection carrier already exist;
PR-A.2 imports them rather than redeclaring:

| Concept             | Authority                                                   | Notes                                                  |
|---------------------|-------------------------------------------------------------|--------------------------------------------------------|
| `PortId`            | `src/v3/std/substrate_minimal.dag:26`                       | opaque atomic id; realized as Rust newtype in `dag.rs` |
| `NodeId`            | `src/v3/std/substrate_minimal.dag:25`                       | opaque atomic id                                       |
| `DeclarationId`     | `src/v3/std/substrate_minimal.dag:27`                       | opaque atomic id                                       |
| `LiteralBits`       | `src/v3/std/substrate.dag:30`                               | needed transitively via `LiteralValue`                 |
| `LoopBound`         | `src/v3/std/substrate.dag:342`                              | needed transitively via `CardinalityValue`             |
| `List<T>`           | `src/v3/std/list.dag:43`                                    | `EvalStateStack.frames: List<EvalFrame>` consumes this |
| `Map<K, V>`         | `dsl/std/types.dag:213` (`= PartialFunction<key, value>`)    | `EvalFrame.bindings: Map<PortId, Value>`; importable from `std.types` (precedent: `src/v3/std/list.dag:33`, `lookup.dag:21`) |
| `TestClaim` / `Compiles` / `TestSuite` | `std.verification`                            | already used by the slice-0 fixture                    |

## The single remaining blocker: PR-A.1 (`Value` and `NamedField`)

No `Value`, `LiteralValue`, `RecordValue`, `VariantValue`, `NodeRef`,
`CardinalityValue`, or `NamedField` declaration exists in `src/v3/std/`.
Verified via
`grep -rn "type Value\|RecordValue\|LiteralValue\|NamedField" src/v3/std/`
against `origin/main` at audit time (re-verified at `8d451d1a0`: 0 matches for
the runtime `Value` carriers; the hits in `clean_emission.dag`,
`emit_model.dag`, and `substrate.dag` are unrelated `ValueBody`,
`ValueConstructionSyntax`, `ValueNode`, and `OverrideNamedFieldsAtBindingSite`
declarations from other layers). PR-A.1 must land first; PR-A.2 must not invent
a duplicate per `feedback_parallel_representation_debt`.

The `Value` naming side has its own decided upstream path: per the Substrate
Manager disposition (jolly-ram-908), the existing L1 behavior marker `Value`
will be renamed (`Value` → `ValueBehavior`, full marker-set rename preferred)
to free bare `Value` for PR-A.1's runtime declaration. PR-A.2 does not need to
participate in that rename; it consumes whatever PR-A.1 lands.

## On `Map<K, V>` — single authority already exists

An earlier draft of this audit incorrectly claimed no instantiable `Map<K, V>`
existed and proposed a `UniqueBindings<K, V>` parallel carrier. That was a
grep-scope error: the original search only covered `src/v3/std/`. `dsl/std/`
is the L1 authority layer and already declares:

```
type Map<key, value> = PartialFunction<key, value>          // dsl/std/types.dag:213
type List<element>   = FreeMonoid<element>                  // dsl/std/types.dag:211
type Set<element>    = BooleanAlgebra<element>              // dsl/std/types.dag:212
```

with the algebraic-inhabitation comment block immediately above explaining
that `Map<K,V> = K → (1 + V)` is a keyed partial function and inherits
`lookup`, `insert`, `delete`, `merge` from `PartialFunction<K, V>`. v3 modules
already consume `std.types` (e.g., `src/v3/std/list.dag:33` imports `Int`,
`Bool`; `src/v3/std/lookup.dag:21` imports `Int`; `src/v3/std/verification.dag:20`
imports `FilePath`, `NonEmptyStr`), so `import std.types { Map }` is the
existing-convention path.

Introducing a separate `UniqueBindings<K, V>` would create a parallel
keyed-collection authority alongside `Map = PartialFunction`, in violation of
M9 (DFS the concept DAG before declaring a new type) and the single-authority
rule. Retracted.

The `PartialFunctionProfile` tag in `src/v3/std/computation.dag:18` is a
*kernel-algebra profile tag* — metadata describing an existing type's
algebraic shape — not the carrier itself; the carrier is the `Map = PartialFunction`
declaration in `dsl/std/types.dag`. `SurfaceMapEntry` (`src/v3/std/parse_surface.dag:96`)
is parse-surface-only with `String` keys and is structurally wrong for
runtime evaluator state. Neither is a substitute for `Map<PortId, Value>`,
but neither is needed: the canonical `Map` already exists.

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

Assuming PR-A.1 lands `Value` and `NamedField` in `src/v3/std/runtime.dag`,
PR-A.2's runtime-module diff is:

```dag
module std.runtime  // already opened by PR-A.1

import std.substrate_minimal { PortId }
import std.list { List }
import std.types { Map }

type EvalFrame {
  bindings: Map<PortId, Value>
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
this audit; that is post-`.dag`-substrate work. No new keyed-collection
carrier (`Map = PartialFunction` is the single authority).

## Pre-merge checklist for PR-A.2 itself (when unblocked)

1. Confirm PR-A.1 has landed and `Value` / `NamedField` are importable from
   `std.runtime`.
2. Declare `EvalFrame { bindings: Map<PortId, Value> }` and
   `EvalStateStack { frames: List<EvalFrame> }` in `src/v3/std/runtime.dag`,
   importing `Map` from `std.types`.
3. Strengthen
   `src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag` past
   `Compiles` to a structural claim that names the new carriers.
4. No `ClosureValue`. No `List<EvalBinding>`. No `UniqueBindings` parallel
   keyed-collection authority. No second runtime `Value` authority.
