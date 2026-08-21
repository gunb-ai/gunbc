# `gunbc run --entry` is an invalid harness for `fold_lowering_test.dag` (2026-08-21)

| | |
|---|---|
| class | **instrument defect, not subject defect** — a stale RED, which is worse than a stale green because an invented defect gets investigated |
| symptom | `error: evaluating <fn> in src/v2/test/claim/long/fold_lowering_test.dag` / `cause: NoSuchField { type_name: "NormalizedTree", field: "kind" }` |
| harness | `gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/long/fold_lowering_test.dag --function <f>` |
| the claim it reds | `fold_call_lowers_to_terminating_loop` — a **pre-existing, unmodified** claim |
| subject | `e336acc4f0f85b2ddda61ec50176071e972efffd` (`main`), detached, `git status --porcelain` **empty**; the claim file's blob hash `d657e9df9b44dbc48d371c45aa00cecea9f5b4e9` equals `FETCH_HEAD:src/v2/test/claim/long/fold_lowering_test.dag` |

---

## The receipt

```
HEAD:            e336acc4f0f85b2ddda61ec50176071e972efffd
EXPECT:          e336acc4f0f85b2ddda61ec50176071e972efffd
DIRTY:           (empty)
CLAIM-FILE-SHA:  d657e9df9b44dbc48d371c45aa00cecea9f5b4e9
MAIN-CLAIM-SHA:  d657e9df9b44dbc48d371c45aa00cecea9f5b4e9

===== fold_call_lowers_to_terminating_loop on MAIN
error: evaluating fold_call_lowers_to_terminating_loop in src/v2/test/claim/long/fold_lowering_test.dag
  cause: NoSuchField { type_name: "NormalizedTree", field: "kind" }
```

**The subject line is printed rather than asserted, and that is not ceremony here — the first attempt at this receipt was WRONG in exactly the way the subject line catches.** The remote runner had checked out the pushed lane branch, so `HEAD` read `6c06bd5` and a `git checkout --` restored the lane's own modified files. It reproduced the branch a second time and returned the expected error, which is precisely what makes that failure mode dangerous: a measurement bound to the wrong revision agrees with you. The re-run pins `FETCH_HEAD` to main's SHA and carries the blob hash on both sides, so the subject is checkable instead of assumed.

## Why this is filed on its own

It was found while validating an unrelated change (the fold-lowering disposition split, gunbc#8801), and a reader who meets it there will read it as that change's problem. It is not: the claim it reds is one nobody edited, and the field it cannot find belongs to a type that change does not touch.

The cost of leaving it implicit is specific. Anyone who reaches for `gunbc run --entry` on this file — the obvious move, and the one the fold-lowering note's own "verified end-to-end on real source" invites — gets a red that has nothing to do with their change. A stale green wastes the time of whoever eventually finds the gap; a stale red spends someone's whole session investigating a defect that was never there.

## What the harness actually gets wrong

`ingest` returns `normalize(parse_tree: artifact.tree)`'s `Accepted` value and `locate_call` then reads it as a `Node`. Under the floor those resolve; under `--entry` the `NormalizedTree` wrapper survives and the field read refuses. This is the admission divergence between entrypoints in its sharpest form — the same source, the same function, two answers — and the discriminating fact is that it reproduces on a tree with no local modification at all.

## What this does NOT claim

- It does not locate the defect in `gunbc run`, in `normalize`'s return shape, or in the claim file. It establishes only that the two entrypoints disagree on this subject, and that the disagreement is not caused by any change in flight.
- It does not establish the population. Other claim files may or may not be affected; no sweep was run, and a count here would be a guess.

## The harness that IS valid

The witness floor — `claim_executor --required-ci`, which `witnesses.yml` invokes. Its floor phase folds the whole discovered roster through one prepared subject, so these claims are enrolled by discovery. That is where this file's greens and reds mean something.
