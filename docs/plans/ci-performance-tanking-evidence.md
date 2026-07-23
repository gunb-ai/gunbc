# Why CI performance is tanking - the evidence, argued serially (2026-07-10/11 main-red incident)

One-line: main CI went red at 2026-07-10T21:58Z and has stayed red; the cause is NOT one thing — it is one latent-red exposure plus three cost terms stacking inside fixed 60/45-minute job budgets, and the loudest prior narrative (per-witness resolve cost in CI) is refuted by the floor's own receipts. Every claim below carries the receipt that grounds it (GitHub Actions run id + step timing, a floor receipt line from that run's log, or a controlled single-variable A/B). Method note: job WALLS are confounded (concurrent runs, cold caches) — all attribution below is at STEP or receipt grain.

## 1. Baseline: what green main looked like (through 2026-07-10 21:39Z)

Five consecutive green main runs that afternoon; the ci job's wall and its floor step:

| run | merge | ci job wall | release-build step | floor step |
| --- | --- | --- | --- | --- |
| 29115650413 (18:45) | witness cost backout #6437 | 17m44s | 8s (warm) | 16m56s |
| 29120303840 (20:07) | spine reconcile | 18m22s | - | - |
| 29121074294 (20:23) | Wave 1C census | 25m43s | - | - |
| 29124032694 (21:13) | symbol_index | 21m00s | - | - |
| 29124587375 (21:24) | body-producer Stage A | 20m41s | 1m47s | 18m15s |

Floor anatomy at this baseline (from run 29125380927's gantt, the last roster-era floor): batch 1 compile-clean 29s; batch 2 (the witness-bearing batch, incl. a small discovery node) 46 SECONDS; batch 3 (emit-host, ingest, self-host comparison) 2m05s; batch 4 emit_determinism_gate 12m25s. The roster-era floor's wall is the emit-determinism gate, not witnesses: the enrolled surface was the opt-in roster (67 check fns after #6437). Demand receipt at this grain: keyed_calls=2,313,593 (run 29123412337 floor materialization receipt).

## 2. Event 1 - #6441 (my merge, 21:39Z): no floor regression; a 33-minute BUILD term exposed

#6441 landed the demand ledger (trace default-on in claim_executor) and the eval-frame memo. Two separately-receipted facts:

- (2a) The CI floor did NOT slow: floor step 16m04s (inside the 16m56s-18m15s baseline band); batch 2 still 46s (run 29125380927 steps + gantt). Claimed earlier as a +30min memo regression - that attribution was WRONG.
- (2b) The +30min in the job wall was the release-build step: 34m09s cold (the .rs change invalidated sccache) vs 8s/1m47s warm in the baseline runs. Second receipt of the same law: the next run (flip, also post-.rs-change) paid 32m42s. General form: ANY .rs-touching merge pays roughly 33 minutes of build inside the 60-minute job budget, leaving about 27 minutes for everything else.

## 3. Event 2 - #6438 the affected-set flip (21:58Z): enrollment changed, three consequences

The flip replaced the 8-witness opt-in roster with tree-wide discovery shrunk by the diff's affected set (that run enrolled 2 roots + 72 explicit entries out of ~1,721 discoverable). All three consequences are receipted from the first flip run, 29126347183:

- (3a) LATENT-RED EXPOSURE: the discovery-corpus node FAILED (not timed out) on emit_directive_rust_test.dag - a resolve error (names rust_emit_directive_emitted_call / rust_emit_directive_target_model not found in v2.extdeps.languages.rust; stale imports in a witness that had never run under the roster). Exposure is SERIAL - one newly-red witness per CI cycle. This first one was fixed at 23:11 (run 29129707839's merge title). How many more remain is UNKNOWN: nobody has enumerated the corpus.
- (3b) EVAL DEMAND x12: floor materialization receipt keyed_calls=27,933,188 (vs 2.31M roster), unkeyed_calls=37,486,998, duplicated_keys=2,088,310, wasted_ms=3,206,488 - 53 MINUTES of duplicated-demand CPU disclosed - with the memo serving 15,556,157 hits. The corpus node ran 25m45s at spawn_width=2 before its own failure ended it.
- (3c) RESOLVE IS NOT THE CI WALL: the same run's resolve receipt reads 1 entry resolve, 22,266ms. The floor pools resolution in-process (walk memo + shared MultiEntryIndex): twenty-two SECONDS of resolve under twenty-five MINUTES of eval. Any argument that CI is slow because each witness re-resolves its closure is refuted by this receipt - that cost is real but lives in per-invocation dev loops (one process per witness), not in the pooled CI floor.

## 4. Event 3 - the memo identity tax (also #6441): invisible at roster grain, material at corpus grain

The eval memo (and the trace, which shares the same key) pays key construction BEFORE knowing hit/miss. The key is a content hash memoized PER ALLOCATION, so the per-call cost is the fresh-allocation frontier - one level of child lookups per new allocation - NOT the whole tree: quadratic only for clone-style list accumulators (a fresh spine of N elements per fold step), about O(log n) for shared HAMT structure. Fold-heavy witnesses are the clone-style case.

Controlled receipt (single variable, quiet host, release binary, eager-ram-612's harness): the 6-witness fold-heavy roster recipe's summed witness walls are 908.4s memo-on vs 290s memo-off - 3.1x, with the gradient tracking fold-heaviness (std_algebra 5.5x, lens_traversal 2.8x, cost_model 1.3x). Zero of that gap is memory retention (measured separately, section 5). Why the roster-era CI floor never showed it (2a): the roster's 46-second witness batch barely contains fold-heavy eval; the flip-era corpus (3b) is made of it. Honest bound: there is no direct CI-corpus A/B yet - the 3.1x controlled ratio plus the 53-minute wasted_ms line bound the corpus node's tax; the pinning receipt is a local corpus run in both configs (planned acceptance for the admission fix, section 8/P2).

## 5. Event 4 - the retention regression and its fix (#6456, merged 00:29Z): the OOM class, now closed

#6441's memo also evicted at ctx lifetime while batch surfaces share one InterpContext across an entry's witnesses - so stored argument+result values accumulated byte-unbounded across witnesses (count cap, not bytes). Measured: single witness plateaus ~3.4GiB; six witnesses climb ~1GiB/5s past ~17-20GiB to SIGKILL (quiet-host A/B, reproduced 3x). FIXED by witness-frame eviction (eval_call_memo_frame_exit at all four batch call sites; #6456) with an independent execution receipt: the same recipe that died pre-fix runs 6/6 PASS at 6.50GiB VmHWM. This closes the exit-137 OOM class (e.g. PR run 29123412337-era kills); it does NOT touch the timeout class or the wall tax (section 4) - three defects, three receipts, only this one was memory.

## 6. The stack-up: why every main run since 23:11 hits a ceiling

After the emit-directive fix, main's failures are all TIMEOUT class:

| run (main) | merge | failure mode |
| --- | --- | --- |
| 29129707839 (23:11) | emit-directive red fix | rust gate timed out at 45m |
| 29130137992 (23:21) | witness transports 3a | failure - mode UNINSPECTED (open item) |
| 29130433489 (23:28) | affected set processing #6453 | floor 60m timeout AND rust gate 45m timeout |
| 29130984121 (23:42) | namespace Wave 1A re-land | floor 60m timeout |
| 29132731778 (00:29) | #6456 (this lane) | floor 60m timeout |

The arithmetic inside a 60-minute job: ~33m cold build (whenever .rs changed, which last evening was most merges - section 2b) + 25m+ corpus-node eval (section 3b, carrying the section-4 tax) + gates already leaves nothing, and the evening fleet ran 5+ overlapping main runs plus PR runs plus the falsifier on ~50 shared slots (prior findings: host oversubscription kills; a 1,397-module cap-kill on the S2a branch). No single term exceeds the budget; the SUM does. Slot relief has already begun (run 29134156187 at 01:10 removed the rust_tests job to free a runner slot - operator-side action).

## 7. Disambiguation: 'witnesses are slow' is three different pains

- CI-red (acute, this doc): latent reds exposed serially by the flip (3a) + the timeout stack (2b build + 3b corpus eval x 4 tax + fleet saturation). Owners: red enumeration/fixes = anyone (P1 below); tax = this lane (P2); build/slots = measurement-infra lane (P3).
- Dev-loop witness slowness (chronic; the operator's felt pain): request-major resolution - each SEPARATE invocation pays 10-21s cold resolve for its closure prefix (receipt: 10.6s resolve for 15ms of witnesses, local floor log; the CI analog is the pooled 22s). Untouched by everything above. Fix = the cross-process tier: persistent node-table encoding (format authority settled: NodeKeyedGraphArtifact, content-hash-keyed rows) then the resolved-graph cache consumer, then activation. Interim TODAY: batch witnesses into one claim_batch invocation - one resolve amortized.
- Offline recipes (complexity lane): same request-major cost + both #6441 defects; frame-exit landed (5), admission pending (P2).

Re-scope note for the in-flight directive: within-run module-grain resolve sharing is valuable (multi-entry in-process surfaces; substrate for the persistent tier) but it is NOT the CI emergency - the CI floor's resolve is already pooled to one 22-second entry resolve (3c). Sequenced behind P1/P2 on the evidence.

## 8. The plan (ordered; each with owner, acceptance receipt, and what it does NOT fix)

- P1 - enumerate the latent reds in ONE pass (this lane, immediately): run the flip-shaped corpus locally (claim_batch --roster-from-discovery over the flip scan dirs), fleet-independent, producing the complete currently-red list instead of one red per CI cycle. Acceptance: the list, plus a fix/unenroll batch. Does not reduce cost.
- P2 - the admission fix (this lane, next PR): identity-availability admission for BOTH ledger and memo - pay key construction only when every composite argument's hash is already materialized (a decidable presence check, not a cost threshold); skipped calls land as a counted identity_uncomputed receipt class, never silent. Acceptance: eager-ram harness 3.1x -> ~1x; local corpus-node wall drop (the direct CI-corpus A/B this doc lacks); CI corpus node fits its budget. Does not fix builds or latent reds.
- P3 - build + slot relief (measurement-infra lane; receipts from 2b and 6 handed over): ~33m cold release builds inside the job budget, rust-gate 45m timeouts under saturation, slot packing. Acceptance: a .rs-touching merge's job fits its budget with headroom. Does not fix eval cost.
- P4 - the persistent tier for the dev loop (store lane owns encoding; cache consumer next; this lane's resolve-count gate is the acceptance harness): fixes the 10-21s per-invocation resolve pain. Not CI-critical per 3c.

## 9. Open unknowns (typed, so they cannot silently harden into claims)

- How many latent-red witnesses remain in the flip corpus (P1 answers; until then serial-exposure risk stands).
- The corpus node's exact tax share (bounded by the controlled 3.1x and the 53min wasted_ms line; pinned by P2's local A/B).
- Failure mode of run 29130137992 (23:21) - uninspected; and the first flip run's rust_tests failure mode - uninspected.
- The rust_tests-job removal (01:10) rationale and its coverage consequence - operator-side action, outside this lane's view.
- Flip-corpus width/packing interaction under the S2a resolver-memory work (their lane's calibration triple; single-witness-per-process sequencing already agreed).

## Dissolution trigger (DESIGN §6)

This is an incident-evidence document, not a standing design. It dissolves when (a) main is green with the affected-set flip ON for a sustained window, and (b) the two pinning receipts it names have landed: the P1 complete latent-red enumeration and the P2 direct corpus A/B (admission fix acceptance). At that point its durable content lives in the carriers it cites (the floor receipts, the admission increment's rows, the resolver design docs) and this narrative retires.
