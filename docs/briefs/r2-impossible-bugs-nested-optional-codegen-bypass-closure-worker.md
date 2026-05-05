# T-ImpossibleBugs — Nested-optional codegen-bypass closure worker brief `(S; git-metadata-unavailable receipt)`

> **Worker brief.** Reports through Impossible-Bugs Manager. This is a
> narrowed follow-up to
> [`r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md).
> It does **not** reopen the allocator / lowering route, which is already
> live. Scope is only the remaining bypassable construction surface for
> `CardinalityPayload::new_unchecked` and generated cardinality
> construction sites.
>
> **Git metadata note:** this brief was authored from a checkout whose
> `.git` pointer is broken. Treat this authoring receipt as
> `git-metadata-unavailable`: file/`rg` evidence only, no branch
> cleanliness or diff-base claim.

## Read first

- [`docs/briefs/r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md) — canonical class brief. Consume its design authority; do not re-scope the class.
- [`docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md`](t-impossiblebugs-nested-optional-flatten-design.md) — design authority for the substrate-constructor invariant.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) — `cardinality_idempotent_target` is already present.
- [`src/v3/compiler/src/dag/builder.rs`](../../src/v3/compiler/src/dag/builder.rs) — `Dag::alloc_cardinality_decl` is already present and is the intended constructor authority.
- [`src/v3/compiler/src/dag/cardinality_payload.rs`](../../src/v3/compiler/src/dag/cardinality_payload.rs) — live bypass aperture: `CardinalityPayload::new_unchecked`.
- [`src/v3/compiler/src/regen_bootstrap_emit.rs`](../../src/v3/compiler/src/regen_bootstrap_emit.rs) — live codegen bypass: cardinality connectives are emitted as `TypeConnective::Cardinality(CardinalityPayload::new_unchecked(...))`.
- Generated surfaces to audit before editing:
  - [`src/v3/compiler/src/bootstrap_std_generated.rs`](../../src/v3/compiler/src/bootstrap_std_generated.rs)
  - [`src/v3/compiler/src/bootstrap_generated.rs`](../../src/v3/compiler/src/bootstrap_generated.rs)
  - [`src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs`](../../src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs)

## Pre-author verification receipt

Manager dispatch verification found the main invariant already live:

- `cardinality_idempotent_target` exists in `src/v3/compiler/src/dag.rs`.
- `Dag::alloc_cardinality_decl` exists in `src/v3/compiler/src/dag/builder.rs`.
- `SurfaceType::Optional` lowering routes through `alloc_cardinality_decl` in `src/v3/compiler/src/lower.rs`.
- `concretize_decl_with_subst` routes through `alloc_cardinality_decl` in `src/v3/compiler/src/infer.rs`.

Remaining bypass evidence:

- `src/v3/compiler/src/dag/cardinality_payload.rs` exposes `CardinalityPayload::new_unchecked` broadly enough for generated / dag-internal construction.
- `src/v3/compiler/src/regen_bootstrap_emit.rs` emits `TypeConnective::Cardinality(CardinalityPayload::new_unchecked(...))`.
- Generated construction sites exist in `bootstrap_std_generated.rs`, `bootstrap_generated.rs`, and `bootstrap_generated_without_parse_surface.rs`, exceeding the original std-only generated-site audit.

## Frame

The class is structurally closed only when nested `AtMostOne` cardinality
cannot be constructed through any production path. The allocator route
already enforces idempotence. This worker closes the remaining bypasses:
generated constructors and any dag-internal helper paths that can still
materialize `TypeConnective::Cardinality(CardinalityPayload::new_unchecked(...))`
without passing through `alloc_cardinality_decl`.

This is not a substrate escalation. It is API/codegen closure around an
already-landed substrate-constructor invariant.

## Slice

1. **Re-audit construction sites** with `rg` before editing:
   - `CardinalityPayload::new_unchecked`
   - `TypeConnective::Cardinality`
   - `alloc_cardinality_decl`
   - `cardinality_idempotent_target`
2. **Narrow `new_unchecked` authority** so ordinary code cannot bypass
   `Dag::alloc_cardinality_decl`. Keep a tightly justified escape hatch
   only where unavoidable for generated bootstrap materialization, and
   document why it cannot construct a nested `AtMostOne` without an
   adjacent normalization pass.
3. **Update regen emission** in `regen_bootstrap_emit.rs` so generated
   bootstrap cardinality construction either:
   - routes through `alloc_cardinality_decl`, or
   - emits a generated-local normalizing constructor that delegates to, or
     is mechanically generated from, the canonical
     `cardinality_idempotent_target` / `alloc_cardinality_decl` authority.
     Do not hand-maintain a second idempotence rule; if a generated-local
     helper is unavoidable, the PR must name its dissolution path back to
     the canonical constructor authority.
4. **Regenerate or mechanically update generated files** consistently for
   all affected generated surfaces, not just `bootstrap_std_generated.rs`.
   At minimum audit and account for:
   - `bootstrap_std_generated.rs`
   - `bootstrap_generated.rs`
   - `bootstrap_generated_without_parse_surface.rs`
5. **Add/extend a focused regression test** that attempts the generated
   path shape and proves nested `AtMostOne` normalizes to the inner
   declaration. Existing allocator/lowering tests are not enough for this
   follow-up.
6. **Run focused checks**:
   - `cargo fmt --all --check`
   - existing `cardinality_idempotent` / `cardinality_idempotence` tests
   - the new generated-path regression

## Acceptance

- [ ] No production construction path can create nested
  `AtMostOne(AtMostOne(T))` without applying the same idempotence rule as
  `alloc_cardinality_decl`.
- [ ] `CardinalityPayload::new_unchecked` is visibility-/callability-
  narrowed so ordinary code cannot bypass `Dag::alloc_cardinality_decl`.
  Any remaining generated/bootstrap-only escape hatch is intentionally
  narrow and paired with normalization evidence that is delegated to,
  mechanically generated from, or explicitly dissolves back into the
  canonical constructor authority.
- [ ] `regen_bootstrap_emit.rs` no longer emits raw unchecked cardinality
  construction as the final authority for generated cardinality nodes.
- [ ] All affected generated files are regenerated or mechanically kept
  consistent, with the PR body naming which path was used.
- [ ] Regression coverage exercises the generated/bootstrap construction
  path, not only hand-written lowering and substitution.
- [ ] Checks above pass, or the PR reports exact tool unavailability.

## STOP-AND-ESCALATE

- **Generated bootstrap materialization cannot call `alloc_cardinality_decl`**
  because the generated DAG is not available at construction time, and no
  generated-local normalizer can preserve declaration identity. Stop and
  report the specific generated function / construction order blocker.
- **Narrowing `new_unchecked` breaks read-side pattern matching or
  non-cardinality generated constructors.** Stop and report exact compiler
  errors; do not broaden this into a general TypeConnective refactor.
- **Regeneration drifts unrelated bootstrap spans or DB-8 fixed-point
  receipts.** Stop with the drift evidence.
- **New non-generated hand-Rust bypasses appear.** Stop and report file /
  function evidence; do not patch them ad hoc unless they are clearly in
  this brief's constructor-closure scope.

## Non-goals

- Not changing the semantic rule: `AtMostOne` is idempotent under nesting.
- Not reopening parser/lowering behavior for `T??`.
- Not changing unhandled diagnostic paths or unenumerated effects.
- Not adding textual sentinels as enforcement.

## Reporting

Single narrow PR. Suggested title:
`fix(v3): close nested-optional cardinality codegen bypass`.

PR body must include:

- `git-metadata-unavailable` if authored from a broken-git worktree.
- Construction-site audit receipt.
- Generated-surface disposition for all three generated files.
- Test list and any regeneration caveat.
