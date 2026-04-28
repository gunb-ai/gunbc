# R1 Surface Manager Brief

> **🔄 SUPERSEDED 2026-04-26 by [`r1-closure-manager.md`](r1-closure-manager.md).**
> R1 gate-close authority now lives with the R1 Closure Manager (PR #847)
> under strict-interpretation reading: every gate in `ROADMAP.md §"Lane
> acceptance — .dag gates"` must be a `.dag` `TestClaim` that compiles AND
> evaluates true. Implementation receipts are necessary but **not sufficient**.
> The "Working state" section below was originally written under a more
> permissive reading where receipt-landed counted as gate-closed; that
> conflation is corrected in the table — implementation receipts and `.dag`
> gate status are now tracked as separate columns. Gates remain owned by
> R1 Closure Manager lanes (R1C-A through R1C-F) until they evaluate.
> This brief stays in-tree as a historical receipt of the lane sequencing;
> new R1 dispatch happens under R1 Closure Manager, not here.

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
- `CharClass` in `std.unicode` phase-1 landed in PR #693. The
  remaining structural scanner-row closeout is not a T-Sub-only
  surface fix: quiet-gull-882 confirmed it is blocked on top-level
  `ValueBody` list/sum support plus a `std.unicode` bootstrap/load-set
  decision.
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
- **Handoff.** T-Sub `sub_charclass_in_std_unicode` phase-1 landed.
  Phase-2 should be handed to substrate/self-hosting capability work with
  the minimal fixture `data xs: List<Int> = [1, 2]` and target fixture
  `data ascii_scan_order: List<CharClass> = [Whitespace, Digit,
  IdentStart, IdentContinue]`.
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
- **Up to director / substrate manager.** `sub_charclass_in_std_unicode`
  phase-2 is a substrate/load-set handoff: top-level structural
  `ValueBody` support for list/sum bodies, plus a decision on including
  `dsl/std/unicode.dag` in the runtime bootstrap/load set.

## Working state

> **Reading note (2026-04-26 SUPERSEDED amendment).** Implementation receipts
> (`Impl` column) record that the underlying feature work is in-tree. `.dag`
> gate status (`Gate` column) records whether the corresponding `TestClaim`
> in `ROADMAP.md §"Lane acceptance — .dag gates"` exists, compiles, and
> evaluates true under the strict-interpretation reading. Gate columns are
> closed only when both halves are true; the implementation-only `[x]` markings
> below are deliberately separated from gate close so the conflation that
> existed before this amendment doesn't recur. Gate-close authority is
> R1 Closure Manager (`docs/briefs/r1-closure-manager.md`).

**T-P0 — implementation closed on current ancestry; `.dag` gates owned by R1 Closure R1C-B:**

| Item | Impl | Gate (`.dag` TestClaim) | Owner |
|---|---|---|---|
| `repeat_string` | [x] (brief: `p0-render-repeat-string.md`) | [ ] `p0_repeat_string_correct` `[Day 1]` (structural); interim `p0_repeat_string_v2_oracle_rust_bridge` in `r1_gates` | R1C-B |
| `REST_OPS` drift | [x] (brief: `p0-rest-ops-drift.md`) | [ ] `p0_rest_ops_aligned` `[ext]` | R1C-B |
| `no_profile_sentinel` | [x] (brief: `p0-bug-no-profile-sentinel.md`) | [ ] `p0_no_fabrication_sentinel` `[ext]` | R1C-B |

**T-Sub — `.dag` gates: 1 closed, 1 receipt-only, 1 substrate-deferred:**

| Item | Impl | Gate (`.dag` TestClaim) | Owner |
|---|---|---|---|
| `sub_match_over_user_sum` (Day-1) | [x] PR #702 | [x] `TestClaim` + suite in `r1_gates.dag` runs through `TestRunner` | gate evaluates; closed |
| `sub_type_alias_where_lowers` (`[ext]`) | [x] PR #703 (DB-11 parse+lower) | [x] `DeclarationHasRefinement("PositiveInt")` on DB-11 witness; `sub_type_alias_where_lowers_gate` in `r1_gates.dag` + `test_runner_runs_sub_type_alias_where_lowers_gate` (#879) | R1C-C (closed) |
| `sub_charclass_in_std_unicode` phase-2 | [x partial] PR #693 (phase-1 tokenizer half) | [ ] reclassified to **R2 T-Substrate** per 2026-04-24 amendment (Class 5 Gap 3) — no longer an R1 gate | R2 Substrate Manager |

**T-Emit — implementation in flight; all three `.dag` gates owned by R1 Closure R1C-E:**

| Item | Impl | Gate (`.dag` TestClaim) | Owner |
|---|---|---|---|
| Rust harden | [x] PR #694 (host-harness gate test, `#[ignore]`-able sweep) | [ ] `emit_rust_fixtures_rustc_green` `[ext: ExecuteCommand]` — host harness is **not** the `.dag` gate; R1C-E wraps it | R1C-E |
| Generic-bound fidelity | [partial] PR #676 in review | [ ] `emit_generic_bounds_survive` `[ext]` | R1C-E |
| Python/Go reconcile | [partial] #691 (Python parity) + #692 (`Behavior::Loop`) | [ ] `emit_omni_demo_fixtures_green` `[ext: ForAllTargets + ExecuteCommand]` | R1C-E |

Decisions log (append as they happen):

- `2026-04-26` — T-Sub strict `.dag` gate for `sub_type_alias_where_lowers`
  closed via PR #879 (`adda0eac`): `DeclarationHasRefinement("PositiveInt")`
  on the same witness as `test_db11_type_alias_where_survives_parse_and_lower`;
  `sub_type_alias_where_lowers_gate` green under `TestRunner`.
- `2026-04-24` — T-Sub wave landed. `sub_match_over_user_sum` Day-1 gate
  closed via #702 (with #690 prior receipt). DB-11 parse+lower substrate
  landed via #703 (type-alias RHS `where`); the `.dag` gate itself awaits
  the [ext] predicate path. `sub_charclass_in_std_unicode` phase-1 closure
  via #693 + ROADMAP row #706 — tokenizer authority + mirror; syntax
  consumer wiring and structural `.dag` read remain (blocked on M1(2.8)
  class-5 gap #3).
- `2026-04-24` — T-Emit partial progress. `emit_rust_fixtures_rustc_green`
  named gate test landed in #694 (ignored baseline sweep; non-ignored
  roundtrips green). Python/Go cross-target progress via #691 (Python
  operator-realization parity) and #692 (`Behavior::Loop` emission for
  Python + Go). `emit_generic_bounds_survive` (#676) still in review;
  `emit_omni_demo_fixtures_green` pending.
- `2026-04-23` — T-P0 reclassified from dispatchable R1 work to already-landed
  closure on current ancestry after direct source audit (`dsl/std/render.dag`,
  `src/v2/tests/src/effects.rs`,
  `src/v2/tests/src/bug_sentinel_ratchet.rs`). Keep the lane in the brief only
  as an enabling receipt for downstream R1 work.
- `2026-04-24` — T-Sub receipt cleanup: `sub_match_over_user_sum` is landed
  via PR #702, and `sub_type_alias_where_lowers` is landed via PR #703. The
  only open T-Sub dispatch is `sub_charclass_in_std_unicode` phase-2
  reproduction/triage.
- `2026-04-24` — quiet-gull-882 confirmed `sub_charclass_in_std_unicode`
  phase-2 is not blocked by parser list syntax. It is blocked by top-level
  `ValueBody` list/sum support and by `std.unicode` not being in the default
  bootstrap/load set.

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- **Self-hosting Manager**: first T-Sub gate landed — `sub_match_over_user_sum`
  Day-1 via #702 (merged 2026-04-24). `match` lowering is live for
  consumer self-hosting surfaces; T-PB-A match-emit-dependent clusters
  (`regen-emits-match`, variant-constructor templates) can now plan
  consumption. ⬅ **SEND**
- **Self-hosting / Substrate Manager**: `sub_type_alias_where_lowers`
  parse+lower landed via #703, and CharClass phase-2 is a substrate/load-set
  handoff: top-level structural `data` list/sum bodies plus `std.unicode`
  bootstrap/load-set inclusion.
- **Testgen Manager**: T-Emit `emit_omni_demo_fixtures_green` still
  pending — predicate shape coordination not yet triggered. Cross-
  target progress continues in #691 / #692 but does not change the
  predicate shape ask.
