# PR #726 (quiet-eagle-364) — E-I / E-C carrier staging: accepted debt receipt

**Status:** Ship-with-debt closure (2026-04-24). This file is the canonical follow-up list so the review loop does not re-litigate the same marginal findings on merge.

**Authority:** OpenAI-pro loop summary (SHIP_WITH_DEBT); in-tree facts from the landed diff (Peano carriers, fail-closed numeric helpers, `CallPattern` lowering, substrate tests).

## What already landed (not debt)

- Raw shrink/divisor slots moved toward **structural Peano carriers** (`PositiveDescentAmount`, `ProportionalDivisor`) with M9-style caps and tests.
- **Fail-closed** cost bridges: `int_pow_bounded` / `ceil_log` / `master_theorem` with `bounded_int_pow_exponent`, degenerate-base fast paths for `int_pow_bounded`, and regression tests (Peano cap, `int_pow`, lattice factors).
- **`SubValueRelation` meet/join** require matching shrink factors; commutativity / mismatch tests lock the lattice rule.
- **`lower_call_pattern`**: `CallPattern` carries **forwarded** `String` slots (`collection`, `ring_param`, `witness`, `element`, `outer_collection`) so lowering does not fabricate placeholder names like `"n"` / `"collection"` for those paths.
- **Receipt consumers:** `m2_substrate_inhabitance_test`, `e_i_lane_induction_preflight_test`, `sub_value_lattice_factor_test`, Peano / `int_pow_bounded` v2 tests — enough to prove carriers parse, mirror, and lower structurally; **not** enough to claim v3 cost/complexity lenses are behaviorally complete.

## Tracked follow-ups (explicit triggers)

| Item | Debt class | Dissolution trigger | Next owner |
|------|------------|---------------------|------------|
| **Live `SumBound` consumer** | E-I vocabulary ahead of behavior | Lane **E-P** attaches per-call cost / evidence producers; then a real cost pass consumes `SumBound` semantically, not only `cost_bound_is_sum_bound` shape tests | E-P lane (`docs/design-substrate-carrier-port-program.md`) |
| **`SizeBound.param` / `CallPattern` strings** | P2 string bridges | Structural size-parameter refs on substrate (same wave as E-P reflected params) | E-P + modeling |
| **`promote_to_strict` name** | P5 misleading bridge | Parser progress threads **`Strict`** at the witness; **rename** helper to match behavior or **delete** call sites | E-T / parser lane + termination.dag |
| **Unified numeric / M9 authority** | Layer-agnostic invariant gap | Single refinement or shared `PositiveInt`-style authority for literal bridges across `std.termination` / `std.computation` / `std.induction` (collapses duplicate 256-cap story) | E-P lane; see ROADMAP P4 Peano ratchet row |
| **Cost / complexity lenses** | PROXY honesty | **E-P** (per-call evidence) + **E-M** (`MethodSemantics`) per `docs/v3-lens-capability-register.md` | Register + port program |

## Merge posture

Further **review-only** passes on this PR should target **one** of: structural param refs, `promote_to_strict` dissolution, a live `SumBound` consumer, or shared refinement — not additional local numeric-only patches unless a regression appears.
