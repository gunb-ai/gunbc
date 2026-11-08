PATCH 6 posted, comment-only: #16006 issuecomment-6707094100. Commit 56ead5acee36dd72854f296a6e7fa39b934e87c4,
parent 66abf3838424b306ad42a2faf69264e0fde24a5e (P5), tree fa9c77c01883f4a723b3fd2bf4321a28d8089d01, base
ed007978dc2a67096df619b1b91566458dfb2ffd. P5 -> P6 is 2 files, 30 changed content lines, 0 non-comment lines
(grep -E '^[+-]' over the delta, stripped, excluding ^// and blanks: 0). Cumulative base -> HEAD: 6 files, 79,401
bytes, sha256 71ba2135abd5c88562edff2425918aa54b31ebebeff6013d3fb49b7fe635d732; the P5 cumulative remains
50326385400cd4b4d3b339af6f24012c1e15a0357ad6b95ca6eca34cb72172ce. Raw commit bytes (2,074 B, sha256
3a7dfff31d329b6e8bacf1a4eef0174dab033c09b400a1d325a8952f0f9a7008), the delta (5,323 B, sha256
c61d8149dc5c781fb0d2526a00301c119078d72110cf4e3ca6eb7478c9588ba5) and the cumulative are all posted under
sentinels with the recipe. Verified from the POSTED text: hash-object on the decoded commit gives 56ead5acee3...,
the two patch sha256 values match, and the cumulative applies to ed007978dc2a with git write-tree returning
fa9c77c01883.

Only N1-N3 were touched, as scoped: N1 the demotion-deletion prediction in the new control's header (reused becomes
1, miss_declared_refused stays 0; parsed stays 0 and no receipt, because the Reused arm reaches
population_parse_accepted's roster arm too); N2 the receipt shape, now quoted as "parse-verdict" TAB source_digest
TAB compiler TAB "Accepted" from parse_verdict_stored_accepted_line, with parse_verdict_receipt_lines' line-folded
Map key as the stronger reason UNOBSERVABLE stands; N3 the production population_parse_member_decided header, which
now distinguishes the fixture-root walk_population_controls from the workflow's parse-verdict steps and claims
nothing about the new publishing route having run. PATCH 5's text is left intact and section 2 of PATCH 6 carries
the erratum. No control, no production behaviour, no style sweep.

ONE ITEM I DID NOT FOLD IN, for your decision rather than mine. The locator comment I authored says the candidates
are "always ancestors of the start and never the start", and that ordinal 0 is reachable "which no caller asks for".
Both are false on the source: data parse_verdict_restore_ancestor_ordinals = [0..11] (dag/gunbc/parse_verdict_schema
.dag:119) and the emitted locator runs 'git rev-list --first-parent --skip=0 --max-count=1 "$BASE_SHA"', which
returns the START itself. The invariant that actually matters is narrower: the start must not be the commit under
verification. On the pull-request route that holds (base.sha is an ancestor of the head); on the push route it holds
only when github.event.before differs from github.sha, which is exactly the force-push / batched-push residual
already declared as a review claim. So the honest options are (a) correct the comment to the narrower invariant and
keep the residual as a declared review claim, or (b) add the trivial guard before != sha. I have made neither change
and added no guard, per "no implementation/control change". Say which and I will do that one thing.

Status: nothing executed. Regeneration, claims, real-producer inhabitance, same-subject cold/reused parity,
generation, host contact, push and LAND remain OPEN and withheld.
