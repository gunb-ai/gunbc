Two more revisions posted on #16006, both comment-only, both verified from the POSTED text.

PATCH 6 (the three minors you relayed): issuecomment-6707094100 — commit 56ead5acee36dd72854f296a6e7fa39b934e87c4,
parent 66abf3838424 (P5), tree fa9c77c01883f4a723b3fd2bf4321a28d8089d01. N1 the demotion-deletion prediction, N2 the
receipt line shape, N3 the production population_parse_member_decided header. 2 files, 30 changed content lines, 0
non-comment lines.

PATCH 7 (one further text correction, authorized by keen-otter as option (a) after I raised it): issuecomment-6707115673
— commit bda8255e415bf924e0bf941f8538f2ddec4c1770, parent 56ead5acee36 (P6 preserved), tree
96103c723b4c5096142c50847bd18b3d14020e9a. One file, dag/gunbc/parse_verdict_restore.dag, comment-only. My locator
comment had claimed the ancestor candidates are "never the start" and that ordinal 0 is never asked for; both are
false (ordinals are [0..11] and the emitted locate row runs 'git rev-list --first-parent --skip=0 --max-count=1
<start>', which returns the start). It now states the selection facts exactly as keen-otter specified — ordinal 0 is
the start, PR start is event.pull_request.base.sha, push start is event.before, no exclusion of the judged commit is
enforced by this code, no blanket "PR base is an ancestor of head", force-push before can be a non-ancestor while a
batched push neither establishes equality nor fails anything — with the standing contract stated first and unchanged,
and an explicit statement that none of those event shapes has been executed. No guard, no control, no refreeze.

Scope proof for reviewers: PATCH 5 (the version sleek-owl graded READY) -> PATCH 7 = 3 files, 0 non-comment changed
lines; patch sha256 0c42baf23a8fc861cb7e233c9cc46542d1661506bb119f673fa7121a754d169d. Cumulative base ed007978dc2a ->
HEAD: 6 files, 80,771 bytes, sha256 777ae6561298ff0f88e9a694ec35f7a7d929f96b4fbd855934fef5a28c66e0b0; applies cleanly
and git write-tree returns 96103c723b4c. Raw commit bytes for both revisions are posted with sha256, lengths and the
sentinel extraction recipe.

NEW READ-ONLY EVIDENCE on the brief's self-reinforcing-loop lead, from the Actions cache API (read-only; no run,
dispatch or push). 296 published receipt caches exist, keyed gunbc-materialization-parse-verdict-<parse-compiler
identity>-<commit sha>, grouped into 16 identity eras (identity changes whenever the tokenize+parse+schema closure
changes). Joining against the last 30 main commits: the 12 commits merged while the timeout was 120 (07:10Z–10:01Z)
ALL have published receipts; every one of the 8 after the revert to 90 (49d4bc36 at 10:01Z onward: 0cfb85d8, f793a798,
ed007978, a7de1afe, cce400af, 2340e0af, 8d22701a, bef04740) has NONE. The run list explains it: 50445 (0cfb85d8) and
50451 (f793a798) are conclusion=cancelled, and six push runs from 11:30Z onward are still in_progress, the oldest
(50453 at my base ed007978) started 11:12Z. So publication is exactly as the brief hypothesised — it happens only when
a main push reaches its save steps — and at 90 minutes the current main tip publishes nothing, which is what pushes the
PR-side locator deeper with every merge. Kept as an observation, not a justification for raising the ceiling: the
baseline stays 90, and Change 1 is what lets a push finish inside it by making the push walk reuse instead of
re-parsing.

Also sent earlier and unchanged: the emit-step correction (each emit step is ~20–22 min of ONE compile_transaction
whose recorded children total ~50 s, so my Change 2 estimate of 1,150–1,900 s of duplicated reconcile+emit is NOT
supported — the recorded shared setup is ~50 s per step and the rest is an unattributed floor).

Holding now. Nothing executed by me: no build, claim, regen, run, dispatch or push; the only writes are local commits
in my own worktree and these comments. Awaiting sleek-owl's scope confirmation and keen-otter's decisions; generation,
execution, push and LAND stay with keen-otter.
