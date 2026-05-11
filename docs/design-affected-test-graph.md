# Affected Test Graph

## Problem

The question CI should answer is not "which broad bucket should run:
analysis, integration, demo, or full suite?" It is:

> Given this change, which test targets depend on the changed facts?

That requires a real dependency graph with recursive reverse edges. A
test is selected because it depends on an affected target, not because a
human placed it in a coarse cost category.

## Target Model

The build/test graph has four first-class node kinds:

- `SourceTarget`: an owned source file or generated source unit.
- `GeneratedTarget`: an artifact produced from one or more sources.
- `LibraryTarget`: a compilable unit, such as a Rust crate module set or
  generated compiler surface.
- `TestTarget`: a runnable test binary, module slice, `.dag` `TestSuite`,
  or external boundary receipt.

Edges are explicit and recursive:

- `SourceTarget -> GeneratedTarget` when a generator consumes the source.
- `SourceTarget -> LibraryTarget` when code is compiled directly.
- `GeneratedTarget -> LibraryTarget` when generated code enters a crate.
- `LibraryTarget -> TestTarget` when a test imports or executes that
  library surface.
- `TestTarget -> TestTarget` only for harness aggregation, such as the
  consolidated `src/v3/compiler/tests/integration.rs` binary wrapping
  per-module test targets.

Cost is metadata on `TestTarget`, not the selector:

- `estimated_wall_ms`
- `layer` (`unit`, `integration`, `boundary`, `demo`, `ratchet`)
- `requires_tools`
- `main_only_allowed`

The selector may use cost to order or cap work, but it must not use cost
as the primary dependency relation.

## Query

The affected-test query is:

1. Compute changed paths against a base ref.
2. Map each path to one or more graph nodes.
3. Walk reverse dependencies to every reachable `TestTarget`.
4. Collapse harness children into runnable commands.
5. Sort selected tests by descending expected or last-observed cost.

That gives:

```text
changed file -> source/generated/library target -> reachable tests -> commands
```

The important property is closure. If `dsl/std/types.dag` changes, every
generated Rust artifact and test that consumes the generated type surface
is selected through graph edges. If `src/v3/compiler/src/lens_cost*.rs`
changes, only tests depending on the cost lens surface are selected,
plus any aggregate harness needed to execute them.

## Bazel/Buck2 Fit

Bazel or Buck2 is the right external shape if we commit to target-level
ownership rather than one broad `cargo test` target.

Required mapping:

- Rust crate/library targets map to `rust_library`.
- Rust test binaries map to `rust_test`.
- `.dag` files map to source targets plus generated targets for emitted
  Rust snapshots or parser/bootstrap artifacts.
- Generator invocations map to actions with declared inputs and outputs.
- Test modules currently hidden inside `tests/integration.rs` need
  synthetic child targets so the affected graph can select
  `lane2_stage_2d_symbolic_cost_test` without selecting every demo.

The consolidated Rust integration binary can remain as an execution
optimization, but the graph must still represent its child modules as
selectable logical test targets. The runner can then execute one binary
with filters for the affected child targets.

## Near-Term Implementation

Do this in slices before adopting a full Bazel/Buck2 migration:

1. Emit a repository-local target manifest.
   - Start with `cargo metadata` for crates and Rust test targets.
   - Add a checked-in or generated manifest for `.dag` sources,
     generated artifacts, and integration-module child tests.

2. Add `affected-tests` query tooling.
   - Input: `base_ref`, `head_ref`.
   - Output: ordered runnable commands plus selected target IDs.
   - Fail closed when a changed path has no owning target.

3. Wire PR CI to the query.
   - Run selected targets for PRs.
   - If ownership is incomplete, fall back to the current full v3 suite
     and report the unmapped paths.

4. Keep main-push full coverage.
   - Main still runs the full suite while graph coverage matures.
   - The per-test timing ratchet feeds `estimated_wall_ms`, sorted
     descending so the most expensive reachable tests are visible first.

5. Replace the manifest backend with Bazel or Buck2.
   - Once the target model is accurate, Bazel/Buck2 can become the
     execution backend rather than the place where we discover the model
     by hand.

## Acceptance Criteria

- A changed path maps to at least one owning target or fails closed.
- The query returns a deterministic, recursively closed set of tests.
- Output is sorted by descending observed or estimated cost.
- PR CI runs affected tests, not every demo.
- Main CI still runs the full coverage set until affected-test coverage
  has proved complete.
- Adding a new test requires declaring its dependencies, not adding it to
  a broad category list.

