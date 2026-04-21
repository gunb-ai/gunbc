# SG-4b-1-fix — Declaration-lookup authority cleanup `(S)`

## Context

PR #609 (SG-4b-1) merged despite a REQUEST_CHANGES review. The infer-helper tranche landed, but it introduced a **parallel declaration-lookup authority** that duplicates the existing `Dag::declaration(id)` accessor and changes the failure contract:

```rust
// src/v3/compiler/src/infer_helpers_generated.rs (from #609)
pub enum DeclarationLookup {
    MissingDeclaration,
    FoundDeclaration { _0: Declaration },
}

pub fn find_declaration(p0: &[Declaration], p1: &DeclarationId) -> DeclarationLookup {
    match p0 {
        [] => DeclarationLookup::MissingDeclaration,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).id == (*(p1))) {
                DeclarationLookup::FoundDeclaration { _0: __list_head.clone() }
            } else {
                find_declaration(__list_tail, p1)
            }
        }
    }
}
```

**Two problems**:

1. **Parallel authority**: `Dag` already exposes `pub fn declaration(&self, id: DeclarationId) -> &Declaration` as the canonical single-authority accessor. `find_declaration` creates a second id-resolution path via O(n) list walk — violates `feedback_parallel_representation_debt` and the Single Authority invariant.

2. **Fail-open contract**: `Dag::declaration` panics on unknown id (it's a witness-bearing handle; invalid ids are constructor-layer violations per Track 9 discipline). The new `DeclarationLookup::MissingDeclaration` treats not-found as a normal inference outcome — silently masks substrate-integrity violations, violating C-8 (fail-closed).

## Read first

- `src/v3/compiler/src/infer_helpers_generated.rs` — current state after #609 merge
- `src/v3/lenses/infer_helpers.dag` — the authority source (contains the `find_declaration` / `DeclarationLookup` definitions to remove)
- `src/v3/compiler/src/dag.rs` — `pub fn declaration(&self, id: DeclarationId) -> &Declaration` is the canonical accessor
- My REQUEST_CHANGES review on #609 for the full rationale
- `INVARIANTS.md` §C-8 (fail-closed discipline) and §Track 9 (constructor-validation)

## Work

1. **Delete `DeclarationLookup` enum and `find_declaration` fn** from `infer_helpers.dag`.
2. **Replace consumer sites** in `infer_helpers.dag` that match on `DeclarationLookup` with direct `Dag::declaration(id)` consumption (or equivalent `.dag` accessor realization that compiles to the same Rust call).
3. **If any consumer genuinely needs to tolerate "id doesn't exist" as non-error**: STOP and surface. That's a deeper substrate-integrity concern — workaround with a parallel lookup path is not the fix.
4. **Regenerate** `infer_helpers_generated.rs` via the regen binary.
5. **Verify** existing inference tests pass identically (behavior should be unchanged modulo the lookup path).

## Acceptance

- `DeclarationLookup` enum removed from `infer_helpers.dag` and `infer_helpers_generated.rs`
- `find_declaration` fn removed
- All consumers call `Dag::declaration(id)` (or a typed accessor resolving to it)
- Inference test suite green
- `infer_helpers_generated.rs` freshness ratchet still passes
- No new parallel id-resolution paths introduced

## STOP-AND-ESCALATE

- If any consumer genuinely needs optional lookup semantics (id may or may not exist) — STOP. That's a substrate-integrity question. Likely means the id-carrying type is too weak; propose either strengthening the witness or adding a substrate-level "may be absent" carrier. Don't re-introduce a parallel lookup.
- If removing `find_declaration` breaks the regen_infer_helpers ratchet in a way that can't be trivially fixed — STOP. The ratchet's contract should hold regardless.
- If the fix requires touching outside `infer_helpers.dag` / `infer_helpers_generated.rs` (e.g., changing `Dag::declaration` signature) — STOP. Wrong scope.

## Non-goals

- Not extending infer tranche beyond SG-4b-1's original set
- Not touching `infer.rs` directly (only regen output)
- Not changing `Dag::declaration` semantics
- Not introducing new substrate carriers

## Size

S. Scope is narrow: remove an enum + fn + their callers in one `.dag` file, regenerate, verify tests. Expected ~50-100 LOC deleted from `infer_helpers.dag`, corresponding deletion in `infer_helpers_generated.rs`, call sites in the `.dag` consumer walk redirected.

## Dispatch note

Director reviews. Primary acceptance: the parallel authority is gone and inference tests are unchanged. If this reveals substrate-integrity questions, surface them as separate lanes rather than paper over.
