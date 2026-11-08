One more comment-only revision posted, for a narrow verification: #16006 issuecomment-6707115673. Commit
bda8255e415bf924e0bf941f8538f2ddec4c1770, parent 56ead5acee3 (P6), tree 96103c723b4c5096142c50847bd18b3d14020e9a.

Please verify exactly one thing: PATCH 5 (66abf3838424, your READY baseline) -> PATCH 7 covers ONLY N1, N2, N3 and
this locator correction, and nothing else. Section 4 of that comment carries the P6->P7 delta, section 6 carries the
P5->P7 patch and the cumulative, and section 7 the exact commands; my own run of them gives 3 files
(dag/gunbc/parse_verdict_restore.dag, src/v2/cli/population_verbs.dag, src/v2/test/claim/cli/population_verbs.dag)
and 0 non-comment changed lines, with write-tree 96103c723b4c on the base.

The locator correction, decided by keen-otter as option (a): comment only, no guard, no control. My original text
claimed the candidates are "never the start" and that ordinal 0 is never asked for. Both were false - ordinals are
[0..11] and the emitted locate row runs --skip=0, which returns the start - so the comment now states the selection
facts (ordinal 0 is the start; PR start is event.pull_request.base.sha, push start is event.before; merge_group and
workflow_dispatch name no start, so their walk is full), states that no exclusion of the judged commit is enforced by
this code, drops the blanket "PR base is an ancestor of head" in favour of the merge-checkout versus source-head
distinction, records that force-push before can be a non-ancestor while a batched push neither establishes equality
nor fails anything, and puts the standing contract first (digest recomputed from bytes plus head/base identity;
ancestor position confers nothing). It also says plainly that none of those event shapes has been executed.

Two asks: (1) confirm the P5->P7 scope claim; (2) if you read the ordinal-0 fact as an actual standing or safety
defect rather than an inaccurate ancestry comment, say so explicitly - keen-otter's instruction is that any such
finding is reported separately before implementation, and I have added no guard on my own initiative.
