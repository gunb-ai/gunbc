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
  both `[ext]` on testgen extensions per `ROADMAP.md:68`) unblocks
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
  The numbered **pre–Rust-deletion** coordination items (runner
  entrypoint, `requires`, M1(2.8), first deletion batch) are maintained
  only in **`docs/briefs/r1-testgen-manager.md`** (*Hand-off points* →
  **Sideways to Self-hosting Manager**) — keep that list single-source
  until Testgen replies.
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
- [x] T-PB-B-1 — Landed `src/v3/compiler/tests/dag/*.dag` + brief
      `docs/briefs/t-pb-b-1.md` (former compile-only smoke
      `t_pb_b_1_tests_dag_smoke_test` retired as redundant with
      `t_pb_b_1_dag_runner_test` — 2026-04-25).
      **Not** Rust test deletion and **not** `pb_*` — those stay unchecked below
      until Testgen signs off.
- [x] T-PB-B-1 — Runner-backed integration test landed (PR #736,
      `t_pb_b_1_dag_runner_test`): closes pre–Rust-deletion checklist items (1)
      runner accepts `tests/dag/` layout via `compile_to_dag`, (2) `requires: []`
      lowers to a runner-consumable shape; (3) M1(2.8) inlining is structurally
      enforced in `t_pb_b_1_contract_port_cost.dag`. Item (4) first-deletion batch
      still owned by Testgen per single-source checklist in
      `docs/briefs/r1-testgen-manager.md`.
- [x] Brief D — Draft `.v3` `TestClaim` / `TestSuite` for pipeline smoke
      (compile-smoke + inventory only; Rust integration tests remain canonical;
      see `docs/briefs/t-pb-b-brief-d.md` +
      `src/v3/compiler/tests/fixtures/t_pb_b_brief_d/pipeline_smoke.v3`)
- [x] Brief D — Draft `.v3` declarations for contract tests (same meaning as
      previous row; `contract_diagnostic_smoke.v3` + `contract_port_cost_smoke.v3`)
- [x] Identify and scope the two TESTING.md residual categories
      per-test (extended D/G/A/B matrix in Brief D)
- [ ] Land **Rust deletions** for the remaining T-PB-B-1 / thesis inventory
      (checklist item (4) follow-on) once Testgen records sign-off in
      `docs/briefs/r1-testgen-manager.md` (first slice: redundant compile-smoke
      file **landed** 2026-04-25; see *Decisions log*).
- [ ] `pb_test_file_generated_from_dag` + `pb_rust_tests_outside_residual_zero`
      predicates evaluate true

Decisions log (append as they happen):

- 2026-04-24: **T-PB-B-1 runner wiring (PR #736)** — `t_pb_b_1_dag_runner_test`
  evaluates all three landed `tests/dag/*.dag` suites through `TestRunner::run_suite`
  (all claims pass); mechanically closes pre–Rust-deletion checklist items (1) runner
  entrypoint / layout and (2) `requires: []` lowering. Also aligns the `contract_port_cost`
  cost witness to the runner-evaluated `Eq 3` value (was a placeholder `Eq 8` copied from
  the unevaluated `m1_5_verification` smoke) across the landed `.dag` and Brief D `.v3`
  siblings. Rust deletion and `pb_*` still gated on Testgen checklist item (4).
- 2026-04-24: **T-PB-A reduction wave active.** Non-test hand-Rust shims
  retired today: lens_idempotency wrapper (#699), lens_parallelism wrapper
  (#715), lens_parallelism compatibility alias (#724), lower pass-through
  scaffold + `regen_lower.rs` + non-authoritative `lower_generated.rs`
  (#729), infer payload span shim (#730), plus T-PB-A-E (#731) and
  T-PB-A-F (#732). SG-0 census + SG-6 ratchets shrunk accordingly. Live
  floor read from `src/v3/compiler/tests/integration/sg0_census_test.rs`;
  the ≤5 irreducible-shim target not yet reached.
- 2026-04-24: **T-PB-B-1** — landed `src/v3/compiler/tests/dag/*.dag` (`data` `TestClaim` /
  `TestSuite` per `std.verification`); redundant compile-smoke file later
  retired. Further Rust deletions / `pb_*` coordination: `docs/briefs/t-pb-b-1.md`.
- 2026-04-23: **Brief D** (`docs/briefs/t-pb-b-brief-d.md`) — extended T-PB-B inventory +
  D/G/A/B matrix + draft `TestClaim` fixtures (`tests/fixtures/t_pb_b_brief_d/*.v3`);
  compile smoke via `t_pb_b_brief_d_fixture_smoke_test` (no `pb_*`, Rust tests unchanged).
- 2026-04-23: SG-0 file-level census split landed as non-test/test
  sub-ratchets; `tokenize.rs` shim retired from the non-test subset.

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- _(none yet — watching Testgen for runner readiness signal)_
