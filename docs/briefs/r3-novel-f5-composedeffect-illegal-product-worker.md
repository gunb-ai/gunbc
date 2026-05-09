# R3 Novel-Finding Worker Brief — F5 `ComposedEffect { idempotent, breaking_operation }` illegal product

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope.
**Authority parent**: gpt-5-5-pro reflective analysis Finding 5; PM dispatch at gunbc#846 c#4413701937.
**Priority**: HIGH — Class C illegal-state-representable; soundness adjacent.

---

## §0. Problem statement

`dsl/std/effects.dag:135` declares:
```
type ComposedEffect { ... }
```

with fields including `idempotent: Bool` and `breaking_operation: ...`. These two fields can both be set in inconsistent ways — e.g., a "breaking operation" claimed `idempotent: true` is contradictory; the type permits the illegal state.

P1 Modeling Faithfulness: typed carriers should not admit semantically-illegal field combinations. Class C illegal-product instance.

## §1. Required outcome

`ComposedEffect` shape redesigned so contradictory field combinations are unrepresentable.

## §2. Fix options

**Option A (sum split)**: Replace product with sum:
```
type ComposedEffect =
  | IdempotentEffect { ... }
  | BreakingEffect { breaking_operation: OperationEffect, ... }
  | NeutralEffect { ... }
```

The sum forecloses contradictory field combinations.

**Option B (refinement)**: Keep product; add `where` refinement constraining valid (idempotent, breaking_operation) pairs. Requires DB-11 alias-RHS `where` parsing already landed (#703).

PM-recommended: Option A — eliminates the illegal state shape entirely (per `feedback_load_bearing_ratchet_preservation` / structural-illegal preference). Option B retains the product shape with downstream refinement load.

## §3. Files

**Option A**:
- `dsl/std/effects.dag:135` (replace product with sum)
- `src/v2/effect_derivation.dag` consumers
- `src/v2/stage0/src/v2_compiler_effect_derivation.rs` (consumer; v2-Class-E)
- consumers across .dag effect_derivation paths
- new `.dag` `TestClaim` cementing legal-state-only construction

## §4. Cross-cutting constraints

- v2-side consumer (`src/v2/...`) migration; coordinate with PB Mgr on v2-retirement timeline interaction.
- Cross-references Class C row 5 in sweep doc.
- STOP-and-PING via Mgr inbox if `OperationEffect` shape needs co-evolution.

## §5. Receipt

- `ComposedEffect` redesigned (Option A or B); illegal field combinations no longer representable.
- Consumers updated.
- Cementing `TestClaim` for legal-construction-only.
- Sweep-doc Class C row 5 updated.

---

**End of brief.**
