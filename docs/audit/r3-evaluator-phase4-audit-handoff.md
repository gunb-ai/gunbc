# R3 Evaluator Phase 4 Audit Handoff

**Status:** handoff packet, 2026-05-06. Docs/audit only. This packet
does not implement evaluator behavior, edit substrate declarations, touch
generated manifests, alter CI, or change runner code.

**Phase 3 anchor:** PR #1838 merged on 2026-05-06 as the
`docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md` receipt for the
#1275-#1500 evaluator slice. That receipt is the source for the live queue
below; this file turns it into the next actionable audit shape.

**Sibling receipt:** PR #1839
(`docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md`) merged on
2026-05-06 as the sibling #1500-#1803 receipt.

## Live Residuals From #1838

These items remain live after the merged #1275-#1500 receipt:

- **G1.a static lens fold/report production:** `docs/briefs/r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md`
  is the static top-level `Lens<C>` fold/report slice. The live pressure is
  report production through declared substrate values, not a host-side mirror.
- **G1.b generic dispatch:** generic `fold_lens<C>` remains held on X1.b
  S1/S3. The relevant boundary is in `docs/briefs/x1b-evaluator-impact-audit.md`
  and the G1.a dispatch packet: no parameter-headed runtime callee shortcut.
- **Descent proof consumer:** `LoopBound::Descent` stays a fail-closed
  evaluator residual. `docs/briefs/r3-pr-e5-loop-readiness-audit.md` keeps
  Descent outside cardinality-loop execution until a Substrate termination
  proof carrier exists and the evaluator consumes it.
- **SymbolicCost runner predicate:** `SymbolicCostExprEquals` is outside the
  body evaluator. It belongs to ROADMAP Pattern B / runner predicate authority
  and `docs/briefs/r2-evaluator-test-runner-authority-ratchet.md`, not to
  `src/v3/compiler/src/lib.rs` body-evaluator closure.

## Next Audit Queue

The next queue should not reopen #1275-#1500 or #1500-#1803. It should
consume #1838 plus merged sibling #1839 and then sweep the
**post-#1803 evaluator authority surface**.

**Queue name:** R3 Evaluator post-#1803 authority sweep.

**Primary range:** merged PRs **#1804 onward** that touch evaluator-owned
or evaluator-consuming surfaces:

- `src/v3/compiler/src/lib.rs` evaluator functions and runtime `Value`;
- `src/v3/compiler/src/lens_apply.rs` only when it is cited as an evaluator
  compatibility seam, not as generic fold authority;
- `src/v3/compiler/src/test_runner.rs` predicate/producer arms;
- evaluator dispatch/readiness docs under `docs/briefs/`;
- evaluator audit receipts under `docs/audit/`;
- ROADMAP Pattern B / Pattern C rows when they name evaluator or runner
  predicate authority.

**Trigger to fire the queue:** start this sweep when both of these are true:

1. The merged #1839 receipt is available to the Phase 4 compile.
2. The next evaluator implementation/audit PR after #1803 lands or receives
   blocking review that changes one of the live residuals above.

If no implementation PR fires the second condition, the Director / Evaluator
Manager can still fire the queue manually for the next R3 debt-sweep compile
pass.

## STOP Conditions

Stop and route back to the Evaluator Manager instead of landing a local
classification if the sweep finds any of these:

- **Callable state contradiction:** live `eval_transform_node` behavior and
  evaluator dispatch/readiness docs disagree on whether a callable surface is
  executable, retired, or still fail-closed. Record the conflict; do not choose
  one authority locally.
- **G1.a/G1.b boundary blur:** any PR claims generic `fold_lens<C>` behavior
  while X1.b S1/S3 is still held, or uses host Rust to bypass runtime-callee
  dispatch.
- **Descent without proof carrier:** any evaluator consumer treats
  `LoopBound::Descent` as executable without a named Substrate termination
  proof carrier and fail-closed residual taxonomy.
- **Runner predicate expansion:** any new `test_runner.rs` predicate/producer
  arm lands without a dissolution hook into evaluator/PB-runtime authority or
  without a Pattern B row.
- **New runtime mirror:** any new host-side `Value`, report, witness, lens, or
  dimension carrier appears without a same-PR bridge class and dissolution
  trigger.
- **Unverifiable PR-range claim:** any receipt cites off-branch PR comments,
  issue comments, or stale line numbers as sole authority for a live state.
  Replace with in-repo files, PR metadata, or STOP.

## Phase 4 Compile Handoff

The Phase 4 compile should combine:

- #1838 / `docs/audit/r3-evaluator-pr-1275-1500-debt-sweep.md` for the
  #1275-#1500 slice;
- #1839 / `docs/audit/r3-evaluator-pr-1500-1803-debt-sweep.md` for the
  #1500-#1803 slice;
- this packet's post-#1803 queue definition.

Do not infer global debt counts from this packet. Its job is to preserve the
next evaluator audit boundary and STOP criteria so the broader
`docs/audit/r3-debt-sweep-2026-05-06.md` compile can consume evaluator evidence
without silently closing live residuals.
