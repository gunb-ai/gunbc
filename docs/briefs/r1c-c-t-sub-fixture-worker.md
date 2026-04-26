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

The `[ext]` tag in ROADMAP suggests a new predicate may be needed, but the simplest predicate (`Compiles` on a fixture program that exercises type-alias `where` lowering) is DB-15 schema and dispatchable Day-1. **Verify at brief authoring** which predicate the gate evaluates.

## Single deliverable

Author `sub_type_alias_where_lowers` TestClaim in `r1_gates.dag` (or sibling). Predicate: most likely `Compiles` on a fixture program declaring a type alias with a `where` clause that lowers correctly per PR #703 receipts. Wire runner dispatch in `test_runner.rs`.

## Slice — single PR

1. Read `test_db11_type_alias_where_*` integration tests for the canonical input/output pair.
2. Author the `.dag` fixture program that exercises type-alias `where` lowering.
3. Author the `sub_type_alias_where_lowers` TestClaim referencing the fixture; pick predicate (`Compiles` likely).
4. Wire runner dispatch.
5. Verify gate evaluates `Pass`.

## Acceptance

- [ ] `sub_type_alias_where_lowers` `.dag` TestClaim authored + runner dispatch + gate evaluates `Pass`.
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
