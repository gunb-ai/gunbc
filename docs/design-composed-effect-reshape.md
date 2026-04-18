> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2 Stage 2a follow-up) | Unblocks: Lane 2 Stage 2b (workflow idempotency lens), Track 17a REST wiring

# Design Brief — `ComposedEffect` reshape

**Brief:** Brief A (Lane 2 Stage 2a / Track 17a boundary follow-up)
**Consumers:** Stage 2b workflow-idempotency lens; Stage 2c test-obligation materialization; Track 17a REST consumers (once wired).
**Status:** Implemented — with an R2 revision in response to reviewer feedback on commit `b42edc15`. The initial reshape (R1) lifted the `Bool + String?` summary into `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: OperationEffect }`; the reviewer correctly pointed out that `OperationEffect` still admits any `EffectShape`, including idempotent ones, so `BrokenBy` remained state-space-unsound and the Stage 2a boundary was "improved rather than fully cleared." R2 goes to root cause: partition `EffectShape` into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` and narrow the `BrokenBy` payload to `BreakingOperation { shape: BreakingShape }`. Naming an idempotent op as the workflow breaker is now unrepresentable, not merely disallowed.

---

## Problem

`src/v3/std/effects.dag` ships `ComposedEffect` as:

```dag
type ComposedEffect {
  operations: List<OperationEffect>
  idempotent: Bool
  breaking_operation: String?
}
```

The record admits illegal state combinations:

| `idempotent` | `breaking_operation` | Meaning                                      |
|--------------|----------------------|----------------------------------------------|
| `true`       | `None`               | ✅ Workflow is idempotent.                    |
| `false`      | `Some(name)`         | ✅ Workflow broken by `name`.                 |
| `true`       | `Some(name)`         | ❌ Contradiction — idempotent but has breaker. |
| `false`      | `None`               | ❌ Non-idempotent but no breaker to blame.    |

`compose_effects` happens to produce only valid combinations today, but that is a *behavioral* invariant (an `iff` the constructor maintains) rather than a *state-space* invariant (an `iff` the type enforces). This is the same bug class `feedback_state_space_vs_behavioral_invariants` names: a `Bool` + correlated `Option<T>` pair should be a single sum whose variants are exactly the valid combinations.

The roadmap deferral at `src/v3/ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" already prescribes the direction:

> ... before any Track 17a consumer lands. ... at minimum drop redundant `idempotent`, or **better encode "no breaker" versus "broken by operation X" in the type shape itself**.

Stage 2b's idempotency lens (which consumes `compose_effects`) and Stage 2c's test-emission layer (which consumes `generate_idempotency_obligations`) are not yet written. Stage 2b's `WorkflowIdempotencyReport` (designed in `lane2-compile-time-proofs.md` §Stage 2b) wraps `ComposedEffect` and re-exposes the same boolean + optional breaker pair; locking the structural shape *before* the lens ships keeps the illegal states out of the lens report too.

No current consumer reads `.idempotent` or `.breaking_operation` — grep shows the type is only parsed (smoke test) or re-exported (v2 bootstrap). This is the window.

---

## Design

### Recommendation — Option B (sum-shaped verdict on a retained record wrapper)

Add a `CompositionVerdict` sum; replace the two summary fields with one verdict field.

```dag
// The effects-algebra verdict for a composed workflow.
//
// 🟢 TERMINAL. The composition of idempotent lattice meets is
// idempotent; a single non-idempotent operation poisons the chain.
// The judgment space is therefore exactly two states: the workflow
// converges, or some operation broke convergence. No third state.
type CompositionVerdict
  = IdempotentComposition
  | BrokenBy { first_breaker: OperationEffect }

type ComposedEffect {
  operations: List<OperationEffect>   // evidence chain (walk)
  verdict: CompositionVerdict         // projection of the walk
}
```

`compose_effects` updates to:

```dag
fn compose_effects(effects: List<OperationEffect>) -> ComposedEffect {
  let breakers = effects |> filter(op => operation_is_breaking(op: op))
  let verdict = match breakers |> first {
    Some(op) => BrokenBy { first_breaker: op }
    None => IdempotentComposition
  }
  ComposedEffect {
    operations: effects,
    verdict: verdict
  }
}
```

Properties:

- **State-space sound.** `(idempotent: true, breaking_operation: Some(_))` and `(idempotent: false, breaking_operation: None)` are no longer representable.
- **Evidence preserved.** `operations` stays on the record — Stage 2c's obligation generator walks it; a future diagnostic renderer needs the chain to explain *where* the break happened.
- **Verdict carries the OperationEffect, not just the name.** `first_breaker: OperationEffect` lets downstream diagnostics project the breaking op's shape, cause, and key source without a name-keyed lookup back into `operations`. (`breaking_operation: String?` today forces that lookup.)
- **Single authority.** `compose_effects` is the only constructor of `ComposedEffect`; the sum-shaped verdict replaces the correlated-field invariant with a structural one.

### Alternatives considered

**Option A — drop `idempotent`, keep `breaking_operation`.** Let the presence of `Some(name)` encode non-idempotency; `None` encodes idempotency. Cheapest reshape. Rejected because it keeps the `String?` back-reference (name-keyed lookup into `operations`) and because the meaning of "idempotent" hides inside `Option`'s shape — readers have to know the convention. The sum makes the convention structural.

**Option C — drop `ComposedEffect` entirely.** Return `(List<OperationEffect>, CompositionVerdict)` or `CompositionVerdict` alone. Rejected: positional pairs aren't named (poor readability), and the sum-only return loses the walk evidence Stage 2c needs. The record is not convenience — it pairs evidence with projection.

**Option D — store the verdict only; derive the walk on demand.** Consumers that want the chain re-walk the input. Rejected: the input is the same `List<OperationEffect>` the record would carry, so "derive on demand" is indistinguishable from "store it" at the boundary, except the caller has to remember to thread the list alongside the verdict. Record is lighter at the consumer site.

**Option E — pure sum with `operations` inside each variant.**

```dag
type ComposedEffect
  = IdempotentComposition { operations: List<OperationEffect> }
  | BrokenBy { operations: List<OperationEffect>, first_breaker: OperationEffect }
```

Rejected: the `operations` field is then duplicated across variants, inviting drift if the shape evolves (e.g., adding a second evidence kind). The record-plus-sum form keeps evidence shared.

### What stays

- `type OperationEffect { operation_name: String; shape: EffectShape }` — unchanged.
- `fn operation_is_breaking(op: OperationEffect) -> Bool` — unchanged.
- `fn generate_idempotency_obligations(ops: List<OperationEffect>) -> List<IdempotencyTestObligation>` — unchanged (consumes `List<OperationEffect>` directly, not `ComposedEffect`).
- `derive_op_effect`, `derive_effect_shape`, modifier-check machinery — unchanged.

### What changes

- `type ComposedEffect` (v3) — two summary fields → one `verdict: CompositionVerdict` field.
- `fn compose_effects` (v3) — builds the sum via match on first breaker.
- New `type CompositionVerdict` (v3).

### v2 scope — do not mirror

The Stage 2a `DerivedOpEffect` collapse (PR #521) only touched v3. v2's `dsl/std/effects.dag` still ships `DerivedOpEffect` and the old `ComposedEffect` shape, and `src/v2/stage0/src/std_effects.rs` generates from the v2 authority, so the two sides are already in structural drift inside Stage 2a. That precedent is the right one: v2 is bootstrap-only, not end-user-facing, and mirroring every v3 reshape back into v2 doubles the reshape work for zero downstream benefit. This brief recommends the same discipline — **v3-only reshape**. v2 `ComposedEffect` stays on the `Bool + String?` shape until v2 retires. If something on the v2 critical path ever reads `.idempotent` or `.breaking_operation` (nothing does today), the implementer can revisit; otherwise the v2 divergence is paid-off debt, not live risk.

One related asymmetry, flagged for later: v2's `OperationEffect` carries an `evidence: IdempotencyEvidence` field that v3 projects on demand via `derive_idempotency_evidence`. That is a separate cleanup (evidence-vs-shape duplication) and is **not** in scope for this brief.

---

## Consumer impact

Grep at brief time:

| Site                                                          | Reads `.idempotent` / `.breaking_operation`? | Impact |
|---------------------------------------------------------------|---------------------------------------------|--------|
| `src/v3/std/effects.dag` (authoring)                          | n/a — it IS the authoring site              | reshape |
| `src/v3/compiler/tests/lane2_stage_2a_effects_smoke.rs`       | No — parse-only smoke                       | update fixture strings; still name-checks `ComposedEffect` + new `CompositionVerdict` |
| `src/v2/stage0/src/std_effects.rs`                            | No — generated from v2 `dsl/std/effects.dag` | **unchanged** (v2 authority not touched by this reshape) |
| `src/v2/stage0/src/v2_compiler_effect_derivation.rs`          | No — re-export only from v2 bootstrap       | **unchanged** |
| `src/v2/effect_derivation.dag`                                | No — imports v2 `ComposedEffect` name only  | **unchanged** |
| Stage 2b lens (`src/v3/lenses/idempotency.dag`, not yet written) | Will read verdict                         | match on `CompositionVerdict` instead of branching on Bool |

No consumer currently depends on the `.idempotent` / `.breaking_operation` names. The reshape is a pre-consumer window: after Stage 2b lands, `match` consumers would have to be rewritten.

---

## Acceptance

Implementation PR must satisfy:

1. `src/v3/std/effects.dag` defines `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: OperationEffect }` and `type ComposedEffect { operations: List<OperationEffect>; verdict: CompositionVerdict }`.
2. `compose_effects` returns a `ComposedEffect` whose `verdict` is `IdempotentComposition` iff no operation satisfies `operation_is_breaking`, and `BrokenBy { first_breaker }` otherwise with `first_breaker` structurally equal to `effects |> filter(operation_is_breaking) |> first`.
3. v2 is explicitly **not** touched (see §v2 scope). `dsl/std/effects.dag` and `src/v2/stage0/src/std_effects.rs` stay on the old shape.
4. `lane2_stage_2a_effects_smoke.rs` asserts `CompositionVerdict`, `IdempotentComposition`, `BrokenBy` are present in the parsed DAG alongside the existing names.
5. `src/v3/ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" moves from **Deferral** to **Cleared** with a pointer to the merge commit; `lane2-compile-time-proofs.md` §Stage 2a boundary note updates likewise; the Stage 2b pre-start gate language drops `ComposedEffect` from the open-gate list.
6. Stage 2b's `WorkflowIdempotencyReport` design (in `lane2-compile-time-proofs.md` §Stage 2b) is not touched by this PR — but the lens implementer, when Stage 2b lands, must not reintroduce a parallel `Bool + String?` pair; the report's `breaking_op: String?` field is a separate question resolved when Stage 2b is built.

Non-goals:

- `OperationEffect.evidence` v2/v3 divergence (separate cleanup).
- Any Stage 2b lens logic.
- Diagnostic rendering of the breaker (Stage 2b scope).

---

## Open question (resolved at land)

Did this warrant a numbered DB? No. The reshape introduced one new carrier (`CompositionVerdict`) but the brief was small enough to serve as the full design doc on its own; promoting to `DB-N` would have added bureaucratic overhead without adding design clarity. The ROADMAP deferral now points at this brief as the design anchor and reads "Cleared (this PR)." Future reshapes of similar scope can follow the same pattern — brief + implementation in one PR, no separate DB promotion — unless the carrier footprint is larger or more consumers cut across lanes.

---

## Pointers

- Current shape: `src/v3/std/effects.dag` lines 294–298 (`type ComposedEffect`), 318–331 (`fn compose_effects`).
- v2 parallel authority (explicitly not mirrored by this reshape): `dsl/std/effects.dag` lines 131–135, 145–162.
- Roadmap deferral: `src/v3/ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary".
- Stage 2a boundary note: `docs/lane2-compile-time-proofs.md` §Stage 2a, lines 41–45, 90.
- Pattern authority: `feedback_state_space_vs_behavioral_invariants` — "dissolve the bools into a single enum whose variants are only the valid combinations."
