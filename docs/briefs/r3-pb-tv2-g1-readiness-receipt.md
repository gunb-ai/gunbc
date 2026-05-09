# R3 PB — T-V2-Retirement G-1 readiness receipt + STOP+PING (docs-only)

**Status:** READINESS RECEIPT (docs-only, STOP+PING). Authored 2026-05-01 by PB Manager continuation per dispatch on inbox #1149 (T-V2-Retirement G-1 first-consumer implementation check).

**Goal of this PR:** record verified `S-1` state on current `origin/main` HEAD, classify whether any G-1 first-consumer migration can honestly begin now, and pin the exact next-unblock order. **No code slice in this PR;** the dispatch explicitly authorized a docs-only STOP receipt if no code slice is honest.

## Live state verification (origin/main HEAD)

`origin/main` HEAD at receipt-authoring time: `f66334729 test(v3): add Tier-3 mirror perf bench skeleton (#1362)`.

| Item | Status | Authority / verification |
|---|---|---|
| `T-V2-Retirement` audit (parent doc) | LIVE | `docs/audit/t-v2-retirement-audit.md` — landed via #1338. |
| `T-V2-Retirement` migration matrix | LIVE | `docs/audit/t-v2-retirement-migration-matrix.md` — landed via #1346/#1379. |
| R3 lane row for T-V2-Retirement | LIVE | `docs/r3-structure.md` line 35 (lane named NEW 2026-04-30) + line 122 (lane structure row). |
| **S-1: PM-authored `T-V2-Retirement` worker brief landed under `docs/briefs/`** | **NOT MET** | `find docs/briefs -type f \| grep -iE "t.?v2\|tv2\|v2.?retire"` returns only `pb-substrate-pilot-v2-arithmeticop.md` (an unrelated substrate-pilot brief, not the PM-authored T-V2-Retirement worker brief). `find docs -type f -exec grep -l "T-V2-Retirement worker brief\|^# T-V2-Retirement"` excluding audit returns nothing. |
| S-2 / S-3 / S-4 | NOT MET (per audit §1) | Not blocking G-1; only blocking G-2 per audit lines 13-14. |
| Population B test consumers (§3.1, §3.2) | EXIST (substantive v2 deps) | `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs` (`use v2_compiler::*`) + `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` line 991-1005 (`v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`). |
| Population B Cargo edges (§3.3) | EXIST | `src/v3/compiler/Cargo.toml:32-33` — `v2-compiler = { path = "../../v2/stage0" }` + `v2-compiler-tests = { path = "../../v2/tests" }`. |
| v3 evaluator runtime surface | NOT YET PRODUCING the equivalence target | R2-Evaluator PR-A through PR-E lane in flight per `docs/briefs/r2-evaluator-manager.md`; PR-A.1 (`Value` carrier) and Item 4 (PB-Runtime interpreter-as-data) gating the equivalence-corpus surface §3.1's "Replace with a v3 evaluator equivalence-corpus row" disposition consumes per `docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md` Seed (1)–(3). |

## Disposition: STOP+PING — no honest code slice today

Per the matrix's STOP rules:

- **§3.1 STOP:** "S-1 unmet → no migration." (`p0_std_render_repeat_string_test.rs` disposition.)
- **§3.2 STOP:** "S-1 unmet → no migration. Substrate-side authority migration not landed → parity test cannot be safely retired (would lose drift detection)." (`m2_substrate_inhabitance_test.rs` disposition.)
- **§3.3 STOP:** "Either §3.1 or §3.2 still has substantive `\bv2_compiler(_tests)?\b` references → cannot delete without breaking the build." (Cargo edges.)

S-1 is the gate that opens **either** §3.1 or §3.2. Without S-1's PM-authored disposition routing, neither test consumer can be migrated honestly:

- **§3.1 (`p0_std_render_repeat_string_test.rs`):** the matrix offers two dispositions ("replace with v3 evaluator equivalence-corpus row" *vs.* "delete as redundant with structural-guarantee receipt"). Picking between them is S-1's job, not PB's unilateral call. Beyond S-1's choice, the "replace" path additionally consumes the v3 evaluator surface (R2-Evaluator) which is in flight, not landed.
- **§3.2 (`m2_substrate_inhabitance_test.rs`):** the migration is cross-program (Substrate Manager owns the `kernel_algebra_profile` authority migration to `dsl/std/algebra.dag` or named successor; PB owns the parity-test retirement). Both sides need S-1's routing before either acts; PB cannot retire the parity test before Substrate's authority migration lands or drift detection is lost.

A `.dag` / Rust code change today would either fabricate S-1's disposition choice (forbidden by the audit's STOP rules) or ship an unrouted code edit (would silently make a unilateral PB call on a cross-program migration). Neither is honest. Therefore: **STOP+PING with this docs-only readiness receipt.**

## Next-unblock order (verbatim from audit dependency DAG, §3 / line 119)

1. **S-1 lands** — PM authors the `T-V2-Retirement` worker brief under `docs/briefs/` (PM-owned per audit table line 19).
2. **G-1 dispositions authorized** — S-1 picks per-test dispositions for §3.1 (replace vs. delete) and §3.2 (cross-program migration sequencing).
3. **G-1 implementation begins** — PB worker dispatched per S-1's brief; §3.1 first slice (smaller, no Substrate-side coupling), §3.2 second slice (cross-program, gated on Substrate authority migration), §3.3 last (mechanical Cargo edge deletion once both §3.1 + §3.2 are green).
4. **G-1 green** — both Population B test files dropped + Cargo edges removed; `cargo test -p v3-compiler` passes without v2 crates.
5. **G-2 prerequisites** (per audit §3.2): S-1 + S-2 + S-3 + S-4 + G-1 all green.
6. **G-2 implementation** — `src/v2/stage0` + `src/v2/tests` workspace members removed; `src/v2/` directory deleted.

This receipt closes step 0 (the readiness check). Nothing else can land from PB territory until S-1 lands.

## Constraints honored (verbatim from dispatch)

- ✅ No deletion of `src/v2/` or workspace members (no code slice).
- ✅ No removal of `v2-compiler` / `v2-compiler-tests` Cargo edges (no code slice; §3.3 STOP cited).
- ✅ No migration of `kernel_algebra_profile` (cross-program; Substrate-side authority not landed).
- ✅ No `verification.dag` convergence decision from PB (out of scope).

## What this PR is

A single new docs-only file (`docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`) plus a link-only registration in `docs/briefs/r2-pure-bootstrap-manager.md` sub-briefs list. No implementation-progress claim.

## Cross-refs

- Parent audit: [`docs/audit/t-v2-retirement-audit.md`](../audit/t-v2-retirement-audit.md) (#1338).
- Per-surface migration matrix: [`docs/audit/t-v2-retirement-migration-matrix.md`](../audit/t-v2-retirement-migration-matrix.md) (#1346/#1379).
- R3 lane structure: [`docs/r3-structure.md`](../r3-structure.md) (T-V2-Retirement row at line 35 + line 122).
- v3 evaluator surface (gating §3.1 "replace" disposition): [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md).
- Equivalence-corpus seed for §3.1 "replace" disposition: [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md).
- Substrate-fact-introduction procedure (§3.2 cross-program escalation): [`INVARIANTS.md`](../../INVARIANTS.md) §P1.
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md).
- Live source paths cited (anchor for future workers): `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs`, `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:991`, `src/v3/compiler/Cargo.toml:32-33`.
