# R3 PB — T-V2-Retirement Population A + B coverage spot-check audit (docs-only)

**Status:** AUDIT artifact (docs-only). Authored 2026-05-02 by PB Manager continuation per dispatches on inbox #1149 (Pop A 4363271189 + Pop B 4363833311/4363833641 — combined since both are parallel coverage spot-checks against the same authority chain).

**This is NOT S-1.** S-1 (PM-authored worker brief) remains absent on `origin/main` HEAD at audit time; this audit feeds the future S-1 and the G-2 worker. No code changes; no `src/v2/` edits; no Cargo-edge removal; no `kernel_algebra_profile` migration; no `verification.dag` convergence decision from PB.

## Live state verification

`origin/main` HEAD at audit-authoring time: same lineage as the S-1 input packet (PR #1462 merged). S-1 PM-authored worker brief still missing under `docs/briefs/` (only `pb-substrate-pilot-v2-arithmeticop.md`, `r3-pb-tv2-g1-readiness-receipt.md`, `r3-pb-tv2-s1-input-packet.md`, and this audit).

> **Line-anchor freshness pointer (added 2026-05-05; refreshed 2026-05-06).** Per-row `:NNN` line cites in §A.1–§A.4 below are author-time anchors against the audit's original HEAD. Latest drift offsets vs current `origin/main` HEAD `2c7d82031` are in §Delta (2026-05-06) at the bottom of this file (induction.dag decls drifted -3 to -5 lines, termination.dag decls drifted +103 to +106 lines, B.2 oracle structurally identical, Cargo edges unchanged). Earlier §Delta entries (530c76ea7 / B.2 reclassification) preserved for audit history. Section/symbol anchors are the load-bearing identity per `feedback_section_anchors_over_line_numbers`.

## Population A — internal `src/v2/tests/src/` named tests (4 spot-checked per dispatch)

**Audit framing:** these tests live INSIDE `src/v2/tests` and fall with G-2 (workspace-member removal of `src/v2/tests`) per audit §1 / matrix §2.1. They count against G-2, not G-1. The dispatch question is: when G-2 deletes the crate, are the *semantic properties* these tests guarantee already covered on the v3 side, or does G-2 need a coverage migration first?

> **Single-authority pointer (P2 — added 2026-05-05).** The per-row "Disposition recommendation for S-1" cells in §A.1–§A.4 below carry the **substrate-presence** finding accurate at audit-authoring time. **For Pop A's gate set and dispatch readiness, the single canonical authority is the "Post-#1715 reclassification" subsection that follows §A.4 / §A summary** (constructor/value evaluator dependency post-PR #1715 / `52dcd5529`). The per-row cells remain useful for substrate-shape context but must not be read as standalone dispatch readiness; that is the reclassification subsection's job. Same pointer applies to the Pop A row in the §"Net dispatch order" lower in this audit.

### A.1 `src/v2/tests/src/derive_bound_fail_closed_test.rs`

| Field | Value |
|---|---|
| Tested behavior | `derive_bound(param, branches, factor, work_exponent) -> CostBound`: P3 fail-closed; rejects non-positive branch counts (`0`, `-3`), invalid work exponents, plus `master_theorem` boundary cases. From v2 `std_induction`. |
| v3-side substrate analog | **LIVE** — `fn derive_bound(...)` declared at `src/v3/std/induction.dag:897`; `fn master_theorem(form: RecurrenceForm) -> CostBound` at `:823`. |
| v3-side test coverage | **MISSING** — call-site-capable grep `grep -rnE '\b(derive_bound\|master_theorem)\b' src/v3/compiler/tests/` (matches both Rust call sites like `derive_bound(...)` and `.dag` test invocations without an `fn` prefix) returns zero. Substrate is live; behavior tests are not. |
| Disposition recommendation for S-1 | Before G-2 deletes `src/v2/tests`: PB Manager dispatches a v3-side property-test worker that asserts the fail-closed semantics on `src/v3/std/induction.dag::derive_bound` + `master_theorem` (parallel `Int`-input cases: 0 branches → `ErrorBound`; negative branches → `ErrorBound`; etc.). Without that v3-side coverage, G-2 silently drops the fail-closed receipt for the `Int`-input cost-algebra boundary. |
| Counter-default cost | Skipping the migration would let `derive_bound`/`master_theorem` regress on those boundary inputs without a test ratchet. |

### A.2 `src/v2/tests/src/int_pow_bounded_test.rs`

| Field | Value |
|---|---|
| Tested behavior | `int_pow_bounded(base, exp) -> Int?`: negative exponent → `None`; non-negative matches `pow`; overflow at `2^63` → `None`. Plus degenerate-base cap (`0`, `1`, `-1`) doesn't deep-recurse. Also `ceil_log`. From v2 `std_induction`. |
| v3-side substrate analog | **LIVE** — `fn int_pow_bounded(base: Int, exp: Int) -> Int?` at `src/v3/std/induction.dag:767`; `fn ceil_log` at `:802`; `fn ceil_log_iter` at `:808`. |
| v3-side test coverage | **MISSING** — `grep -rnE '\b(int_pow_bounded\|ceil_log)\b' src/v3/compiler/tests/` (call-site-capable; matches Rust + `.dag` invocations) returns zero. |
| Disposition recommendation for S-1 | Same shape as A.1: v3-side property-test worker re-asserts negative-exp / overflow / degenerate-base / `ceil_log` semantics against `src/v3/std/induction.dag` directly. The semantic surface is identical (both versions take `Int` / `Int?`); the v3 fixture is a near-mechanical port of the v2 test body. |
| Counter-default cost | Skipping migration loses the Int-overflow / negative-exp boundary ratchet; v3 currently has no test coverage of these edge cases despite the substrate being live. |

### A.3 `src/v2/tests/src/peano_materialization_cap_test.rs`

| Field | Value |
|---|---|
| Tested behavior | M9 / P4: Peano literal bridges cap at 256 (oversize `Int` inputs → `none`, not deep-recurse / wrap). `positive_descent_amount_from_positive_int(k)` rejects > 256; `proportional_divisor_from_int_at_least_two(k)` rejects > 256; `master_theorem` work_exponent capped before `int_pow_bounded`. From v2 `std_induction` + `std_termination`. |
| v3-side substrate analog | **LIVE with the cap declared** — `fn peano_literal_materialization_cap() -> Int { 256 }` at `src/v3/std/termination.dag:140`; `fn positive_descent_amount_from_positive_int` at `:146`; `fn proportional_divisor_from_int_at_least_two` at `:162`. The cap value (256) is a single-source v3 declaration. |
| v3-side test coverage | **MISSING** — `grep -rnE '\b(peano_literal_materialization_cap\|positive_descent_amount_from_positive_int\|proportional_divisor_from_int_at_least_two)\b' src/v3/compiler/tests/` (call-site-capable) returns zero. |
| Disposition recommendation for S-1 | v3-side property-test worker asserts `positive_descent_amount_from_positive_int(257) == None`, `… (256) != None`, `… (1) != None`, plus parallel `proportional_divisor_from_int_at_least_two` cases. Substrate is live; just needs test wiring. **Bonus benefit**: a v3-side test would bind `peano_literal_materialization_cap()` as the cap source-of-truth (currently a magic 256 in v2 test bodies); v3 already has the named constant, so the v3 test cites `peano_literal_materialization_cap()` and the cap is grep-clean across the codebase. |
| Counter-default cost | Skipping migration loses fail-closed receipt for the 256-cap; the cap function itself stays live but un-tested at the boundary. |

### A.4 `src/v2/tests/src/sub_value_lattice_factor_test.rs`

| Field | Value |
|---|---|
| Tested behavior | P2 / single-authority: `meet_sub_value` and `join_sub_value` must not drop cost-relevant `ShrinkFactor` when field/param keys align. Tests strict + non-strict aligned-key cases on `SubValueRelation`. From v2 `std_induction`. |
| v3-side substrate analog | **LIVE** — `fn meet_sub_value` at `src/v3/std/induction.dag:281`; `fn join_sub_value` at `:329`. Same `SubValueRelation` / `RecursionShape` / `InductiveField` substrate carriers (also live in `src/v3/std/induction.dag`; ratchet at `m2_substrate_inhabitance_test.rs`). |
| v3-side test coverage | **MISSING** — `grep -rnE '\b(meet_sub_value\|join_sub_value)\b' src/v3/compiler/tests/` (call-site-capable) returns zero. |
| Disposition recommendation for S-1 | v3-side property-test worker ports the meet/join cases against `src/v3/std/induction.dag::{meet_sub_value, join_sub_value}` with v3 `SubValueRelation` / `InductiveField` / `RecursionShape` constructors. The `ShrinkFactor`-preservation invariant is identical between versions; the test body is a near-mechanical port. |
| Counter-default cost | Skipping migration loses the lattice-factor-preservation receipt; without it the meet/join behavior could silently regress without a test catching it. |

### Population A summary

All 4 named tests have **substrate live on v3 side** (every function has a parallel `src/v3/std/induction.dag` or `termination.dag` declaration), but **none have v3-side test coverage**. The migration shape was originally classified "mechanical port" on substrate-presence grounds; see the **Post-#1715 reclassification** below for the corrected disposition.

### Post-#1715 reclassification — blocked on runtime constructor/value execution (2026-05-05)

This audit's original "Substrate live; just needs test wiring" disposition for the four Pop A rows conflated two distinct readiness conditions:

1. **Substrate-presence** — the `.dag` declarations exist (still true).
2. **Executability** — the v3 evaluator can run the `.dag` functions on test inputs and compare results.

Re-verified at `origin/main` HEAD `52dcd5529` (post-#1715 `b13378e60` + #1716):

- **PR #1715 advanced executability** for one shape: `TransformTarget::Callable(callee_decl)` where `callee_decl` is an `Arrow` with a `UserDefined` body now evaluates the bind body in a fresh frame (`src/v3/compiler/src/lib.rs:579-617`). User-function calls land. **Direct calls to `derive_bound`, `master_theorem`, `int_pow_bounded`, `ceil_log[_iter]`, `peano_literal_materialization_cap`, `positive_descent_amount_from_positive_int`, `proportional_divisor_from_int_at_least_two`, `meet_sub_value`, `join_sub_value` are now in scope as Arrow-target callables.**

- **Non-Arrow `Callable` targets still fail-closed** with `BadTransformOperands { reason: "Callable target declaration is not an Arrow type" }` (`src/v3/compiler/src/lib.rs:581-585`; ratcheted by unit test assertion at `:2035-2040`).

- **Surface variant / record construction lowers to non-Arrow `Callable`.** `lower_constructor_invocation(dag, target, inputs, span)` at `src/v3/compiler/src/lower.rs:7103-7119` produces `TransformTarget::Callable(target)` where `target` is the variant constructor or record declaration id — not an Arrow. Every constructor expression in a `.dag` test body (`ErrorBound`, `Some { value: ... }`, `none`, `OneStep`, `ConstantShrink { steps: ... }`, `StrictSubValue { field: ..., factor: ... }`, etc.) goes through this path.

Population A's value flow is dominated by these constructor values:

- **A.1 derive_bound / master_theorem** — output is `CostBound` variants (`ErrorBound`, `cost_linear`, `ForeverBound`, `cost_polynomial { ... }`); input includes `ShrinkFactor` variants (`UnitShrink`, `ConstantShrink { steps: ... }`, `ProportionalShrink { divisor: ... }`).
- **A.2 int_pow_bounded / ceil_log / ceil_log_iter** — output is `Int?` constructed via `Some { value: ... }` / `none`. Even the success path emits a non-Arrow `Callable` for the `Some` constructor.
- **A.3 peano cap + descent / divisor** — output is `PositiveDescentAmount?` / `ProportionalDivisor?` (nested `Some`/`OneStep`/`AdditionalStep { previous: ... }` / `DivideByTwo`/etc.).
- **A.4 meet_sub_value / join_sub_value** — both ends of the function (input args and return value) are `SubValueRelation` variants (`StrictSubValue { field, factor }`, `ArithmeticDescent { param, factor }`, `SubValueUnknown`); inputs nest `InductiveField` records and `ShrinkFactor` variants.

Without runtime constructor/value execution for variants and records, every behavioral assertion needs a constructor result the evaluator cannot produce, so the property tests cannot run. Authoring decorative shape-only assertions on the lowered Dag would not catch the regressions the v2 tests catch (per dispatch's no-fake-tests clause).

**Reclassified disposition for Pop A (all four rows):**

> Substrate live; **requires v3 evaluator runtime constructor/value execution** (variant constructors + record constructors) before the behavioral property-test migration can land. PR #1715's `Callable` arm covers Arrow user-function calls only; the constructor-target arm remains parked.

**Routing per current PB/Director coordination (inbox #1134):**

- Pop A migration is held pending an evaluator slice that lands constructor/value execution for variants and records (or an explicit substrate decision that introduces a different executable path). PB does not author Rust mirrors as an interim surface — that would require a Substrate / Evaluator authority decision PB does not own and would violate the intended single-authority migration shape.
- The audit's original A.1–A.4 row text remains accurate as substrate-presence; this section is the executability correction. The original "single v3-side property-test PR landing all 4 ports" recommendation above is **not actionable today** — defer until the constructor evaluator arm lands.

## Population B — substantive G-1 consumers outside `src/v2/` (2 test files)

**Audit framing:** these are the §3.1 + §3.2 consumers from the migration matrix. The S-1 input packet's Decisions 1 + 2 already enumerated the *disposition choice* for each. This section spot-checks the v3-side coverage **available today** so S-1's author can pick replace-vs-delete (Decision 1) / authority-migration sequencing (Decision 2) with the live state visible.

### B.1 `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs` (matrix §3.1)

| Field | Value |
|---|---|
| Current dependency | `use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult}; use v2_compiler::v2_interpreter::{self, Value}; use v2_compiler_tests::helpers::resolve_imports_transitively;` (verified at file head). |
| Property under test | `dsl/std/render.dag` `repeat_string(s, n)` and `indent_text` semantics; lower-time fold; `String` result. The v2 oracle compiles a small program through the v2 pipeline and runs the v2 interpreter to assert output. |
| v3-side substrate analog | **LIVE** — `dsl/std/render.dag::repeat_string` is declared in v3 substrate (referenced in matrix §3.1's "Proposed migration"). The v3 evaluator surface that would HOST the equivalence row is in flight per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E; not yet landed. |
| v3-side test coverage | **PARTIAL** — `p0_std_render_repeat_string_test.rs` itself uses v2 as oracle; no parallel v3-evaluator-side test exists. The "Replace" disposition (S-1 Decision 1 default) lands the missing v3-side fixture as a corpus row consuming the in-flight evaluator. The "Delete" disposition (alternate) requires a structural-guarantee receipt naming the v3 typed primitive composition that makes the test redundant. |
| Disposition recommendation for S-1 | **Replace** (S-1 input packet Decision 1's default), gated on R2-Evaluator surface landing far enough to host the equivalence row. If R2-Evaluator timeline is uncertain, **Delete with structural-guarantee receipt** is the fallback if the v3 lower-time fold is statically guaranteed to produce the right `String` result by typed primitive composition (PB cannot make that call alone — needs Substrate Manager confirmation that the `dsl/std/render.dag::repeat_string` lowering is structurally total). |
| Counter-default cost | Pre-emptive deletion without structural-guarantee receipt drops oracle without replacement (loses `repeat_string` lower-time fold ratchet). Pre-emptive replacement before R2-Evaluator surface is ready ships a fixture pointing at a non-functional substrate (same fail-red-permanently pattern audits #1235 / #1347 / #1368 / #1415 reject). |

### B.2 `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` (matrix §3.2)

| Field | Value |
|---|---|
| Current dependency | `let v2_map = v2_compiler::std_algebra::kernel_algebra_profile();` (line 1005) plus the `v2_profile_to_v3` shim at line 991-1004 mapping `v2_compiler::std_algebra::AlgebraProfile` variants to v3 `AlgebraProfile`. |
| Property under test | Drift ratchet: v3 `dag::kernel_algebra_profile` matches v2 stage0 `std_algebra::kernel_algebra_profile()` row-for-row, treating v2 stage0 as the authority. |
| v3-side substrate analog | **PARTIALLY LIVE** — `kernel_algebra_profile` is declared on the v3 side (in `dsl/std/algebra.dag`), but the audit/matrix names v2 stage0 as the *authority* the v3 mirror is checked against; the cross-program migration is to make v3 the single authority and retire the parity test. Substrate Manager owns the authority migration per S-1 input packet Decision 2. |
| v3-side test coverage | **WILL BECOME REDUNDANT** — once v3 single-authority `kernel_algebra_profile` lands (Substrate continuation work), the parity test is structurally meaningless (no mirror to compare against). The other tests in the same file (`m2_substrate_inhabitance_test.rs`) are unaffected and continue to ratchet v3-side substrate inhabitance. |
| Disposition recommendation for S-1 | **Migrate authority + retire parity test** (S-1 input packet Decision 2's default). Sequencing per matrix §3.2 STOP cell: Substrate-side authority migration **first** (Substrate Manager dispatches), then PB retires the parity test (in this same file, atomic with the Substrate PR or a follow-up depending on PR boundaries). The parity test's drop is mechanical once `v2_compiler::std_algebra::kernel_algebra_profile()` is no longer the authority. |
| Counter-default cost | Reverse ordering (PB drops parity test before Substrate authority lands) loses drift detection. PB acting unilaterally (without Substrate-Manager-routed authority migration) makes a cross-program decision PB doesn't own. |

### Population B summary

Both are substantively gated on cross-lane work (R2-Evaluator surface for B.1's "Replace" path; Substrate-side `kernel_algebra_profile` authority migration for B.2). PB cannot dispose of either honestly until S-1 routes the disposition + the prerequisite cross-lane work lands. The two Cargo edges (matrix §3.3) drop atomically once both B.1 and B.2 dispose.

## Net dispatch order this audit implies

Combining Pop A + Pop B per the matrix's prerequisite DAG + this audit:

1. **S-1 lands** (PM-authored T-V2-Retirement worker brief).
2. **Three lanes can dispatch in parallel once S-1 lands** (each retains its own lane-specific dependency chain — S-1 is the common gate, not the only gate per lane):
   - **B.1 disposition** per Decision 1 of S-1 input packet (#1462) — additionally gated on R2-Evaluator surface for the Replace path, or a Substrate structural-guarantee receipt for the Delete fallback. Lane-specific.
   - **B.2 disposition** per Decision 2 — additionally gated on Substrate-side `kernel_algebra_profile` authority migration landing first. Lane-specific.
   - **Pop A v3-side property-test migration** — **superseded by the "Post-#1715 reclassification" subsection in §A** (2026-05-05). The original "S-1 only / no evaluator dependency / mechanical port" framing was correct on substrate-presence grounds but conflated substrate-presence with executability. After PR #1715 the disposition reads: gated on S-1 **and** on v3 evaluator runtime constructor/value execution for variants and records (Arrow user-function calls land per #1715; non-Arrow `Callable` constructor targets remain parked at `src/v3/compiler/src/lib.rs:581-585`). PB does not author Rust mirrors as an interim authority. The §A reclassification is the single authority for Pop A's gate set; this lane row defers to it.
3. **§3.3 Cargo edges drop** atomically with B.1 + B.2 closure.
4. **G-1 green** — `cargo test -p v3-compiler` passes without v2 crates.
5. **Pop A migration green** — must precede G-2; without it, G-2 silently drops the property ratchets when `src/v2/tests` is removed. Independent of G-1 close ordering otherwise.
6. **G-2 prereq stack green** (S-1 + S-2 + S-3 + S-4 + G-1; per audit §3.2). Pop A coverage from step 5 is implicit in this stack via Pop A's "before G-2" anchoring.
7. **G-2 implementation** — `src/v2/stage0` + `src/v2/tests` workspace members removed; `src/v2/` deleted.

Recommended sequencing for the S-1 author: explicitly enumerate Pop A migration scope alongside the B.1/B.2 dispositions so all three lane-specific dispatches surface in one document and the Pop A property ratchet doesn't get lost between G-1 close and G-2 deletion.

## Constraints honored (verbatim from dispatches)

- ✅ No code changes.
- ✅ No `src/v2/` deletion or workspace-member removal.
- ✅ No `v2-compiler` / `v2-compiler-tests` Cargo edge removal.
- ✅ No `kernel_algebra_profile` migration decision from PB (Decision 2 routes to Substrate; this audit only spot-checks the live state for Substrate's reference).
- ✅ No `verification.dag` convergence decision from PB.
- ✅ No claim of G-1 implementation unblocked.
- ✅ No claim of S-1 authored.

## What this PR is

A single new docs-only file (`docs/briefs/r3-pb-tv2-population-coverage-audit.md`) plus a link-only registration in `docs/briefs/r2-pure-bootstrap-manager.md` sub-briefs list. Combined Pop A + Pop B audit since the dispatches are parallel; PM/Director can split into two artifacts later if useful.

## Cross-refs

- Parent audit: [`docs/audit/t-v2-retirement-audit.md`](../audit/t-v2-retirement-audit.md) (#1338).
- Per-surface migration matrix: [`docs/audit/t-v2-retirement-migration-matrix.md`](../audit/t-v2-retirement-migration-matrix.md) (#1346/#1379).
- G-1 readiness receipt: [`docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`](r3-pb-tv2-g1-readiness-receipt.md) (#1446).
- S-1 input packet (Decision 1 + 2 referenced): [`docs/briefs/r3-pb-tv2-s1-input-packet.md`](r3-pb-tv2-s1-input-packet.md) (#1462).
- R2-Evaluator manager (B.1 prerequisite): [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md).
- Equivalence-corpus seed (B.1 "replace" path framing): [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md).
- Substrate-fact-introduction procedure (B.2 routing): [`INVARIANTS.md`](../../INVARIANTS.md) §P1.
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md).
- Live v3 substrate authorities: `src/v3/std/induction.dag:281` (`meet_sub_value`), `:329` (`join_sub_value`), `:767` (`int_pow_bounded`), `:802` (`ceil_log`), `:823` (`master_theorem`), `:897` (`derive_bound`); `src/v3/std/termination.dag:140` (`peano_literal_materialization_cap`), `:146` (`positive_descent_amount_from_positive_int`), `:162` (`proportional_divisor_from_int_at_least_two`).
- Live Pop A test sources (anchor for future v3-port worker): `src/v2/tests/src/derive_bound_fail_closed_test.rs`, `src/v2/tests/src/int_pow_bounded_test.rs`, `src/v2/tests/src/peano_materialization_cap_test.rs`, `src/v2/tests/src/sub_value_lattice_factor_test.rs`.
- Live Pop B test sources: `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs`, `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:991`.

## §Delta (2026-05-05+) — re-execution vs `origin/main` HEAD `530c76ea7`

**Verdict:** **No material delta.** All audit findings (substrate-presence, missing v3-side coverage, post-#1715 reclassification, Pop B v2 dependencies, Cargo edges) reproduce against current `origin/main`. Only drift is minor line-number shift inside still-live declarations; no semantic change to the audit's dispatch implications.

**Re-executed methodology** (commands replayed verbatim from §Population A row tables and §Post-#1715 reclassification):

| Check | Audit text | Live state at `530c76ea7` | Drift |
|---|---|---|---|
| `meet_sub_value` / `join_sub_value` decl in `induction.dag` | `:281` / `:329` | `:282` / `:330` | +1 line each (cosmetic) |
| `int_pow_bounded` / `ceil_log` / `ceil_log_iter` | `:767` / `:802` / `:808` | `:768` / `:803` / `:809` | +1 line each |
| `master_theorem` / `derive_bound` | `:823` / `:897` | `:824` / `:898` | +1 line each |
| `peano_literal_materialization_cap` / `positive_descent_amount_from_positive_int` / `proportional_divisor_from_int_at_least_two` in `termination.dag` | `:140` / `:146` / `:162` | `:140` / `:146` / `:162` | none |
| `grep -rnE '\b(derive_bound\|master_theorem)\b' src/v3/compiler/tests/` | zero | zero | none |
| `grep -rnE '\b(int_pow_bounded\|ceil_log)\b' src/v3/compiler/tests/` | zero | zero | none |
| `grep -rnE '\b(peano_literal_materialization_cap\|positive_descent_amount_from_positive_int\|proportional_divisor_from_int_at_least_two)\b' src/v3/compiler/tests/` | zero | zero | none |
| `grep -rnE '\b(meet_sub_value\|join_sub_value)\b' src/v3/compiler/tests/` | zero | zero | none |
| Non-Arrow `Callable` fail-closed in `src/v3/compiler/src/lib.rs` | `:581-585` `BadTransformOperands { reason: "Callable target declaration is not an Arrow type" }` | present at the analogous arm (block ~`:579-585`); fail-closed branch + Arrow-bind dispatch unchanged | line range shifted within same function |
| `lower_constructor_invocation` produces `TransformTarget::Callable(target)` | `src/v3/compiler/src/lower.rs:7103-7119` | function present at `:7103+`; same `TransformTarget::Callable(target)` shape | none |
| B.1 v2 imports in `p0_std_render_repeat_string_test.rs` | `use v2_compiler::v2_compiler_compile::…; use v2_compiler::v2_interpreter::…; use v2_compiler_tests::helpers::resolve_imports_transitively;` | identical (file head) | none |
| B.2 v2 oracle line in `m2_substrate_inhabitance_test.rs` | `:1005` (`v2_compiler::std_algebra::kernel_algebra_profile()`) + `v2_profile_to_v3` shim `:991-1004` | `:1222` / `:1209-1219` (test fn at `:1208`) | +~217 lines (file grew); `v2_compiler::std_algebra::kernel_algebra_profile()` call + shim shape unchanged |
| Cargo edges (`src/v3/compiler/Cargo.toml`) | `v2-compiler = { path = "../../v2/stage0" }` + `v2-compiler-tests = { path = "../../v2/tests" }` | present at `:37-38` | none |

**Net dispatch order (§"Net dispatch order this audit implies"):** unchanged. Pop A still gated on S-1 + v3 evaluator runtime constructor/value execution (per §Post-#1715 reclassification); B.1 still gated on R2-Evaluator surface (or Substrate structural-guarantee receipt); B.2 still gated on Substrate `kernel_algebra_profile` authority migration; §3.3 Cargo edges still drop atomically with B.1 + B.2.

**Single-authority pointer (P2 from §A) reaffirmed:** §"Post-#1715 reclassification" remains the canonical Pop A dispatch-readiness authority. Per-row §A.1–§A.4 cells continue to carry substrate-presence context only.

**HEAD commit verified:** `530c76ea7 docs: E6-G1.a static lens fold dispatch packet (post-cleanup)`.

### B.2 reclassification — substrate authority migration already landed (correction 2026-05-06)

Per blocking review on PR #1805 (codex sha:`72667918`): the §Delta row for B.2 above incorrectly inferred from the surviving v2 oracle that Substrate authority migration is still pending. Re-grep at HEAD `530c76ea7` shows the migration **already landed**:

- `src/v3/compiler/src/dag.rs:1789-1791` (doc comment on `pub fn kernel_algebra_profile`): *"Semantic authority is `dsl/std/algebra.dag` (`data kernel_algebra_profile`). `v2_compiler::std_algebra::kernel_algebra_profile` remains only as a [drift ratchet]"*.
- `src/v3/compiler/src/dag.rs:3596-3605` — typed accessor `Dag::kernel_algebra_profile` reads the lowered `data kernel_algebra_profile` Map directly from `dsl/std/algebra.dag`.
- `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:1239` — `fn v3_kernel_algebra_profile_reads_lowered_dag_map_authority` ratchets v3 reading the lowered-Dag Map authority (P0 invariant).
- The surviving `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` test at `:1208` is the **drift ratchet** (test name + doc both label v3 as authority; v2 stage0 is the mirror, regenerated from `dsl/std/algebra.dag`). The original audit's framing that v2 stage0 is "the *authority* the v3 mirror is checked against" is reversed by the live state.

**Reclassified disposition for B.2:**

> Substrate `kernel_algebra_profile` authority migration **DONE** at HEAD `530c76ea7`. B.2's remaining R3 PB-lane work is **parity-test retirement + Cargo-edge drop**, not authority migration:
> - retire `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` (and its `v2_profile_to_v3` shim) once the cross-program decision allows; the test is structurally redundant given `v3_kernel_algebra_profile_reads_lowered_dag_map_authority` already ratchets the v3-side authority read.
> - drop `v2-compiler` / `v2-compiler-tests` Cargo edges in `src/v3/compiler/Cargo.toml:37-38` once B.1 + B.2 parity-test retirement both close.

**Net dispatch order correction:** the original §"Net dispatch order this audit implies" §3.2 sequencing assumed Substrate-Manager-routed authority migration is the prerequisite. With migration already landed, B.2 collapses to a PB-lane-internal mechanical retirement (atomic with Cargo-edge drop), no Substrate-Manager dispatch needed for B.2 disposition.

**§A reclassification single-authority pointer reaffirmed for Pop A** — this B.2 correction does not affect Pop A's gate set (still §"Post-#1715 reclassification" — v3 evaluator runtime constructor/value execution). Authority changes here are B.2 (Pop B) only.

## §Delta (2026-05-06) — re-execution vs `origin/main` HEAD `2c7d82031`

**Verdict:** **No material delta.** All audit findings (substrate-presence, missing v3-side coverage, B.2 authority-migration-already-landed reclassification, Pop B v2 dependencies, Cargo edges) reproduce against current `origin/main` HEAD `2c7d82031` ("docs(r3): refresh §3 T-Debt-Paydown row after #1807/#1892/#1903 (#1911)"). Line-anchor drift continues per the load-bearing pattern; symbol/section anchors remain the canonical identity.

**Re-executed methodology** (verbatim from prior §Delta tables):

| Check | Audit text | Live state at `2c7d82031` | Drift vs §Delta `530c76ea7` |
|---|---|---|---|
| `meet_sub_value` / `join_sub_value` decl in `induction.dag` | `:281` / `:329` (audit time) | `:277` / `:325` | -5 lines each (file shrank since 530c76ea7's `:282` / `:330`) |
| `int_pow_bounded` / `ceil_log` / `ceil_log_iter` | `:767` / `:802` / `:808` | `:765` / `:800` / `:806` | -3 lines each |
| `master_theorem` / `derive_bound` | `:823` / `:897` | `:821` / `:895` | -3 lines each |
| `peano_literal_materialization_cap` / `positive_descent_amount_from_positive_int` / `proportional_divisor_from_int_at_least_two` in `termination.dag` | `:140` / `:146` / `:162` | `:243` / `:251` / `:268` | **+103 / +105 / +106 lines** (`termination.dag` grew substantially between 530c76ea7 and 2c7d82031; symbols still live, named declarations unchanged) |
| `grep -rcE '\b(derive_bound\|master_theorem)\b' src/v3/compiler/tests/` non-zero hits | zero | zero | none |
| `grep -rcE '\b(int_pow_bounded\|ceil_log)\b' src/v3/compiler/tests/` non-zero hits | zero | zero | none |
| `grep -rcE '\b(peano_literal_materialization_cap\|positive_descent_amount_from_positive_int\|proportional_divisor_from_int_at_least_two)\b' src/v3/compiler/tests/` non-zero hits | zero | zero | none |
| `grep -rcE '\b(meet_sub_value\|join_sub_value)\b' src/v3/compiler/tests/` non-zero hits | zero | zero | none |
| Non-Arrow `Callable` fail-closed in `src/v3/compiler/src/lib.rs` | `:581-585` originally; `~:579-585` at 530c76ea7 | `:674` (primary fail-closed branch) **plus a second site at `:2338`** with identical `reason: "Callable target declaration is not an Arrow type"` text — fail-closed shape unchanged, surface duplicated for additional dispatch path | line shifted within same function; **new second site** at `:2338` (same fail-closed semantics) |
| `lower_constructor_invocation` produces `TransformTarget::Callable(target)` | `:7103-7119` (audit) / `:7103+` (530c76ea7) | `:7192-7202` | +~89 lines |
| B.1 v2 imports in `p0_std_render_repeat_string_test.rs` | L9: `use v2_compiler::v2_compiler_compile::…`; L10: `use v2_compiler::v2_interpreter::…`; L11: `use v2_compiler_tests::helpers::…` | identical (L9-L11 unchanged) | none |
| B.2 v2 oracle line in `m2_substrate_inhabitance_test.rs` | `:1222` oracle / shim block at `:1208-1219` per 530c76ea7 §Delta | `:1208` test fn declaration / `:1209` shim fn / `:1222` `kernel_algebra_profile()` call — **structurally identical to 530c76ea7**; the v3-authority test `v3_kernel_algebra_profile_reads_lowered_dag_map_authority` still resides at `:1239` | none — file did not grow further between `530c76ea7` and `2c7d82031` in this region |
| Cargo edges (`src/v3/compiler/Cargo.toml`) | `:37-38` | `:37-38` | none |
| `dag.rs` authority-migration markers (B.2 reclassification) | `pub fn kernel_algebra_profile` authority comment + `Dag::kernel_algebra_profile` typed accessor | comment block at `:1786-1798`, typed accessor at `:3587-3606`, drift-ratchet pointer at `:1796` — all present | line shifts only; semantic content unchanged |

**Pop A net dispatch order:** unchanged. Still gated on S-1 (PM-authored worker brief, still absent under `docs/briefs/`) + v3 evaluator runtime constructor/value execution per §"Post-#1715 reclassification". This is the load-bearing single-authority statement for Pop A; per-row §A.1–§A.4 cells continue to carry substrate-presence context only.

**Pop B net dispatch order:** unchanged from the 2026-05-06 B.2 reclassification correction above. Substrate `kernel_algebra_profile` authority migration still **DONE**; B.2 remaining work is **parity-test retirement + Cargo-edge drop atomic with B.1**, no Substrate-Manager dispatch needed.

**HEAD-delta narrative:** between `530c76ea7` (2026-05-05) and `2c7d82031` (2026-05-06) main absorbed substantial bootstrap/substrate churn (Q-MachineConstraint #1856, T-Numeric-Construction S9 #1840, T-Debt-Paydown row refreshes #1807/#1892/#1903, T-V2 inventory #1848/#1850, etc.). None of that motion changed the audit's findings — every cited surface still resolves, every zero-coverage grep still returns zero, the B.2 authority-migration markers landed in `dag.rs` are still present and load-bearing.

**HEAD commit verified:** `2c7d82031 docs(r3): refresh §3 T-Debt-Paydown row after #1807/#1892/#1903 (#1911)`.
