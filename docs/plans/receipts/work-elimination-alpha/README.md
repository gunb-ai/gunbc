# Receipts: work-elimination alpha sample

Raw data backing [work-elimination-alpha-measurement.md](../../work-elimination-alpha-measurement.md).

`merged-prs-sample.tsv` — the 60 most recently merged pull requests on `gunb-ai/gunbc` as of
2026-08-20T05:30Z (query time), pulled verbatim from the GitHub API (an external, versioned
system of record — not a measurement copied from the working tree). Columns: PR number,
`createdAt`/`mergedAt` (ISO-8601, UTC, GitHub's own timestamps), `admitted_unit_minutes`
(`mergedAt - createdAt`, in minutes), `additions`/`deletions`/`changed_lines`/`changed_files`
(GitHub's diff stat for the PR), and title.

Regeneration command (idempotent up to which 60 PRs are most recent at run time):

```
gh pr list --repo gunb-ai/gunbc --state merged --limit 60 \
  --json number,title,createdAt,mergedAt,additions,deletions,changedFiles
```

No row was hand-edited, filtered, or excluded after the pull.
