# R1 Surface Manager Brief

## Orient before reading

- Product direction: [PR #672](https://github.com/gunb-ai/gunbc/pull/672)
  — `docs/thesis/compositional-modeling.md`. This manager's slice
  covers what developers actually see: real bugs surface today as
  real bugs (P0 sweep), missing surface features land as first-
  class `.dag` (T-Sub), and multi-target emission produces code
  that external toolchains accept (T-Emit). If these drift, the
  story doc's claims stop being true at the user's terminal.
- Coordination context: [R1 Director Brief](r1-director-brief.md).
- Scope authority: [`THESIS.md`](../../THESIS.md) +
  [`ROADMAP.md`](../../ROADMAP.md). This brief does not author R1
  scope; it sequences and coordinates what those docs already name.

## Slice

This manager owns three lanes:

- **`T-P0`** (`ROADMAP.md:47`) — P0 sweep:
  - `repeat_string` — real bug in existing emission.
  - `REST_OPS` — drift between the declared REST ops surface and
    what's actually emitted.
  - `no_profile_sentinel` — fabrication sentinel audit.
  - Size **S**. Brief-level receipts live at `docs/briefs/p0-*.md`.
- **`T-Sub`** (`ROADMAP.md:48`) — surface capability receipts:
  - `match` over user sums (`sub_match_over_user_sum` — `[Day 1]`
    gate; landed PR #702).
  - `CharClass` in `std.unicode` (`sub_charclass_in_std_unicode` —
    `[ext]` gate; phase-2 reproduction/triage remains open).
  - type-alias `where` (`sub_type_alias_where_lowers` — `[ext]`
    gate; landed PR #703).
  - Size **S**.
- **`T-Emit`** (`ROADMAP.md:49`) — emission hardness:
  - Rust harden.
  - PR #650 generic-bound fidelity.
  - Python/Go reconcile.
  - Gates: `emit_rust_fixtures_rustc_green`,
    `emit_generic_bounds_survive`, `emit_omni_demo_fixtures_green`.
  - Size **M**.

## Framing question this manager answers

**Do the user-surface capabilities — bug fixes, missing-feature
closures, and multi-target emission — land such that generated code
runs under external toolchains and no "it works around the gap"
scaffolding survives?**

Today:
- T-P0 is already closed on current ancestry: `repeat_string` is fixed in
  `dsl/std/render.dag`, `REST_OPS` now derives from extdep authority in
  `src/v2/tests/src/effects.rs`, and the `__BUG_NO_PROFILE_` fabrication
  sentinel is removed with a ratchet in
  `src/v2/tests/src/bug_sentinel_ratchet.rs`. Keep the lane listed as an
  R1 enabler receipt, not active dispatch.
- `match` over user sums landed in PR #702. It has a Day-1
  `TestClaim` in `src/v3/compiler/tests/fixtures/r1_gates.dag`, a
  runner receipt in `test_runner_runs_sub_match_over_user_sum_gate`,
  and a rustc boundary receipt in `sub_match_over_user_sum_links_and_runs`.
- Type-alias `where` landed in PR #703. Alias-RHS predicates now parse
  and lower; receipts are the `test_db11_type_alias_where_*`
  integration tests in `m2_feature_parity_test.rs`.
- `CharClass` in `std.unicode` is the character-level consumption
  gap per `ROADMAP.md:353` — the types exist in `dsl/std/`; the
  tokenizer and syntax authorities aren't using them yet.
- Cross-target emission green-ness is the external-toolchain
  receipt: generated Rust compiles under `rustc`; generated Python
  runs under CPython; generated Go builds under `go build`.

The ask: close each. Surface is where "we can generate code that
runs" is literally true, so it's the manager whose work is easiest
for a principal engineer to verify in one evening.

## Sequence + dispatch

- **Landed.** T-Sub `sub_match_over_user_sum` landed in PR #702.
  The `[Day 1]` gate compiles against today's DB-15 schema and runs
  through `TestRunner`.
- **Day 1.** T-Emit Rust-harden dispatches. Rust is the primary
  target; Python and Go reconcile against it. No cross-manager
  blocker.
- **Landed.** T-Sub `sub_type_alias_where_lowers` landed in PR #703.
  DB-11 alias-RHS parse/lower receipts are in `test_db11_type_alias_where_*`.
- **Parallel.** T-Sub `sub_charclass_in_std_unicode` dispatches.
  Add `CharClass = Whitespace | Digit | IdentStart | IdentContinue`
  (or superset) to `std.unicode` per `ROADMAP.md:353`;
  retype the opaque-string fields in `tokenize.dag` / `syntax.dag`
  (`suffix: Char`, `output_codepoint: Char`, `pattern: List<Char>`,
  etc.); rewire `regen_tokenize` to read class predicates
  structurally.
- **Parallel.** T-Emit Python/Go reconcile dispatches after Rust
  harden has a stable baseline. The goal is `emit_omni_demo_fixtures_green`
  across all three targets.

## Hand-off points

- **Sideways to Self-hosting Manager.** `sub_match_over_user_sum`
  landed in PR #702 and unblocks `match` lowering in the compiler's
  own sum-type code paths.
- **Sideways to Self-hosting Manager.** `sub_type_alias_where_lowers`
  landed in PR #703 and unblocks refinement patterns used by `std/`
  types the compiler consumes.
- **Sideways to Testgen Manager.** `emit_omni_demo_fixtures_green`
  requires the `ExecuteCommand` + `ForAllTargets` predicates —
  those are T-TestGen `[ext]` extensions. Coordinate with Testgen
  on predicate shape when Rust harden lands; that predicate wiring
  is jointly owned.
- **Up to director.** If P0 fixes surface a class of similar bugs
  (not individual incidents but a systemic miss), flag to director
  — that's an R1 scope question, not a lane-owner call.
- **Up to director.** If the `CharClass` lane reveals more
  character-level consumption gaps beyond what's cited at
  `ROADMAP.md:353`, flag to director so the cardinality-substrate
  ledger row can be amended rather than expanding the lane.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-P0 (closed on current ancestry):**
- [x] `repeat_string` fix landed (brief: `p0-render-repeat-string.md`)
- [x] `REST_OPS` drift resolved (brief: `p0-rest-ops-drift.md`)
- [x] `no_profile_sentinel` audit completed (brief: `p0-bug-no-profile-sentinel.md`)

**T-Sub:**
- [x] `sub_match_over_user_sum` gate compiles + passes (Day-1; PR #702)
- [x] `sub_type_alias_where_lowers` gate compiles + passes (PR #703)
      (DB-11 alias-RHS path)
- [ ] `sub_charclass_in_std_unicode` gate compiles + passes

**T-Emit:**
- [ ] Rust harden — `emit_rust_fixtures_rustc_green` passes
- [ ] PR #650 generic-bound fidelity — `emit_generic_bounds_survive`
      passes
- [ ] Python/Go reconcile — `emit_omni_demo_fixtures_green` passes
      across all three targets

Decisions log (append as they happen):

- `2026-04-23` — T-P0 reclassified from dispatchable R1 work to already-landed
  closure on current ancestry after direct source audit (`dsl/std/render.dag`,
  `src/v2/tests/src/effects.rs`,
  `src/v2/tests/src/bug_sentinel_ratchet.rs`). Keep the lane in the brief only
  as an enabling receipt for downstream R1 work.
- `2026-04-24` — T-Sub receipt cleanup: `sub_match_over_user_sum` is landed
  via PR #702, and `sub_type_alias_where_lowers` is landed via PR #703. The
  only open T-Sub dispatch is `sub_charclass_in_std_unicode` phase-2
  reproduction/triage.

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- Self-hosting can consume the landed T-Sub `match` and alias-`where` receipts
  from PR #702 / PR #703.
- Coordinate with Testgen when T-Emit `emit_omni_demo_fixtures_green`
  predicate shape needs defining.
