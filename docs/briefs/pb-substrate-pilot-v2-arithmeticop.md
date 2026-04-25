# PB-Substrate Pilot v2 — `ArithmeticOp` round-trip via existing generator `(genuinely-S)`

> **Replacement worker brief** for the withdrawn PB-Substrate pilot ([rolled
> back in #774](https://github.com/gunb-ai/gunbc/pull/774); original brief
> at #772 had an invalidated premise).
>
> **Pre-promotion Deliverable 4 (a)-supplement** for
> [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
> (PROPOSAL). Director-approved per
> [#766 escalation thread](https://github.com/gunb-ai/gunbc/pull/766) +
> [#775 second message on (a)](https://github.com/gunb-ai/gunbc/pull/775).
> Reports through Zero-Floor Program Manager (`stern-swift-335`).

## Read first (the lessons from v1)

The withdrawn v1 brief targeted `CardinalityBound` round-trip on the
premise that `dag.rs` was wholly hand-authored and substrate generation
was a future pattern needing proof. **Both premises were wrong.** Codex
BLOCKING + an independent worker STOP-AND-ESCALATE on `quick-tern-80`
established:

- `dag.rs:497` includes `dag_scalar_generated.rs` (already generated).
- `CardinalityBound`, `LiteralBits`, `PortState`, `TemplateArgument`,
  `TypeShape`, plus `BranchPattern` / `Cluster` / `IntraClusterCall` /
  `LoopBound` / `MemberDescent` / `PayloadBinding` are **already
  generated** from `substrate.dag` via `scripts/regen_runtime_mirrors.py`.
- The generation pattern is shipping; this pilot **extends** it, not
  builds it.

See [`docs/design-pure-bootstrap-zero-audit.md`](../design-pure-bootstrap-zero-audit.md)
§"Substrate generation is already proven and shipping" for the
load-bearing reframe + 38-type substrate.dag coverage survey.

## Read first (the artifacts you'll touch)

- [`scripts/regen_runtime_mirrors.py`](../../scripts/regen_runtime_mirrors.py) — the existing Python generator. Specifically `render_dag_scalar_module` around `:760` reading `sums[...]` from `substrate.dag` and rendering `dag_scalar_generated.rs`.
- [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) `:134-164` — `ArithmeticOp` / `ComparisonOp` / `LogicalOp` / `OperatorKind` declarations (the source-of-truth).
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) `:694-725` — current hand-authored Rust counterparts. Pilot retires these on cementing.
- [`src/v3/compiler/build.rs`](../../src/v3/compiler/build.rs) `:408-440` — `REGEN_OUTPUTS` array. New generated file (or extension to existing) lands here.
- [`src/v3/compiler/src/dag_scalar_generated.rs`](../../src/v3/compiler/src/dag_scalar_generated.rs) — example of existing generated output for the scalar module.
- [`src/v3/compiler/src/operators_generated.rs`](../../src/v3/compiler/src/operators_generated.rs) — note: this is generated from `operators.dag` via `emit_rust_module` (different source, different generator path). It **consumes** `ArithmeticOp` / `OperatorKind` (e.g., `OperatorKind::Arithmetic(ArithmeticOp::Add)` at `:6`) but does **not** declare them. Pilot scope is the type declarations, not the dispatch tables.

## Frame

The cascade promotion PR cites the existing 23-file generation fleet as
primary evidence (per [#775](https://github.com/gunb-ai/gunbc/pull/775)
characterization). This pilot adds **incremental** evidence: a freshly
ungenerated TERMINAL substrate type migrated to the existing pattern,
in a single small PR, demonstrating the pattern extends cleanly.

Pattern-extend, not pattern-build. **Genuinely-S.**

## Slice — `ArithmeticOp` (and its three siblings)

`ArithmeticOp` is the smallest slice: 4 variants (`Add`/`Sub`/`Mul`/`Div`),
no payloads, declared in `substrate.dag:134`, hand-authored in `dag.rs:694`.

**Strongly suggested: bundle `ArithmeticOp` + `ComparisonOp` + `LogicalOp` +
`OperatorKind` into one PR.** Reasoning:
- All four declared together in `substrate.dag:134-164`, all
  hand-authored together in `dag.rs:694-725`.
- `OperatorKind = Arithmetic(ArithmeticOp) | Comparison(ComparisonOp) |
  Logical(LogicalOp)` references the other three; partial migration
  forces a hybrid `OperatorKind` straddling generated + hand-authored
  variants — worse churn than landing all four together.
- Total surface is ~32 lines of hand-authored Rust; emission is ~32 lines
  of generated Rust.

If bundling proves to surface an unexpected blocker, fall back to
`ArithmeticOp` only and leave `ComparisonOp` / `LogicalOp` / `OperatorKind`
for follow-up. Manager-call at execution time; surface the choice in
the PR description.

> **Note on `OperatorKind`'s `🟡 SCAFFOLD` annotation.** `substrate.dag:158-160`
> marks `OperatorKind` as `🟡 SCAFFOLD. OperatorKind is the current structural
> carrier...`. The annotation indicates the type is expected to dissolve in
> a future modeling pass, not that it's hand-authored-pending-generation.
> Migrating the **declaration** to generation is orthogonal to the eventual
> dissolution — the migrated declaration retires when the carrier dissolves,
> same as any other generated type. Don't let the annotation block the slice.

## Round-trip mechanics

1. **Read**: `scripts/regen_runtime_mirrors.py` already parses
   `substrate.dag` into `sums[...]` / `records[...]` dicts. Verify
   `sums["ArithmeticOp"]` etc. are populated before extending the
   renderer.
2. **Render**: extend `render_dag_scalar_module` (or author a sibling
   `render_dag_operators_module` — worker's call based on which fits
   the existing structure cleaner) with `render_sum` calls for the four
   sums. Pattern (from `:759`):
   ```python
   render_sum(
       "ArithmeticOp",
       sums["ArithmeticOp"],
       "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]",
       output_name="ArithmeticOp",
   ),
   ```
   Match the **derive set** from the hand-authored counterpart at
   `dag.rs:693` (`#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`)
   so consumers compile unchanged.
3. **Wire**: if a new module (e.g., `dag_operators_substrate_generated.rs`),
   register in `build.rs` `REGEN_OUTPUTS` (`:408-440`) and add an
   `include!()` in `dag.rs` adjacent to the existing four substrate-shape
   includes (`:497`, `:1678`, `:1699`, `:1710`). If extending
   `render_dag_scalar_module`, no new include needed.
4. **Retire**: delete `dag.rs:694-725` hand-authored declarations;
   their generated equivalents now satisfy consumers.
5. **Regenerate**: run the regen path (likely `cargo run --bin regen_v3`
   or whichever the existing pattern uses; verify against `build.rs`
   regen invocation).

## Cementing — SG-0 producer-owned-partition (automatic)

**No new test to author.** The existing SG-0 invariant is the cementing:

- The new generated file (or extension) is registered in
  `build.rs` `REGEN_OUTPUTS`.
- SG-0 census (`src/v3/compiler/tests/integration/sg0_census_test.rs`)
  enforces partition: a file in `REGEN_OUTPUTS` cannot also be a
  hand-authored entry in `EXPECTED_HAND_AUTHORED_NON_TEST`.
- Hand-edits to the generated file fail census on next run.
- Producer-ownership is structural, not byte-match.

This is the cementing pattern proven by all 23 existing generated
files. The pilot inherits it; no separate cementing test needed in this
PR.

## Type-mapping note

`substrate.dag` declares variant payloads in its own type vocabulary;
the existing generator already encodes mappings (e.g.,
`CardinalityBound::Exact(Int)` → `Exact(u32)` per
`render_dag_scalar_module` `:763` `overrides={"Int": "u32"}`).

`ArithmeticOp` / `ComparisonOp` / `LogicalOp` are unit variants (no
payloads), so no mapping decision is required for the pilot. If
`OperatorKind`'s payload variants need an override (each carries an
inner sum: `Arithmetic(ArithmeticOp)` etc.), inherit whatever the
existing generator does for sum-payload sums; do **not** invent a new
mapping pattern in the pilot.

If a mapping question genuinely surfaces (variant payload doesn't
match an existing override pattern), STOP-AND-ESCALATE rather than
absorb — Director wants type-mapping consistency surfaced for cascade
PR notes.

## Acceptance

- [ ] `sums["ArithmeticOp"]` / `sums["ComparisonOp"]` / `sums["LogicalOp"]` / `sums["OperatorKind"]` populated by existing parser; verified in worker first-action grep.
- [ ] `regen_runtime_mirrors.py` extended (existing module or new sibling); runs without manual invocation per existing build hooks.
- [ ] Generated declarations match hand-authored derive set + variant order.
- [ ] If new module: registered in `REGEN_OUTPUTS`; `include!()` added in `dag.rs`.
- [ ] Hand-authored `ArithmeticOp` / `ComparisonOp` / `LogicalOp` / `OperatorKind` deleted from `dag.rs:694-725` (or partial fallback per "manager-call at execution time" above).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean (pre-push hook will enforce).
- [ ] DB-8 `self_host_fixed_point` converges bit-identically.
- [ ] **SG-0 census deltas correct**: the four types' source no longer in `dag.rs` hand-authored partition; new generated file (if any) in `REGEN_OUTPUTS` partition. SG-0 cementing is producer-owned-partition; the partition shift is the proof.

Note: `dag.rs` itself stays in `EXPECTED_HAND_AUTHORED_NON_TEST` (the file isn't retired by this pilot — only its operator-declaration content slice is). No SG-0 census array edits required.

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager (`stern-swift-335`).

- **If `sums["ArithmeticOp"]` etc. are missing from the existing parser's substrate.dag dict** — STOP. Parser support for these declarations may need extending; that's a separate prerequisite, not pilot scope.
- **If migrating `OperatorKind` requires a new variant-payload type-mapping pattern** not already in the generator — STOP. Director wants type-mapping consistency surfaced for cascade PR notes.
- **If consumers of `ArithmeticOp` / `OperatorKind` (e.g., `operators_generated.rs:6`, `dag_branch_generated.rs`, etc.) break on the migrated types** — STOP. Indicates the derive set or visibility contract drifted; surface the specifics.
- **If DB-8 fixed-point drifts** — STOP immediately.
- **If the bundle (4 types) surfaces an unexpected blocker** — fall back to `ArithmeticOp` only; surface in PR description; manager-call whether to follow up the others in this PR or split.
- **If pilot scope balloons beyond the four operator types** — STOP. Pilot is genuinely-S; preserve it.

## Non-goals

- **Not extending the pattern beyond the four operator types.** Other 23 uncovered substrate.dag types (per #775 survey) are PB-Substrate proper, post-cascade.
- **Not changing the cementing pattern.** SG-0 producer-owned-partition is the canonical pattern; pilot inherits.
- **Not retiring `dag.rs` itself** — only its operator-declaration slice. File stays in census.
- **Not amending `substrate.dag`** — declarations there are already correct.
- **Not touching `operators_generated.rs`** — different generator (`emit_rust_module` from `operators.dag`); orthogonal.
- **Not picking the N=0 resolution shape** — orthogonal; design doc Open call.

## Reporting

- Single PR for the pilot. Title pattern: `feat(v3): PB-Substrate pilot v2 — ArithmeticOp/ComparisonOp/LogicalOp/OperatorKind via existing regen pattern (Pre-promotion Deliverable 4(a) for #762)`.
- PR description cites this brief + the type-mapping check + which slice variant landed (full bundle vs. ArithmeticOp-only fallback).
- On merge: Zero-Floor Manager signals Director that Deliverable 4(a) is closed and cascade-eligible.
- On STOP-AND-ESCALATE: surface to Zero-Floor Manager; manager decides next move.

## Cross-manager note

Grounding Manager (`crisp-seal-366`) was heads-up'd at v1 pilot start
on [#768](https://github.com/gunb-ai/gunbc/pull/768). v2 scope is even
more additive than v1 (extending an existing Python script + retiring
~32 lines from `dag.rs` operator-declaration slice). No further
cross-manager signal required for this slice unless the four types
surface unexpected substrate-shape concerns mid-execution.
