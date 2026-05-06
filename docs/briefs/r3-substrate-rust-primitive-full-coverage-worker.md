---
status: dispatchable per Director Path A RATIFICATION at gunbc#1739 #issuecomment-4392731264 (2026-05-06)
authority parent: R3 Substrate Manager (#1739)
ratification: Path A RATIFIED at gunbc#1739 #issuecomment-4392731264 (zesty-bear-812, 2026-05-06). Bundled substrate prerequisite for Grounding G2 Phase 2 full-coverage (u128 / isize / usize). Three-axis structurally-coupled slice authored as one bundled brief per "necessary structural fix" exception (bundled-scope discipline at gunbc#1739 #issuecomment-4392225548).
roadmap row: §1.8 ledger row TBD slot — gate `rust_primitive_full_coverage` (Director-ratified name); cluster TBD (T-Numeric-Construction-adjacent OR new T-Interval-Representation cluster — Mgr-discretion at PR-authoring time)
authority docs:
  - gunbc#1739 #issuecomment-4392731264 (Director Path A RATIFICATION; named gate; named scope)
  - gunbc#1907 #issuecomment-4392585090 (loyal-stag-699 STOP findings — substrate gaps surfaced)
  - gunbc#1745 #issuecomment-4392594242 (Grounding STOP receipt; 3 substrate gaps enumerated)
  - dsl/extdeps/languages/rust/primitives.dag (current Rust integer primitive rows; 9-row ratchet)
  - src/v3/compiler/src/int_literal_ranges.rs (current i128 host repr; explicit u128 deferral)
  - src/v3/spec/rust.dag (current Rust target spec; missing isize/usize PlatformDependent rows)
gates:
  - `rust_primitive_full_coverage` (Director-ratified name; closure gate for full Rust integer primitive coverage)
worker pin: valiant-ibex-312 (freed-pool; substrate-authoring discipline fresh; numeric-domain-adjacent post-#1842 merge)
---

# R3 Substrate — Rust integer primitive full coverage (bundled substrate-fact-introduction)

## Context

Per Director Path A RATIFICATION at gunbc#1739 #issuecomment-4392731264:
Grounding G2 Phase 1 lands narrowed scope (i8-i64 + u8-u64) on existing
substrate; full-coverage (u128 / isize / usize) gates on three structurally-
coupled substrate gaps that this brief lands as **one bundled slice**.

### Substrate gaps (per Grounding loyal-stag-699 STOP at gunbc#1907)

1. **`IntervalInt::ExactInterval` host-repr gap**: `src/v3/compiler/src/int_literal_ranges.rs` parses all integer ranges as `i128` host representation; `u128::MAX` exceeds `i128` range. `dsl/extdeps/languages/rust/primitives.dag` explicitly defers `u128` because of this. Substrate widening required (BigInt-based or alternative encoding).

2. **Bound-carrier wiring gap on actual row surface**: the live row surface used by `src/v3/spec/rust.dag` is `TargetIntegerTypeInhabitance` (with bound-field type `TargetIntegerInhabitanceBound`) imported from `v3.std.emit_model` — NOT a generic `RustPrimitive` carrier. Per codex BLOCKING (PR #1910) verified at HEAD: live shape is `TargetIntegerInhabitanceBound = BoundUnspecified | StaticBoundFact(IntInterval)`. No PlatformDependent variant exists. `BoundDeclaration` is live in `src/v3/std/substrate.dag` (`StaticBound(Interval<Int>) | PlatformDependent`) but NOT wired into `TargetIntegerInhabitanceBound`. Carrier refactor is **required** (not optional) to unblock isize/usize rows.

3. **`spec/rust.dag` PlatformDependentFact row population**: `src/v3/spec/rust.dag` has no Rust PlatformDependentFact row entries for isize / usize. Co-located with (2) — same brief / PR per "necessary structural fix" exception.

### Why bundled

All three gaps address the same axis: "support wider/platform-dependent integer primitives." (2) consumes (1)'s widened interval representation; (3) consumes (2)'s structural BoundDeclaration field. Splitting across separate PRs would create cross-PR dependency chains and partial-substrate intermediate states. Bundled-scope discipline allows same-PR per "necessary structural fix" exception.

## Scope

### Deliverable 1 — `IntervalInt::ExactInterval` host-repr widening

Refactor `src/v3/compiler/src/int_literal_ranges.rs` to support integer ranges beyond `i128` representable scope. Two shape options worker DFS-decides at dispatch:

- **Option α (BigInt-based host repr)**: switch `ExactInterval` to BigInt (`num::BigInt` or workspace-equivalent). All existing 9 ratcheted rows continue parsing identically. Pro: simplest; covers all integer widths uniformly. Con: BigInt arithmetic vs native i128 — minor compile-time perf cost; minor dependency surface
- **Option β (typed-by-row variants)**: `enum ExactInterval { I128(i128, i128), U128(u128, u128), ... }` with per-row dispatch. Pro: native arithmetic preserved; explicit width-typing. Con: variant explosion; more complex pattern-matching at consumers

**Mgr recommendation: α (BigInt)** — simpler; uniform; no variant explosion; perf cost is brief-time-only (parsing literal ranges; not hot-path runtime). Worker confirms via DFS at dispatch.

Update consumers of `ExactInterval` at HEAD (worker greps for usage sites; refactor signatures; preserve semantic equivalence on the existing 9 i8-i64+u8-u64 rows via bootstrap snapshot + parse corpus manifest verification).

### Deliverable 2 — `TargetIntegerInhabitanceBound` carrier alignment

Refactor target: `v3.std.emit_model` carrier `TargetIntegerInhabitanceBound` (the actual bound-field type used by `TargetIntegerTypeInhabitance` rows in `src/v3/spec/rust.dag` lines 169/180/191/199/207).

Per codex BLOCKING at PR #1910 sha 98507c432 inline finding (verified at HEAD): live shape is `TargetIntegerInhabitanceBound = BoundUnspecified | StaticBoundFact(IntInterval)`. No PlatformDependent variant exists. Earlier Option (ii) (populate via existing variants) is therefore INFEASIBLE — only carrier refactor unblocks isize/usize rows.

**Required path — wire BoundDeclaration into TargetIntegerInhabitanceBound**:

Refactor `TargetIntegerInhabitanceBound` in `src/v3/std/emit_model.dag` to gain PlatformDependent semantics. Two structural shapes worker DFS-decides:

- **(i.a) — embed BoundDeclaration**: replace `TargetIntegerInhabitanceBound` body with `BoundDeclaration` (or `BoundDeclaration` newtype wrapper). Single source of truth for bound facts; existing variants `BoundUnspecified` / `StaticBoundFact(IntInterval)` map to `BoundDeclaration` variants (BoundUnspecified ≈ Phase 1 placeholder; StaticBoundFact ≈ StaticBound). Migration: 5 existing rows + new isize/usize/u128 rows populate via `BoundDeclaration` variants.
- **(i.b) — extend variant set**: keep TargetIntegerInhabitanceBound name but add PlatformDependent variant (3 variants: BoundUnspecified | StaticBoundFact(IntInterval) | PlatformDependent). Less integration with substrate.dag's BoundDeclaration carrier; worker confirms whether (i.a) is structurally preferred (single-authority) at dispatch.

**Mgr recommendation: (i.a) embed BoundDeclaration** — single-authority discipline + consumes existing substrate (P2 boundary discipline). Worker confirms via DFS at dispatch.

Existing 5 rows (rust_integer_inhabit_u32 / i32 / int64 / i128 / i32_at_program_bound) migrate to the chosen shape; bootstrap snapshot + parse corpus manifest verification on existing-row semantic equivalence is non-negotiable.

Both options consume Deliverable 1's widened `Interval<Int>` representation for the u128 row.

`dsl/extdeps/languages/rust/primitives.dag` is a separate file with its own `RustPrimitive` shape — worker confirms whether that file's rows are also a target for refactor at dispatch (per loyal-stag-699's STOP that surfaced both files as gap sites). If primitives.dag also needs structural-bound-field migration, scope expands to both files; bundled-scope check applies (likely allowed as "necessary structural fix" — same axis).

### Deliverable 3 — `src/v3/spec/rust.dag` `TargetIntegerTypeInhabitance` rows for isize / usize

Add `TargetIntegerTypeInhabitance` rows in `src/v3/spec/rust.dag` for `isize` / `usize` using `TargetIntegerInhabitanceBound` PlatformDependent-equivalent variant (post-Deliverable-2 carrier alignment if Option (i)). Per S7 Phase 1 brief framing — `PlatformDependent` is the substrate-side variant for compile-time-unknown bound facts.

Worker DFS-catalogs existing row population convention at `src/v3/spec/rust.dag` lines 169-207 (5 existing rows: u32 / i32 / int64 / i128 / i32_at_program_bound) and aligns isize/usize rows to that convention.

### Deliverable 4 — u128 row addition

Add `u128` row to `dsl/extdeps/languages/rust/primitives.dag` consuming Deliverable 1's widened `Interval<Int>` (now BigInt-backed). Row shape: `RustPrimitive { name: "u128", bound: StaticBound(Interval<Int> { min: BigInt::from(0), max: BigInt::from(u128::MAX) }) }` (or canonical project-equivalent shape; worker confirms at dispatch).

`int_literal_ranges.rs` ratchet update: 9 rows → 12 rows (+ u128, isize, usize).

### Deliverable 5 — Practice 4 checkpoint

Per `docs/modeling-discipline.md#4-coproduct-dissolution`:
- `BoundDeclaration` is existing 🟢 GREEN substrate (no new variants added; Deliverable 2/3 just consume existing variants on RustPrimitive rows)
- `ExactInterval` widening (α BigInt or β typed-variants) — α is type-substitution (no Practice 4 implication); β adds variants and needs 🟢/🟡/🔴 classification + checkpoint comment
- Worker authors checkpoint comment if β chosen; α path doesn't require new checkpoint

### Deliverable 6 — §1.8 ledger row receipt

Add `rust_primitive_full_coverage` to §1.8 ledger; advance DECLARED → CONSUMER_LANDED on merge (consumer = Grounding G2 Phase 2 full coverage; Grounding-side wiring may be same-PR co-authored OR follow-on per Mgr-Grounding discretion at dispatch). Cluster: T-Numeric-Construction-adjacent OR new T-Interval-Representation cluster — Mgr-discretion at PR-authoring time per Director ratification.

## Slice — single PR

Phase ordering (PR-internal):
1. DFS-catalog `ExactInterval` consumers + `RustPrimitive` row shape + `rust.dag` row population convention at HEAD
2. Choose α vs β for ExactInterval widening; author Practice 4 checkpoint if β
3. Author Deliverable 1 (ExactInterval widening) + verify existing 9 rows hold via bootstrap snapshot
4. Author Deliverable 2 (RustPrimitive bound field refactor) + migrate existing 9 rows; bootstrap snapshot + manifest verification
5. Author Deliverable 3 (rust.dag PlatformDependent rows for isize/usize)
6. Author Deliverable 4 (u128 row) + ratchet update 9 → 12 rows
7. §1.8 ledger row receipt (Deliverable 6)
8. Bootstrap snapshot regen + parse corpus manifest refresh
9. Cross-program handoff receipt to Grounding Mgr (#1745) for G2 Phase 2 full-coverage dispatch

## Acceptance

- `IntervalInt::ExactInterval` widened (α BigInt-based recommended; β typed-variants if α structurally infeasible) with existing 9 i8-i64+u8-u64 rows holding semantic equivalence
- `RustPrimitive` rows carry structural `BoundDeclaration` field; existing 9 rows migrated from static-string to `StaticBound(Interval<Int>)` form
- `dsl/extdeps/languages/rust/primitives.dag` has 12 rows (added u128, isize, usize); ratchet at `int_literal_ranges.rs` updated
- `src/v3/spec/rust.dag` has PlatformDependent rows for isize / usize
- Practice 4 checkpoint comment if β chosen for ExactInterval; α path no checkpoint required
- §1.8 row `rust_primitive_full_coverage` advances DECLARED → CONSUMER_LANDED upon merge (cluster: Mgr-discretion T-Numeric-Construction-adjacent OR new T-Interval-Representation)
- Cross-program handoff receipt to Grounding Mgr (#1745) in PR body — G2 Phase 2 full-coverage dispatch unblocked
- `cargo test --workspace --exclude v2-compiler-tests` green (3 pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body
- P1 substrate-fact-introduction receipt (DFS-of-concept-DAG; named consumer demand = Grounding G2 Phase 2; carrier-shape rationale per α/β decision)

## STOP-AND-ESCALATE

- **α BigInt dependency adds workspace surface beyond minor**: surface to Substrate Mgr; β typed-variants may be the right shape if dependency cost is high
- **`BoundDeclaration` variant set insufficient** (e.g., isize/usize needs a third variant beyond `StaticBound`/`PlatformDependent`): STOP — substrate-fact-introduction cascade; surface to Substrate Mgr with proposed third variant + named consumer demand + DFS-of-concept-DAG receipt
- **Bootstrap snapshot drift** during ExactInterval widening or RustPrimitive bound field refactor: root-cause; do NOT bridge with placeholder; semantic equivalence on existing 9 rows is non-negotiable
- **u128 / isize / usize add surfaces additional substrate gaps** (e.g., parser doesn't recognize `u128` literal suffix in `.dag` source): STOP — surface scope expansion; bundled-scope check (necessary-structural-fix vs parallel-infrastructure)
- **Bundled-scope drift on consumer side**: do NOT bundle Grounding G2 Phase 2 lowering rules / emit verification into this PR. Per Director bundled-scope ratification — parallel infrastructure DISALLOWED. Grounding G2 Phase 2 PR is downstream consumer

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `BoundDeclaration` carrier landed (`src/v3/std/substrate.dag`) ✓
   - `RustPrimitive` rows + 9-row ratchet in `dsl/extdeps/languages/rust/primitives.dag` ✓ (current static-string form)
   - `IntervalInt::ExactInterval` in `int_literal_ranges.rs` with i128 host repr ✓ (current narrow form; widening target)
   - `spec/rust.dag` row population convention exists ✓ (current; missing isize/usize entries)
   - This brief is producer of widened ExactInterval + structural-bound RustPrimitive + isize/usize/u128 rows
2. **Existing brief?** No prior brief on this axis at HEAD. T-E-P-Producer-Broadening (`r3-t-e-p-producer-broadening-worker.md`) and S7 PR-F (`r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`) are adjacent precedents — neither covers this scope
3. **Design-doc match?** Director Path A RATIFIED scope at gunbc#1739 #issuecomment-4392731264 names all three gaps + bundling decision verbatim
4. **Citations live?** Worker re-verifies at dispatch; substrate-state-grep confirms Path A finding shape unchanged
5. **Carrier dissolves the bridge?** Yes — widened `ExactInterval` + structural `BoundDeclaration` field + populated isize/usize rows together dissolve the "u128 / isize / usize cannot be represented faithfully in current substrate" bridge. Ad-hoc string encoding (the bridge anti-pattern Director rejected) avoided structurally

## Provenance

Drafted 2026-05-06 by Substrate Mgr (quick-crab-830) per Director Path A RATIFICATION at gunbc#1739 #issuecomment-4392731264. Brief queues bundled substrate-prerequisite for Grounding G2 Phase 2 full-coverage; authored same window as Path A ratification per pre-authored-brief-queue discipline.

Cross-references:
- S7 PR-F brief (`r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`) — BoundDeclaration consumer; this brief is the Rust-target row-population side
- Grounding G2 Phase 1 narrowed dispatch (loyal-stag-699; bold-ferret-748 #1745) — runs in parallel on existing 8-primitive substrate; this brief unblocks Phase 2
- Grounding loyal-stag-699 STOP at gunbc#1907 — surfaced the three gaps; this brief is the structural resolution
