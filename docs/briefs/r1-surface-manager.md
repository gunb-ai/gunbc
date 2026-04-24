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

- **`T-P0`** (`ROADMAP.md:45`) — P0 sweep:
  - `repeat_string` — real bug in existing emission.
  - `REST_OPS` — drift between the declared REST ops surface and
    what's actually emitted.
  - `no_profile_sentinel` — fabrication sentinel audit.
  - Size **S**. Brief-level receipts live at `docs/briefs/p0-*.md`.
- **`T-Sub`** (`ROADMAP.md:46`) — three missing surface
  capabilities:
  - `match` over user sums (`sub_match_over_user_sum` — `[Day 1]`
    gate).
  - `CharClass` in `std.unicode` (`sub_charclass_in_std_unicode` —
    `[ext]` gate).
  - type-alias `where` (`sub_type_alias_where_lowers` — `[ext]`
    gate; alias-RHS parser skip per DB-11 at `ROADMAP.md:231`).
  - Size **S**.
- **`T-Emit`** (`ROADMAP.md:48`) — emission hardness:
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
- P0 bugs are real (each has a brief under `docs/briefs/p0-*.md`).
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
  gap per `ROADMAP.md:305`+ — the types exist in `dsl/std/`; the
  tokenizer and syntax authorities aren't using them yet.
- Cross-target emission green-ness is the external-toolchain
  receipt: generated Rust compiles under `rustc`; generated Python
  runs under CPython; generated Go builds under `go build`.

The ask: close each. Surface is where "we can generate code that
runs" is literally true, so it's the manager whose work is easiest
for a principal engineer to verify in one evening.

## Sequence + dispatch

- **Day 1.** T-P0 sweep dispatches fully — all three P0 items have
  per-bug briefs; lane-owners can go. Size S.
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
  (or superset) to `std.unicode` per `ROADMAP.md:305+`;
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
  `ROADMAP.md:305+`, flag to director so the cardinality-substrate
  ledger row can be amended rather than expanding the lane.

## Unscheduled gaps relevant to this slice

None as of R1 launch. All three T-Sub deliverables cite DB-11 or
`ROADMAP.md:305+`. T-Emit deliverables cite standing emit-honesty
predicates. T-P0 items each have their own brief.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-P0:**
- [ ] `repeat_string` fix landed (brief: `p0-render-repeat-string.md`)
- [ ] `REST_OPS` drift resolved (brief: `p0-rest-ops-drift.md`)
- [ ] `no_profile_sentinel` audit completed (brief: `p0-bug-no-profile-sentinel.md`)

**T-Sub:**
- [ ] `sub_match_over_user_sum` gate compiles + passes (Day-1)
- [ ] `sub_type_alias_where_lowers` gate compiles + passes
      (DB-11 alias-RHS path)
- [ ] `sub_charclass_in_std_unicode` gate compiles + passes

**T-Emit:**
- [ ] Rust harden — `emit_rust_fixtures_rustc_green` passes
- [ ] PR #650 generic-bound fidelity — `emit_generic_bounds_survive`
      passes
- [ ] Python/Go reconcile — `emit_omni_demo_fixtures_green` passes
      across all three targets

Decisions log (append as they happen):

- _(none yet)_

Open questions for director:

- _(none yet)_

Cross-manager notifications queued:

- _(none yet — coordinate with Self-hosting when T-Sub first gate
  lands; coordinate with Testgen when T-Emit `emit_omni_demo_fixtures_green`
  predicate shape needs defining)_
