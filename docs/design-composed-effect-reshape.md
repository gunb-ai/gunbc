> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2 Stage 2a follow-up) | Unblocks: Lane 2 Stage 2b (workflow idempotency lens), Track 17a REST wiring

# Design Brief — `ComposedEffect` reshape

**Brief:** Brief A (Lane 2 Stage 2a / Track 17a boundary follow-up)
**Consumers:** Stage 2b workflow-idempotency lens; Stage 2c test-obligation materialization; Track 17a REST consumers (once wired).
**Status:** Implemented — with two revisions in response to PR review on commit `b42edc15`.
- **R1 (landed at `b42edc15`):** lifted the `Bool + String?` summary into `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: OperationEffect }` wrapped in `ComposedEffect { operations, verdict }`.
- **R2 (responding to codex review):** partitioned `EffectShape` into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` and narrowed `BrokenBy` to `BreakingOperation { shape: BreakingShape }`. Fixed the inner-variant hole (idempotent ops could no longer be named as the breaker).
- **R3 (responding to ChatGPT review):** dropped `ComposedEffect` entirely. `compose_effects` now returns `CompositionVerdict` directly. The R2 record still correlated two fields — the `operations` walk and the `verdict` — that the constructor could keep coherent by convention but the type could not. `.dag` records are directly constructible, so any caller could build an incoherent `ComposedEffect`. R3 fixes the outer-record hole by deleting the record.

After R3 the verdict carrier is structurally sound all the way down: `CompositionVerdict` is a sum whose each variant is internally coherent, and there is no correlated sibling field for it to disagree with. The evidence chain (`List<OperationEffect>`) stays at its natural site — the caller's input — not duplicated onto the output.

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

The roadmap deferral at `ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" already prescribes the direction:

> ... before any Track 17a consumer lands. ... at minimum drop redundant `idempotent`, or **better encode "no breaker" versus "broken by operation X" in the type shape itself**.

Stage 2b's idempotency lens (which consumes `compose_effects`) and Stage 2c's test-emission layer (which consumes `generate_idempotency_obligations`) are not yet written. Stage 2b's `WorkflowIdempotencyReport` (designed in `lane2-compile-time-proofs.md` §Stage 2b) wraps `ComposedEffect` and re-exposes the same boolean + optional breaker pair; locking the structural shape *before* the lens ships keeps the illegal states out of the lens report too.

No current consumer reads `.idempotent` or `.breaking_operation` — grep shows the type is only parsed (smoke test) or re-exported (v2 bootstrap). This is the window.

---

## Design

### R3 — drop the outer record; partition `EffectShape`; narrow the breaker carrier

The final shape has three structural pieces — the partitioned `EffectShape`, the narrowed `BreakingOperation`, and the sum-shaped `CompositionVerdict` — and no outer record around them:

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

// The effects-algebra verdict for a composed workflow. Returned
// directly by `compose_effects` — no enclosing record.
type CompositionVerdict
  = IdempotentComposition
  | BrokenBy { first_breaker: BreakingOperation }
```

`compose_effects` projects each op via `operation_to_breaker` and returns the verdict directly:

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

fn compose_effects(effects: List<OperationEffect>) -> CompositionVerdict {
  let breaker_candidates = effects |> flat_map(op =>
    match operation_to_breaker(op: op) {
      Some(b) => [b]
      None => []
    }
  )
  match breaker_candidates |> first {
    Some(b) => BrokenBy { first_breaker: b }
    None => IdempotentComposition
  }
}
```

Properties (R3):

- **State-space sound at the verdict.** `CompositionVerdict` is a sum whose each variant is internally coherent. `IdempotentComposition` carries no payload; `BrokenBy` carries exactly one `BreakingOperation`, and `BreakingOperation.shape: BreakingShape` cannot admit idempotent variants. Nothing beside the verdict can be made to disagree with it because nothing *is* beside it.
- **State-space sound at the shape boundary.** `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` makes the idempotent/breaking partition structural. Consumers that need the classification (`is_idempotent_effect`, `operation_is_breaking`) read the outer variant directly rather than enumerating variants and deciding each case. Adding a new `EffectShape` member requires choosing which side of the partition it inhabits — the compiler enforces this via exhaustive match on the two-arm outer sum.
- **Evidence at its natural site.** `List<OperationEffect>` is the caller's own input to `compose_effects`. Callers that want both the walk and the verdict keep the input in scope and read the return value — two facts, two sites, zero correlation to enforce. Stage 2c's obligation generator already consumes `List<OperationEffect>` directly (not `ComposedEffect`), so nothing regresses.
- **Single-authority by type shape, not by convention.** There is no carrier for a caller to incoherently construct. `BrokenBy { first_breaker: some_BreakingOperation }` is locally coherent: the variant carries what the variant carries. `.dag` lets anyone construct it, but the constructed value's state-space is the variant itself — there is no sibling field to correlate with.
- **Cleaner consumers.** `classify_idempotent_disagreement` takes `shape: BreakingShape` directly — the previous "dead arms" for idempotent shapes (unreachable but required by exhaustive match on the old flat `EffectShape`) dissolve. `check_modifier_vs_derivation` matches the op's outer partition variant instead of calling `is_idempotent_effect` and then re-destructuring.

### Known trade-off — `BreakingOperation` is a copy, not a carrier-relative witness

`BrokenBy.first_breaker: BreakingOperation` is a structural copy of an element of `compose_effects`'s input `List<OperationEffect>`, not a carrier-relative witness pointing at that element. If a caller holds both the input list and the verdict, the type system does not enforce `first_breaker ∈ effects`: anyone can construct a standalone `BreakingOperation { operation_name: "fabricated", shape: AppendEffect }` and wrap it in `BrokenBy`. The discipline "go through `compose_effects`" is convention, not shape, at the `BreakingOperation` level.

**Why not a witness here.** The project's `feedback_state_space_vs_behavioral_invariants` discipline would ordinarily push for a handle — the right shape for "this field must refer to an element of that carrier" is an `ElementRef<T>` handle whose constructor takes the list and a slot, analogous to `ParamRef` / `TransformRef` in Track 9 (`ROADMAP.md:763-772`). `ElementRef<T>` is **deliberately not yet declared** per the same Track 9 entry: "the handle lands when a concrete consumer needs it, not speculatively." No consumer of `CompositionVerdict` exists yet (Stage 2b's lens is not written, Track 17a REST wiring is not written), so there is no concrete shape to anchor an `ElementRef<OperationEffect>` graduation against. Landing one now would be speculative.

**What this means for the boundary.** The Stage 2a / Track 17a boundary is cleared at the wrapper / record level — the R1 `Bool + String?` correlation, the R1 breaker-carrier inner-variant hole, and the R2 outer-record-pairs-two-fields hole are all dissolved. The deeper single-authority concern — whether `first_breaker` should be a handle rather than a copy — is a separate design axis that belongs with the first real consumer (most likely the Stage 2b lens). When Stage 2b lands, if the report needs to render "position of the breaker in the workflow" or tie the verdict back to its evidence chain structurally, that is the moment to promote `BreakingOperation` to `ElementRef<OperationEffect>` (and ship the `ElementRef<T>` graduation alongside). This brief explicitly does not pre-declare the handle.

**Tracked follow-up.** `ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" names this trade-off and points at Track 9's deferred `ElementRef<T>` graduation as the place where a witness-based breaker would land if and when a Stage 2b / Track 17a consumer justifies it. The boundary is cleared for the specific shape contract Stage 2a owns; the "is the breaker a handle or a copy" design axis is an independent, consumer-driven decision deferred to the Stage 2b / Track 17a implementation PRs.

### Revision trail

- **R1 (landed at `b42edc15`).** `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: OperationEffect }` wrapped in `type ComposedEffect { operations, verdict }`. codex review flagged: `BrokenBy.first_breaker: OperationEffect` admits idempotent shapes.
- **R2.** Partitioned `EffectShape` into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`; narrowed `BrokenBy.first_breaker: BreakingOperation { shape: BreakingShape }`. ChatGPT review flagged: the `{ operations, verdict }` record still admits incoherent pairs — `operations: [breaker]` + `verdict: IdempotentComposition`, or `BrokenBy { first_breaker }` where the breaker isn't the first breaker in `operations` or isn't in `operations` at all. Records are directly constructible in `.dag`; the constructor's coherence is behavioral, not structural.
- **R3 (this land).** Dropped `ComposedEffect`. `compose_effects` returns `CompositionVerdict` directly. The record-level correlation problem dissolves because there is no record.

Each revision goes one layer deeper into the same critique: *make illegal states unrepresentable at every structural level*, not just the outer one or the inner one. R3 hits all levels at once.

### Alternatives considered (revisited at R3)

**Option A — drop `idempotent`, keep `breaking_operation`.** Let the presence of `Some(name)` encode non-idempotency; `None` encodes idempotency. Cheapest reshape. Rejected because it keeps the `String?` back-reference (name-keyed lookup into `operations`) and because the meaning of "idempotent" hides inside `Option`'s shape — readers have to know the convention. The sum makes the convention structural.

**Option C — drop `ComposedEffect` entirely.** Return `CompositionVerdict` alone. **This is R3.** The R1/R2 briefs rejected this on the grounds that "Stage 2c needs the walk," but that reasoning was wrong: Stage 2c's `generate_idempotency_obligations` consumes `List<OperationEffect>` directly, not `ComposedEffect`. No consumer actually needed the pairing to be materialized at the verdict boundary. The record was a convenience that admitted incoherence; removing it removes the correlation risk.

**Option D — store the verdict only; derive the walk on demand.** Consumers that want the chain re-walk the input. Essentially R3 for the single-consumer case: the input is the caller's own list, so "derive on demand" is just "keep using the variable you already have."

**Option E — pure sum with `operations` inside each variant.**

```dag
type ComposedEffect
  = IdempotentComposition { operations: List<OperationEffect> }
  | BrokenBy { operations: List<OperationEffect>, first_breaker: OperationEffect }
```

Rejected: duplicates the evidence across variants; still admits a version of the R2 critique — `BrokenBy.operations` could disagree with `first_breaker`. R3 (no wrapper) dominates this option.

**Option F — structural tie between `first_breaker` and `operations`.** Encode `first_breaker` as an index into `operations` plus a refinement that the index is in-bounds and points at a `BreakingShape`, or split the record into `preceding: List<IdempotentOperation>`, `breaker: BreakingOperation`, `following: List<OperationEffect>`. Rejected: substantially more machinery; no consumer today needs the structural tie; R3 gets the same soundness with less surface by not pairing the two in a carrier at all.

### What stays

- `type OperationEffect { operation_name: String; shape: EffectShape }` — signature unchanged (`shape` still reads `EffectShape`; just the internal structure of `EffectShape` partitioned).
- `fn generate_idempotency_obligations(ops: List<OperationEffect>) -> List<IdempotencyTestObligation>` — unchanged signature (still consumes `List<OperationEffect>`).
- `derive_op_effect`, `derive_effect_shape` — signatures unchanged; constructor call sites adjusted to the partitioned `EffectShape`.
- `type IdempotencyEvidence`, `type ModifierAgreement`, `type ModifierAxisCheck`, `type ModifierCheck` — unchanged.

### What changes (R3-final)

- `type EffectShape` (v3) — flat five-variant sum → `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`.
- New `type IdempotentShape`, `type BreakingShape`, `type BreakingOperation` (v3).
- New `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }` (v3).
- **`type ComposedEffect` is removed.** The old `{ operations, idempotent, breaking_operation }` is gone; the intermediate `{ operations, verdict }` is also gone.
- `fn compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` — returns the verdict directly; no enclosing record.
- `fn is_idempotent_effect` — collapses to a two-arm match on the outer partition variant.
- `fn derive_idempotency_evidence` — becomes a nested match (outer partition, then inner variant).
- `fn operation_is_breaking` — reads the partition directly off `op.shape` (no evidence round-trip).
- New `fn operation_to_breaker(op: OperationEffect) -> BreakingOperation?` (v3).
- `fn classify_idempotent_disagreement` — argument type narrows from `EffectShape` to `BreakingShape` (dead-arm cleanup).
- `fn classify_readonly_agreement` — nested match; the `IsBreaking` arm collapses to one case.
- `fn check_modifier_vs_derivation` — matches `op.shape`'s outer partition variant directly for the idempotent check; readonly still delegates to `classify_readonly_agreement`.

### v2 scope — do not mirror

The Stage 2a `DerivedOpEffect` collapse (PR #521) only touched v3. v2's `dsl/std/effects.dag` still ships `DerivedOpEffect` and the old `ComposedEffect` shape, and `src/v2/stage0/src/std_effects.rs` generates from the v2 authority, so the two sides are already in structural drift inside Stage 2a. That precedent is the right one: v2 is bootstrap-only, not end-user-facing, and mirroring every v3 reshape back into v2 doubles the reshape work for zero downstream benefit. This brief recommends the same discipline — **v3-only reshape**. v2 `ComposedEffect` stays on the `Bool + String?` shape until v2 retires. If something on the v2 critical path ever reads `.idempotent` or `.breaking_operation` (nothing does today), the implementer can revisit; otherwise the v2 divergence is paid-off debt, not live risk.

One related asymmetry, flagged for later: v2's `OperationEffect` carries an `evidence: IdempotencyEvidence` field that v3 projects on demand via `derive_idempotency_evidence`. That is a separate cleanup (evidence-vs-shape duplication) and is **not** in scope for this brief.

---

## Consumer impact

| Site                                                          | Reads `ComposedEffect` fields? | Impact |
|---------------------------------------------------------------|--------------------------------|--------|
| `src/v3/std/effects.dag` (authoring)                          | n/a — it IS the authoring site | reshape |
| `src/v3/compiler/tests/lane2_stage_2a_effects_smoke.rs`       | No — parse-only smoke          | name-check list updated to R3 types; `ComposedEffect` removed |
| `src/v2/stage0/src/std_effects.rs`                            | No — generated from v2 `dsl/std/effects.dag` | **unchanged** (v2 authority not touched by this reshape) |
| `src/v2/stage0/src/v2_compiler_effect_derivation.rs`          | No — re-export only from v2 bootstrap       | **unchanged** |
| `src/v2/effect_derivation.dag`                                | No — imports v2 `ComposedEffect` name only  | **unchanged** |
| Stage 2b lens (`src/v3/lenses/idempotency.dag`, not yet written) | Will read verdict           | consume `CompositionVerdict` directly from `compose_effects`; pair with caller-held `List<OperationEffect>` if the lens report needs the walk |

No v3 consumer currently depends on any `ComposedEffect` field — the type was new in R1/R2 and is being removed before its first consumer lands. The reshape is a pre-consumer window.

---

## Acceptance

Implementation PR must satisfy (R3-final):

1. `src/v3/std/effects.dag` defines:
   - `type IdempotentShape = ReadEffect | UpsertEffect { key_source: KeySource } | DeleteEffect { key_source: KeySource }`
   - `type BreakingShape = CreateEffect { cause: CreateCause } | AppendEffect`
   - `type EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
   - `type BreakingOperation { operation_name: String; shape: BreakingShape }`
   - `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }`
   - No `type ComposedEffect`. The outer record is gone; `compose_effects` returns `CompositionVerdict` directly.
2. `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` returns `IdempotentComposition` iff `effects |> flat_map(operation_to_breaker) |> is_empty`, and `BrokenBy { first_breaker }` otherwise with `first_breaker` structurally equal to the first projected `BreakingOperation`.
3. v2 is explicitly **not** touched (see §v2 scope). `dsl/std/effects.dag` and `src/v2/stage0/src/std_effects.rs` stay on the old shape.
4. `lane2_stage_2a_effects_smoke.rs` asserts the following top-level types are present in the parsed DAG: `EffectShape`, `IdempotentShape`, `BreakingShape`, `CreateCause`, `KeySource`, `IdempotencyEvidence`, `CompositionVerdict`, `OperationEffect`, `BreakingOperation`, `ModifierAgreement`, `ModifierAxisCheck`, `ModifierCheck`. (Variants of sum types are not asserted separately — the v3 bootstrap does not expose variants as standalone declarations.)
5. `ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary" reads **Cleared (this PR)** with a description that names the partition *and* the removal of the outer record; `lane2-compile-time-proofs.md` §Stage 2a boundary note updates to the R3 shape; the Stage 2b pre-start gate language names `CompositionVerdict` (not `ComposedEffect`) as the algebra's output and tells the lens implementer to pair it with the caller-held `List<OperationEffect>` rather than recreate the pairing at the lens boundary.
6. Stage 2b's `WorkflowIdempotencyReport` design (in `lane2-compile-time-proofs.md` §Stage 2b) is not touched by this PR — but the lens implementer, when Stage 2b lands, must match on `CompositionVerdict` and project through `BrokenBy.first_breaker.shape: BreakingShape` rather than reintroduce a parallel `Bool + String?` pair; the draft `breaking_op: String?` field in the §Stage 2b report is a placeholder from before this reshape landed and should be replaced with a structural carrier derived from `CompositionVerdict`.

Non-goals:

- `OperationEffect.evidence` v2/v3 divergence (separate cleanup).
- Any Stage 2b lens logic.
- Diagnostic rendering of the breaker (Stage 2b scope).

---

## Open question (resolved at land)

Did this warrant a numbered DB? No, even after R3. The final reshape introduces four new carriers (`IdempotentShape`, `BreakingShape`, `BreakingOperation`, `CompositionVerdict`) but they are all tight refinements of shapes already in the same file; no cross-lane footprint, no new substrate concept. The brief is the design doc.

**Lessons from the R1 → R2 → R3 arc.** Each revision went one layer deeper into the same critique:
- **R1 → R2 (codex review):** the outer summary was a sum, but the sum's *payload* still admitted incoherent inner shapes. State-space soundness has to reach the nested carriers, not stop at the outer variant.
- **R2 → R3 (ChatGPT review):** the payload was sound, but the *record wrapper* paired it with a sibling field (the walk) that the type system couldn't correlate. State-space soundness across sibling fields of a record is convention unless the record's own construction is restricted — and `.dag` records don't restrict construction.

The brief-review discipline that falls out: when adding a new carrier, audit *every field* and *every sibling-field pair* against `feedback_state_space_vs_behavioral_invariants`. "This sum is clean" is not enough; the sum's payload, and anything paired with the sum in a record, needs the same scrutiny. If the pairing requires a constructor discipline to stay coherent, the pairing is API-level and should be removed or replaced with a structural tie (index-plus-refinement, split variants, or just "return one, let the caller keep the other in scope").

---

## Pointers

- Authoring site: `src/v3/std/effects.dag` — `type EffectShape`, `type IdempotentShape`, `type BreakingShape`, `type OperationEffect`, `type BreakingOperation`, `type CompositionVerdict`, `fn compose_effects`, `fn operation_to_breaker`, `fn operation_is_breaking`.
- v2 parallel authority (explicitly not mirrored by this reshape): `dsl/std/effects.dag` (`ComposedEffect` still on the `Bool + String?` shape; `EffectShape` still flat).
- Roadmap: `ROADMAP.md` §"Lane 2 Stage 2a / Track 17a boundary".
- Stage 2a boundary note: `docs/lane2-compile-time-proofs.md` §Stage 2a.
- Pattern authority: `feedback_state_space_vs_behavioral_invariants` — "dissolve the bools into a single enum whose variants are only the valid combinations." Extended by the R1→R3 arc: soundness must reach *every* structural level — the outer variant, every field of every carrier, and every sibling-field pair within any record.
