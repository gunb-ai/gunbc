# R1 Self-hosting Manager Brief

## Orient before reading

- Product direction: [PR #672](https://github.com/gunb-ai/gunbc/pull/672)
  — `docs/thesis/compositional-modeling.md` explains *why*
  self-hosting matters: if the compiler isn't written in its own
  language, the reward-structure pitch ("modeling pays back with
  compile-time guarantees") breaks at the author boundary. The
  compiler's own reasoning should live in `.dag` so the guarantees
  it provides to user code also apply to it.
- Coordination context: [R1 Director Brief](r1-director-brief.md).
- Scope authority: [`THESIS.md`](../../THESIS.md) +
  [`ROADMAP.md`](../../ROADMAP.md). This brief does not author R1
  scope; it sequences and coordinates what those docs already name.

## Slice

This manager owns two lanes:

- **`T-PB-A`** (`ROADMAP.md:53`) — compiler self-emits (fixed-
  point); **non-test** hand-Rust surface reaches the ≤5 irreducible-
  shim floor per [`docs/design-pure-bootstrap.md`](../design-pure-bootstrap.md).
  Generated escape hatch OK. Live baseline is the non-test subset
  of the SG-0 census (`EXPECTED_HAND_AUTHORED_NON_TEST` file-level +
  `EXPECTED_HAND_AUTHORED_FRAGMENTS` crate-root scaffolds) in
  `src/v3/compiler/tests/integration/sg0_census_test.rs`. This
  brief does not freeze the count.
- **`T-PB-B`** (`ROADMAP.md:54`) — tests-as-data. Pipeline and
  contract tests port to `.dag`. The two `TESTING.md §Post-R2 shape`
  residual categories (compiler-internal unit tests for Rust-only
  helpers; external-toolchain boundary tests invoking
  rustc/go/python) remain Rust-authored. Size **M**.

## Framing question this manager answers

**Does gunbc compile itself from a named minimal Rust shim, with
the residual Rust floor structurally bounded and mechanically
checkable, including tests-as-data?**

Today:
- Residual Rust count is tracked by the SG-0 census with a **mechanical**
  non-test vs test split: `EXPECTED_HAND_AUTHORED_NON_TEST` +
  `EXPECTED_HAND_AUTHORED_FRAGMENTS` (T-PB-A) and `EXPECTED_HAND_AUTHORED_TEST`
  (T-PB-B) in `sg0_census_test.rs` (landed on `main` via PR #683).
- `pb_hand_rust_at_shim_floor` and `pb_rust_tests_outside_residual_zero`
  still need to **compile and evaluate** against that partition once
  T-TestGen extensions support them (`[ext]` predicates).
- Pipeline / contract tests largely remain Rust-authored until the
  testgen runner lands; T-PB-B owns draft `.dag` claims in the interim.

The ask: close both halves. Non-test ≤5 irreducible-shim floor.
Tests live as `.dag` data evaluated by the testgen runner, except
the two acknowledged residuals.

## Sequence + dispatch

- **T-PB-A — SG-0 partition (done).** Non-test vs test sub-ratchets
  (`EXPECTED_HAND_AUTHORED_NON_TEST` and `EXPECTED_HAND_AUTHORED_TEST` in
  `sg0_census_test.rs`) are mechanically checked on `main` (PR #683).
- **T-PB-A — ongoing.** Dispatch non-test hand-Rust reduction (PB program),
  compiler–`std/` consolidation, and cementing per sections below.
- **Day 1 — T-PB-B pipeline porting, Rust-side.** Identify which
  pipeline / contract tests are candidates for `.dag` conversion
  (vs. the two TESTING.md residuals). Draft `TestClaim`
  declarations in conversational `.dag` so they're ready when
  Testgen's runner closes. Don't land the conversion yet — the
  runner needs to exist to evaluate them.
- **Gated on Testgen Manager.** T-PB-B's landing gate
  (`pb_test_file_generated_from_dag` + `pb_rust_tests_outside_residual_zero`,
  both `[ext]` on testgen extensions per `ROADMAP.md:65`) unblocks
  when Testgen's runner lands. Until then: draft `.dag` test
  declarations, don't convert Rust tests yet.
- **Cementing.** Maintain cementing tests that compare v3 compiler
  self-emission against its own generated form (fixed-point
  property — `pb_self_compile_fixed_point`). These should pass
  before and after each non-test shim reduction.

## Hand-off points

- **Sideways from Surface Manager.** Several T-Sub closures unblock
  self-hosting paths. Specifically `sub_match_over_user_sum` gates
  `match` lowering needed by the compiler's own sum-type code
  paths; `sub_type_alias_where_lowers` gates refinement patterns
  used by std/ types the compiler consumes. Coordinate with
  Surface for Day-1 prioritization of the T-Sub sub-deliverables
  that unblock the most self-hosting surface.
- **Sideways from Testgen Manager.** T-TestGen's runner closure is
  the gate for T-PB-B. Testgen manager signals readiness; this
  manager dispatches the Rust→.dag test conversion at that signal.
- **Up to director.** Any proposal to adjust the two TESTING.md
  residual categories (what counts as compiler-internal unit test,
  what counts as external-toolchain boundary test). These are
  scope decisions that live in ROADMAP / TESTING.md.
- **Up to director.** If the SG-0 census partition implementation
  needs a broader refactor than expected, flag for director before
  proceeding — the ratchet is load-bearing.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-PB-A:**
- [x] Non-test SG-0 census partition (`EXPECTED_HAND_AUTHORED_NON_TEST`
      / `EXPECTED_HAND_AUTHORED_TEST` split in `sg0_census_test.rs`)
- [ ] Hand-Rust non-test reduction toward ≤5 irreducible-shim floor
- [ ] `pb_hand_rust_at_shim_floor` predicate compiles once T-TestGen
      extensions land
- [ ] `pb_self_compile_fixed_point` predicate compiles once T-TestGen
      extensions land
- [ ] `pb_compiler_std_ratchet_zero` predicate compiles once
      T-TestGen extensions land

**T-PB-B:**
- [ ] Draft `.dag` `TestClaim` declarations for pipeline tests
      (non-landing — wait on Testgen runner)
- [ ] Draft `.dag` `TestClaim` declarations for contract tests
      (non-landing — wait on Testgen runner)
- [ ] Identify and scope the two TESTING.md residual categories
      per-test
- [ ] Land `.dag` test conversion once Testgen signals runner
      readiness
- [ ] `pb_test_file_generated_from_dag` + `pb_rust_tests_outside_residual_zero`
      predicates evaluate true

Decisions log (append as they happen):

- _(none yet)_
- 2026-04-23: SG-0 file-level census split landed as non-test/test
  sub-ratchets; `tokenize.rs` shim retired from the non-test subset.

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- _(none yet — watching Testgen for runner readiness signal)_
