# PB §1.8 Status drift-sweep — post-T-LP closure wave (pre-authored brief)

**State:** PROPOSAL — dispatch-gated on T-LensProducer-Retirement reaching CONSUMER_LANDED + PASSING (G5 #2374 + G7 #2378 retirements landed + sg0_non_test_zero ratchet trip).

**Pattern:** PR #2399 (`docs(r3): §1.8 ledger Status drift sweep — 10 promotion candidates`). Single PB-side drift-sweep PR ratcheting Status discipline; no cross-Mgr coordination required.

**Authority:** Director ratification at gunbc#828 #issuecomment-4415877726 of PB Mgr reconciliation (#issuecomment-4415872125), explicit pre-author endorsement per `feedback_pre_authored_brief_queue`.

## Gates queued for DECLARED → CONSUMER_LANDED promotion

| Gate | Name | Evidence | Status path |
|---|---|---|---|
| #6 | `lens_testgen_dot_rs_retired` | PR #2392 (`b11c8014`, 06:03Z producer-retirement) + PR #2594 (`484c719e`, 15:40Z consumer regression-guard) | DECLARED → CONSUMER_LANDED + PASSING |
| #33 | `bridge_canonical_lens_name_dispatch_retired` | PR #2449 (`MERGED 01:14Z`) | DECLARED → CONSUMER_LANDED |
| #34 | `bridge_include_str_side_channels_retired` | PR #2459 (`MERGED 00:05Z`) — `pipeline.dag` slice retirement | DECLARED → CONSUMER_LANDED (slice scope only; standalone closure brief #1976 remains STOP-blocked on Substrate-T1 / #1939) |
| #66 | `lens_producer_retirement_executable_witness` | PR #2595 (`a085f6e9`, 15:52Z substrate-impl: TestRunner reports residual) | DECLARED → CONSUMER_LANDED (substrate-plumbing only); F3 deferral keeps PASSING gated on aggregate Row-4 evidence |
| #5 | `lens_apply_dot_rs_retired` | PR-pending (worker valiant-otter-715 on #2374) | DECLARED → CONSUMER_LANDED (when PR lands) |
| #7 | `regen_lens_dot_rs_retired` | PR-pending (worker warm-crab-600 on #2378) | DECLARED → CONSUMER_LANDED (when PR lands) |
| #8 | `sg0_non_test_zero` | PR #2569 (definition alignment) + ratchet trip post T-PB-A non-test = 0 | DECLARED → CONSUMER_LANDED + PASSING (when ratchet trips zero) |

## Excluded (cross-Mgr or not yet evidenced)
- **#31** `bridge_source_span_file_participation_retired` — Substrate charter (#2068).
- **#36** `bridge_retirement_ledger_zero` — Verification charter (#2075).
- **#41 / #42 / #60 / #71** — T-V2-Retirement HELD on PM-authored S-1 brief #1974.

## PR shape
- Single docs-only PR amending §1.8 row Status fields per the table above.
- Cite-link each PR-evidence in the row's Evidence cell.
- No code changes; no test changes; no cross-Mgr blockers.
- Squash-merges as `docs(r3): §1.8 ledger Status drift sweep — T-LP / T-Bridge wave`.

## Dispatch trigger
Spawn worker sub-issue under #2074 when:
1. PR for G5 #2374 (lens_apply.rs retirement) MERGED; AND
2. PR for G7 #2378 (regen_lens.rs retirement) MERGED.

(G6 / G33 / G34 / G66 evidence already exists; G8 ratchet trip + #5/#7 PRs are gating events. Note: ledger gate id `#66` per `r3-program-plan.md` §1.8 row #66 — `#64` is `substrate_gap_reflection_closure_closed`, a separate row.)

If T-LP cascades earlier (e.g., G5 + G7 land together with sg0_non_test_zero ratchet trip in same wave), this brief absorbs all rows in one PR. If T-LP cascades partial, the brief can split: T-LP wave first, then T-Bridge follow-on.

## Override path
If new substrate facts emerge invalidating any row's CONSUMER_LANDED evidence (e.g., regression / revert), surface to PB Mgr (#2074) before authoring. Do NOT promote rows whose evidence has been reverted.
