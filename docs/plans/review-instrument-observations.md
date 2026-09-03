# Review-instrument observations (not a census of this repository)

Two findings about the **review and merge-readiness instrument**, recorded because they were each
found by accident while doing other work and are the kind of thing that is otherwise rediscovered
expensively. They are facts about the tooling, not about the seed or the `.dag` corpus, which is why
they are here rather than in [seed-string-decode-census.md](seed-string-decode-census.md).

Both were observed on `gunb-ai/gunbc#10180` and independently reproduced by a second session.

## 1. `stale_provider_count` does not fire, even where both operands are local

A single `dashboard-ops reviews 10180` payload carried **three different shas at once**:

| field | value |
|---|---|
| `reviews[0].sha` (the approving review) | `60aefd9d3acb684e` |
| `merge_criteria.head_sha` (the dashboard's own head) | `aff564ca8a7d2fc7` |
| `gh pr view --json headRefOid` (actual) | `2c9c08a439b5b0c2` |

and, in that same object: `stale_provider_count: 0`, `stale_providers: []`, `approvals: 1`,
`meets_approval_rule: true`.

**What is observed, stated before what explains it.** The approval sha differed from the dashboard's
own `head_sha`, both fields present in one object, and `stale_provider_count` still read 0. A later
payload on the same PR DID report `stale_provider_count: 1`, so the field is not simply inert.

**Correction to an earlier reading.** This document first concluded "the comparison does not fire even
when it has everything it needs". Two observations now constrain it better, and the second refutes
that sentence as written:

| observation | `head_sha` | provider's latest review sha | `stale_provider_count` |
|---|---|---|---|
| A | `aff564ca8a7` | claude @ `60aefd9d3ac` | **0** |
| B | `dee613521e4` | claude @ `dee613521e4`, codex @ `c29a33ce28e` | **1** (codex) |

In B the field fires correctly. **Hypothesis that fits both, and it is a hypothesis, not a
measurement:** staleness is evaluated when a review is INGESTED, against the head known at that
moment, and stored — while `head_sha` is read live at query time. Under that reading A is a stored
verdict that was true when written and went false underneath, not a comparison that failed to run.
Distinguishing it would take a payload sampled at a known ingest boundary, which has not been done.

**The operative rule is unchanged either way**, which is why the correction does not disturb it: a
stored-and-gone-stale verdict and a non-firing comparison are indistinguishable to a reader, and both
report 0 on an approval that is not on the current head.

**Consequence for any merge decision.** Comparing the dashboard's `head_sha` to `gh`'s `headRefOid` is
NECESSARY BUT NOT SUFFICIENT — it catches the lag case and would have passed observation A the moment
the dashboard caught up, with an approval three heads old. The sufficient check is computed by the
reader:

```
gh pr view <N> --repo <owner/repo> --json headRefOid --jq .headRefOid
dashboard-ops reviews <N>          # compare each reviews[].sha to that value yourself
```

`stale_provider_count` is not evidence of anything on its own — it is right in B and wrong in A, and
nothing in the payload distinguishes the two cases for you.

**The same applies to `request_changes_count` reaching 0.** On this PR a codex REQUEST_CHANGES on
`c29a33ce28e` disappeared from the counter after the next push. The findings WERE addressed in that
push — but the counter would read 0 either way, because a REQUEST_CHANGES is superseded by any push
regardless of whether anything was fixed. The commit and the reply are the evidence that the findings
were addressed; the zero is not.

**A second, softer form.** An approval can also be superseded in PREMISE rather than in sha. #10180's
approving review reasoned explicitly from *"census-only … no code, no `.dag`, no seed changes"*; a
later commit added seed code. Even had the shas matched, the verdict's stated grounds no longer
described the diff. Read what a review says it approved, not only what it approved.

## 2. A refused review burns its sha slot, invisibly to the counters

In the same payload, review `59066` carried `status: "failed"` with:

```
worktree freshness check failed: HEAD is 95ef0941… but PR #10180 head is 44efc6eb…
— refusing to review a stale/wrong checkout
```

That refusal is correct behaviour: reviewing a checkout that is not the PR head would produce a verdict
about a different object. But the slot is consumed — `44efc6eb` is never reviewed, and no retry occurs.
A PR can therefore accumulate refused reviews that leave no trace in `approvals`,
`request_changes`, or `stale_provider_count`; only the `reviews[].status` / `error` fields carry it.

**The pairing is the finding.** One half of this system refuses to review unless its checkout matches
the PR head EXACTLY; the other half reports staleness as 0 across a three-sha spread. One half is
strict about precisely the question the other half does not ask. Classify a review by its `status` and
`error` fields before reading any counter.
