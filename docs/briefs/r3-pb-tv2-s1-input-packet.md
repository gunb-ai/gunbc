# R3 PB — T-V2-Retirement S-1 input packet (input to PM/Director; not S-1 itself)

**Status:** INPUT PACKET (docs-only). Authored 2026-05-02 by PB Manager continuation per dispatch on inbox #1149 (post-#1446 follow-up).

**This is NOT the S-1 worker brief.** S-1 is the PM-authored `T-V2-Retirement` worker brief that audit/migration-matrix STOP conditions name as the prerequisite for G-1 implementation. PB cannot author S-1 (the audit names PM as owner). This packet is what S-1's author needs from PB territory: a decision checklist enumerating each call S-1 must make, with PB's recommended default per existing audit support, and the owner (PM / PB / Substrate / Director) per decision.

## Live state verification

`origin/main` HEAD at packet-authoring time: `66edec52` (same lineage as #1446 readiness receipt, refreshed via `git fetch origin main`).

| Item | Status |
|---|---|
| `T-V2-Retirement` audit (#1338) | LIVE |
| `T-V2-Retirement` migration matrix (#1346/#1379) | LIVE |
| G-1 readiness receipt (#1446) | LIVE |
| **S-1 PM-authored worker brief** | **STILL NOT MET** — `find docs/briefs -type f \| grep -iE "t.?v2\|tv2\|v2.?retire"` returns only `pb-substrate-pilot-v2-arithmeticop.md` (substrate-pilot, unrelated) and `r3-pb-tv2-g1-readiness-receipt.md` (PB receipt, not the PM brief) and this input packet. **If a PM-authored S-1 worker brief landed since #1446, this packet is itself a STOP**: PB resumes G-1 implementation per the dispatch chain, not authors more docs. |

## Decision checklist for the PM-authored S-1 worker brief

Each row is a decision S-1 must make. **Recommended default** is what existing audit / migration-matrix already converge on; S-1's job is to either ratify that default or pick differently with a receipt. **Owner** is who makes the call (and who executes if different — split named where applicable).

### Decision 1 — §3.1 (`p0_std_render_repeat_string_test.rs`) disposition

| Field | Value |
|---|---|
| **Question** | Replace the v2 oracle with a v3 evaluator equivalence-corpus row, OR delete the test as redundant with a structural-guarantee receipt? |
| **Recommended default** | **Replace.** Aligns with the broader PB-Runtime ↔ R2-Evaluator equivalence-corpus direction (cf. migration-matrix §3.1 "Proposed migration" + `r3-pb-runtime-equivalence-corpus-seed-audit.md` general framing — the seed audit's three named seed classes are not binding here; `repeat_string` is a `std.render` / lower-time fold / `String` result and doesn't map onto a specific seed class). Preserves the property under test (lower-time fold of `repeat_string` to a string literal). The "delete with receipt" path is acceptable only if the lower-time fold is structurally guaranteed by v3 typed primitive composition such that no oracle is needed. |
| **Pre-requisite for the "replace" path** | R2-Evaluator surface live enough to host an equivalence-corpus row — currently in flight per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E. The "delete" path has no R2-Evaluator dependency. |
| **Owner** | **PM** picks (audit §3.1 explicitly names this as S-1's choice). **PB** executes once chosen. |

### Decision 2 — §3.2 (`m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`) cross-program routing

| Field | Value |
|---|---|
| **Question** | Which v3-side authority replaces `v2_compiler::std_algebra::kernel_algebra_profile()` as single source? Who owns the authority migration vs. the parity-test retirement? What's the sequencing? |
| **Recommended default (authority)** | **`dsl/std/algebra.dag`** (or its successor as v3 substrate inhabitance consolidates). Matches matrix §3.2's "Proposed migration" row + the broader §P1 substrate-fact-introduction discipline (single authority on v3 side, no Rust↔.dag mirror to drift). |
| **Recommended default (ownership split)** | **Substrate Manager** owns the authority migration (substrate shape — single-source `kernel_algebra_profile` lands on v3 side per §P1). **PB Manager** owns the parity-test retirement once the v2 mirror is no longer the authority. |
| **Recommended default (sequencing)** | Substrate-side authority migration **first**; then PB retires the parity test. Reverse order would lose drift detection (per matrix §3.2 STOP cell). |
| **Pre-requisite** | Substrate Manager dispatch the authority-migration worker; v3-side single-authority `kernel_algebra_profile` lands. |
| **Owner** | **PM** picks routing (per audit §3.1 ownership table). **Substrate** executes authority migration. **PB** executes parity-test retirement. |

### Decision 3 — §3.3 Cargo edges deletion mechanics

| Field | Value |
|---|---|
| **Question** | Atomic deletion with §3.1 + §3.2 in the same retirement PR, OR follow-up PR after both tests dissolve? |
| **Recommended default** | **Atomic with the second-of-§3.1-or-§3.2-to-land** (whichever closes last). Per matrix §3.3 STOP: "Either §3.1 or §3.2 still has substantive `\bv2_compiler(_tests)?\b` references → cannot delete without breaking the build." Pre-emptive deletion breaks the live tests; post-hoc-follow-up deletion leaves the audit's INVARIANTS §P2 (Boundary Discipline / single authority) parallel-authority residue alive between the two PRs. Atomic-with-the-last is the cleanest. |
| **Pre-requisite** | Both §3.1 + §3.2 dispositions complete. |
| **Owner** | **PB** executes; **PM** ratifies sequencing if it differs from the recommended default (no PM-only choice required if default is taken). |

### Decision 4 — Legacy emit chain (`rust_method_template_contracts.dag` header note)

| Field | Value |
|---|---|
| **Question** | When does the legacy `rust_simple_method_specs` / `rust_method_templates()` / `rust_method_wraps_result()` chain delete from `dsl/extdeps/languages/rust/emit.dag` (and parallel python/go chains)? What's the cross-lane coupling with T-Ground-LanguageSpec scope E? |
| **Recommended default** | **Delete once PB-Runtime trampoline is the live bootstrap (S-4) AND v3-side `MethodTemplateContract` rows are consumed by the v3 emitter end-to-end.** Matrix §"Legacy emit chain" already converges on this gate. Per matrix's "Owner" cell: PB Manager + cross-ref `r3-pb-binshim-retirement-worker.md` + T-Ground-LanguageSpec scope-E lineage. |
| **Pre-requisite** | S-4 (PB-Runtime trampoline live) + v3 emitter end-to-end consumption of `MethodTemplateContract` rows under `src/v3/std/{rust,python,go}_method_template_contracts.dag`. |
| **Owner** | **PM** marks the gate (S-1 brief enumerates the prerequisite chain). **PB** executes deletion once gate clears. Cross-ref T-Ground-LanguageSpec lane authority for the v3-emitter consumption side. |

### Decision 5 — `verification.dag` convergence routing

| Field | Value |
|---|---|
| **Question** | Does v3's `TestPredicate` / `TestSuite` model fully replace v2's `AssertKind` / `TestClaim` / `TestCase` model, OR does some v2 surface continue under a renamed module path? Who owns the design call? |
| **Recommended default** | **Routed to Substrate Manager (design call); PB does not pre-empt.** Matrix §"verification.dag convergence" explicitly names this as Substrate's call. The audit section explicitly says: "convergence of `verification.dag` is a **prerequisite for G-2** (deleting `src/v2/` requires that no surviving authority depends on the v2 `verification.dag` surface). It is **NOT** a prerequisite for G-1." S-1 should explicitly route this to Substrate Manager (or escalate to Director if Substrate cannot scope) rather than silently deferring; otherwise G-2 stalls without a named owner. |
| **Pre-requisite** | None for G-1 (this decision is G-2-only per audit §"verification.dag convergence" position). |
| **Owner** | **Substrate Manager** (design call). **PB** for v2-side cleanup once the call lands. **PM** explicitly routes the decision to Substrate in the S-1 brief; **Director** arbitrates if Substrate cannot scope it. |

### Decision 6 — S-1 brief scope coverage

| Field | Value |
|---|---|
| **Question** | Does S-1 cover only G-1 dispositions (§3.1 + §3.2 + §3.3), or also enumerate the S-2 / S-3 / S-4 prereq chain that gates G-2? |
| **Recommended default** | **Cover both.** Audit §1 G-2 prereq stack is `S-1 + S-2 + S-3 + S-4 + G-1`. S-1 brief enumerating only G-1 work leaves G-2 without a single-doc dispatch surface; later workers would have to reconstruct the S-2/S-3/S-4 chain from the audit. Cleaner to bundle once at S-1 authoring time. |
| **Pre-requisite** | None (this is meta-scope). |
| **Owner** | **PM** picks. (PB recommends; not PB's call to make.) |

## Summary table — owners and sequencing

| Decision | Owner | Executor | Recommended default | Counter-default cost |
|---|---|---|---|---|
| 1 — §3.1 replace vs delete | PM | PB | Replace (corpus-row) | "Delete" path needs structural-guarantee receipt; without it, drops oracle without replacement. |
| 2 — §3.2 routing | PM | Substrate (authority) + PB (test) | Substrate-side authority migration first → PB retires parity test | Reverse order loses drift detection during the gap. |
| 3 — §3.3 Cargo edges | PB | PB | Atomic with last-of-§3.1-or-§3.2 PR | Pre-emptive deletion breaks build; post-hoc leaves parallel-authority residue. |
| 4 — Legacy emit chain | PM (gate) + PB (execute) | PB | Delete on S-4 + v3 emitter end-to-end consumption | Earlier deletion breaks emit; later leaves dead code. |
| 5 — `verification.dag` convergence | Substrate (design); Director (arb if stuck) | PB (v2-side cleanup) | Route to Substrate explicitly | Silent deferral stalls G-2 without named owner. |
| 6 — S-1 scope (G-1 only vs G-1 + G-2 chain) | PM | PM (authoring) | Cover both | G-1-only leaves G-2 reconstructed by later workers. |

## What this packet is

A single new docs-only file (`docs/briefs/r3-pb-tv2-s1-input-packet.md`) plus a link-only registration in `docs/briefs/r2-pure-bootstrap-manager.md` sub-briefs list. Explicitly labeled as **input to S-1, not S-1 itself**. PB does not author S-1; PM does.

If this packet's defaults are useful, S-1's author can adopt them and cite this packet as the rationale source. If S-1's author picks differently, this packet records the alternative each row's "Recommended default" was chosen against.

## Constraints honored (verbatim from dispatch)

- ✅ No code changes.
- ✅ No `src/v2/` deletion.
- ✅ No `v2-compiler` / `v2-compiler-tests` Cargo edge removal.
- ✅ No `kernel_algebra_profile` migration decision from PB (Decision 2 routes to Substrate per §P1; PB recommends, doesn't decide).
- ✅ No `verification.dag` convergence decision from PB (Decision 5 routes to Substrate; PB explicitly does not pre-empt).
- ✅ No claim of G-1 implementation unblocked.

## Cross-refs

- Parent audit: [`docs/audit/t-v2-retirement-audit.md`](../audit/t-v2-retirement-audit.md) (#1338).
- Per-surface migration matrix: [`docs/audit/t-v2-retirement-migration-matrix.md`](../audit/t-v2-retirement-migration-matrix.md) (#1346/#1379).
- G-1 readiness receipt: [`docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`](r3-pb-tv2-g1-readiness-receipt.md) (#1446).
- Equivalence-corpus seed (Decision 1 "replace" path): [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md).
- R2-Evaluator manager (Decision 1 prereq): [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md).
- T-Ground-LanguageSpec / `MethodTemplateContract` (Decision 4 cross-ref): `src/v3/std/rust_method_template_contracts.dag`, `src/v3/std/python_method_template_contracts.dag`, `src/v3/std/go_method_template_contracts.dag`.
- BinShim retirement program (Decision 4 cross-ref): [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
- Substrate-fact-introduction procedure (Decision 2 routing): [`INVARIANTS.md`](../../INVARIANTS.md) §P1.
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md).
