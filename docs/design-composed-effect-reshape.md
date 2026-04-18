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

### R2 — partition `EffectShape`; narrow `BrokenBy` payload to `BreakingShape`

The final shape partitions `EffectShape` by idempotency class and narrows the `BrokenBy` carrier to the breaking subset:

```dag
// Partition: idempotent-subset.
type IdempotentShape
  = ReadEffect
  | UpsertEffect { key_source: KeySource }
  | DeleteEffect { key_source: KeySource }

// Partition: breaking-subset (non-idempotent by construction).
type BreakingShape
  = CreateEffect { cause: CreateCause }
  | AppendEffect

// EffectShape is the disjoint union of the two subsets.
type EffectShape
  = IsIdempotent(IdempotentShape)
  | IsBreaking(BreakingShape)

type OperationEffect {
  operation_name: String
  shape: EffectShape
}

// Breaking-subset projection of OperationEffect. Cannot carry an
// idempotent shape — that combination is unrepresentable, not merely
// disallowed by a constructor convention.
type BreakingOperation {
  operation_name: String
  shape: BreakingShape
}

type CompositionVerdict
  = IdempotentComposition
  | BrokenBy { first_breaker: BreakingOperation }

type ComposedEffect {
  operations: List<OperationEffect>   // evidence chain (walk)
  verdict: CompositionVerdict         // state-space-sound judgment
}
```

`compose_effects` now projects each op via `operation_to_breaker` and takes the first breaker:

```dag
fn operation_to_breaker(op: OperationEffect) -> BreakingOperation? {
  match op.shape {
    IsIdempotent => None
    IsBreaking(breaking) =>
      Some(BreakingOperation {
        operation_name: op.operation_name,
        shape: breaking
      })
  }
}

fn compose_effects(effects: List<OperationEffect>) -> ComposedEffect {
  let breaker_candidates = effects |> flat_map(op =>
    match operation_to_breaker(op: op) {
      Some(b) => [b]
      None => []
    }
  )
  let verdict = match breaker_candidates |> first {
    Some(b) => BrokenBy { first_breaker: b }
    None => IdempotentComposition
  }
  ComposedEffect { operations: effects, verdict: verdict }
}
```

Properties (R2):

- **State-space sound at the verdict boundary.** `BrokenBy` carries a `BreakingOperation` whose `shape: BreakingShape` cannot admit `ReadEffect` / `UpsertEffect` / `DeleteEffect`. Naming an idempotent op as the workflow breaker is not representable by the type.
- **State-space sound at the shape boundary.** `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` makes the idempotent/breaking partition structural. Consumers that need the classification (`is_idempotent_effect`, `operation_is_breaking`) read the outer variant directly rather than enumerating variants and deciding each case. Adding a new `EffectShape` member requires choosing which side of the partition it inhabits — the compiler enforces this via exhaustive match on the two-arm outer sum.
- **Evidence preserved.** `operations: List<OperationEffect>` stays on the record — Stage 2c's obligation generator walks it; diagnostic rendering can project the full chain.
- **Cleaner consumers.** `classify_idempotent_disagreement` now takes `shape: BreakingShape` directly — the previous "dead arms" for idempotent shapes (unreachable but required by exhaustive match on the old flat `EffectShape`) dissolve. `check_modifier_vs_derivation` matches the op's outer partition variant instead of calling `is_idempotent_effect` and then re-destructuring.
- **Single authority.** `compose_effects` remains the only constructor of `ComposedEffect`; the sum-shaped verdict plus the partitioned shape replace two correlated invariants with two structural ones.

### Why not R1 (flat `BrokenBy { first_breaker: OperationEffect }`)

R1 (the first-landed shape) kept `EffectShape` flat and let `BrokenBy.first_breaker` carry any `OperationEffect`. The payload type admitted `OperationEffect { shape: ReadEffect }` — an idempotent op in the breaker slot. `compose_effects` filtered those out by behavioral invariant (`filter(operation_is_breaking)`), but the type itself allowed the illegal combination. Per `feedback_state_space_vs_behavioral_invariants`: when the type admits a state the constructor cannot justify, the invariant is API-level, not structural. The reshape has to reach the type, not the call site. Reviewer feedback on commit `b42edc15` caught this; R2 is the root-cause fix.

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

- `type OperationEffect { operation_name: String; shape: EffectShape }` — signature unchanged (`shape` still reads `EffectShape`; just the internal structure of `EffectShape` partitioned).
- `fn generate_idempotency_obligations(ops: List<OperationEffect>) -> List<IdempotencyTestObligation>` — unchanged signature (still consumes `List<OperationEffect>`).
- `derive_op_effect`, `derive_effect_shape` — signatures unchanged; constructor call sites adjusted to the partitioned `EffectShape`.
- `type IdempotencyEvidence`, `type ModifierAgreement`, `type ModifierAxisCheck`, `type ModifierCheck` — unchanged.

### What changes

- `type EffectShape` (v3) — flat five-variant sum → `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`.
- New `type IdempotentShape`, `type BreakingShape`, `type BreakingOperation` (v3).
- `type ComposedEffect` (v3) — two summary fields → `operations` + `verdict: CompositionVerdict`.
- New `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }` (v3).
- `fn is_idempotent_effect` — collapses to a two-arm match on the outer partition variant.
- `fn derive_idempotency_evidence` — becomes a nested match (outer partition, then inner variant).
- `fn operation_is_breaking` — reads the partition directly off `op.shape` (no evidence round-trip).
- New `fn operation_to_breaker(op: OperationEffect) -> BreakingOperation?` (v3).
- `fn compose_effects` — projects via `flat_map(operation_to_breaker)`; builds the sum verdict from the first breaker.
- `fn classify_idempotent_disagreement` — argument type narrows from `EffectShape` to `BreakingShape` (dead-arm cleanup).
- `fn classify_readonly_agreement` — nested match; the `IsBreaking` arm collapses to one case.
- `fn check_modifier_vs_derivation` — matches `op.shape`'s outer partition variant directly for the idempotent check; readonly still delegates to `classify_readonly_agreement`.

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

Implementation PR must satisfy (R2):

1. `src/v3/std/effects.dag` defines:
   - `type IdempotentShape = ReadEffect | UpsertEffect { key_source: KeySource } | DeleteEffect { key_source: KeySource }`
   - `type BreakingShape = CreateEffect { cause: CreateCause } | AppendEffect`
   - `type EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
   - `type BreakingOperation { operation_name: String; shape: BreakingShape }`
   - `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }`
   - `type ComposedEffect { operations: List<OperationEffect>; verdict: CompositionVerdict }`
2. `compose_effects` returns a `ComposedEffect` whose `verdict` is `IdempotentComposition` iff `effects |> flat_map(operation_to_breaker) |> is_empty`, and `BrokenBy { first_breaker }` otherwise with `first_breaker` structurally equal to the first projected `BreakingOperation`.
3. v2 is explicitly **not** touched (see §v2 scope). `dsl/std/effects.dag` and `src/v2/stage0/src/std_effects.rs` stay on the old shape.
4. `lane2_stage_2a_effects_smoke.rs` asserts `EffectShape`, `IdempotentShape`, `BreakingShape`, `ComposedEffect`, `CompositionVerdict`, `OperationEffect`, `BreakingOperation` (and the existing `CreateCause`, `KeySource`, `IdempotencyEvidence`, `ModifierAgreement`, `ModifierAxisCheck`, `ModifierCheck`) are present in the parsed DAG.
5. `src/v3/ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" reads **Cleared (this PR)** with a description of the partition; `lane2-compile-time-proofs.md` §Stage 2a boundary note updates to the partition shape; the Stage 2b pre-start gate language names the partition clearly so the lens implementer cannot reintroduce a flat-payload design.
6. Stage 2b's `WorkflowIdempotencyReport` design (in `lane2-compile-time-proofs.md` §Stage 2b) is not touched by this PR — but the lens implementer, when Stage 2b lands, must match on `CompositionVerdict` and project through `BrokenBy.first_breaker.shape: BreakingShape` rather than reintroduce a parallel `Bool + String?` pair.

Non-goals:

- `OperationEffect.evidence` v2/v3 divergence (separate cleanup).
- Any Stage 2b lens logic.
- Diagnostic rendering of the breaker (Stage 2b scope).

---

## Open question (resolved at land)

Did this warrant a numbered DB? No, even after R2. The reshape introduces four new carriers (`IdempotentShape`, `BreakingShape`, `BreakingOperation`, `CompositionVerdict`) but they are all tight refinements of shapes already in the same file; no cross-lane footprint, no new substrate concept. The brief is the design doc. The ROADMAP deferral now points at this brief as the design anchor and reads "Cleared (this PR)." Future reshapes of similar scope can follow the same pattern — brief + implementation in one PR, no separate DB promotion — unless the carrier footprint is larger or more consumers cut across lanes. **Lesson from R1 → R2:** the first-round brief recommended a sum-shaped verdict but missed that `first_breaker: OperationEffect` still left the illegal-state boundary unsound. "Make illegal states unrepresentable" has to reach every carrier in the chain, not just the outermost one; a future brief reviewer should check every field of every new carrier against the feedback-state-space-vs-behavioral-invariants audit.

---

## Pointers

- Authoring site: `src/v3/std/effects.dag` — `type EffectShape`, `type IdempotentShape`, `type BreakingShape`, `type BreakingOperation`, `type CompositionVerdict`, `type ComposedEffect`, `fn compose_effects`, `fn operation_to_breaker`.
- v2 parallel authority (explicitly not mirrored by this reshape): `dsl/std/effects.dag` (`ComposedEffect` still on the `Bool + String?` shape; `EffectShape` still flat).
- Roadmap: `src/v3/ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary".
- Stage 2a boundary note: `docs/lane2-compile-time-proofs.md` §Stage 2a.
- Pattern authority: `feedback_state_space_vs_behavioral_invariants` — "dissolve the bools into a single enum whose variants are only the valid combinations." Extended in R2 to: *every field* of a verdict carrier must be state-space-sound; outer-only soundness is not enough.
