# R1C-C — T-Sub `sub_type_alias_where_lowers` fixture `(XS, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-C. Closes 1 unwired T-Sub gate. Reports to R1 Closure Manager.

## Read first

- **[`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md)** — T-Sub row: `sub_type_alias_where_lowers` [ext, landed PR #703]. The feature is landed (PR #703); the fixture wrapper is what's missing.
- **PR #703** — type-alias `where` lowering. Integration tests at `test_db11_type_alias_where_*` are the reference for input + expected behavior.
- **[`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag)** — current R1 gate fixture file. Pattern to mirror.
- **[`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs)** — runner dispatch.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Frame

PR #703 landed type-alias `where` lowering with `test_db11_type_alias_where_*` integration tests as the behavioral receipt. The `.dag` TestClaim wrapper just needs to author against that landed feature with the appropriate predicate.

The `[ext]` tag in ROADMAP originally allowed a new predicate. **`Compiles` alone is insufficient** for this gate name: it does not assert `Declaration.refinement` survives lowering. The landed receipt uses **`DeclarationHasRefinement("…")`** on the same witness program as `test_db11_type_alias_where_survives_parse_and_lower` (plus `test_runner` / M1.5 harness wiring).

## Single deliverable

Author `sub_type_alias_where_lowers` TestClaim in `r1_gates.dag` (or sibling). Predicate: **`DeclarationHasRefinement`** on the alias identifier so the gate fails closed if alias-`where` lowering silently drops the refinement edge. Wire runner dispatch in `test_runner.rs`.

## Slice — single PR

1. Read `test_db11_type_alias_where_*` integration tests for the canonical input/output pair.
2. Author the `.dag` fixture program that exercises type-alias `where` lowering.
3. Author the `sub_type_alias_where_lowers` TestClaim referencing the fixture; predicate **`DeclarationHasRefinement`** (not `Compiles` alone).
4. Wire runner dispatch (`DeclarationHasRefinement` arm in `TestRunner::run_claim`).
5. Verify gate evaluates `Pass`.

## Acceptance

- [x] `sub_type_alias_where_lowers` `.dag` TestClaim authored + runner dispatch + gate evaluates `Pass`.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status table updated to "R1C-C: 1/1 gates green."

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **The `[ext]` predicate turns out to need a shape not in DB-15 + not in R1C-A's scope** → STOP. Escalate to R1 Closure Manager for cross-lane scoping coordination.
- **Type-alias `where` regression discovered while authoring fixture** (existing PR #703 receipts fail) → STOP immediately. The fixture authoring is supposed to be additive; regression indicates an earlier landing is not actually held.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-C.
- Feature receipt: PR #703 + `test_db11_type_alias_where_*` integration tests.
- Gate authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) T-Sub row.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
