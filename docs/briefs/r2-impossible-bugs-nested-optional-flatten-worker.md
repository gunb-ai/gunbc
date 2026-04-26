# T-ImpossibleBugs — Nested-optional flatten implementation worker brief `(M; ungated — substrate-constructor invariant per design doc)`

> **Worker brief.** Reports through Impossible-Bugs Manager (post-R2
> spin-up) / Director (pre-spin-up). T-ImpossibleBugs Goal 4 class 1
> of 3.
>
> **NOT GATED on substrate.** Per
> [`docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md`](t-impossiblebugs-nested-optional-flatten-design.md)
> §Q2: v3 substrate is past the cardinality bridge —
> `TypeConnective::Cardinality { element, bound }` is first-class
> (`src/v3/compiler/src/dag.rs:395-398`); `CardinalityBound::AtMostOne`
> is the carrier for `Option`. The dissolution is a substrate-constructor
> invariant, not new substrate work. Dispatch immediately.

## Read first

- **[`docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md`](t-impossiblebugs-nested-optional-flatten-design.md)** — design doc with full Q1-Q4 audit, construction-site enumeration, and canonical implementation sketch. **Read in full before slicing.**
- **[`THESIS.md` lines 342-344](../../THESIS.md)** — class definition.
- **[`src/v3/compiler/src/dag.rs:395-398`](../../src/v3/compiler/src/dag.rs)** — `TypeConnective::Cardinality { element, bound }` first-class.
- **[`src/v3/compiler/src/dag_scalar_generated.rs:21-25`](../../src/v3/compiler/src/dag_scalar_generated.rs)** — `CardinalityBound::AtMostOne` (the carrier for `Option`).
- **[`src/v3/compiler/src/lower.rs:1949-1968, :2044-2047`](../../src/v3/compiler/src/lower.rs)** — `SurfaceType::Optional` lowering arms (call sites).
- **[`src/v3/compiler/src/infer.rs:2902-2916`](../../src/v3/compiler/src/infer.rs)** — `concretize_decl_with_subst` (the **killer construction site** per design doc — substitution path that bypasses `lower.rs`).
- **[`docs/modeling-discipline.md`](../modeling-discipline.md)** practice 6 — API-level enforcement over convention.
- **`feedback_state_space_vs_behavioral_invariants`** — illegal states unrepresentable, not validated.

## Frame

Per design doc §Q3: the algebraic property is `AtMostOne ∧ AtMostOne = AtMostOne` — `Option<Option<T>>` is observationally equivalent to `Option<T>` because `AtMostOne` cardinality is idempotent under self-composition.

The implementation **closes the constructor API** so that `Cardinality { element: <Cardinality{bound: AtMostOne}>, bound: AtMostOne }` literally **cannot be constructed**. Per `feedback_state_space_vs_behavioral_invariants` + INVARIANTS.md P2: the dissolution is structural, not API-convention.

**No new substrate.** The work is:
1. One predicate (`cardinality_idempotent_target`) that owns the rule.
2. One allocator (`alloc_cardinality_decl`) — the **single substrate-constructor authority**.
3. API closure on `TypeConnective::Cardinality`'s payload so the variant cannot be constructed via struct-init outside the allocator.
4. All call sites route through the allocator (3 hand-Rust + ~22 codegen sites per design doc audit).

## Slice (per design doc §Q3 canonical sketch)

1. **Author the predicate** `cardinality_idempotent_target(dag, element, bound) -> Option<DeclarationId>` in `src/v3/compiler/src/dag/builder.rs` (or adjacent). Returns `Some(inner_decl)` when the rule fires; else `None`. **Single rule authority.**
2. **Author the allocator** `alloc_cardinality_decl(dag, element, bound, span) -> DeclarationId`. If the rule fires, returns the inner decl; else allocates fresh `Cardinality { element, bound }`.
3. **Close the constructor API** — refactor `TypeConnective::Cardinality`'s payload so the variant cannot be struct-init'd outside `alloc_cardinality_decl`'s body. Per design doc canonical sketch + `docs/modeling-discipline.md` practice 6.
4. **Migrate the 3 hand-Rust call sites:**
   - `lower.rs:1949` (`type_to_declaration_id` Optional arm)
   - `lower.rs:2044` (`type_to_connective` Optional arm)
   - `infer.rs:2902` (`concretize_decl_with_subst` — the killer case for generic instantiation)
5. **Migrate the codegen path** — `regen_bootstrap_emit.rs` emits `alloc_cardinality_decl` calls instead of literal `TypeConnective::Cardinality { ... }` struct-init in regen output. Affects ~22 sites in `bootstrap_std_generated.rs`.
6. **Regression tests:**
   - `Option<Option<T>>` accessor patterns lower to `Option<T>` accessor patterns (constructive — try to allocate the nested form, observe it returns the inner).
   - `T??` source surface produces `Option<T>` after lowering (not `Option<Option<T>>`).
   - Generic instantiation: `fn foo<T>(x: T?) -> T?` instantiated with `T = Int?` produces `Option<Int>`, not `Option<Option<Int>>`.
   - Spoofing test: a non-`AtMostOne` Cardinality (e.g., `Exact(2)`) is unaffected by the idempotent rule.
7. **DB-8 fixed-point bit-identical** — existing programs with `T?` continue to compile to identical DAGs; `T??` programs that previously produced nested-cardinality now produce flat (this is the dissolution; record the existence of any).
8. **Surface-syntax `T??` decision** — per design doc §Q1: tokenizer admits `?` `?` sequence at parse time. Decision: should the parser emit a diagnostic ("nested optional is structurally flattened — write `T?` directly") OR silently normalize? Worker picks; surface in PR. Director-lean: silent normalize (the dissolution makes it identical; user-facing diagnostic adds noise).

## Acceptance

- [ ] Predicate `cardinality_idempotent_target` authored as single rule authority.
- [ ] Allocator `alloc_cardinality_decl` is the single substrate-constructor authority for `Cardinality` declarations.
- [ ] `TypeConnective::Cardinality`'s payload mechanically closed so struct-init outside the allocator is impossible (per design doc + modeling-discipline practice 6).
- [ ] All 3 hand-Rust call sites migrated.
- [ ] Codegen path (`regen_bootstrap_emit.rs`) emits allocator calls; ~22 codegen sites covered.
- [ ] Regression tests: `T??` flatten / generic-instantiation / spoofing.
- [ ] DB-8 fixed-point bit-identical.
- [ ] Surface-syntax `T??` decision documented in PR body.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **Audit reveals additional construction sites not enumerated in the design doc** — surface; the API closure may need extension. Don't migrate ad-hoc.
- **`TypeConnective::Cardinality`'s payload closure breaks more than the construction sites** — e.g., pattern-match destructuring elsewhere. Surface; may need a separate accessor pattern (read-side struct-match remains open; only write-side construction is closed).
- **Generic instantiation case (`infer.rs:2902`) reveals a deeper substitution pattern** that needs substrate work — surface; the design doc's Q4 may need extension.
- **DB-8 drifts on a program that wasn't expected to flatten** — surface; may indicate a legitimate distinction the dissolution shouldn't erase.
- **Codegen migration is too large for one PR** — surface; consider splitting hand-Rust + codegen as B4-style stacked PRs (substrate-constructor first, then codegen migration).

## Non-goals

- Not extending `OptionalOf` algebra-template semantics (per design doc §Q2 note: `OptionalOf` is a parallel construct in `algebra.dag`, not user-surface — out of scope).
- Not adding new substrate (v3 already has first-class Cardinality).
- Not authoring full DB-11 alias-`where` closure (adjacent, not subsumed).
- Not addressing other T-ImpossibleBugs classes.

## Cross-program note

- **No producer prerequisite** — substrate is past the cardinality bridge.
- **Producer:** this brief produces the constructor invariant.
- **Consumer:** existing v3 type-checker / lens consumers (no migration needed; they already walk Cardinality recursively, just stop seeing the nested form).
- **Downstream signal:** lane close → Impossible-Bugs Manager → R2 Release Manager (Goal 4 nested-optional class).

## Reporting

Single PR. Title: `feat(v3): T-ImpossibleBugs nested-optional flatten — substrate-constructor invariant for AtMostOne idempotence`. Body cites this brief + design doc + construction-site audit receipt + surface-syntax `T??` decision + DB-8 disposition.

On merge: signal R2 Release Manager + close THESIS R2+ nested-optional bullet.
