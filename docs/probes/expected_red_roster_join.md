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
| `not_evaluated` | no subject verdict (not in manifest, host tool missing, marginal budget, …) |

`not_evaluated` is load-bearing: folding unevaluable rows into either other bucket is
fail-open. A nonzero `not_evaluated` count is a headline number, not a footnote.

## How verdicts are obtained

**Not log parsing.** On the CI path the in-floor join **consumes the verdict the fold already produced** — it records `ClaimOutcome` from the single `run_claim_measured` call per claim, with no second execution. The standalone bin runs join-only (one eval per enrolled identity). `run_head` in the TSV is `git rev-parse HEAD` at execution time.

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

**Sequencing:** open branches may still classify rows as `not_evaluated` with
`host_tool_unresolved` because PR workflows run from the branch tip (rust-cache cleanup
distortion; fixed on main at #8420). Do not use a join run from that window as the basis
for roster pruning.

## Authority

- Implementation: `src/v1/stage0/src/expected_red_roster_join.rs`
- Floor hook: `run_required_floor` when join env vars are set
- Related: #8411 (visible-rows subset prune — complementary, not duplicated here)
