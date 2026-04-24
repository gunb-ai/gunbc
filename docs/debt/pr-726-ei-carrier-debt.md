# PR #726 (quiet-eagle-364) — E-I / E-C carrier staging: accepted debt receipt

**Status:** Ship-with-debt closure (2026-04-24). This file is the canonical follow-up list so the review loop does not re-litigate the same marginal findings on merge.

**Authority:** OpenAI-pro loop summary (SHIP_WITH_DEBT); in-tree facts from the landed diff (Peano carriers, fail-closed numeric helpers, `CallPattern` lowering, substrate tests).

## Ingested dashboard relays (audit log)

Scheduled / relayed GitHub **api-review** context is copied here so the queue does not need re-parsing; **verdicts bind to the SHA named in the relay**, not necessarily current `HEAD`.

| Ingested (UTC) | Relay | SHA reviewed | Verdict | Summary |
|----------------|-------|--------------|---------|---------|
| 2026-04-24 19:33 | `[api-review]` codex / codex-default (briansrls on #726) | `8f8ab27c` | **APPROVE** | No invariant / modeling-discipline / coding / testing violations asserted; scaffolds documented with bounded triggers; Peano / int bridges fail-closed. Verified tests cited in relay: `v2-compiler-tests` (int pow, Peano cap, sub-value lattice), `v3-compiler` E-I bootstrap CostBound test. Dashboard indicated **+12 more queued** after this comment — not expanded here; triage under **§ Pending dashboard** on `HEAD` if still open. |
| 2026-04-24 20:18 | `[api-meta-review]` openai-pro / gpt-5-5-pro (manual; briansrls on #726) | `8f8ab27c` | **SHIP_WITH_DEBT** | Aggregate loop summary (~45 review artifacts / ~10h window in relay); direction converged — remaining risk is **consumer** debt (E-P / E-M), not wrong carriers. Next useful work is a **structural slice** (E-P per-call evidence consuming `SumBound` / lattice semantically, `promote_to_strict` rename/delete, structural param refs, or unified numeric refinement) — aligns with **§ Tracked follow-ups**. External conversation: `https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69ebce73-ab5c-83ea-b605-89b6e3be759c`. Meta-review suggests making **E-P / E-M** the next convergence gate in `ROADMAP.md` (optional doc follow-up). Dashboard: **+11 queued** after this comment. |
| 2026-04-24 20:20 | `[api-review]` claude / claude-opus-4-7 (schedule; briansrls on #726) | `d743ec7a` | **APPROVE** | E-I carrier staging: Peano / fail-closed numerics / receipts / tests match modeling + TESTING.md posture; scaffolds have named triggers in this file. **Superseded at HEAD:** relay’s minor note that `ParserAdvanceCall` / `WorklistDrainCall` lowered only to `CollectionSize` — later commits use **`ParserStreamSize` / `WorklistDrainSize`** (see **§ What already landed**). Dashboard: **+10 queued** after this comment. |

## What already landed (not debt)

- Raw shrink/divisor slots moved toward **structural Peano carriers** (`PositiveDescentAmount`, `ProportionalDivisor`) with M9-style caps and tests.
- **Fail-closed** cost bridges: `int_pow_bounded` / `ceil_log` / `master_theorem` with `bounded_int_pow_exponent`, degenerate-base fast paths for `int_pow_bounded`, and regression tests (Peano cap, `int_pow`, lattice factors).
- **`SubValueRelation` meet/join** require matching shrink factors; commutativity / mismatch tests lock the lattice rule.
- **`lower_call_pattern`**: `CallPattern` carries **forwarded** `String` slots (`collection`, `ring_param`, `witness`, `element`, `outer_collection`) so lowering does not fabricate placeholder names like `"n"` / `"collection"` for those paths. Parser advance vs worklist drain lower to **distinct** `SizeBound` variants (`ParserStreamSize` / `WorklistDrainSize`) so stream vs set-removal identities are not collapsed into generic `CollectionSize` (dashboard review 2026-04-24).
- **`ProportionalDivisor` (Rust mirror):** `src/v3/compiler/src/dag.rs` carries a **TERMINAL** dissolution receipt aligned with `std.termination` (same review pass).
- **Receipt consumers:** `m2_substrate_inhabitance_test`, `e_i_lane_induction_preflight_test`, `sub_value_lattice_factor_test`, Peano / `int_pow_bounded` v2 tests — enough to prove carriers parse, mirror, and lower structurally; **not** enough to claim v3 cost/complexity lenses are behaviorally complete.

## Tracked follow-ups (explicit triggers)

| Item | Debt class | Dissolution trigger | Next owner |
|------|------------|---------------------|------------|
| **Live `SumBound` consumer** | E-I vocabulary ahead of behavior | Lane **E-P** attaches per-call cost / evidence producers; then a real cost pass consumes `SumBound` semantically, not only `cost_bound_is_sum_bound` shape tests | E-P lane (`docs/design-substrate-carrier-port-program.md`) |
| **`SizeBound` / `CallPattern` string bridges** | P2 string bridges | `ParserStreamSize.witness` / `WorklistDrainSize.element` / `CollectionSize.param` / … still use `String` until structural size-parameter refs on substrate (same wave as E-P reflected params) | E-P + modeling |
| **`promote_to_strict` name** | P5 misleading bridge | Parser progress threads **`Strict`** at the witness; **rename** helper to match behavior or **delete** call sites | E-T / parser lane + termination.dag |
| **Unified numeric / M9 authority** | Layer-agnostic invariant gap | Single refinement or shared `PositiveInt`-style authority for literal bridges across `std.termination` / `std.computation` / `std.induction` (collapses duplicate 256-cap story) | E-P lane; see ROADMAP P4 Peano ratchet row |
| **Cost / complexity lenses** | PROXY honesty | **E-P** (per-call evidence) + **E-M** (`MethodSemantics`) per `docs/v3-lens-capability-register.md` | Register + port program |

## Merge posture

Further **review-only** passes on this PR should target **one** of: structural param refs, `promote_to_strict` dissolution, a live `SumBound` consumer, or shared refinement — not additional local numeric-only patches unless a regression appears.

## Pending dashboard / inline review (follow-up only)

**Intent:** Anything still showing as “queued” or “pending” in GitHub/dashboard **after** the 2026-04-24 inline pass on this branch should be treated as **follow-up triage on HEAD**, not a merge gate, unless a fresh review run reproduces a real defect.

| Source | Recorded action |
|--------|-----------------|
| Inline threads queued **after** parser/worklist lowering + `ProportionalDivisor` receipt | Owner: re-verify on latest commit; reply **stale** only with a HEAD cite, or open a **new** tracked row above if still real. |
| API-review exploratory notes (numeric literal caps in `induction.dag`, `shrink_factor_eq` cost if Peano caps rise, synthetic `_fold_body` / `_scc_arithmetic` names) | Already bounded by rows **Unified numeric / M9** and **string bridges**; no separate PR unless scope grows. |
