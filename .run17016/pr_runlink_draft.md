## Execution receipts — egress-effects-1 witness (run route), bold-crane-719

Supersedes the earlier run-link comment on this PR (that one is stale: its stage-1 build was voided by the 07:47:21Z SIGTERM sweep and its route probes predate the claim-route finding).

Claim route (verified against the binary's own help at this head; corroborated by the roadmap row text on main):
`gunbc run --source-root dag --source-root src/v1 --source-root src/v2 --entry dag/test/claim/filesystem_effect_reach_wall.dag --function <claim>`, once per claim, one script in one heavy-slot acquisition.

Base under test: head `a8d7e7a4eab815982f75a3b086bb084ad822f942` (this PR).
Binary: `~/scratch/bold-crane-719/head-a8d7e7a4eab.../target/release/gunbc`, sha256 `c7f7e18e5e1d63d70f74ca154b4fc06cb3d5b5c50bd14e8679ff034b454d4c18`, 41120248 B, built from the head tree (freshness asserted against the ruling epoch with its own rc line; forced rebuild recorded).
Scripts (single authority = `~/scratch/bold-crane-719/run/`, local==srv1 sha256 parity printed at leg start): stage2_run a4ebb8fb..., run_stage 1f7d54e8..., stage_common fbc4eebe....

### Instruments that failed and were fixed, kept visible
- 11:59:30Z binary-grep instrument VOID: category error (arm lives in the .dag corpus loaded from the TREE; PR changes no Rust). Replaced by tree-side counts (VOID if any 0).
- 12:55Z / 13:32Z test-route legs: rc 2 typed refusals + one cwd panic — instrument/route defects, not tree verdicts.
- 15:29:xxZ RC=1 SCRATCH unbound; 15:34:43Z RC=1 HEAD unbound — my rewrite regressions; my own guards fired within a second; no verdict claimed; explained deaths, not infra.
- 15:46:00Z leg voided itself: `DECLARED_COUNT=0` + `VOID_STOP_CLAIM_COUNT_UNEXPECTED expected=32` — remembered constant (32 census sites) used as claim-count gate; claim population is measured `grep -cE '^test fn ' ... = 34` at this head (manager independently measured the same). RC line for that leg: RC_LINE_LOST (wrapper defect; since closed — leg script appends `STAGE2_LEG_RC=... APPENDED_BY_SCRIPT_AT=<UTC>` and run_stage forces RC=6 with VOID_RC_LINE_ABSENT if the line is missing).
- Negative test on the population gate (watched failing, in-leg): `NEGATIVE_TEST gate fired as required on wrong expected=100 (declared=34)`, `NEGATIVE_TEST_RC=6`.

### Fail-closed controls (test route; coloured by their own rc lines, never asserted as verdicts)
Observed 14:21–14:49Z against this head, rc 2 typed refusals, no panic:
`gunbc test <label> --source-root dag --source-root src/v1 --source-root src/v2` → rc 2 with
"gunbc test: --source-root is a claim-route option; an instrument subject is one registry binding, run by its own producer. the execution mode and the corpus are options on claim evaluation, which is the claim route" (all three label spellings). `claim_batch --help` rc 2 (route deleted in ad5c67a316f / gunbc#16872). Unknown target name rc 2, no panic.

### 34-claim loop (verdict leg)
RAW PASS LINES POSTED HERE WHEN THE LEG LANDS (per-claim rc lines from each invocation's own $?, aggregate SELECTED=34 DECLARED=34 N_RC_NONZERO=0 N_PANIC=0, STAGE2_LEG_RC line, heavy-slot acquisition line, real rc from run_stage).
