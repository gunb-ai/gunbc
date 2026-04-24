# SG-4b-1-fix — Declaration-lookup authority cleanup `(S)`

> **Historical receipt.** This lane is closed by SG-4b-1-fix. The live code now
> imports `declaration_by_id` from `std.substrate`, and
> `DeclarationLookup` / `find_declaration` are gone from
> `src/v3/lenses/variant_payload.dag`. This brief remains as the dispatch
> record; do not treat the Work section below as open.

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

1. **Parallel authority**: `Dag` (Rust) exposes `pub fn declaration(&self, id: DeclarationId) -> &Declaration` as the canonical single-authority accessor. `find_declaration` creates a second id-resolution path via O(n) list walk — violates `feedback_parallel_representation_debt` and the Single Authority invariant.

2. **Fail-open contract**: `Dag::declaration` treats unknown id as a constructor-layer violation (the id is a witness-bearing handle; invalid ids can't legitimately exist). The new `LookupMissing` variant treats not-found as a normal lens-consumer outcome — silently masks substrate-integrity violations, violating C-8 (fail-closed).

**Prerequisite gap** (flagged by reviewer at 2026-04-21T06:57): the reflected substrate (`src/v3/std/substrate.dag`) currently exposes accessors for `port`, `node`, `resolve_producer`, `lane2_workflow_at` — **but not for declarations by id**. `.dag` lens consumers have no `fn declaration_by_id(d: Dag, id: DeclarationId) -> Declaration?` to call; they must either walk `d.declarations` or use `find_declaration`. This is the real root cause: the parallel authority exists because the reflected substrate is incomplete. SG-4b-1-fix must close the gap as well as delete the workaround — see Work section below.

## Read first

- `src/v3/lenses/variant_payload.dag` — the authority source (contains `type DeclarationLookup` + `fn find_declaration` to remove, plus their consumers in the same file)
- `src/v3/std/substrate.dag` — **currently exposes `port`, `node`, `resolve_producer`, `lane2_workflow_at` but NOT a declaration-by-id accessor**. This is the root cause of the parallel authority and the prerequisite work for this lane.
- `src/v3/compiler/src/dag.rs` — `pub fn declaration(&self, id: DeclarationId) -> &Declaration` is the canonical Rust accessor (the Host realization the new reflected accessor will bind to)
- `src/v3/lenses/infer_helpers.dag` (sanity-check only — confirm it no longer has `DeclarationLookup`; the pattern migrated/was cleaned up here)
- My REQUEST_CHANGES review on #609 for the original rationale (same concern, different file)
- `INVARIANTS.md#c-8` (fail-closed discipline) and `INVARIANTS.md#p2-boundary-discipline` (constructor-validation / witness-carrying boundary discipline)

## Work — two-step lane

### Step 1: Substrate extension (the actual prerequisite)

Add a reflected declaration-by-id accessor to `src/v3/std/substrate.dag` so `.dag` lens consumers have a single-authority path:

```
// Pattern mirrors the existing port / node accessors
fn declaration_by_id(d: Dag, id: DeclarationId) -> Declaration? {
  host declaration_by_id
}
```

Bind the Rust realization (`host declaration_by_id`) to `Dag::declaration(id)` or the equivalent that returns `Option<Declaration>`. Decision for the worker: should the reflected accessor be `Declaration?` (optional — permissive, matches the existing `port` accessor shape) or panic-on-miss matching `Dag::declaration`'s current Rust behavior? Preferred: `Declaration?` for reflected-side symmetry with `port` / `node`; the C-8 fail-closed contract is enforced at the **caller** (consumers that treat `None` as a bug still fail closed via a diagnostic, not by converting it to a normal outcome like `LookupMissing` does).

Verify the accessor is wired: bootstrap loads the realization binding; a pilot lens consumer can successfully resolve a known declaration id.

### Step 2: variant_payload migration

1. **Delete `DeclarationLookup` enum and `find_declaration` fn** from `src/v3/lenses/variant_payload.dag`.
2. **Replace consumer sites** in the same file with calls to the new `declaration_by_id` accessor from Step 1. Use typed `match` on `Declaration?` — `Some(decl) => ...` / `None => ...`.
3. **For `None` handling**: treat as a constructor-layer violation (emit a typed diagnostic, not a normal lens outcome). The id being present means the declaration should exist — if lookup returns `None`, that's a substrate-integrity violation worth surfacing.
4. **Regenerate** the affected generated file (`variant_payload_generated.rs` or equivalent per the lens regen pattern).
5. **Verify** existing variant-payload consumers + tests pass identically (behavior should be unchanged modulo the lookup path).

## Acceptance

**Step 1 (substrate extension)**:
- `fn declaration_by_id(d: Dag, id: DeclarationId) -> Declaration?` exists in `src/v3/std/substrate.dag` with `host declaration_by_id` realization
- Rust-side host binding compiles + resolves known ids correctly
- Existing substrate accessor tests pass

**Step 2 (variant_payload migration)**:
- `DeclarationLookup` type removed from `src/v3/lenses/variant_payload.dag`
- `find_declaration` fn removed from the same file
- All same-file consumers call `declaration_by_id` with typed `match Declaration?`
- `None` branch emits a typed diagnostic (constructor-layer violation), not a normal lens outcome
- `variant_payload_generated.rs` freshness ratchet still passes
- Variant-payload + inference + emit test suites green

**End-state**:
- No new parallel id-resolution paths introduced
- Sanity: `git grep "DeclarationLookup\|find_declaration" src/v3/lenses/` returns zero hits after merge
- Reflected substrate now has id-keyed lookup parity with `port` / `node` accessors

## STOP-AND-ESCALATE

- If any consumer genuinely needs optional lookup semantics (id may or may not exist) — STOP. That's a substrate-integrity question. Likely means the id-carrying type is too weak; propose either strengthening the witness or adding a substrate-level "may be absent" carrier. Don't re-introduce a parallel lookup.
- If removing `find_declaration` breaks the variant_payload regen ratchet (or equivalent lens freshness check) in a way that can't be trivially fixed — STOP. The ratchet's contract should hold regardless.
- If the fix requires touching outside `src/v3/lenses/variant_payload.dag` / `variant_payload_generated.rs` (e.g., changing `Dag::declaration` signature, touching `infer_helpers.dag`, editing emitter code) — STOP. Wrong scope; this lane is variant_payload-only cleanup.

## Non-goals

- Not touching `infer_helpers.dag` or `infer_helpers_generated.rs` (those were cleaned up independently; confirm via sanity grep but do not modify)
- Not touching `lower.rs` / `parse.rs` / `infer.rs` directly
- Not changing `Dag::declaration` semantics
- Not introducing new substrate carriers
- Not extending the variant_payload lens beyond what's required to remove the parallel lookup

## Size

**S-M** (was S; upsized after substrate-extension prerequisite surfaced).

- Step 1 (substrate extension): ~10-20 LOC in `substrate.dag` + corresponding Rust host realization + bootstrap wiring + pilot test. Small but non-trivial (touches substrate + host binding).
- Step 2 (variant_payload migration): ~30-80 LOC deleted from `variant_payload.dag`, corresponding deletion in generated projection, ~5-10 consumer sites redirected to `declaration_by_id`.

Total expected delta: ~-20 to -60 LOC hand-authored `.dag`. Small lane, one PR.

## Dispatch note

Director reviews. Primary acceptance: the parallel authority is gone from `variant_payload.dag` and variant-payload + adjacent tests are unchanged. If this reveals substrate-integrity questions (e.g., a consumer genuinely wants optional lookup), surface them as separate lanes rather than paper over.
