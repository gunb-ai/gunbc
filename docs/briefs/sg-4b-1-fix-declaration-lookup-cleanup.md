# SG-4b-1-fix — Declaration-lookup authority cleanup `(S)`

## Context

PR #609 (SG-4b-1) surfaced a **parallel declaration-lookup authority** pattern. Per a later cleanup on main, `infer_helpers.dag` / `infer_helpers_generated.rs` no longer carry it (only `TemplateArgumentLookup` remains there). **But the pattern persists on main in `src/v3/lenses/variant_payload.dag`**:

```
type DeclarationLookup
  = LookupMissing
  | LookupFound(Declaration)

fn find_declaration(decls: List<Declaration>, target: DeclarationId) -> DeclarationLookup =
  match decls {
    Empty => LookupMissing
    Cons(payload) =>
      if payload.head.id == target then LookupFound(payload.head)
      else find_declaration(payload.tail, target)
  }
```

`find_declaration` is called from within the same file to resolve variant declarations by id.

**Two problems** (same as the original #609 concern, just in a different file):

1. **Parallel authority**: `Dag` already exposes `pub fn declaration(&self, id: DeclarationId) -> &Declaration` as the canonical single-authority accessor (or the `.dag`-side equivalent). `find_declaration` creates a second id-resolution path via O(n) list walk — violates `feedback_parallel_representation_debt` and the Single Authority invariant.

2. **Fail-open contract**: `Dag::declaration` treats unknown id as a constructor-layer violation (the id is a witness-bearing handle; invalid ids can't legitimately exist). The new `LookupMissing` variant treats not-found as a normal lens-consumer outcome — silently masks substrate-integrity violations, violating C-8 (fail-closed).

## Read first

- `src/v3/lenses/variant_payload.dag` — the authority source (contains `type DeclarationLookup` + `fn find_declaration` to remove, plus their consumers in the same file)
- `src/v3/compiler/src/dag.rs` — `pub fn declaration(&self, id: DeclarationId) -> &Declaration` is the canonical accessor (or its `.dag` equivalent that the lens consumer can reach)
- `src/v3/lenses/infer_helpers.dag` (sanity-check only — confirm it no longer has `DeclarationLookup`; the pattern migrated/was cleaned up here)
- My REQUEST_CHANGES review on #609 for the original rationale (same concern, different file)
- `INVARIANTS.md` §C-8 (fail-closed discipline) and §Track 9 (constructor-validation)

## Work

1. **Delete `DeclarationLookup` enum and `find_declaration` fn** from `src/v3/lenses/variant_payload.dag`.
2. **Replace consumer sites** in the same file (grep for `find_declaration(` within `variant_payload.dag`) with direct consumption of the canonical declaration accessor — whatever `.dag`-side surface resolves to `Dag::declaration(id)` without going through a parallel enum.
3. **If any consumer genuinely needs to tolerate "id doesn't exist" as non-error**: STOP and surface. That's a deeper substrate-integrity concern — workaround with a parallel lookup path is not the fix. The invariant here is: a `DeclarationId` is a witness — if it's present, the declaration exists; absence is a constructor-layer violation, not a lens outcome.
4. **Regenerate** the affected generated file (`variant_payload_generated.rs` or equivalent per the lens regen pattern).
5. **Verify** existing variant-payload consumers + tests pass identically (behavior should be unchanged modulo the lookup path).

## Acceptance

- `DeclarationLookup` type removed from `src/v3/lenses/variant_payload.dag`
- `find_declaration` fn removed from the same file
- All same-file consumers call the canonical declaration accessor (or its typed realization)
- Variant-payload + inference + emit test suites green
- `variant_payload_generated.rs` (or equivalent) freshness ratchet still passes
- No new parallel id-resolution paths introduced
- Sanity: `git grep "DeclarationLookup\|find_declaration" src/v3/lenses/` returns zero hits after merge

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
