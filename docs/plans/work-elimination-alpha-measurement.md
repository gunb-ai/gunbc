# Work-elimination alpha: baseline vs. admitted unit-minutes, measured on gunbc

**Status: measurement note, receipts-backed. No code lands from this document.** It answers one
question — how much wall-clock work does the agent-session pipeline eliminate versus a
non-agentic baseline, on one real repository — with the receipts that back every number, and it
is explicit about which numbers are measured and which are estimated. Following DESIGN §5's
test-oracle rule (a merge-blocking comparison needs an independent referent, never a literal
copied from the same tree being measured): the **admitted** side is measured directly from an
external system of record (GitHub's own PR timestamps); the **baseline** side cannot be measured
this way — this repository's visible history starts already agent-driven (see §4), so there is no
in-repo pre-agent era to sample — so it is instead grounded in a cited, external, published
estimate, and reported as a range rather than a single fabricated point.

## 1. Definitions

- **Unit** — one admitted change: a merged pull request on `gunb-ai/gunbc`. This is the smallest
  thing the repository's own process treats as a discrete deliverable (one review, one merge
  commit, one entry in `git log`).
- **Admitted unit-minutes** — the wall-clock minutes a unit actually spent in this repository's
  real admission pipeline, `mergedAt - createdAt` on the GitHub PR, taken verbatim from the
  GitHub API. This *includes* review latency, CI wait, and any idle time between PR-open and
  merge — it is not "agent thinking time" alone, so if anything it **overstates** the pipeline's
  true minimum and understates alpha (§5).
- **Baseline unit-minutes** — the wall-clock minutes an equivalent unit would take a working
  engineer to produce and land *without* an agent-session pipeline: design, implement, self-test,
  open a PR, and get it reviewed, for the same diff. There is no way to observe this baseline
  inside this repository (§4), so it is derived from the diff size and a cited external
  production rate (§3).
- **Alpha** — the fraction of baseline unit-minutes eliminated: `1 - admitted / baseline`, i.e.
  the work-elimination the agent-session pipeline achieves relative to that baseline.

## 2. Sample and receipts

The 60 most recently merged PRs on `gunb-ai/gunbc` as of 2026-08-20T05:30Z, pulled unfiltered
from the GitHub API:

```
gh pr list --repo gunb-ai/gunbc --state merged --limit 60 \
  --json number,title,createdAt,mergedAt,additions,deletions,changedFiles
```

Full row-level receipts (PR number, `createdAt`, `mergedAt`, computed `admitted_unit_minutes`,
`additions`, `deletions`, `changed_lines`, `changed_files`, title) are checked in at
[`receipts/work-elimination-alpha/merged-prs-sample.tsv`](receipts/work-elimination-alpha/merged-prs-sample.tsv),
with regeneration instructions in that directory's `README.md`. No row was excluded or hand-edited
after the pull.

Sample window: `createdAt` of the oldest PR in the sample is `2026-08-19T07:00:18Z`; `mergedAt`
of the newest is `2026-08-20T05:17:24Z` — about 22 hours of wall-clock throughput, 60 merged units,
totalling 24,044 changed lines (additions + deletions) across 60 PRs.

Admitted unit-minutes, measured directly (n=60):

| stat | minutes |
|---|---|
| min | 13.4 |
| p25 | 52.7 |
| median | 126.4 |
| p75 | 243.4 |
| max | 805.1 |
| **sum** | **11,294.6** (188.2 hours) |

## 3. Baseline: a cited external rate, not a fabricated one

There is no controlled fixture for "how long would a human take to write this diff" — it is a
counterfactual. To avoid the exact failure DESIGN §5 names (a merge-blocking or fact-stating
number with no independent referent), the baseline is anchored to a published, external,
industry-wide production-rate estimate rather than invented for this document: Steve McConnell,
*Code Complete*, 2nd ed. (Microsoft Press, 2004), ch. 27, citing multiple industry studies —
professional software engineers deliver on the order of **10–50 lines of production code per
staff-day** across the full lifecycle (design, implementation, self-test, integration), not just
typing time. That range, not a single number, is carried through the computation, at an assumed
8-hour (480-minute) staff-day:

`baseline_minutes_per_line = 480 / LOC_per_day`

## 4. Why no within-repo baseline exists

The natural stronger receipt would be this same repository's own pre-agent era: measure real
human PR throughput here, before agent sessions, as the baseline. That receipt does not exist —
`git log --reverse` shows the earliest visible commit already at PR #1450, dated 2026-05-01, and
the repository's own commit-author distribution over its full history (3,062 "Brian Searls",
1,688 `gunbai-bot[bot]`, 2 "Claude" as raw `git log` author strings — agent-authored commits in
this repo are attributed to the operator's or a bot's git identity, not distinguishable from
human commits by author field alone) gives no clean pre/post split to sample. This is recorded
here rather than papered over: the baseline in §3 is the best available *external* referent, not
a claim that a stronger internal one was checked and matched it.

## 5. Result

Total admitted unit-minutes (measured): **11,294.6** (60 PRs, 24,044 changed lines).

| baseline rate | baseline minutes | baseline hours | alpha | admitted is 1/N of baseline |
|---|---|---|---|---|
| 50 LOC/staff-day (upper end of McConnell's range — fastest baseline) | 230,822 | 3,847 | 95.1% | 20.4x |
| 25 LOC/staff-day (midpoint) | 461,645 | 7,694 | 97.6% | 40.9x |
| 10 LOC/staff-day (lower end — slowest baseline) | 1,154,112 | 19,235 | 99.0% | 102.2x |

Reading the range conservatively (use the *fastest* baseline, 50 LOC/day, which produces the
*smallest* alpha): on this sample, the admitted pipeline eliminates **at least ~95%** of the
baseline unit-minutes a McConnell-rate human process would need for the same merged diffs — and,
per §1, `admitted_unit_minutes` here still includes review/CI wait time it did not need to, so
this 95% figure is itself a floor, not a ceiling.

## 6. Limitations (stated so the number is not over-read)

- **`changed_lines` is not authored-effort-equivalent.** A diff stat counts every added or
  deleted line identically, whether hand-reasoned or produced by this repository's own
  regeneration/emission tooling (§ "Building & checks" in `DESIGN.md` — `claim_executor`,
  witness generation, `regen`). A baseline built by multiplying line count by a human authoring
  rate implicitly assumes every line required human-rate authoring; to the extent generated
  lines are over-represented in this repo's diffs relative to the industry corpus McConnell's
  rate was measured on, the baseline (and therefore alpha) is inflated. No correction is applied
  here because there is no reliable way to partition "authored" vs. "generated" lines from the
  GitHub diff stat alone — this is named as an open gap, not silently absorbed into the number.
- **`admitted_unit_minutes` measures PR lifetime, not session compute time.** It is a real,
  externally-timestamped receipt, but it conflates review wait, CI wait, and actual working time.
  It is a defensible upper bound on the pipeline's true cost and therefore, per §1, a
  conservative (alpha-deflating) choice — but it is not a decomposition of where the minutes went.
- **One repository, one 22-hour window.** This is exactly what the work item asked for — "one
  real repository, with receipts" — not a claim of generality across repositories, teams, or
  problem domains. `gunb-ai/gunbc` is a compiler/language project with unusually heavy tooling
  (regen, witnesses, lenses) already amortizing effort DESIGN.md's own §2 exists to eliminate;
  a codebase without that infrastructure would likely show a different multiple.
- **The baseline range spans 5x (10 vs. 50 LOC/day) and alpha still only moves from 95.1% to
  99.0%.** The result is not sensitive to where in McConnell's published range the baseline sits
  — which is a mild robustness check on the direction of the finding, not a claim that a McConnell
  rate is the *right* baseline in an absolute sense.

## 7. Reproducing this measurement later

Re-run the `gh pr list` command in §2 against a fresh window (the same 60-PR trailing sample, or
any explicit date range via `--search "merged:<range>"`), recompute `admitted_unit_minutes` and
`changed_lines` from the JSON exactly as `receipts/work-elimination-alpha/README.md` describes,
and hold §3's baseline rate constant across runs so alpha changes reflect the pipeline, not a
moving referent.
