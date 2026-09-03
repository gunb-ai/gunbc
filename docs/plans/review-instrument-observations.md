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

**The reading that matters.** The obvious diagnosis is a lag: the dashboard has not yet noticed the
newest push, so `stale = 0` means *not yet noticed* rather than *judged current*. That is true and is
itself worth knowing — a summary can report a clean 1/1 on code no reviewer has seen. But it is not the
whole defect: the approval sha differs from **the dashboard's own `head_sha`**, both fields present in
one object, and staleness still reads 0. The comparison does not fire even when it has everything it
needs.

**Consequence for any merge decision.** Comparing the dashboard's `head_sha` to `gh`'s `headRefOid` is
NECESSARY BUT NOT SUFFICIENT — it catches the lag case and would pass this payload the moment the
dashboard caught up, with an approval three heads old. The sufficient check is computed by the reader:

```
gh pr view <N> --repo <owner/repo> --json headRefOid --jq .headRefOid
dashboard-ops reviews <N>          # compare each reviews[].sha to that value yourself
```

`stale_provider_count` is not evidence of anything.

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
