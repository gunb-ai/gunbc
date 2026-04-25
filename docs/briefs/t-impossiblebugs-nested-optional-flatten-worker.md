# T-ImpossibleBugs — nested-optional flatten `(S, R2)`

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 1 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes — any worker can dispatch
> in parallel. Reports to Director (`zesty-bear-812`).

## Read first

- **[`THESIS.md` §"Enumerable impossible-bug classes" lines 342-344](../../THESIS.md)** — class definition: *"`Option<Option<T>>` accessor patterns requiring hand-unwrapping in normal languages."* The bug class is: a user writes code that requires multiple-level option unwrapping (`x?.y?.z`); gunbc instead **flattens `Option<Option<T>>` to `Option<T>` structurally**, making the nested-unwrap pattern impossible to write. THESIS.md:343 explicitly tags: *"Gated on cardinality refinement substrate work."*
- **[`docs/r2-structure.md` §"Goal 4"](../r2-structure.md)** — sub-lane scoping; tagged `[R2+]` per ROADMAP T-Demo row.
- **[`dsl/std/types.dag`](../../dsl/std/types.dag)** + **[`dsl/std/algebra.dag`](../../dsl/std/algebra.dag)** — current Option / Maybe-equivalent type declarations. Locate where `Option<T>` is declared today and assess whether nested forms are constructible.
- **[`docs/architecture.md`](../architecture.md) §"How the compiler knows it" — cardinality bridge** (`return_cardinality` enum on Node; 142 construction sites). The dissolution path: cardinality refines into edge-existence patterns; once refined, `Option<Option<T>>` becomes un-expressible at construction time.
- **[`docs/db-history/db-11.md`](../db-history/db-11.md)** — alias-RHS `where` (PR #703) — closest **structural precedent** for type-construction filtering via predicates. The pattern (predicate at construction site rejects invalid inhabitants) generalizes to the cardinality-collapse mechanism this brief needs.
- **[`MODELING.md`](../../MODELING.md)** — especially M9 (DFS the concept DAG) for where `Option` lives in the existing concept hierarchy.
- **[`INVARIANTS.md`](../../INVARIANTS.md)** — `feedback_construction_over_ratchets`: model first, violations dissolve.

## Frame

The bug class is **structurally-impossible to express** rather than runtime-detected. The user can't write `Option<Option<T>>` because the type system collapses it at construction time. This is *"impossible by construction"* per the discipline — no heuristic, no warning, just the type doesn't exist.

Today: v3 has no mechanism to detect or flatten nested options; the type-algebra constructor for `Option<Option<_>>` is allowed (as far as discoverable). Cardinality substrate exists in `architecture.md` but is not yet refined into edge-existence patterns at the type-algebra layer.

The dissolution requires structural change at the type-algebra construction site so `Option<Option<T>>` simplifies to `Option<T>` *during type checking*, not at access time. Once the substrate is in place, no diagnostic is needed at the user-code level — the nested form is unrepresentable.

## Three consumer-side requirements

1. **Cardinality-aware Option construction.** When the type-algebra constructor sees `Option<Option<T>>`, it returns `Option<T>` (idempotent flatten). Substrate-level mechanism — extends type-resolution / inhabitance check / algebra-attachment, depending on how Option is currently expressed.
2. **No diagnostic at user-code level for the nested form.** The collapsed type is the only valid inhabitant; the user wrote `Option<Option<T>>` syntactically and got `Option<T>` semantically. This may produce a name-resolution surprise (the alias they wrote is not the alias they got); decide whether to surface a `info`-level "flattened to" hint or leave silent. Worker-call; surface in PR description.
3. **Smoke + integration test demonstrating the collapse.** A user-authored type `type Nested = Option<Option<Int>>` resolves to `Option<Int>` structurally; matching against it requires only one level of unwrap; attempting two levels of pattern-match is rejected at the type-checker layer.

## Slice — cardinality flatten on Option

1. Locate Option declaration in `dsl/std/`. Audit its type-algebra attachment.
2. Add idempotent-flatten substrate mechanism — likely a new fact on the type-algebra declaration ("idempotent under self-nesting" → constructor flattens) or a generalized cardinality-refinement at the algebra layer.
3. Wire the constructor / type-resolver to apply the flatten when seen.
4. Smoke + integration tests (per req 3).
5. Any retired hand-Rust off SG-0 census; doc-comment updates.

## Acceptance

- [ ] All 3 consumer-side requirements satisfied + documented in PR body.
- [ ] `Option<Option<T>>` flattens to `Option<T>` at type-construction time.
- [ ] No silent-fail or partial-flatten path; tests confirm one level only.
- [ ] Decision on info-level "flattened to" hint documented (worker-call).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas as needed.

## STOP-AND-ESCALATE

Surface to Director.

- **Cardinality-substrate dependency surfaces** — if implementing Option-flatten requires the full cardinality-refinement substrate (which is named in r2-structure.md as outside-R2-scope for the full capability), STOP. May need substrate-extension first; or may need a narrower bypass for this specific case.
- **Option declaration is in extdeps not std/** — if Option lives per-target rather than in `std/`, the flatten substrate needs to attach at a different layer; surface for Director routing.
- **Flatten interacts poorly with existing pattern-match** — if the user-written `Option<Option<T>>` is already accepted by surface parser + lowerer in ways that produce nested patterns, retroactive flattening may break parse-output expectations. STOP; coordinate with parser owners.
- **Generalization concern** — if the `idempotent under self-nesting` flatten generalizes to other types (`List<List<T>>`? `Set<Set<T>>`?) in unexpected ways, STOP. Director-call on scope.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not implementing the full cardinality-refinement substrate** — only the subset that closes Option-flatten.
- **Not refactoring Option's runtime representation** beyond what's needed for the type-construction collapse.
- **Not implementing the other two T-ImpossibleBugs classes** (unhandled-diagnostic-paths, unenumerated-effects) — independent briefs.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs — Option<Option<T>> flattens to Option<T> at construction (closes nested-optional flatten class)`.
- PR body cites this brief + addresses the 3 reqs + documents the info-hint policy.
- On merge: signal Director.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if substrate.dag-adjacent.
- **Grounding Manager**: no current overlap.
