# R3 Wave-1 Substrate lane — retractions (S2 + S4)

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-12
**Authority**: codex BLOCKING review on PR #2782 (sha post-fix-forward 4-batch). Per `feedback_redirect_noop_prs`: retract briefs that route workers into wrong work; document the retraction so the trail survives.

## Retraction 1 — S2 (#85 + #86 quantifier + generator carriers)

**Reason**: Stale against gate-state authority. Per `docs/r3-program-plan.md:311-312`:
- **#85** `forall_exists_quantifier_substrate_landed`: DECLARED; **carriers already landed via PR #2647** (`Quantifier { ForAll, Exists }`, `QuantifiedTestClaim`, `SuiteClaim` in `src/v3/std/verification.dag`). Remaining work is **consumer-side** (SuiteClaim wrapper migration of `TestSuite.claims` per design §6 line 344 + V Mgr #87 generated/runner consumer) — that's NOT Substrate-lane scope; it's Verification-lane / V-Mgr cascade.
- **#86** `program_generator_carrier_landed`: **CONSUMER_LANDED + PASSING**. Already complete. No work remains.

**Action**: original S2 brief (`docs/briefs/r3-wave1-s2-quantifier-and-generator-carriers-worker.md`) deleted. No replacement worker brief authored — Substrate lane has no work on #85/#86 at this point. If V Mgr surfaces a Substrate-side substrate-prereq for the #85 SuiteClaim wrapper migration, that's a separate brief.

## Retraction 2 — S4 (#36 bridge_retirement_ledger_zero)

**Reason**: Wrong lane ownership. Per `docs/r3-structure.md:214` + `docs/r3-program-plan.md:391` (Director-locked 2026-04-28 distribute-work-centralize-ledger discipline):

- **#36 `bridge_retirement_ledger_zero` is Verification-owned** — coordinates cross-program reporting cadence; audit gate for the 5-bridge distribution map
- The 5 distributed bridges are NOT #36; they're separate gates:
  - 2 Substrate-owned: `SourceSpan.file` participation + `mark_bootstrap_secret_nominal_opacity()`
  - 3 PB-owned: canonical lens-name dispatch + `include_str!` side channels + `patch_lower_helpers_*`

**Action**: original S4 brief (`docs/briefs/r3-wave1-s4-bridge-retirement-ledger-zero-worker.md`) deleted. If the 2 Substrate-owned bridges need brief authoring, that's separate scope; PM (deep-wolf-155) coordinates with V Mgr (clever-tern-670) for the ledger audit gate.

## Wave-1 Substrate lane queue post-retraction

Revised queue (5 items, down from 7):
- ~~S2 (#85 + #86)~~ — **RETRACTED** (already landed / consumer-side is V-lane)
- ~~S4 (#36)~~ — **RETRACTED** (Verification lane ownership)
- S1 #81 parallelism walker port (substrate-ready)
- S3 #82 F-β.1 canvas (Director ratification target)
- S5 #73 lens behavioral parity (gates on S1 + F-β.2 landing for full 4-lens coverage)
- S6 #103 Slice 7 affected-set impl (gates on PR #2766)
- S7 #98 Slice 5 BinaryShim body pre-stage (gates on PR #2774 merge)

## Cross-link

PM (deep-wolf-155) msg_a63f5e7e: original 7-item Wave-1 Substrate catalog. This retraction document is the audit-trail receipt for the 2-item drop.

Surface to PM: Substrate lane capacity-budget post-retraction is +13 (was +15; spawn 5 instead of 7 once Wave-1 brief PR #2782 merges).
