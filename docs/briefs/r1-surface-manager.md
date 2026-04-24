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
- **`T-Sub`** (`ROADMAP.md:48`) — three missing surface
  capabilities:
  - `match` over user sums (`sub_match_over_user_sum` — `[Day 1]`
    gate).
  - `CharClass` in `std.unicode` (`sub_charclass_in_std_unicode` —
    `[ext]` gate).
  - type-alias `where` (`sub_type_alias_where_lowers` — `[ext]`
    gate; alias-RHS parser skip per DB-11 at `ROADMAP.md:231`).
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
- `match` over user sums is a live surface capability gap that
  blocks self-hosting surface (Self-hosting manager's T-PB-A half
  will consume it as soon as it lands).
- Type-alias `where` is the DB-11 gap — parser `skip_where_clause`
  in `src/v3/compiler/parse_parser_body.txt:651` (called from
  `parse_type_rhs_after_eq:597` at `:620`; generated counterpart at
  `src/v3/compiler/src/parse_generated.rs:678`). Alias-form
  refinements are advertised in the story doc as `[target]`;
  closing this lane moves them toward `[live]`.
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

- **Day 1.** T-Sub `sub_match_over_user_sum` dispatches. `[Day 1]`
  gate — compiles against today's DB-15 schema.
- **Day 1.** T-Emit Rust-harden dispatches. Rust is the primary
  target; Python and Go reconcile against it. No cross-manager
  blocker.
- **Parallel.** T-Sub `sub_type_alias_where_lowers` dispatches.
  Requires DB-11 alias-RHS parse path closure
  (`src/v3/compiler/parse_parser_body.txt:651` `skip_where_clause`
  is the drop; `parse_type_rhs_after_eq:597` at `:620` is the call
  site). Once this lane lands, the story doc's alias-refinement
  claims tighten from `[target]` toward `[live]` — that's a
  visible integration win (PR-level, not scope-amendment-level).
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
  closure unblocks `match` lowering in the compiler's own sum-type
  code paths. Sideways to Self-hosting: "T-Sub's first gate is
  landing — coordinate which self-hosting surfaces consume it
  first."
- **Sideways to Self-hosting Manager.** `sub_type_alias_where_lowers`
  closure unblocks refinement patterns used by `std/` types the
  compiler consumes. Notify when the lane lands so Self-hosting
  can plan consumer updates.
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
- [x] `sub_match_over_user_sum` gate compiles + passes (Day-1)
      (PR #702, merged 2026-04-24 — `TestClaim` + suite in
      `src/v3/compiler/tests/fixtures/r1_gates.dag` run through
      `TestRunner`; #690 prior receipt confirmed structural match)
- [ ] `sub_type_alias_where_lowers` gate compiles + passes
      (DB-11 parse + lower substrate landed in PR #703, 2026-04-24 —
      `SurfaceItem::TypeAlias` carries `refinement`; `.dag` gate still
      to be authored once [ext] predicate path is named)
- [ ] `sub_charclass_in_std_unicode` gate compiles + passes
      (phase-1 tokenizer half landed in PR #693 + ROADMAP row #706,
      2026-04-24 — `CharClass` + `char_in_class` in `std.unicode`,
      tokenizer calls `tokenize_char_class::byte_matches`; follow-ups
      open: `syntax.dag` / `std.syntax` consumer wiring and structural
      `CharClass` consumption from lowered `tokenize.dag` — blocked by
      M1(2.8) class-5 gap #3)

**T-Emit:**
- [x] Rust harden — `emit_rust_fixtures_rustc_green` gate test landed
      (PR #694, merged 2026-04-24 — `#[ignore]`d named gate sweeps
      9 program fixtures + 5 reflected-module fixtures through the
      batched rustc roundtrip; baseline `rustc_roundtrip_*` tests all
      green)
- [ ] PR #650 generic-bound fidelity — `emit_generic_bounds_survive`
      passes (PR #676 in review — session `vivid-cat-794`)
- [ ] Python/Go reconcile — `emit_omni_demo_fixtures_green` passes
      across all three targets (cross-target progress 2026-04-24:
      `Behavior::Loop` emission for Python + Go in #692; Python
      operator-realization parity for `*` / `!=` / int comparisons
      in #691; omni gate still pending)

Decisions log (append as they happen):

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

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- _(none yet — coordinate with Self-hosting when T-Sub first gate
  lands; coordinate with Testgen when T-Emit `emit_omni_demo_fixtures_green`
  predicate shape needs defining)_
