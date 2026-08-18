# Expected-red roster identity join

**Status:** mechanism landed; **do not prune the roster from this output yet.**

The floor's failure log only surfaces a **subset** of enrolled identities that are now
passing (`test.claim.*` on a recent main run, while `v2.test.*` rows on the same roster
can pass without producing a failure line). Pruning exactly the visible rows leaves stale
enrollments behind and makes the tree look correct while the roster still asserts fixed
defects.

## Three outcomes (identity grain)

Every identity in `v2.workflow.floor_expected_red` receives exactly one:

| disposition | meaning |
| --- | --- |
| `still_red` | witness ran and failed on its subject |
| `now_passes` | witness ran and returned true — **retire the enrollment, keep the witness** |
| `not_evaluated` | no subject verdict (not in manifest, host tool missing, …) |

`not_evaluated` is load-bearing for rows with **no subject verdict** (infra gaps, not-in-manifest).
Budget kills and other witness failures classify as `still_red` — 675ms is a **hard cutoff**
(`v2.workflow.required_floor` `required_floor_claim_budget_ms`, operator 2026-08-17, raised
to 550 then 675 on 2026-08-18 for shared-runner contention in the single-fold prepared-subject
job; receipts runs 32116564202 and 32123958902);
a witness near that line is over-large regardless of host-load jitter.

## Witness cost (operator ruling, 2026-08-18)

675ms HARD, 100ms warn (`v2.workflow.required_floor` `required_floor_claim_budget_ms`). The
target is measured wall time, not
the failure list: ~267 witnesses exceed the hard cutoff on main while only ~11–15 fail at the
deadline (unpollable host builtins). Rank by measured ms and pare over-large witnesses into
smaller discriminating fixtures; whole-corpus scans belong on a cadence, not per-PR.

## How verdicts are obtained

**Not log parsing.** On the CI path the in-floor join **consumes the verdict the fold already produced** — it records `ClaimOutcome` from the single `run_claim_measured` call per claim, with no second execution. The standalone bin runs join-only (one eval per enrolled identity). `run_head` in the TSV is `git rev-parse HEAD` when available; the header field is empty when git is unavailable.

**CI wiring:** `GUNBC_EXPECTED_RED_ROSTER_JOIN=expected_red_roster_join.tsv` in `gunbc.witness_floor_workflow` → `.github/workflows/witnesses.yml`.

## Terminal consumer

**(a) Terminal instrument** — consumed by `run_required_floor` on every witnesses CI run. Completes the floor's existing expected-red accounting (held + now-passing) with the required `not_evaluated` third bucket. The `expected_red_roster_join` bin alone is interim operator transport; dissolve-on when floor emits the join report by default.

## Run (after rebase wave)

```bash
cargo run -p v1-compiler --bin expected_red_roster_join -- \
  docs/probes/expected_red_roster_join.tsv
```

Environment:

- `GUNBC_EXPECTED_RED_ROSTER_JOIN=<path>` — write TSV receipt
- `GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY=1` — evaluate only enrolled identities present in
  the manifest (set by the bin)

**Sequencing:** eight identities that passed on CI run 32103473552 were removed from
`v2.workflow.floor_expected_red` (2026-08-18). Further pruning waits until host-tool
verdicts are trustworthy post-rebase (#8420).

## Authority

- Implementation: `src/v1/stage0/src/expected_red_roster_join.rs`
- Floor hook: `run_required_floor` when join env vars are set
- Related: #8411 (visible-rows subset prune — complementary, not duplicated here)
