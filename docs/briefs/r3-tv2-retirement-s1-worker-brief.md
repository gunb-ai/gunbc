# R3 — T-V2-Retirement S-1 worker brief (PM-authored)

**Status**: ACTIVE worker brief; PM-authored 2026-05-04 per Director ask at [`gunb-ai/gunbc#828` inbox-4374518930](https://github.com/gunb-ai/gunbc/issues/828) (relayed to PM at #846 inbox-4375374044).

**Lane**: T-V2-Retirement (R3 lane #11; PB Manager-owned per `docs/r3-structure.md` §"Manager structure" + lane structure table).

**Authority chain**: this brief is the missing **S-1: PM-authored T-V2-Retirement worker brief** that the audit + migration-matrix STOP conditions name as the prerequisite for G-1 implementation. PB cannot author S-1; the audit explicitly names PM as owner (per `docs/audit/t-v2-retirement-audit.md` §3.1 ownership table). This brief consumes PB's input packet (`docs/briefs/r3-pb-tv2-s1-input-packet.md`) + audit + migration matrix + G-1 readiness receipt to make the PM-owned decisions worker dispatch needs.

## What this brief unblocks

S-1 unblocks the **G-1 implementation chain** for T-V2-Retirement:

- **G-1**: deletion of Cargo.lock v2 dev-deps. Cascade-gated on §3.1 + §3.2 dispositions completing (one of which needs Substrate-side authority migration; the other needs PM choice between replace vs delete).
- **G-2 prereq stack**: S-1 + S-2 + S-3 + S-4 + G-1. S-1 brief enumerates the entire chain (per Decision 6 below) so workers don't reconstruct it from the audit.

The narrowest immediately-dispatchable worker action under S-1 is **Pop A v3 property-test migration** (§"Worker dispatch sequence" below) — port the **four audited Pop A coverage items** (per `docs/briefs/r3-pb-tv2-population-coverage-audit.md` §"Population A") onto live v3 `src/v3/std/induction.dag` + `src/v3/std/termination.dag` surfaces (audit-canonical root; v3-staged authority that v3 compiler reads). S-1-only; no Evaluator/Substrate authority migration prerequisite; prevents G-2 from silently dropping the ratchets.

## Consumes (authoritative inputs)

This brief is grounded in the following authorities:

- **PB input packet**: [`docs/briefs/r3-pb-tv2-s1-input-packet.md`](r3-pb-tv2-s1-input-packet.md) — 6-decision checklist with PB-recommended defaults; PM ratifies/counters per row below.
- **Lane audit**: [`docs/audit/t-v2-retirement-audit.md`](../audit/t-v2-retirement-audit.md) (#1338) — full retirement audit; STOP conditions for G-1 + G-2.
- **Per-surface migration matrix**: [`docs/audit/t-v2-retirement-migration-matrix.md`](../audit/t-v2-retirement-migration-matrix.md) (#1346/#1379) — per-test-file disposition with proposed migrations.
- **G-1 readiness receipt**: [`docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`](r3-pb-tv2-g1-readiness-receipt.md) (#1446) — current state of G-1-prereq-ci (now green via #1701) + remaining blockers.
- **G-2 deletion plan + guardrails**: [`docs/audit/t-v2-g2-deletion-plan-and-guardrails.md`](../audit/t-v2-g2-deletion-plan-and-guardrails.md) — full `src/v2/` deletion plan + guardrails.

Lane structure context: [`docs/r3-structure.md`](../r3-structure.md) §T-V2-Retirement (lane #11) + closure gates `v2_oracle_no_remaining_test_consumers` + `v2_directory_deleted`.

## PM decisions

Each decision corresponds to a row in PB's input packet. Format: ratify default OR counter with receipt.

### Decision 1 — §3.1 (`p0_std_render_repeat_string_test.rs`) disposition

**PM ratification**: **REPLACE** (per PB recommended default).

Replace the v2 oracle with a v3 evaluator equivalence-corpus row. Aligns with PB-Runtime ↔ R2-Evaluator equivalence-corpus direction; preserves the property under test (lower-time fold of `repeat_string` to a string literal). Delete-with-receipt path requires structural-guarantee receipt that lower-time fold is by construction; that's a stronger receipt than a corpus-row replace and isn't currently authored.

**Pre-requisite**: R2-Evaluator surface live enough to host an equivalence-corpus row.

**Owner**: PB executes once R2-Evaluator dependency lands.

**Sequencing**: PB authors the corpus-row replace **after** R2-Evaluator hits the surface readiness threshold per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E. Pre-Evaluator dispatch of this slice would force a deferred-disposition pattern that's harder to track than just waiting.

### Decision 2 — §3.2 (`m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`) cross-program routing

**PM ratification**: **Authority on `dsl/std/algebra.dag`** (per PB recommended default).

**PM ratification on ownership split**: Substrate Manager owns the authority migration; PB Manager owns the parity-test retirement once v2 mirror is no longer authority.

**PM ratification on sequencing**: Substrate-side authority migration **first**; then PB retires parity test. Per matrix §3.2 STOP cell — reverse order would lose drift detection during the gap.

**Routing**: this brief explicitly **routes the authority-migration worker dispatch to Substrate Manager (jolly-ram-908 #1130)** as a cross-program ask. PM relays this routing in parallel with this brief landing.

**Owner**: Substrate Manager executes authority migration; PB Manager executes parity-test retirement.

### Decision 3 — §3.3 Cargo edges deletion mechanics

**PM ratification**: **Atomic with the second-of-§3.1-or-§3.2-to-land** (per PB recommended default).

Whichever closes last bundles the Cargo edges deletion. Per matrix §3.3 STOP — pre-emptive deletion breaks build; post-hoc-follow-up deletion leaves parallel-authority residue between PRs.

**Owner**: PB executes; PM ratification of sequencing already encoded above (no PM-only choice required beyond this ratification).

### Decision 4 — Legacy emit chain (`rust_method_template_contracts.dag` header note)

**PM ratification**: **Delete on S-4 + v3 emitter end-to-end consumption of `MethodTemplateContract` rows** (per PB recommended default).

Cross-lane coupling: T-Ground-LanguageSpec scope E owns the v3-emitter consumption side; PB owns the legacy emit chain deletion.

**Pre-requisite chain** (PM enumerates):
1. S-4 (PB-Runtime trampoline live as bootstrap path)
2. v3 emitter consumes `src/v3/std/{rust,python,go}_method_template_contracts.dag` end-to-end
3. T-Ground-LanguageSpec scope E delivers the v3-side consumption

**Owner**: PM marks the gate (this brief enumerates the prerequisite chain); PB executes deletion once gate clears; T-Ground-LanguageSpec scope E delivers the v3-side consumption.

### Decision 5 — `verification.dag` convergence routing

**PM ratification**: **ROUTE to Substrate Manager (design call); escalate to Director if Substrate cannot scope** (per PB recommended default).

Convergence of `verification.dag` is **not** a G-1 prerequisite (per audit §"verification.dag convergence" position) — it's G-2-only. But silent deferral stalls G-2 without a named owner; this brief explicitly names Substrate Manager (jolly-ram-908 #1130) as the design-call owner and Director (zesty-bear-812 #828) as the arbitration owner if Substrate cannot scope.

**Routing**: this brief explicitly **routes the `verification.dag` convergence design call to Substrate Manager** as a cross-program ask, with G-2-only timing context (not blocking G-1; blocking G-2). PM relays this routing in parallel with this brief landing.

**Owner**: Substrate Manager (design call); Director (arb if Substrate cannot scope); PB (v2-side cleanup once design lands).

### Decision 6 — S-1 brief scope coverage

**PM ratification**: **COVER BOTH G-1 dispositions AND S-2 / S-3 / S-4 prereq chain** (per PB recommended default).

This brief covers the full G-2 prereq stack so workers don't reconstruct the chain from the audit. The G-2 chain (S-1 + S-2 + S-3 + S-4 + G-1) is documented in §"G-2 prerequisite chain" below.

**Owner**: PM (this brief covers both per ratification).

## Worker dispatch sequence

PB Mgr's recommended narrowest first dispatch under S-1 is **Pop A v3 property-test migration**. Subsequent dispatches per the cascade chain.

### Dispatch 1 — Pop A v3 property-test migration (S-1-only; immediately dispatchable)

**Scope**: port the **four audited Pop A coverage items** per `docs/briefs/r3-pb-tv2-population-coverage-audit.md` §"Population A" (authoritative source) onto live v3 surfaces:

1. **A.1 — `derive_bound` / `master_theorem` fail-closed boundary coverage** (from v2 `std_induction`; v3 substrate at `src/v3/std/induction.dag:897` + `:823`). v3-side property test asserts fail-closed semantics: 0 branches → `ErrorBound`; negative branches → `ErrorBound`; invalid work exponents; `master_theorem` boundary cases.

2. **A.2 — `int_pow_bounded` / `ceil_log` boundary coverage** (from v2 `std_induction`; v3 substrate at `src/v3/std/induction.dag:767` + `:802` + `:808`). v3-side property test re-asserts: negative-exp → `None`; non-negative matches `pow`; overflow at `2^63` → `None`; degenerate-base cap (`0`/`1`/`-1`) doesn't deep-recurse; `ceil_log` semantics.

3. **A.3 — `peano_literal_materialization_cap` + `positive_descent_amount_from_positive_int` + `proportional_divisor_from_int_at_least_two` cap coverage** (from v2 `std_induction` + `std_termination`; v3 substrate at `src/v3/std/termination.dag:140` (cap=256) + `:146` + `:162`). v3-side property test asserts oversize `Int` inputs → `None`; cap of 256 is single-source v3 declaration. **Bonus**: v3 test cites `peano_literal_materialization_cap()` directly so the cap value is grep-clean across the codebase (replaces magic 256 in v2 test bodies).

4. **A.4 — `meet_sub_value` / `join_sub_value` `ShrinkFactor`-preservation coverage** (from v2 `std_induction`; v3 substrate at `src/v3/std/induction.dag:281` + `:329`). v3-side property test ports meet/join cases against v3 `SubValueRelation` / `InductiveField` / `RecursionShape` constructors; `ShrinkFactor`-preservation invariant is identical between versions; near-mechanical port.

**Why this dispatch is narrowest**:
- S-1-only (does NOT need R2-Evaluator/PB-Runtime/Substrate authority migration)
- Prevents G-2 from silently dropping the property ratchets
- Prerequisite-shaped (preserves test surface for later cleanup) rather than deletion-shaped (which would break G-1)
- Single-author work (no cross-program coordination per dispatch)
- Substrate is **LIVE on v3 side for all 4 items** (verified per `r3-pb-tv2-population-coverage-audit.md` §"Population A summary" — no R2-Evaluator / PB-Runtime / Substrate-authority dependency)

**Live v3 surfaces** the migration targets (per audit substrate-line citations):
- `dsl/std/induction.dag` — A.1 (`derive_bound`:897, `master_theorem`:823), A.2 (`int_pow_bounded`:767, `ceil_log`:802, `ceil_log_iter`:808), A.4 (`meet_sub_value`:281, `join_sub_value`:329)
- `dsl/std/termination.dag` — A.3 (`peano_literal_materialization_cap`:140, `positive_descent_amount_from_positive_int`:146, `proportional_divisor_from_int_at_least_two`:162)

**Recommended worker assignment**: silent-boar or witty-tern (per PB Mgr's read-back at [#1134 comment-4375362161](https://github.com/gunb-ai/gunbc/issues/1134#issuecomment-4375362161)). PB Manager picks specific worker per their cadence + readiness.

**Closure**: dispatch produces one PR (or 4 sub-PRs per coverage item) migrating the four audited Pop A coverage items to v3 surfaces; PB Manager confirms property ratchets preserved on v3 side; PR(s) merge; ratchet status tracked in PB Manager's inventory.

### Dispatch 2 — B.1 (peano_arith consumer) — gated on Decision 1 prerequisite

**Scope**: replace the `p0_std_render_repeat_string_test.rs` v2 oracle with a v3 evaluator equivalence-corpus row (per Decision 1 "replace" disposition).

**Pre-requisite**: R2-Evaluator surface live enough to host an equivalence-corpus row (per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E). NOT immediately dispatchable; queued behind R2-Evaluator dispatch milestones.

**Owner**: PB Manager (executes once R2-Evaluator dependency lands).

### Dispatch 3 — B.2 (workflow-dispatcher parity / kernel_algebra_profile authority) — Substrate-led

**Scope per Decision 2 ratification**:
- Substrate Manager dispatches authority-migration worker; v3-side single-authority `kernel_algebra_profile` lands at `dsl/std/algebra.dag`
- PB Manager retires the parity test once authority migration completes

**Pre-requisite**: Substrate Manager dispatch (cross-program ask routed in parallel with this brief).

**Owner**: Substrate Manager (authority migration executor); PB Manager (parity-test retirement once authority lands).

### Dispatch 4 — G-1 deletion of Cargo.lock v2 dev-deps — cascade-gated

**Scope**: deletion of v2-compiler / v2-compiler-tests Cargo edges per Decision 3 ratification.

**Pre-requisite**: §3.1 (Dispatch 2) + §3.2 (Dispatch 3) both close. Atomic with the second-of-§3.1-or-§3.2-to-land per ratification.

**Owner**: PB Manager.

## G-2 prerequisite chain (full retirement of `src/v2/`)

Per `docs/audit/t-v2-retirement-audit.md` §1 STOP-condition table, G-2 prereq stack is `S-1 + S-2 + S-3 + S-4 + G-1`. This brief covers S-1; the rest enumerated per audit S-N definitions:

### S-2 — T-FixedPoint closed (audit §1 row "S-2")

Per audit: *"T-FixedPoint closed"* (gate per `r3-structure.md` Lane 5: `pb_self_compile_fixed_point` — `compiler.dag` compiles to bit-identical stage0 Rust).

S-2 + S-3 closure is what allows S-4 (PB-Runtime trampoline) to be the live bootstrap — without S-2+S-3, removing `src/v2/stage0` from the workspace breaks the build chain even if PB-Runtime is technically present (audit §1).

PB Manager-owned (T-FixedPoint lane). R3 in flight.

### S-3 — T-LensProducer-Retirement closed (audit §1 row "S-3")

Per audit: *"T-LensProducer-Retirement closed (all 3 sub-gates: `lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs`)"*.

PB Manager-owned (T-LensProducer-Retirement lane; sub-gates per `r2-pure-bootstrap-manager.md` row T-LensProducer-Retirement + `design-pb-runtime-interpreter.md` §5.1). R3 in flight.

### S-4 — PB-Runtime trampoline live as bootstrap path (audit §1 row "S-4")

Per audit: *"PB-Runtime trampoline lands such that bootstrap no longer routes through `src/v2/stage0`"*. Gate per `design-pb-runtime-interpreter.md` §3.

PB Manager-owned. NOT MET; PB-Runtime interpreter-as-data is the gate.

### G-1 — Cargo.lock v2 dev-deps deletion (covered above; Dispatch 4)

G-1's STOP condition per audit §"G-1 STOP condition" is **S-1 only** (PM worker brief — landed via PR #1711). G-1's *implementation* additionally requires §3.1 + §3.2 dispositions (Dispatches 2-3 above; Decisions 1+2 of this brief).

### G-2 — `src/v2/` workspace member removal + directory deletion

Final deletion. Cascade-gated on **S-1 + S-2 + S-3 + S-4 + G-1** all closing per audit §"G-2 STOP condition for G-2 work". PB Manager-owned execution.

Per `docs/audit/t-v2-g2-deletion-plan-and-guardrails.md` for full deletion plan + guardrails.

### Note on Decision 2 + Decision 5 routings

Decisions 2 + 5 (per §"PM decisions" above) are routings within G-1 / G-2 implementation work — **not** equivalent to the S-N gates:

- **Decision 2** (`kernel_algebra_profile` v3-side authority migration) is the §3.2 prerequisite for G-1 (one of the two v2-oracle test consumer dissolutions; lets PB retire the parity test). Substrate-side authority migration; routed to Substrate Manager (jolly-ram-908 #1130). NOT in the S-N chain.
- **Decision 5** (`verification.dag` convergence) is a G-2 prerequisite (independent of S-1 + S-2 + S-3 + S-4) per audit §"verification.dag convergence" position. Substrate Manager design call; routed in parallel. NOT in the S-N chain.

S-N labels match audit definitions (T-FixedPoint / T-LensProducer-Retirement / PB-Runtime trampoline). Decision 2 + Decision 5 are routing decisions within audit §3 surfaces, not the S-N stack.

## Cross-program coordination (PM relays in parallel with this brief)

Two cross-program asks routed in parallel with this brief landing:

1. **Substrate Manager (jolly-ram-908 #1130)** — Decision 2 authority migration ask: dispatch authority-migration worker for `kernel_algebra_profile` v3-side single-authority landing.
2. **Substrate Manager (jolly-ram-908 #1130)** — Decision 5 design call ask: convergence of `verification.dag` v2 → v3 surface (G-2-only timing; not blocking G-1).

If Substrate Manager cannot scope Decision 5, PM escalates to Director (zesty-bear-812 #828) per Decision 5 ratification.

## Closure mapping

S-1 brief lands → unblocks G-1 implementation work (Dispatch 1 immediately; Dispatches 2-4 cascade-gated). Closure of T-V2-Retirement lane is per `docs/r3-structure.md` §T-V2-Retirement closure gates:
- `v2_oracle_no_remaining_test_consumers` — closes once §3.1 + §3.2 + §"verification.dag convergence" all dissolve
- `v2_directory_deleted` — closes once G-2 lands (full `src/v2/` removal)

Lane closure cascades on T-FixedPoint + T-LensProducer-Retirement closing first per `r3-structure.md:412` cascade-gate note. T-Numeric-Construction has reciprocal cascade on T-V2-Retirement landing first (path-(a) v2-refinement-syntax-blocker per `docs/design-numeric-construction.md`).

## Constraints (from S-1 input packet § "Constraints honored")

- ✅ No code changes in this brief PR (docs-only).
- ✅ No `src/v2/` deletion (G-2 cascade).
- ✅ No `v2-compiler` / `v2-compiler-tests` Cargo edge removal (G-1 cascade).
- ✅ No `kernel_algebra_profile` migration decision from PB seat (Decision 2 routes to Substrate per §P1).
- ✅ No `verification.dag` convergence decision from PB seat (Decision 5 routes to Substrate; PB does not pre-empt).
- ✅ Pre-cascade brief authoring per `r3-structure.md:412` Director-discretionary rule (T-V2-Retirement carries internal cascade gate on T-FixedPoint + T-LensProducer-Retirement; pre-cascade brief authoring permitted).

## Cross-refs

- **Director ask**: [`gunb-ai/gunbc#828` inbox-4374518930](https://github.com/gunb-ai/gunbc/issues/828) (relayed at #846 inbox-4375374044)
- **PB Mgr read-back**: [`gunb-ai/gunbc#1134` comment-4375362161](https://github.com/gunb-ai/gunbc/issues/1134#issuecomment-4375362161)
- **#1701 G-2-prereq-ci green** — CI v2 step deleted; 590s parallel CI savings; PB Mgr confirmed at #1134 comment-4375362161
- **PB input packet**: `docs/briefs/r3-pb-tv2-s1-input-packet.md`
- **Lane authority**: `docs/r3-structure.md` §T-V2-Retirement (lane #11)
- **Audit + migration matrix**: `docs/audit/t-v2-retirement-{audit,migration-matrix}.md`
- **G-1 readiness**: `docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`
- **G-2 deletion plan**: `docs/audit/t-v2-g2-deletion-plan-and-guardrails.md`
- **Equivalence-corpus seed (Decision 1)**: `docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`
- **R2-Evaluator dependency (Decision 1)**: `docs/briefs/r2-evaluator-manager.md`
- **T-Ground-LanguageSpec / `MethodTemplateContract` (Decision 4)**: `src/v3/std/{rust,python,go}_method_template_contracts.dag`
- **PB Manager brief**: `docs/briefs/r2-pure-bootstrap-manager.md`
