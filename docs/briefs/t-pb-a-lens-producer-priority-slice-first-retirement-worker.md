# T-PB-A — Lens-producer priority slice: first landable retirement `(S)`

> **Worker brief.** Pure Bootstrap Worker 1 (T-PB-A lens-producer
> priority slice). Authored 2026-04-26 against `sg0_census_test.rs`
> at HEAD `f462b0df8be24fbca420deca381cbac10c49ae53` (census counts
> verified by static inspection; **re-run** `cargo test -p v3-compiler
> sg0_v3_` before merge — the manager preflight requires live census
> confirmation).
>
> **Authority:** `ROADMAP.md` T-PB-A row (lens-producer priority within
> SG-0 non-test census), `docs/design-pure-bootstrap-zero.md` §New
> lanes `PB-Runtime`, `docs/briefs/pure-bootstrap-zero-manager.md`
> PB-Tier1-Sweep checklist (includes `lens_unused_parameters.rs`).
>
> **Does not supersede:** [`docs/briefs/pure-bootstrap-zero-manager.md`](pure-bootstrap-zero-manager.md)
> PB-Runtime program checklist (`lens_apply.rs` / `lens_testgen.rs` /
> `post_emit_verifier.rs` full migrations). This brief scopes the
> **first narrow census retirement** in the lens-producer queue, not
> the entire PB-Runtime lane.

## Read first

- [`docs/briefs/brief-authoring-checklist.md`](brief-authoring-checklist.md) — five-question audit before implementation PR; receipt belongs in that PR body.
- [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) — §New lanes `PB-Runtime` (lens evaluator as data + tiny interpreter end state).
- [`ROADMAP.md`](../ROADMAP.md) T-PB-A row — `per_call_descent_evidence` anchor; `lens_producer_files_remaining` gate notes enumeration will live in `sg0_census_test.rs` when T-TestGen scopes the predicate ([`docs/briefs/r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md)).
- [`src/v3/compiler/tests/integration/sg0_census_test.rs`](../../src/v3/compiler/tests/integration/sg0_census_test.rs) — `EXPECTED_HAND_AUTHORED_NON_TEST`, `EXPECTED_HAND_AUTHORED_FRAGMENTS`.
- [`src/v3/compiler/src/lib.rs`](../../src/v3/compiler/src/lib.rs) — inline `lens_cost` / `lens_cost_symbolic` / `lens_provenance` pattern (generated `include!` + thin public surface).
- [`src/v3/compiler/src/lens_unused_parameters.rs`](../../src/v3/compiler/src/lens_unused_parameters.rs) — current standalone module + `#[cfg(test)]` unit tests.
- [`src/v3/lenses/unused_parameters.dag`](../../src/v3/lenses/unused_parameters.dag) — lens authority (unchanged by this slice).
- [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md) — cost/complexity proxy status + E-P side table narrative.

## Frame

T-PB-A’s **lens-producer priority slice** is the subset of SG-0
non-test hand-Rust that either (a) implements a bounded lens
interpreter / lens testgen, (b) is a **thin host shell** around a
regen-emitted lens body, or (c) is the **E-family / descent-evidence
producer family** called out in ROADMAP (`per_call_descent_evidence`).

Retiring those files dissolves “lens purity by reviewer convention”
into “lens purity by construction” as logic moves behind `.dag` +
producer-owned Rust projections.

This brief’s **first landable slice** is intentionally smaller than
full `PB-Runtime` migration: **collapse the
`lens_unused_parameters.rs` standalone file** into the same **inline
`pub mod` + `include!(…_generated.rs)`** pattern already used for
`lens_cost` in `lib.rs`, then **remove** `src/v3/compiler/src/lens_unused_parameters.rs`
from the tree and from `EXPECTED_HAND_AUTHORED_NON_TEST`.

Rationale: the generated query already lives in
`lens_unused_parameters_generated.rs`; the hand file is mostly a
type wrapper, a `Lens` struct, and **module-local unit tests**. No new
substrate carrier; no change to the `.dag` lens semantics.

## Inventory — `EXPECTED_HAND_AUTHORED_NON_TEST` (36) + `EXPECTED_HAND_AUTHORED_FRAGMENTS` (1)

Verified list: `sg0_census_test.rs` (`f462b0df`).

### Fragment (not lens)

| Path | Note |
|------|------|
| `src/v3/compiler/parse_parser_body.txt` | Parser scaffold for `regen_parse`; SG-2c dissolution track. |

### Lens producers / lens-adjacent (priority-slice candidates)

| Path | Role |
|------|------|
| `src/v3/compiler/src/lens_apply.rs` | T-LensAPI D1 bounded interpreter over `FieldValue` (`ROADMAP` / debt synthesis: large surface). PB-Runtime eventual owner. |
| `src/v3/compiler/src/lens_testgen.rs` | TestClaim generation + cost integration; PB-Runtime eventual owner. |
| `src/v3/compiler/src/lens_unused_parameters.rs` | Thin shell over `lens_unused_parameters_generated.rs` — **this brief’s Slice 1 target.** |
| `src/v3/compiler/src/bin/regen_lens.rs` | Unified regen driver over `regen.dag` `LensRegistryEntry` records; lens-adjacent **producer scaffold**. |
| `src/v3/compiler/src/dag.rs` | ROADMAP anchor: `per_call_descent_evidence` (search symbol in file — line numbers drift). E-P descent-evidence types + producer family. |
| `src/v3/compiler/src/dag/builder.rs` | Test-facing Dag builders; substrate-adjacent (not a `.dag` lens emitter). |
| `src/v3/compiler/src/dag/effects.rs` | Std effects mirror chunk; lenses read `Behavior`. |
| `src/v3/compiler/src/dag/ports.rs` | Ports mirror chunk; lens / projection consumers. |
| `src/v3/compiler/src/dimension.rs` | Symbolic-cost dimension analysis; calls generated `lens_cost_symbolic` (`ROADMAP` “cost … family around the E-family port”). |
| `src/v3/compiler/src/lib.rs` | Host registry; already hosts inline `lens_cost*` / `lens_provenance` / etc. |

### Compiler core (T-PB-A census, not lens-producer priority for `lens_producer_files_remaining` narrative)

Remaining `EXPECTED_HAND_AUTHORED_NON_TEST` entries: `build.rs`,
`regen_bootstrap.rs`, `regen_parse.rs`, `regen_parse_tables.rs`,
`regen_tokenize.rs`, `regen_v3.rs`, `self_host_fixed_point.rs`,
`bootstrap.rs`, `bootstrap_regen_fresh.rs`, `diagnostics.rs`,
`emit.rs`, `emit/python_target.rs`, `emit/rust_target.rs`,
`emit_rust.rs`, `infer.rs`, `int_literal_ranges.rs`, `lower.rs`,
`pipeline_authority.rs`, `post_emit_verifier.rs`,
`regen_bootstrap_emit.rs`, `regen_parse_emit.rs`,
`regen_parse_tables_emit.rs`, `test_runner.rs`,
`tokenize_char_class.rs`, `workflow_idempotency.rs`,
`workflow_parallelism.rs`.

(`post_emit_verifier.rs` is emit-contract harness, not a lens
producer.)

### Queue after Slice 1 (recommended order for follow-on briefs)

1. **PB-Runtime track (program-sized):** `lens_testgen.rs` then
   `lens_apply.rs` (per `pure-bootstrap-zero-manager.md` unchecked
   items) — depends on interpreter-as-data authority design.
2. **Tier-1 / regen:** `regen_lens.rs` — depends on PB-1 + emit
   patterns for generated bin shims (`pure-bootstrap-zero-manager.md`
   PB-Tier1-Sweep).
3. **E-family / dag.rs:** migrating `per_call_descent_evidence` off
   hand-Rust is **not** this slice — it is carrier-port / PB-Substrate
   territory; STOP if a worker discovers substrate gaps (see below).

## Slice — align `lens_unused_parameters` with `lens_cost` module shape

### Deliverables

1. **Inline module in `lib.rs`** — Add `pub mod lens_unused_parameters
   { … }` that:
   - wraps `include!("lens_unused_parameters_generated.rs")` inside a
     `mod generated { … }` with the same `#![allow(…)]` bundle used by
     sibling lens modules;
   - defines `UnusedParametersConfig`, `UnusedParameter`,
     `UnusedParametersLens::new` / `query` as **thin** forwarding to
     `generated::check` (preserve public API used by integration tests
     and embedders).
2. **Delete** `src/v3/compiler/src/lens_unused_parameters.rs`.
3. **Tests (default — no new hand-authored test files).** The current
   file embeds `#[cfg(test)]` unit tests. **Prefer:** keep them
   **colocated** inside the new inline `pub mod lens_unused_parameters
   { … }` in `lib.rs`, mirroring `pub mod lens_cost`’s existing
   `#[cfg(test)] mod tests { … }` pattern. Those tests compile with the
   library test target only; they do **not** add paths under
   `src/v3/compiler/tests/` and do **not** touch
   `EXPECTED_HAND_AUTHORED_TEST` — the T-PB-B ratchet must not grow as
   collateral to a T-PB-A census retirement (census increases are
   STOP per manager dispatch unless Director-approved).

   **Alternatives** (only if colocation is genuinely infeasible): fold
   the assertions into an **already-listed** integration test file
   (no new `EXPECTED_HAND_AUTHORED_TEST` line), or port coverage to
   runner-backed `.dag` `TestClaim`s if expressible without new
   hand-Rust files. **Do not** treat “add a new Rust integration test
   file + ratchet line” as the default path.
4. **SG-0** — Remove the deleted `.rs` path from
   `EXPECTED_HAND_AUTHORED_NON_TEST`; keep counts consistent with
   `sg0_v3_hand_authored_census`.
5. **Regen / build** — No change to `unused_parameters.dag` or
   `REGEN_OUTPUTS` for `lens_unused_parameters_generated.rs` unless a
   mechanical path update is required (should not be).

### Acceptance

- [ ] `lens_unused_parameters.rs` absent from repo; behavior unchanged
  for `UnusedParametersLens::query` callers.
- [ ] All preserved tests pass (colocated `#[cfg(test)]` or chosen
  alternative); no loss of unused-parameters coverage.
- [ ] `cargo test -p v3-compiler sg0_v3_` passes (full census + both
  sub-ratchets).
- [ ] `cargo test -p v3-compiler` (or workspace hand-test command per
  `CLAUDE.md`) passes for affected integration tests
  (`m1_3_lens_unused_parameters_test`, migration tests, etc.).
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all
  --check` clean.

## STOP-AND-ESCALATE

Surface to Pure Bootstrap manager / parent inbox **#884** when:

- **Census ratchet:** implementing this slice would **add** a new
  non-test `.rs` or fragment (forbidden without director sign-off), or
  live census at HEAD disagrees with the counts in this brief.
- **T-PB-B collateral:** this slice would add a new path to
  `EXPECTED_HAND_AUTHORED_TEST` solely to host moved tests — **STOP**
  (use colocated `#[cfg(test)]` in `lib.rs`, an already-counted test
  file, or `.dag` claims instead).
- **Duplicate authority:** another brief already lands the same
  file retirement — close as redundant; do not fight for scope.
- **Substrate / carrier beyond thin shell:** if investigation shows
  `UnusedParametersLens` cannot move without new `ValueBody` /
  `FieldValue` reflection or other Grounding work — STOP; re-scope to
  substrate lane.
- **ROADMAP vs HEAD mismatch** on `per_call_descent_evidence` or other
  cited examples — STOP; update ROADMAP or brief, not silent drift.
- **DB-8 fixed-point drift** on any compile/regen step — STOP
  immediately.

## Non-goals

- Not migrating `lens_apply.rs` or `lens_testgen.rs` (PB-Runtime
  program items).
- Not moving `per_call_descent_evidence` / `dag.rs` producer logic in
  this slice.
- Not changing `unused_parameters.dag` lens logic or generated query
  semantics.
- Not authoring the `lens_producer_files_remaining` testgen predicate
  enumeration (R1C-A owns schema + list constant).

## Reporting

- PR title pattern: `refactor(v3): T-PB-A — fold lens_unused_parameters into lib.rs; retire SG-0 census entry`
- PR body: paste **Authority audit receipt** below.

---

## Authority audit receipt (brief-authoring-checklist.md compliance)

Copy into the PR body that lands the implementation (not this
authoring-only change if the brief merges alone).

1. **Substrate exists?** Grep `src/v3/std/`, `src/v3/spec/`: no new
   carrier proposed. Slice is shell alignment only → **N/A / confirmed
   not a producer brief.**

2. **Existing brief?** Grep `docs/briefs/`: no other brief owns
   `lens_unused_parameters.rs` retirement. PB-Tier1-Sweep lists the
   file as unchecked in `pure-bootstrap-zero-manager.md` — this slice
   advances that checkbox without duplicating PB-Runtime lens_apply /
   lens_testgen authority.

3. **Design-doc recommendation matches?** `design-pure-bootstrap-zero.md`
   §PB-Runtime names eventual generation of `lens_apply.rs` /
   `lens_testgen.rs`; this slice is an **incremental census win** on
   the same program vector (thin host shells disappear). **Verdict:**
   aligned; does not claim PB-Runtime closure.

4. **Citations live?** Verified at `f462b0df8be24fbca420deca381cbac10c49ae53`:
   `per_call_descent_evidence` exists as `pub fn` in
   `src/v3/compiler/src/dag.rs` (use ripgrep symbol anchor, not stale
   line-only cites for ROADMAP examples).

5. **Carrier dissolves the bridge?** **N/A** — no new substrate
   carrier. The “bridge” retired is **parallel module layout** (one lens
   on a standalone `.rs` while peer lenses live inline in `lib.rs`).
